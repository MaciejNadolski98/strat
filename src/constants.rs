pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 700.0;
pub const SHOP_REROLL_COST: u32 = 2;
pub const STARTING_MONEY: i32 = 40;
pub const PLAYER_BASE_MAX_HP: i32 = 20;
pub const BASE_REGENERATION: i32 = 1;
pub const BASE_ATTACK_SPEED: f32 = 1.0;
pub const BASE_LOOT: i32 = 2;
pub const BASE_CRITICAL_CHANCE: f32 = 0.12;
pub const BASE_PIERCING_DAMAGE: f32 = -0.20;
pub const HEX_SIZE: f32 = 28.0;
pub const HEX_SPACING: f32 = HEX_SIZE * 1.7320508;
pub const BACKGROUND_Z: f32 = -10.0;
pub const TERRAIN_Z: f32 = -9.0;
pub const GRID_Z: f32 = -8.0;
pub const PATH_FILLED_Z: f32 = -2.0;
pub const PATH_LINE_Z: f32 = -1.5;

use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::srgb(0.10, 0.15, 0.13);
pub const PATH_FILL_COLOR: Color = Color::srgb(0.43, 0.39, 0.31);
pub const PATH_EDGE_COLOR: Color = Color::srgb(0.24, 0.21, 0.16);
pub const PATH_LINE_COLOR: Color = Color::srgb(0.43, 0.39, 0.31);

pub const MAX_HEALTH_GROWTH: f32 = 0.4;
pub const PRICE_GROWTH: f32 = 0.4;
