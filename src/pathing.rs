use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{PathEdge, PathEndMarker, PathExtensionHint, PathTile, Tower};
use crate::constants::{HEX_SIZE, HEX_SPACING, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{GameOver, GameWon, Money, PathTiles, TowerDraft, TowerDraftPhase};

/// The 6 edge-normal angles of a pointy-top hexagon, in degrees. Each one both
/// points at the center of the neighboring hex across that edge, and is the
/// direction of that edge's midpoint from this hex's center.
const HEX_EDGE_ANGLES_DEG: [f32; 6] = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0];

/// World-space offsets to each of the 6 neighboring hex centers.
pub fn hex_neighbor_offsets() -> [Vec2; 6] {
    HEX_EDGE_ANGLES_DEG.map(|deg| Vec2::from_angle(deg.to_radians()) * HEX_SPACING)
}

pub fn is_buildable_cell(position: Vec2, path_tiles: &PathTiles) -> bool {
    is_in_play_area(position) && !path_tiles.contains(position)
}

pub fn update_path_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    game_over: Res<GameOver>,
    game_won: Res<GameWon>,
    draft: Res<TowerDraft>,
    mut money: ResMut<Money>,
    mut path_tiles: ResMut<PathTiles>,
    towers: Query<&Transform, With<Tower>>,
    mut end_marker: Query<&mut Transform, (With<PathEndMarker>, Without<Tower>)>,
    mut path_visuals: ParamSet<(Query<Entity, With<PathTile>>, Query<Entity, With<PathEdge>>)>,
    hints: Query<Entity, With<PathExtensionHint>>,
) {
    if game_over.value || game_won.value || matches!(draft.phase, TowerDraftPhase::Placing(_)) || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let grid_position = snap_to_grid(world_position);
    if !is_in_play_area(grid_position) || !path_tiles.can_extend_to(grid_position) {
        return;
    }
    if towers
        .iter()
        .any(|t| t.translation.truncate().distance(grid_position) < HEX_SPACING * 0.5)
    {
        return;
    }

    let cost = path_tiles.extension_cost();
    if money.amount < cost {
        return;
    }

    money.amount -= cost;
    path_tiles.extend_to(grid_position);
    for entity in path_visuals.p0().iter() {
        commands.entity(entity).despawn();
    }
    for entity in path_visuals.p1().iter() {
        commands.entity(entity).despawn();
    }
    for entity in &hints {
        commands.entity(entity).despawn();
    }
    let tower_positions: Vec<Vec2> = towers.iter().map(|t| t.translation.truncate()).collect();
    spawn_path_visuals(&mut commands, &mut meshes, &mut materials, &path_tiles, &tower_positions);

    if let Ok(mut marker_transform) = end_marker.single_mut() {
        marker_transform.translation = grid_position.extend(0.0);
    }

    spawn_floating_text(
        &mut commands,
        format!("-${cost}"),
        grid_position + Vec2::new(-30.0, 32.0),
        Color::srgb(1.0, 0.86, 0.20),
        20.0,
    );
}

pub fn spawn_path_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    path_tiles: &PathTiles,
    tower_positions: &[Vec2],
) {
    for tile in &path_tiles.tiles {
        spawn_path_tile(commands, meshes, materials, *tile);
    }

    for (index, tile) in path_tiles.tiles.iter().enumerate() {
        spawn_path_edges(commands, path_tiles, index, *tile);
    }

    let cost = path_tiles.extension_cost();
    let end = path_tiles.end();
    for offset in hex_neighbor_offsets() {
        let candidate = end + offset;
        if !path_tiles.can_extend_to(candidate) || !is_in_play_area(candidate) {
            continue;
        }
        if tower_positions
            .iter()
            .any(|&t| t.distance(candidate) < HEX_SPACING * 0.5)
        {
            continue;
        }
        commands.spawn((
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
            MeshMaterial2d(materials.add(Color::srgba(0.65, 0.60, 0.38, 0.22))),
            Transform::from_translation(candidate.extend(-1.5)),
            PathExtensionHint,
        ));
        commands.spawn((
            Text2d::new(format!("${cost}")),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgba(0.88, 0.82, 0.55, 0.75)),
            TextShadow::default(),
            Transform::from_translation(candidate.extend(-1.0)),
            PathExtensionHint,
        ));
    }
}

pub fn update_path_hints(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    draft: Res<TowerDraft>,
    game_over: Res<GameOver>,
    game_won: Res<GameWon>,
    towers: Query<&Transform, With<Tower>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hints: Query<(&mut Visibility, &Transform, Option<&MeshMaterial2d<ColorMaterial>>), With<PathExtensionHint>>,
) {
    let suppress = game_over.value || game_won.value || matches!(draft.phase, TowerDraftPhase::Placing(_));

    let cursor_world = (!suppress)
        .then(|| {
            let window = windows.single().ok()?;
            let (cam, cam_t) = camera.single().ok()?;
            cam.viewport_to_world_2d(cam_t, window.cursor_position()?).ok()
        })
        .flatten();

    for (mut visibility, transform, material_handle) in &mut hints {
        if suppress {
            *visibility = Visibility::Hidden;
            continue;
        }

        let pos = transform.translation.truncate();
        let has_tower = towers
            .iter()
            .any(|t| t.translation.truncate().distance(pos) < HEX_SPACING * 0.5);
        if has_tower {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        if let Some(handle) = material_handle {
            let hovered = cursor_world
                .map(|c| c.distance(pos) < HEX_SIZE)
                .unwrap_or(false);
            if let Some(material) = materials.get_mut(&handle.0) {
                material.color = if hovered {
                    Color::srgba(0.80, 0.72, 0.42, 0.60)
                } else {
                    Color::srgba(0.65, 0.60, 0.38, 0.22)
                };
            }
        }
    }
}

fn spawn_path_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
) {
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.43, 0.39, 0.31))),
        Transform::from_translation(position.extend(-2.0)),
        PathTile,
    ));
}

fn spawn_path_edges(commands: &mut Commands, path_tiles: &PathTiles, index: usize, position: Vec2) {
    let apothem = HEX_SIZE * 0.8660254; // sqrt(3)/2
    let wall_size = Vec2::new(4.0, HEX_SIZE + 4.0);

    for angle_deg in HEX_EDGE_ANGLES_DEG {
        let angle = angle_deg.to_radians();
        let direction = Vec2::from_angle(angle);

        if is_consecutive_path_neighbor(path_tiles, index, position + direction * HEX_SPACING) {
            continue;
        }

        commands.spawn((
            Sprite::from_color(Color::srgb(0.24, 0.21, 0.16), wall_size),
            Transform::from_translation((position + direction * apothem).extend(-1.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            PathEdge,
        ));
    }
}

fn is_consecutive_path_neighbor(path_tiles: &PathTiles, index: usize, position: Vec2) -> bool {
    let previous = index
        .checked_sub(1)
        .and_then(|neighbor| path_tiles.tiles.get(neighbor));
    let next = path_tiles.tiles.get(index + 1);

    previous
        .into_iter()
        .chain(next)
        .any(|tile| tile.distance_squared(position) < 1.0)
}

/// Enumerates every hex cell (axial coordinate + world center) whose world
/// position falls within `half_extent` of the origin (both axes), for
/// drawing a bounded hex grid or generating terrain over it.
pub fn hex_cells_in_bounds(half_extent: Vec2) -> Vec<(i32, i32, Vec2)> {
    let mut cells = Vec::new();
    let row_height = HEX_SIZE * 1.5;
    let col_width = HEX_SIZE * 3f32.sqrt();

    let r_max = (half_extent.y / row_height).ceil() as i32 + 1;
    for r in -r_max..=r_max {
        let q_center = -(r as f32) * 0.5;
        let q_span = (half_extent.x / col_width).ceil() as i32 + 1;
        for q in (q_center.floor() as i32 - q_span)..=(q_center.ceil() as i32 + q_span) {
            let pos = axial_to_world(q as f32, r as f32);
            if pos.x.abs() <= half_extent.x && pos.y.abs() <= half_extent.y {
                cells.push((q, r, pos));
            }
        }
    }
    cells
}

fn is_in_play_area(position: Vec2) -> bool {
    let extent = WINDOW_WIDTH * 5.0;
    position.x >= -extent + HEX_SIZE
        && position.x <= extent - HEX_SIZE
        && position.y >= -extent + HEX_SIZE
        && position.y <= extent - HEX_SIZE
}

/// Snaps a world position to the center of the nearest hex tile, via the
/// standard cube-coordinate rounding algorithm (world -> fractional axial ->
/// rounded cube -> axial -> world).
pub fn snap_to_grid(position: Vec2) -> Vec2 {
    let (q, r) = world_to_axial(position);
    let (rq, rr) = round_axial(q, r);
    axial_to_world(rq as f32, rr as f32)
}

/// Same rounding as `snap_to_grid`, but returns the axial coordinate itself
/// rather than its world center — for looking up per-cell data (e.g.
/// terrain) keyed by axial coordinate.
pub fn world_to_axial_cell(position: Vec2) -> (i32, i32) {
    let (q, r) = world_to_axial(position);
    round_axial(q, r)
}

fn world_to_axial(position: Vec2) -> (f32, f32) {
    let q = (3f32.sqrt() / 3.0 * position.x - 1.0 / 3.0 * position.y) / HEX_SIZE;
    let r = (2.0 / 3.0 * position.y) / HEX_SIZE;
    (q, r)
}

fn axial_to_world(q: f32, r: f32) -> Vec2 {
    Vec2::new(
        HEX_SIZE * (3f32.sqrt() * q + 3f32.sqrt() / 2.0 * r),
        HEX_SIZE * (1.5 * r),
    )
}

fn round_axial(q: f32, r: f32) -> (i32, i32) {
    let (x, z) = (q, r);
    let y = -x - z;

    let mut rx = x.round();
    let ry = y.round();
    let mut rz = z.round();

    let x_diff = (rx - x).abs();
    let y_diff = (ry - y).abs();
    let z_diff = (rz - z).abs();

    // Whichever coordinate had the largest rounding error is least trustworthy.
    // We only return rx/rz, so we only need to correct one of *them* here: if x
    // (or z) was the least trustworthy, recompute it from the other two so the
    // cube constraint x+y+z=0 holds; if y was, rx/rz are already fine as-is.
    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff <= z_diff {
        rz = -rx - ry;
    }

    (rx as i32, rz as i32)
}
