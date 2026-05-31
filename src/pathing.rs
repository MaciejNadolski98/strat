use bevy::prelude::*;

use crate::constants::{
    GRID_SIZE, HUD_BUILD_LIMIT, PATH, PATH_HALF_WIDTH, SHOP_BUILD_LIMIT, WINDOW_WIDTH,
};

pub fn is_buildable_cell(position: Vec2) -> bool {
    let half_cell = GRID_SIZE * 0.5;
    position.x >= -WINDOW_WIDTH * 0.5 + half_cell
        && position.x <= WINDOW_WIDTH * 0.5 - half_cell
        && position.y >= SHOP_BUILD_LIMIT
        && position.y <= HUD_BUILD_LIMIT
        && !is_on_path(position)
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

fn is_on_path(position: Vec2) -> bool {
    PATH.windows(2).any(|segment| {
        distance_to_segment(position, segment[0], segment[1]) < PATH_HALF_WIDTH + 18.0
    })
}

fn snap_to_cell_center(value: f32) -> f32 {
    (value / GRID_SIZE).floor() * GRID_SIZE + GRID_SIZE * 0.5
}

fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared == 0.0 {
        return point.distance(start);
    }

    let t = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}
