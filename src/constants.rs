use bevy::prelude::*;

pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 700.0;
pub const TOWER_COST: i32 = 40;
pub const STARTING_MONEY: i32 = 120;
pub const PLAYER_BASE_MAX_HP: i32 = 20;
pub const GRID_SIZE: f32 = 48.0;
pub const PATH_HALF_WIDTH: f32 = GRID_SIZE * 0.5;
pub const HUD_BUILD_LIMIT: f32 = WINDOW_HEIGHT * 0.5 - 90.0;

pub const PATH: [Vec2; 8] = [
    Vec2::new(-504.0, 216.0),
    Vec2::new(-264.0, 216.0),
    Vec2::new(-264.0, -120.0),
    Vec2::new(-24.0, -120.0),
    Vec2::new(-24.0, 120.0),
    Vec2::new(264.0, 120.0),
    Vec2::new(264.0, -168.0),
    Vec2::new(504.0, -168.0),
];
