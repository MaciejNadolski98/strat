use std::collections::HashMap;

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy::window::PrimaryWindow;

use crate::components::MainCamera;
use crate::constants::{BACKGROUND_COLOR, BACKGROUND_Z, HEX_SIZE, PATH_EDGE_COLOR, PATH_FILLED_Z, PATH_FILL_COLOR, PATH_LINE_COLOR, PATH_LINE_Z, TERRAIN_Z, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::game::pan_camera;
use crate::pathing::{hex_cells_in_bounds, world_to_axial_cell};
use crate::paths::{PathDefinition, PathId, PathVisual, are_hex_adjacent, despawn_path_visuals, spawn_path_filled, spawn_path_line, validate_paths};
use crate::regions::RegionDefinition;
use crate::setup::spawn_grid;
use crate::terrain::{
    build_hex_fill_mesh, generate_random_kinds, TerrainKind, ALL_TERRAIN_KINDS, DEFAULT_TERRAIN_KIND,
};
use crate::terrain_file::{load_terrain_file, save_terrain_file, TERRAIN_FILE_PATH};

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

const MODE_BUTTON_POS: Vec3 = Vec3::new(WINDOW_WIDTH * 0.5 - 190.0, WINDOW_HEIGHT * 0.5 - 30.0, 20.0);
const MODE_BUTTON_SIZE: Vec2 = Vec2::new(128.0, 40.0);

const REGION_LIST_X: f32 = -WINDOW_WIDTH * 0.5 + 90.0;
const REGION_LIST_START_Y: f32 = WINDOW_HEIGHT * 0.5 - 80.0;
const REGION_LIST_SPACING: f32 = 32.0;
const REGION_SLOT_WIDTH: f32 = 160.0;
const REGION_SLOT_HEIGHT: f32 = 26.0;

const NEW_REGION_BUTTON_Y_OFFSET: f32 = 10.0;

const REGION_OVERLAY_COLOR: Color = Color::srgba(0.9, 0.75, 0.2, 0.55);
const UNASSIGNED_OVERLAY_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorMode {
    Terrain,
    Regions,
    Paths,
}

impl EditorMode {
    fn label(self) -> &'static str {
        match self {
            Self::Terrain => "Terrain",
            Self::Regions => "Regions",
            Self::Paths => "Paths",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Terrain => Self::Regions,
            Self::Regions => Self::Paths,
            Self::Paths => Self::Terrain,
        }
    }
}

#[derive(Component)]
struct EditorSelectionHighlight;

#[derive(Component)]
struct EditorSaveButton;

#[derive(Component)]
struct EditorModeButton;

#[derive(Component)]
struct EditorModeLabel;

#[derive(Component)]
struct EditorHudText;

#[derive(Component)]
struct TerrainPaletteElement;

#[derive(Component)]
struct RegionListElement;

#[derive(Component)]
struct RegionOverlay;

#[derive(Component)]
struct PathListElement;

#[derive(Component)]
struct PathOverlay;

#[derive(Component)]
struct DragRectVisual;

#[derive(Resource)]
struct EditorState {
    mode: EditorMode,
    current_kind: TerrainKind,
    palette_index: usize,
    kinds: HashMap<(i32, i32), TerrainKind>,
    cells: Vec<(i32, i32, Vec2)>,
    layer_meshes: HashMap<TerrainKind, Handle<Mesh>>,
    regions: Vec<RegionDefinition>,
    paths: Vec<PathDefinition>,
    selected_region: Option<usize>,
    selected_path: Option<usize>,
    naming_region: bool,
    naming_path: bool,
    region_name_buffer: String,
    path_name_buffer: String,
    path_errors: Vec<String>,
    consumed_click: bool,
    drag_start: Option<Vec2>,
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
                editor_reset_consumed_click,
                editor_mode_toggle.after(editor_reset_consumed_click),
                editor_select_kind_input,
                editor_terrain_click.after(editor_mode_toggle),
                editor_region_click.after(editor_mode_toggle),
                editor_path_click.after(editor_mode_toggle),
                editor_region_name_input,
                editor_path_name_input,
                editor_drag_select.after(editor_terrain_click).after(editor_region_click),
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

fn cell_position(cells: &[(i32, i32, Vec2)], coord: (i32, i32)) -> Option<Vec2> {
    cells.iter().find(|&&(q, r, _)| (q, r) == coord).map(|&(_, _, pos)| pos)
}

fn editor_setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let camera = commands.spawn((Camera2d, MainCamera)).id();

    let bg_size = Vec2::new(WINDOW_WIDTH * 5.0, WINDOW_HEIGHT * 5.0);
    commands.spawn((
        Mesh2d(meshes.add(bevy::math::primitives::Rectangle::new(bg_size.x, bg_size.y))),
        MeshMaterial2d(materials.add(BACKGROUND_COLOR)),
        Transform::from_translation(Vec3::new(0.0, 0.0, BACKGROUND_Z)),
    ));

    let extent = Vec2::new(WINDOW_WIDTH * 5.0, WINDOW_HEIGHT * 5.0);
    let cells = hex_cells_in_bounds(extent);
    let data = load_terrain_file();
    let kinds: HashMap<(i32, i32), TerrainKind> = cells
        .iter()
        .map(|&(q, r, _)| {
            let kind = data.overrides.get(&(q, r)).copied().unwrap_or(DEFAULT_TERRAIN_KIND);
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
            Transform::from_translation(Vec3::new(0.0, 0.0, TERRAIN_Z)),
            NoFrustumCulling,
        ));
        layer_meshes.insert(kind, mesh_handle);
    }

    let centers: Vec<Vec2> = cells.iter().map(|&(_, _, pos)| pos).collect();
    spawn_grid(&mut commands, &mut meshes, &mut materials, &centers);

    spawn_editor_path_lines(&mut commands, &mut meshes, &mut materials, &data.paths, None, &cells);

    for (index, &kind) in ALL_TERRAIN_KINDS.iter().enumerate() {
        let x = palette_x(index);
        let swatch = commands
            .spawn((
                Sprite::from_color(kind.color(), Vec2::splat(SWATCH_SIZE)),
                Transform::from_translation(Vec3::new(x, palette_y(), 20.0)),
                TerrainPaletteElement,
            ))
            .id();
        commands.entity(camera).add_child(swatch);
    }

    let highlight = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.95, 0.85, 0.25), Vec2::splat(SWATCH_SIZE + 8.0)),
            Transform::from_translation(Vec3::new(palette_x(0), palette_y(), 19.0)),
            EditorSelectionHighlight,
            TerrainPaletteElement,
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

    let mode_button = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.28, 0.30, 0.42), MODE_BUTTON_SIZE),
            Transform::from_translation(MODE_BUTTON_POS),
            EditorModeButton,
        ))
        .id();
    let mode_label = commands
        .spawn((
            Text2d::new("Terrain"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.92, 0.92, 0.88)),
            TextShadow::default(),
            Transform::from_translation(MODE_BUTTON_POS + Vec3::new(0.0, 0.0, 1.0)),
            EditorModeLabel,
        ))
        .id();
    commands.entity(camera).add_child(mode_button);
    commands.entity(camera).add_child(mode_label);

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
        mode: EditorMode::Terrain,
        current_kind: DEFAULT_TERRAIN_KIND,
        palette_index: 0,
        kinds,
        cells,
        layer_meshes,
        regions: data.regions,
        path_errors: {
            let errs = validate_paths(&data.paths);
            errs.iter().map(|e| e.message(&data.paths)).collect()
        },
        paths: data.paths,
        selected_region: None,
        selected_path: None,
        naming_region: false,
        naming_path: false,
        region_name_buffer: String::new(),
        path_name_buffer: String::new(),
        consumed_click: false,
        drag_start: None,
    });
}


fn select_palette_index(state: &mut EditorState, highlight_transform: &mut Transform, index: usize) {
    state.palette_index = index;
    state.current_kind = ALL_TERRAIN_KINDS[index];
    highlight_transform.translation.x = palette_x(index);
}

fn editor_reset_consumed_click(mut state: ResMut<EditorState>) {
    state.consumed_click = false;
}

fn editor_mode_toggle(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut mode_label: Query<&mut Text2d, With<EditorModeLabel>>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    palette_elements: Query<Entity, With<TerrainPaletteElement>>,
    region_elements: Query<Entity, With<RegionListElement>>,
    region_overlays: Query<Entity, With<RegionOverlay>>,
    path_elements: Query<Entity, With<PathListElement>>,
    path_overlays: Query<Entity, With<PathOverlay>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    let mut toggled = keyboard.just_pressed(KeyCode::Tab);

    if !toggled && mouse.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return; };
        let Ok((cam, cam_transform)) = camera.single() else { return; };
        let Some(cursor_pos) = window.cursor_position() else { return; };
        let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        let local_pos = world_pos - cam_transform.translation().truncate();

        let btn = MODE_BUTTON_POS.truncate();
        if (local_pos.x - btn.x).abs() <= MODE_BUTTON_SIZE.x * 0.5
            && (local_pos.y - btn.y).abs() <= MODE_BUTTON_SIZE.y * 0.5
        {
            toggled = true;
        }
    }

    if !toggled {
        return;
    }

    state.consumed_click = true;
    state.mode = state.mode.next();
    state.naming_region = false;
    state.naming_path = false;
    state.region_name_buffer.clear();
    state.path_name_buffer.clear();
    if state.mode != EditorMode::Regions {
        state.selected_region = None;
    }
    if state.mode != EditorMode::Paths {
        state.selected_path = None;
    }

    if let Ok(mut label) = mode_label.single_mut() {
        label.0 = state.mode.label().to_string();
    }

    let Ok(camera_entity) = main_camera.single() else { return; };

    for entity in &palette_elements {
        let vis = if state.mode == EditorMode::Terrain { Visibility::Inherited } else { Visibility::Hidden };
        commands.entity(entity).insert(vis);
    }

    for entity in &region_elements {
        commands.entity(entity).despawn();
    }
    for entity in &region_overlays {
        commands.entity(entity).despawn();
    }
    for entity in &path_elements {
        commands.entity(entity).despawn();
    }
    for entity in &path_overlays {
        commands.entity(entity).despawn();
    }

    if state.mode == EditorMode::Regions {
        spawn_region_list(&mut commands, &mut meshes, &mut materials, &state, camera_entity);
    }
    if state.mode == EditorMode::Paths {
        spawn_path_list(&mut commands, &mut meshes, &mut materials, &state, camera_entity);
    }
}

fn spawn_region_list(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &EditorState,
    camera_entity: Entity,
) {
    for (index, region) in state.regions.iter().enumerate() {
        let y = REGION_LIST_START_Y - index as f32 * REGION_LIST_SPACING;
        let selected = state.selected_region == Some(index);
        let bg_color = if selected {
            Color::srgb(0.35, 0.38, 0.50)
        } else {
            Color::srgb(0.18, 0.20, 0.22)
        };

        let slot = commands
            .spawn((
                Sprite::from_color(bg_color, Vec2::new(REGION_SLOT_WIDTH, REGION_SLOT_HEIGHT)),
                Transform::from_translation(Vec3::new(REGION_LIST_X, y, 20.0)),
                RegionListElement,
            ))
            .id();
        let label = commands
            .spawn((
                Text2d::new(&region.name),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.92, 0.92, 0.86)),
                TextShadow::default(),
                Transform::from_translation(Vec3::new(REGION_LIST_X, y, 21.0)),
                RegionListElement,
            ))
            .id();
        commands.entity(camera_entity).add_child(slot);
        commands.entity(camera_entity).add_child(label);

        if selected {
            let positions: Vec<Vec2> = region.tiles.iter()
                .filter_map(|&coord| cell_position(&state.cells, coord))
                .collect();
            if !positions.is_empty() {
                commands.spawn((
                    Mesh2d(meshes.add(build_hex_fill_mesh(&positions, HEX_SIZE))),
                    MeshMaterial2d(materials.add(REGION_OVERLAY_COLOR)),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.5)),
                    RegionOverlay,
                ));
            }
        }
    }

    let new_y = REGION_LIST_START_Y - state.regions.len() as f32 * REGION_LIST_SPACING - NEW_REGION_BUTTON_Y_OFFSET;
    let new_btn = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.22, 0.40, 0.25), Vec2::new(REGION_SLOT_WIDTH, REGION_SLOT_HEIGHT)),
            Transform::from_translation(Vec3::new(REGION_LIST_X, new_y, 20.0)),
            RegionListElement,
        ))
        .id();
    let new_label = commands
        .spawn((
            Text2d::new("+ New Region"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.88, 0.95, 0.88)),
            TextShadow::default(),
            Transform::from_translation(Vec3::new(REGION_LIST_X, new_y, 21.0)),
            RegionListElement,
        ))
        .id();
    commands.entity(camera_entity).add_child(new_btn);
    commands.entity(camera_entity).add_child(new_label);

    let assigned: std::collections::HashSet<(i32, i32)> = state.regions.iter()
        .flat_map(|r| r.tiles.iter().copied())
        .collect();
    let unassigned: Vec<Vec2> = state.cells.iter()
        .filter(|&&(q, r, _)| !assigned.contains(&(q, r)))
        .map(|&(_, _, pos)| pos)
        .collect();
    if !unassigned.is_empty() {
        commands.spawn((
            Mesh2d(meshes.add(build_hex_fill_mesh(&unassigned, HEX_SIZE))),
            MeshMaterial2d(materials.add(UNASSIGNED_OVERLAY_COLOR)),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.3)),
            RegionOverlay,
        ));
    }
}

fn rebuild_region_ui(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &EditorState,
    region_elements: &Query<Entity, With<RegionListElement>>,
    region_overlays: &Query<Entity, With<RegionOverlay>>,
    camera_entity: Entity,
) {
    for entity in region_elements.iter() {
        commands.entity(entity).despawn();
    }
    for entity in region_overlays.iter() {
        commands.entity(entity).despawn();
    }
    spawn_region_list(commands, meshes, materials, state, camera_entity);
}


fn spawn_editor_path_lines(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    paths: &[PathDefinition],
    skip: Option<usize>,
    cells: &[(i32, i32, Vec2)],
) {
    for (i, path) in paths.iter().enumerate() {
        if skip == Some(i) {
            continue;
        }
        let world_tiles: Vec<Vec2> = path.tiles.iter()
            .filter_map(|&coord| cell_position(cells, coord))
            .collect();
        spawn_path_line(commands, meshes, materials, &world_tiles, PATH_LINE_COLOR, PATH_LINE_Z);
    }
}


fn spawn_path_list(
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    state: &EditorState,
    camera_entity: Entity,
) {
    for (index, path) in state.paths.iter().enumerate() {
        let y = REGION_LIST_START_Y - index as f32 * REGION_LIST_SPACING;
        let selected = state.selected_path == Some(index);
        let bg_color = if selected {
            Color::srgb(0.30, 0.42, 0.50)
        } else {
            Color::srgb(0.18, 0.20, 0.22)
        };

        let slot = commands
            .spawn((
                Sprite::from_color(bg_color, Vec2::new(REGION_SLOT_WIDTH, REGION_SLOT_HEIGHT)),
                Transform::from_translation(Vec3::new(REGION_LIST_X, y, 20.0)),
                PathListElement,
            ))
            .id();
        let label_text = path.name.clone();
        let label = commands
            .spawn((
                Text2d::new(label_text),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.92, 0.92, 0.86)),
                TextShadow::default(),
                Transform::from_translation(Vec3::new(REGION_LIST_X, y, 21.0)),
                PathListElement,
            ))
            .id();
        commands.entity(camera_entity).add_child(slot);
        commands.entity(camera_entity).add_child(label);
    }

    let new_y = REGION_LIST_START_Y - state.paths.len() as f32 * REGION_LIST_SPACING - NEW_REGION_BUTTON_Y_OFFSET;
    let new_btn = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.22, 0.30, 0.45), Vec2::new(REGION_SLOT_WIDTH, REGION_SLOT_HEIGHT)),
            Transform::from_translation(Vec3::new(REGION_LIST_X, new_y, 20.0)),
            PathListElement,
        ))
        .id();
    let new_label = commands
        .spawn((
            Text2d::new("+ New Path"),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.88, 0.90, 0.98)),
            TextShadow::default(),
            Transform::from_translation(Vec3::new(REGION_LIST_X, new_y, 21.0)),
            PathListElement,
        ))
        .id();
    commands.entity(camera_entity).add_child(new_btn);
    commands.entity(camera_entity).add_child(new_label);
}

fn rebuild_path_ui(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    state: &mut EditorState,
    path_elements: &Query<Entity, With<PathListElement>>,
    path_overlays: &Query<Entity, With<PathOverlay>>,
    path_visuals: &Query<Entity, With<PathVisual>>,
    camera_entity: Entity,
) {
    for entity in path_elements.iter() {
        commands.entity(entity).despawn();
    }
    for entity in path_overlays.iter() {
        commands.entity(entity).despawn();
    }
    despawn_path_visuals(commands, path_visuals);

    spawn_editor_path_lines(commands, meshes, materials, &state.paths, state.selected_path, &state.cells);

    if let Some(selected) = state.selected_path {
        if selected < state.paths.len() {
            let positions: Vec<Vec2> = state.paths[selected].tiles.iter()
                .filter_map(|&coord| cell_position(&state.cells, coord))
                .collect();
            spawn_path_filled(commands, meshes, materials, &positions, PATH_FILL_COLOR, PATH_EDGE_COLOR, PATH_FILLED_Z);
        }
    }

    let errors = validate_paths(&state.paths);
    state.path_errors = errors.iter().map(|e| e.message(&state.paths)).collect();

    spawn_path_list(commands, meshes, materials, state, camera_entity);
}

fn editor_path_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    path_elements: Query<Entity, With<PathListElement>>,
    path_overlays: Query<Entity, With<PathOverlay>>,
    path_visuals: Query<Entity, With<PathVisual>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    if state.mode != EditorMode::Paths || state.naming_path || state.consumed_click {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
    let Ok(camera_entity) = main_camera.single() else { return; };
    let local_pos = world_pos - cam_transform.translation().truncate();

    let save_local = SAVE_BUTTON_POS.truncate();
    if (local_pos.x - save_local.x).abs() <= SAVE_BUTTON_SIZE.x * 0.5
        && (local_pos.y - save_local.y).abs() <= SAVE_BUTTON_SIZE.y * 0.5
    {
        save_terrain_file(&sparse_overrides(&state.kinds), &state.regions, &state.paths);
        return;
    }

    for index in 0..state.paths.len() {
        let y = REGION_LIST_START_Y - index as f32 * REGION_LIST_SPACING;
        if (local_pos.x - REGION_LIST_X).abs() <= REGION_SLOT_WIDTH * 0.5
            && (local_pos.y - y).abs() <= REGION_SLOT_HEIGHT * 0.5
        {
            if state.selected_path == Some(index) {
                state.selected_path = None;
            } else {
                state.selected_path = Some(index);
            }
            rebuild_path_ui(&mut commands, &mut meshes, &mut materials, &mut state, &path_elements, &path_overlays, &path_visuals, camera_entity);
            return;
        }
    }

    let new_y = REGION_LIST_START_Y - state.paths.len() as f32 * REGION_LIST_SPACING - NEW_REGION_BUTTON_Y_OFFSET;
    if (local_pos.x - REGION_LIST_X).abs() <= REGION_SLOT_WIDTH * 0.5
        && (local_pos.y - new_y).abs() <= REGION_SLOT_HEIGHT * 0.5
    {
        state.naming_path = true;
        state.path_name_buffer.clear();
        return;
    }

    let Some(selected) = state.selected_path else { return; };
    let coord = world_to_axial_cell(world_pos);
    if !state.kinds.contains_key(&coord) {
        return;
    }

    let path = &mut state.paths[selected];
    if let Some(pos) = path.tiles.iter().position(|&c| c == coord) {
        path.tiles.remove(pos);
    } else if path.tiles.is_empty() {
        path.tiles.push(coord);
    } else if are_hex_adjacent(*path.tiles.last().unwrap(), coord) {
        path.tiles.push(coord);
    } else if are_hex_adjacent(*path.tiles.first().unwrap(), coord) {
        path.tiles.insert(0, coord);
    } else {
        return;
    }

    rebuild_path_ui(&mut commands, &mut meshes, &mut materials, &mut state, &path_elements, &path_overlays, &path_visuals, camera_entity);
}

fn editor_path_name_input(
    mut key_events: EventReader<KeyboardInput>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    path_elements: Query<Entity, With<PathListElement>>,
    path_overlays: Query<Entity, With<PathOverlay>>,
    path_visuals: Query<Entity, With<PathVisual>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    if !state.naming_path {
        return;
    }

    for ev in key_events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(s) => {
                state.path_name_buffer.push_str(s);
            }
            Key::Space => {
                state.path_name_buffer.push(' ');
            }
            Key::Backspace => {
                state.path_name_buffer.pop();
            }
            Key::Escape => {
                state.naming_path = false;
                state.path_name_buffer.clear();
                return;
            }
            Key::Enter => {
                let name = state.path_name_buffer.trim().to_string();
                if !name.is_empty() {
                    let new_index = state.paths.len();
                    state.paths.push(PathDefinition {
                        id: PathId(new_index),
                        name,
                        tiles: Vec::new(),
                        enemies: Vec::new(),
                    });
                    state.selected_path = Some(new_index);
                }
                state.naming_path = false;
                state.path_name_buffer.clear();

                let Ok(camera_entity) = main_camera.single() else { return; };
                rebuild_path_ui(&mut commands, &mut meshes, &mut materials, &mut state, &path_elements, &path_overlays, &path_visuals, camera_entity);
                return;
            }
            _ => {}
        }
    }
}

fn editor_select_kind_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<EditorState>,
    mut highlight: Query<&mut Transform, With<EditorSelectionHighlight>>,
) {
    if state.mode != EditorMode::Terrain {
        return;
    }

    let count = ALL_TERRAIN_KINDS.len();
    let new_index = if keyboard.just_pressed(KeyCode::ArrowRight) {
        Some((state.palette_index + 1) % count)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        Some((state.palette_index + count - 1) % count)
    } else {
        None
    };

    if let Some(index) = new_index {
        let Ok(mut highlight_transform) = highlight.single_mut() else {
            return;
        };
        select_palette_index(&mut state, &mut highlight_transform, index);
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

fn editor_terrain_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut highlight: Query<&mut Transform, With<EditorSelectionHighlight>>,
    mut state: ResMut<EditorState>,
) {
    if state.mode != EditorMode::Terrain || state.consumed_click {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
    let local_pos = world_pos - cam_transform.translation().truncate();

    let save_local = SAVE_BUTTON_POS.truncate();
    if (local_pos.x - save_local.x).abs() <= SAVE_BUTTON_SIZE.x * 0.5
        && (local_pos.y - save_local.y).abs() <= SAVE_BUTTON_SIZE.y * 0.5
    {
        save_terrain_file(&sparse_overrides(&state.kinds), &state.regions, &state.paths);
        state.consumed_click = true;
        return;
    }

    for index in 0..ALL_TERRAIN_KINDS.len() {
        let x = palette_x(index);
        if (local_pos.x - x).abs() <= SWATCH_SIZE * 0.5 && (local_pos.y - palette_y()).abs() <= SWATCH_SIZE * 0.5 {
            if let Ok(mut highlight_transform) = highlight.single_mut() {
                select_palette_index(&mut state, &mut highlight_transform, index);
            }
            state.consumed_click = true;
            return;
        }
    }
}

fn editor_region_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    region_elements: Query<Entity, With<RegionListElement>>,
    region_overlays: Query<Entity, With<RegionOverlay>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    if state.mode != EditorMode::Regions || state.naming_region || state.consumed_click {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
    let Ok(camera_entity) = main_camera.single() else { return; };
    let local_pos = world_pos - cam_transform.translation().truncate();

    let save_local = SAVE_BUTTON_POS.truncate();
    if (local_pos.x - save_local.x).abs() <= SAVE_BUTTON_SIZE.x * 0.5
        && (local_pos.y - save_local.y).abs() <= SAVE_BUTTON_SIZE.y * 0.5
    {
        save_terrain_file(&sparse_overrides(&state.kinds), &state.regions, &state.paths);
        state.consumed_click = true;
        return;
    }

    for index in 0..state.regions.len() {
        let y = REGION_LIST_START_Y - index as f32 * REGION_LIST_SPACING;
        if (local_pos.x - REGION_LIST_X).abs() <= REGION_SLOT_WIDTH * 0.5
            && (local_pos.y - y).abs() <= REGION_SLOT_HEIGHT * 0.5
        {
            if state.selected_region == Some(index) {
                state.selected_region = None;
            } else {
                state.selected_region = Some(index);
            }
            rebuild_region_ui(&mut commands, &mut meshes, &mut materials, &state, &region_elements, &region_overlays, camera_entity);
            state.consumed_click = true;
            return;
        }
    }

    let new_y = REGION_LIST_START_Y - state.regions.len() as f32 * REGION_LIST_SPACING - NEW_REGION_BUTTON_Y_OFFSET;
    if (local_pos.x - REGION_LIST_X).abs() <= REGION_SLOT_WIDTH * 0.5
        && (local_pos.y - new_y).abs() <= REGION_SLOT_HEIGHT * 0.5
    {
        state.naming_region = true;
        state.region_name_buffer.clear();
        state.consumed_click = true;
        return;
    }
}

fn editor_region_name_input(
    mut key_events: EventReader<KeyboardInput>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    region_elements: Query<Entity, With<RegionListElement>>,
    region_overlays: Query<Entity, With<RegionOverlay>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    if !state.naming_region {
        return;
    }

    for ev in key_events.read() {
        if !ev.state.is_pressed() {
            continue;
        }
        match &ev.logical_key {
            Key::Character(s) => {
                state.region_name_buffer.push_str(s);
            }
            Key::Space => {
                state.region_name_buffer.push(' ');
            }
            Key::Backspace => {
                state.region_name_buffer.pop();
            }
            Key::Escape => {
                state.naming_region = false;
                state.region_name_buffer.clear();
                return;
            }
            Key::Enter => {
                let name = state.region_name_buffer.trim().to_string();
                if !name.is_empty() {
                    let new_index = state.regions.len();
                    state.regions.push(RegionDefinition {
                        name,
                        tiles: Vec::new(),
                    });
                    state.selected_region = Some(new_index);
                }
                state.naming_region = false;
                state.region_name_buffer.clear();

                let Ok(camera_entity) = main_camera.single() else { return; };
                rebuild_region_ui(&mut commands, &mut meshes, &mut materials, &state, &region_elements, &region_overlays, camera_entity);
                return;
            }
            _ => {}
        }
    }
}

fn editor_randomize_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut state: ResMut<EditorState>,
) {
    if state.mode != EditorMode::Terrain {
        return;
    }
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

fn hex_tiles_in_rect(
    cells: &[(i32, i32, Vec2)],
    corner_a: Vec2,
    corner_b: Vec2,
) -> Vec<(i32, i32)> {
    let min_x = corner_a.x.min(corner_b.x);
    let max_x = corner_a.x.max(corner_b.x);
    let min_y = corner_a.y.min(corner_b.y);
    let max_y = corner_a.y.max(corner_b.y);
    cells
        .iter()
        .filter(|&&(_, _, pos)| pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y)
        .map(|&(q, r, _)| (q, r))
        .collect()
}

const DRAG_THRESHOLD: f32 = 10.0;

fn editor_drag_select(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut state: ResMut<EditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    drag_visuals: Query<Entity, With<DragRectVisual>>,
    region_elements: Query<Entity, With<RegionListElement>>,
    region_overlays: Query<Entity, With<RegionOverlay>>,
    main_camera: Query<Entity, With<MainCamera>>,
) {
    if !matches!(state.mode, EditorMode::Terrain | EditorMode::Regions) {
        return;
    }

    for entity in &drag_visuals {
        commands.entity(entity).despawn();
    }

    if mouse.just_pressed(MouseButton::Left) && !state.consumed_click {
        let Ok(window) = windows.single() else { return; };
        let Ok((cam, cam_transform)) = camera.single() else { return; };
        let Some(cursor_pos) = window.cursor_position() else { return; };
        let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };
        state.drag_start = Some(world_pos);
        return;
    }

    let Some(start) = state.drag_start else { return; };

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

    if mouse.pressed(MouseButton::Left) {
        let dist = start.distance(world_pos);
        if dist > DRAG_THRESHOLD {
            let center = (start + world_pos) * 0.5;
            let size = Vec2::new((world_pos.x - start.x).abs(), (world_pos.y - start.y).abs());
            commands.spawn((
                Sprite::from_color(Color::srgba(0.9, 0.85, 0.3, 0.2), size),
                Transform::from_translation(center.extend(5.0)),
                DragRectVisual,
            ));
        }
        return;
    }

    if mouse.just_released(MouseButton::Left) {
        state.drag_start = None;
        let dist = start.distance(world_pos);
        let is_drag = dist > DRAG_THRESHOLD;

        match state.mode {
            EditorMode::Terrain => {
                if is_drag {
                    let tiles = hex_tiles_in_rect(&state.cells, start, world_pos);
                    let mut changed_kinds = std::collections::HashSet::new();
                    let new_kind = state.current_kind;
                    for &coord in &tiles {
                        if let Some(&old) = state.kinds.get(&coord) {
                            if old != new_kind {
                                changed_kinds.insert(old);
                                state.kinds.insert(coord, new_kind);
                            }
                        }
                    }
                    changed_kinds.insert(new_kind);
                    for kind in changed_kinds {
                        rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, kind);
                    }
                } else {
                    let coord = world_to_axial_cell(world_pos);
                    if let Some(&old_kind) = state.kinds.get(&coord) {
                        let new_kind = state.current_kind;
                        if old_kind != new_kind {
                            state.kinds.insert(coord, new_kind);
                            rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, old_kind);
                            rebuild_layer_mesh(&mut meshes, &state.layer_meshes, &state.cells, &state.kinds, new_kind);
                        }
                    }
                }
            }
            EditorMode::Regions => {
                let Some(selected) = state.selected_region else { return; };
                if is_drag {
                    let tiles = hex_tiles_in_rect(&state.cells, start, world_pos);
                    let valid_tiles: Vec<(i32, i32)> = tiles
                        .into_iter()
                        .filter(|c| state.kinds.contains_key(c))
                        .collect();
                    let region = &mut state.regions[selected];
                    for coord in valid_tiles {
                        if !region.tiles.contains(&coord) {
                            region.tiles.push(coord);
                        }
                    }
                } else {
                    let coord = world_to_axial_cell(world_pos);
                    if state.kinds.contains_key(&coord) {
                        let region = &mut state.regions[selected];
                        if let Some(pos) = region.tiles.iter().position(|&c| c == coord) {
                            region.tiles.remove(pos);
                        } else {
                            region.tiles.push(coord);
                        }
                    }
                }
                let Ok(camera_entity) = main_camera.single() else { return; };
                rebuild_region_ui(&mut commands, &mut meshes, &mut materials, &state, &region_elements, &region_overlays, camera_entity);
            }
            _ => {}
        }
    }
}

fn editor_update_hud(state: Res<EditorState>, mut hud: Query<&mut Text, With<EditorHudText>>) {
    let Ok(mut text) = hud.single_mut() else { return; };

    if state.naming_region {
        text.0 = format!(
            "Terrain Map Editor — Regions\n\
             Type region name: {}_\n\
             Enter: confirm  |  Esc: cancel",
            state.region_name_buffer,
        );
        return;
    }

    if state.naming_path {
        text.0 = format!(
            "Terrain Map Editor — Paths\n\
             Type path name: {}_\n\
             Enter: confirm  |  Esc: cancel",
            state.path_name_buffer,
        );
        return;
    }

    match state.mode {
        EditorMode::Terrain => {
            text.0 = format!(
                "Terrain Map Editor — Terrain\n\
                 WASD: pan  |  arrows or click: pick tile  |  click: paint  |  drag: fill area  |  G: randomize  |  Tab: switch mode  |  SAVE: write {TERRAIN_FILE_PATH}\n\n\
                 Painting: {}",
                state.current_kind.name(),
            );
        }
        EditorMode::Regions => {
            let region_info = match state.selected_region {
                Some(i) => format!("Selected: {} ({} tiles)", state.regions[i].name, state.regions[i].tiles.len()),
                None => "No region selected".to_string(),
            };
            text.0 = format!(
                "Terrain Map Editor — Regions\n\
                 WASD: pan  |  click list: select region  |  click: toggle tile  |  drag: add area  |  Tab: switch mode  |  SAVE: write {TERRAIN_FILE_PATH}\n\n\
                 {region_info}",
            );
        }
        EditorMode::Paths => {
            let path_info = match state.selected_path {
                Some(i) => {
                    let p = &state.paths[i];
                    format!("Selected: {} ({} tiles, {} enemy groups)", p.name, p.tiles.len(), p.enemies.len())
                }
                None => "No path selected".to_string(),
            };
            let error_info = if state.path_errors.is_empty() {
                String::new()
            } else {
                format!("\n\nErrors (save blocked):\n{}", state.path_errors.join("\n"))
            };
            text.0 = format!(
                "Terrain Map Editor — Paths\n\
                 WASD: pan  |  click list: select path  |  click map: toggle tile  |  Tab: switch mode  |  SAVE: write {TERRAIN_FILE_PATH}\n\n\
                 {path_info}{error_info}",
            );
        }
    }
}
