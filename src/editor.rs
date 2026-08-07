use std::collections::HashMap;

use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::MainCamera;
use crate::constants::{HEX_SIZE, INITIAL_PATH, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::game::pan_camera;
use crate::pathing::{hex_cells_in_bounds, world_to_axial_cell};
use crate::setup::build_hex_ring_mesh;
use crate::terrain::{
    build_hex_fill_mesh, generate_random_kinds, TerrainKind, ALL_TERRAIN_KINDS, DEFAULT_TERRAIN_KIND,
};
use crate::terrain_file::{load_terrain_overrides, save_terrain_overrides, TERRAIN_FILE_PATH};

const SWATCH_SIZE: f32 = 28.0;
const SWATCH_SPACING: f32 = 36.0;

fn palette_y() -> f32 {
    -WINDOW_HEIGHT * 0.5 + 40.0
}

fn palette_x(index: usize) -> f32 {
    let start_x = -(ALL_TERRAIN_KINDS.len() as f32 - 1.0) * SWATCH_SPACING * 0.5;
    start_x + index as f32 * SWATCH_SPACING
}

const SAVE_BUTTON_POS: Vec3 = Vec3::new(WINDOW_WIDTH * 0.5 - 66.0, WINDOW_HEIGHT * 0.5 - 30.0, 20.0);
const SAVE_BUTTON_SIZE: Vec2 = Vec2::new(108.0, 40.0);

#[derive(Component)]
struct EditorSelectionHighlight;

#[derive(Component)]
struct EditorSaveButton;

#[derive(Component)]
struct EditorHudText;

#[derive(Resource)]
struct EditorState {
    current_kind: TerrainKind,
    kinds: HashMap<(i32, i32), TerrainKind>,
    cells: Vec<(i32, i32, Vec2)>,
    layer_meshes: HashMap<TerrainKind, Handle<Mesh>>,
    painting: bool,
}

pub fn run_editor() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Terrain Map Editor".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, editor_setup)
        .add_systems(
            Update,
            (
                pan_camera,
                editor_select_kind_input,
                editor_click_input,
                editor_randomize_input,
                editor_update_hud,
            ),
        )
        .run();
}

fn positions_for_kind(
    cells: &[(i32, i32, Vec2)],
    kinds: &HashMap<(i32, i32), TerrainKind>,
    kind: TerrainKind,
) -> Vec<Vec2> {
    cells
        .iter()
        .filter(|&&(q, r, _)| kinds[&(q, r)] == kind)
        .map(|&(_, _, pos)| pos)
        .collect()
}

fn editor_setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let camera = commands.spawn((Camera2d, MainCamera)).id();

    let extent = Vec2::new(WINDOW_WIDTH * 5.0, WINDOW_HEIGHT * 5.0);
    let cells = hex_cells_in_bounds(extent);
    let overrides = load_terrain_overrides();
    let kinds: HashMap<(i32, i32), TerrainKind> = cells
        .iter()
        .map(|&(q, r, _)| {
            let kind = overrides.get(&(q, r)).copied().unwrap_or(DEFAULT_TERRAIN_KIND);
            ((q, r), kind)
        })
        .collect();

    let mut layer_meshes = HashMap::new();
    for &kind in &ALL_TERRAIN_KINDS {
        let positions = positions_for_kind(&cells, &kinds, kind);
        let mesh_handle = meshes.add(build_hex_fill_mesh(&positions, HEX_SIZE));
        commands.spawn((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(materials.add(kind.color())),
            Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        ));
        layer_meshes.insert(kind, mesh_handle);
    }

    let centers: Vec<Vec2> = cells.iter().map(|&(_, _, pos)| pos).collect();
    commands.spawn((
        Mesh2d(meshes.add(build_hex_ring_mesh(&centers, HEX_SIZE, HEX_SIZE - 2.0))),
        MeshMaterial2d(materials.add(Color::srgba(0.68, 0.76, 0.70, 0.16))),
        Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
    ));

    spawn_initial_path_preview(&mut commands, &mut meshes, &mut materials);

    for (index, &kind) in ALL_TERRAIN_KINDS.iter().enumerate() {
        let x = palette_x(index);
        let swatch = commands
            .spawn((
                Sprite::from_color(kind.color(), Vec2::splat(SWATCH_SIZE)),
                Transform::from_translation(Vec3::new(x, palette_y(), 20.0)),
            ))
            .id();
        let label = commands
            .spawn((
                Text2d::new((index + 1).to_string()),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.92, 0.92, 0.88)),
                TextShadow::default(),
                Transform::from_translation(Vec3::new(x, palette_y() - 24.0, 20.0)),
            ))
            .id();
        commands.entity(camera).add_child(swatch);
        commands.entity(camera).add_child(label);
    }

    let highlight = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.95, 0.85, 0.25), Vec2::splat(SWATCH_SIZE + 8.0)),
            Transform::from_translation(Vec3::new(palette_x(0), palette_y(), 19.0)),
            EditorSelectionHighlight,
        ))
        .id();
    commands.entity(camera).add_child(highlight);

    let save_button = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.20, 0.45, 0.22), SAVE_BUTTON_SIZE),
            Transform::from_translation(SAVE_BUTTON_POS),
            EditorSaveButton,
        ))
        .id();
    let save_label = commands
        .spawn((
            Text2d::new("SAVE"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.95, 0.98, 0.92)),
            TextShadow::default(),
            Transform::from_translation(SAVE_BUTTON_POS + Vec3::new(0.0, 0.0, 1.0)),
        ))
        .id();
    commands.entity(camera).add_child(save_button);
    commands.entity(camera).add_child(save_label);

    commands.spawn((
        Text::new(""),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::srgb(0.92, 0.94, 0.88)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
        EditorHudText,
    ));

    commands.insert_resource(EditorState {
        current_kind: DEFAULT_TERRAIN_KIND,
        kinds,
        cells,
        layer_meshes,
        painting: false,
    });
}

fn spawn_initial_path_preview(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>) {
    for &position in &INITIAL_PATH {
        commands.spawn((
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
            MeshMaterial2d(materials.add(Color::srgb(0.43, 0.39, 0.31))),
            Transform::from_translation(position.extend(-0.3)),
        ));
    }

    let start = INITIAL_PATH[0];
    let end = INITIAL_PATH[INITIAL_PATH.len() - 1];
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE + 6.0, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.35, 0.13, 0.12))),
        Transform::from_translation(start.extend(-0.2)),
    ));
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE + 9.0, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.12, 0.35, 0.36))),
        Transform::from_translation(end.extend(-0.2)),
    ));
}

fn select_palette_index(state: &mut EditorState, highlight_transform: &mut Transform, index: usize) {
    state.current_kind = ALL_TERRAIN_KINDS[index];
    highlight_transform.translation.x = palette_x(index);
}

fn editor_select_kind_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut highlight: Query<&mut Transform, With<EditorSelectionHighlight>>,
) {
    const KEYS: [KeyCode; 8] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
    ];
    let Ok(mut highlight_transform) = highlight.single_mut() else {
        return;
    };
    for (index, &key) in KEYS.iter().enumerate() {
        if keyboard.just_pressed(key) {
            select_palette_index(&mut state, &mut highlight_transform, index);
        }
    }
}

fn rebuild_layer_mesh(
    meshes: &mut Assets<Mesh>,
    layer_meshes: &HashMap<TerrainKind, Handle<Mesh>>,
    cells: &[(i32, i32, Vec2)],
    kinds: &HashMap<(i32, i32), TerrainKind>,
    kind: TerrainKind,
) {
    let Some(handle) = layer_meshes.get(&kind) else { return; };
    let positions = positions_for_kind(cells, kinds, kind);
    meshes.insert(handle, build_hex_fill_mesh(&positions, HEX_SIZE));
}

fn editor_click_input(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    save_button: Query<&GlobalTransform, With<EditorSaveButton>>,
    mut highlight: Query<&mut Transform, With<EditorSelectionHighlight>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut state: ResMut<EditorState>,
) {
    if mouse.just_released(MouseButton::Left) {
        state.painting = false;
    }

    let just_clicked = mouse.just_pressed(MouseButton::Left);
    if !(just_clicked || state.painting && mouse.pressed(MouseButton::Left)) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

    if just_clicked {
        if let Ok(button_transform) = save_button.single() {
            let pos = button_transform.translation().truncate();
            if (world_pos.x - pos.x).abs() <= SAVE_BUTTON_SIZE.x * 0.5
                && (world_pos.y - pos.y).abs() <= SAVE_BUTTON_SIZE.y * 0.5
            {
                save_terrain_overrides(&sparse_overrides(&state.kinds));
                return;
            }
        }

        for index in 0..ALL_TERRAIN_KINDS.len() {
            let x = palette_x(index);
            if (world_pos.x - x).abs() <= SWATCH_SIZE * 0.5 && (world_pos.y - palette_y()).abs() <= SWATCH_SIZE * 0.5 {
                if let Ok(mut highlight_transform) = highlight.single_mut() {
                    select_palette_index(&mut state, &mut highlight_transform, index);
                }
                return;
            }
        }

        state.painting = true;
    }

    let coord = world_to_axial_cell(world_pos);
    let Some(&old_kind) = state.kinds.get(&coord) else { return; };
    if old_kind == state.current_kind {
        return;
    }
    let new_kind = state.current_kind;
    state.kinds.insert(coord, new_kind);
    rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, old_kind);
    rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, new_kind);
}

fn editor_randomize_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut state: ResMut<EditorState>,
) {
    if !keyboard.just_pressed(KeyCode::KeyG) {
        return;
    }

    state.kinds = generate_random_kinds(&state.cells);
    for &kind in &ALL_TERRAIN_KINDS {
        rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, kind);
    }
}

fn sparse_overrides(kinds: &HashMap<(i32, i32), TerrainKind>) -> HashMap<(i32, i32), TerrainKind> {
    kinds
        .iter()
        .filter(|&(_, &kind)| kind != DEFAULT_TERRAIN_KIND)
        .map(|(&coord, &kind)| (coord, kind))
        .collect()
}

fn editor_update_hud(state: Res<EditorState>, mut hud: Query<&mut Text, With<EditorHudText>>) {
    let Ok(mut text) = hud.single_mut() else { return; };
    text.0 = format!(
        "Terrain Map Editor\n\
         WASD: pan  |  1-8 or click a swatch: pick tile type  |  click/drag: paint  |  G: randomize  |  click SAVE: write {TERRAIN_FILE_PATH}\n\n\
         Painting: {}",
        state.current_kind.name(),
    );
}
