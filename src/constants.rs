use bevy::prelude::*;

pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 700.0;
pub const SHOP_REROLL_COST: u32 = 2;
pub const PATH_EXTENSION_BASE_COST: i32 = 3;
pub const PATH_EXTENSION_COST_STEP: i32 = 2;
pub const STARTING_MONEY: i32 = 40;
pub const PLAYER_BASE_MAX_HP: i32 = 20;
pub const BASE_REGENERATION: i32 = 1;
pub const BASE_ATTACK_SPEED: f32 = 1.0;
pub const BASE_LOOT: i32 = 2;
pub const BASE_CRITICAL_CHANCE: f32 = 0.12;
pub const GRID_SIZE: f32 = 48.0;
pub const PATH_HALF_WIDTH: f32 = GRID_SIZE * 0.5;

pub const MAX_HEALTH_GROWTH: f32 = 0.4;
pub const PRICE_GROWTH: f32 = 0.4;

pub const INITIAL_PATH: [Vec2; 7] = [
    Vec2::new(-456.0, 216.0),
    Vec2::new(-408.0, 216.0),
    Vec2::new(-360.0, 216.0),
    Vec2::new(-312.0, 216.0),
    Vec2::new(-264.0, 216.0),
    Vec2::new(-216.0, 216.0),
    Vec2::new(-168.0, 216.0),
];
