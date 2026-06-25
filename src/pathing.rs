use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{PathEdge, PathEndMarker, PathExtensionHint, PathTile, Tower};
use crate::constants::{GRID_SIZE, PATH_HALF_WIDTH, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{GameOver, GameWon, Money, PathTiles, TowerDraft, TowerDraftPhase};

pub fn is_buildable_cell(position: Vec2, path_tiles: &PathTiles) -> bool {
    is_in_play_area(position) && !path_tiles.contains(position)
}

pub fn update_path_input(
    mut commands: Commands,
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
        .any(|t| t.translation.truncate().distance(grid_position) < GRID_SIZE * 0.5)
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
    spawn_path_visuals(&mut commands, &path_tiles, &tower_positions);

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

pub fn spawn_path_visuals(commands: &mut Commands, path_tiles: &PathTiles, tower_positions: &[Vec2]) {
    for tile in &path_tiles.tiles {
        spawn_path_tile(commands, *tile);
    }

    for (index, tile) in path_tiles.tiles.iter().enumerate() {
        spawn_path_edges(commands, path_tiles, index, *tile);
    }

    let cost = path_tiles.extension_cost();
    let end = path_tiles.end();
    for dir in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
        let candidate = end + dir * GRID_SIZE;
        if !path_tiles.can_extend_to(candidate) || !is_in_play_area(candidate) {
            continue;
        }
        if tower_positions
            .iter()
            .any(|&t| t.distance(candidate) < GRID_SIZE * 0.5)
        {
            continue;
        }
        commands.spawn((
            Sprite::from_color(Color::srgba(0.65, 0.60, 0.38, 0.22), Vec2::splat(GRID_SIZE)),
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
    mut hints: Query<(&mut Visibility, &Transform, Option<&mut Sprite>), With<PathExtensionHint>>,
) {
    let suppress = game_over.value || game_won.value || matches!(draft.phase, TowerDraftPhase::Placing(_));

    let cursor_world = (!suppress)
        .then(|| {
            let window = windows.single().ok()?;
            let (cam, cam_t) = camera.single().ok()?;
            cam.viewport_to_world_2d(cam_t, window.cursor_position()?).ok()
        })
        .flatten();

    for (mut visibility, transform, sprite_opt) in &mut hints {
        if suppress {
            *visibility = Visibility::Hidden;
            continue;
        }

        let pos = transform.translation.truncate();
        let has_tower = towers
            .iter()
            .any(|t| t.translation.truncate().distance(pos) < GRID_SIZE * 0.5);
        if has_tower {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;

        if let Some(mut sprite) = sprite_opt {
            let hovered = cursor_world
                .map(|c| {
                    (c.x - pos.x).abs() < GRID_SIZE * 0.5 && (c.y - pos.y).abs() < GRID_SIZE * 0.5
                })
                .unwrap_or(false);
            sprite.color = if hovered {
                Color::srgba(0.80, 0.72, 0.42, 0.60)
            } else {
                Color::srgba(0.65, 0.60, 0.38, 0.22)
            };
        }
    }
}

fn spawn_path_tile(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.43, 0.39, 0.31), Vec2::splat(GRID_SIZE)),
        Transform::from_translation(position.extend(-2.0)),
        PathTile,
    ));
}

fn spawn_path_edges(commands: &mut Commands, path_tiles: &PathTiles, index: usize, position: Vec2) {
    let directions = [
        (
            Vec2::X,
            Vec2::new(PATH_HALF_WIDTH, 0.0),
            Vec2::new(4.0, GRID_SIZE + 4.0),
        ),
        (
            Vec2::NEG_X,
            Vec2::new(-PATH_HALF_WIDTH, 0.0),
            Vec2::new(4.0, GRID_SIZE + 4.0),
        ),
        (
            Vec2::Y,
            Vec2::new(0.0, PATH_HALF_WIDTH),
            Vec2::new(GRID_SIZE + 4.0, 4.0),
        ),
        (
            Vec2::NEG_Y,
            Vec2::new(0.0, -PATH_HALF_WIDTH),
            Vec2::new(GRID_SIZE + 4.0, 4.0),
        ),
    ];

    for (direction, offset, size) in directions {
        if is_consecutive_path_neighbor(path_tiles, index, position + direction * GRID_SIZE) {
            continue;
        }

        commands.spawn((
            Sprite::from_color(Color::srgb(0.24, 0.21, 0.16), size),
            Transform::from_translation((position + offset).extend(-1.0)),
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

fn is_in_play_area(position: Vec2) -> bool {
    let half_cell = GRID_SIZE * 0.5;
    let extent = WINDOW_WIDTH * 5.0;
    position.x >= -extent + half_cell
        && position.x <= extent - half_cell
        && position.y >= -extent + half_cell
        && position.y <= extent - half_cell
}

pub fn snap_to_grid(position: Vec2) -> Vec2 {
    Vec2::new(
        snap_to_cell_center(position.x),
        snap_to_cell_center(position.y),
    )
}

pub fn snap_axis(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

fn snap_to_cell_center(value: f32) -> f32 {
    (value / GRID_SIZE).floor() * GRID_SIZE + GRID_SIZE * 0.5
}
