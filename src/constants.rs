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
pub const BASE_PIERCING_DAMAGE: f32 = -0.20;
pub const HEX_SIZE: f32 = 28.0;
pub const HEX_SPACING: f32 = HEX_SIZE * 1.7320508;

pub const MAX_HEALTH_GROWTH: f32 = 0.4;
pub const PRICE_GROWTH: f32 = 0.4;

const fn axial_to_world(q: f32, r: f32) -> Vec2 {
    Vec2::new(
        HEX_SPACING * (q + 0.5 * r),
        HEX_SIZE * (1.5 * r),
    )
}

pub const INITIAL_PATH: [Vec2; 7] = [
    axial_to_world(-12.0, 5.0),
    axial_to_world(-11.0, 5.0),
    axial_to_world(-10.0, 5.0),
    axial_to_world(-9.0, 5.0),
    axial_to_world(-8.0, 5.0),
    axial_to_world(-7.0, 5.0),
    axial_to_world(-6.0, 5.0),
];
