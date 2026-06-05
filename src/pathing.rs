use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{PathEndMarker, PathTile};
use crate::constants::{
    GRID_SIZE, HUD_BUILD_LIMIT, PATH_HALF_WIDTH, SHOP_BUILD_LIMIT, WINDOW_WIDTH,
};
use crate::effects::spawn_floating_text;
use crate::resources::{GameOver, Money, PathTiles};

pub fn is_buildable_cell(position: Vec2, path_tiles: &PathTiles) -> bool {
    is_in_play_area(position) && !path_tiles.contains(position)
}

pub fn update_path_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    game_over: Res<GameOver>,
    mut money: ResMut<Money>,
    mut path_tiles: ResMut<PathTiles>,
    mut end_marker: Query<&mut Transform, With<PathEndMarker>>,
) {
    if game_over.value || !mouse.just_pressed(MouseButton::Left) {
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

    let cost = path_tiles.extension_cost();
    if money.amount < cost {
        return;
    }

    money.amount -= cost;
    path_tiles.extend_to(grid_position);
    spawn_path_tile(&mut commands, grid_position);

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

pub fn spawn_path_tile(commands: &mut Commands, position: Vec2) {
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.43, 0.39, 0.31),
            Vec2::splat(PATH_HALF_WIDTH * 2.0),
        ),
        Transform::from_translation(position.extend(-2.0)),
        PathTile,
    ));
}

fn is_in_play_area(position: Vec2) -> bool {
    let half_cell = GRID_SIZE * 0.5;
    position.x >= -WINDOW_WIDTH * 0.5 + half_cell
        && position.x <= WINDOW_WIDTH * 0.5 - half_cell
        && position.y >= SHOP_BUILD_LIMIT
        && position.y <= HUD_BUILD_LIMIT
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
