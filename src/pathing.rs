use bevy::prelude::*;

use crate::constants::{HEX_SIZE, WINDOW_WIDTH};
use crate::paths::PathMap;

pub fn is_buildable_cell(position: Vec2, path_map: &PathMap) -> bool {
    is_in_play_area(position) && !path_map.contains_world(position)
}

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

pub fn snap_to_grid(position: Vec2) -> Vec2 {
    let (q, r) = world_to_axial(position);
    let (rq, rr) = round_axial(q, r);
    axial_to_world(rq as f32, rr as f32)
}

pub fn world_to_axial_cell(position: Vec2) -> (i32, i32) {
    let (q, r) = world_to_axial(position);
    round_axial(q, r)
}

fn world_to_axial(position: Vec2) -> (f32, f32) {
    let q = (3f32.sqrt() / 3.0 * position.x - 1.0 / 3.0 * position.y) / HEX_SIZE;
    let r = (2.0 / 3.0 * position.y) / HEX_SIZE;
    (q, r)
}

pub fn axial_to_world(q: f32, r: f32) -> Vec2 {
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

    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff <= z_diff {
        rz = -rx - ry;
    }

    (rx as i32, rz as i32)
}
