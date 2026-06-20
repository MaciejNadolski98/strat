use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;

pub struct SprayerPlugin;

impl Plugin for SprayerPlugin {
    fn build(&self, _app: &mut App) {}
}

pub const TOWER_SPRAYER: TowerDefinition = TowerDefinition {
    name: "Sprayer",
    range: 125.0,
    cooldown: 0.32,
    damage_formula: DamageFormula {
        flat: 11,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 1.0,
    },
    projectile_speed: 520.0,
    explosion_radius: 0.0,
    angular_speed: 4.2,
    base_color: Color::srgb(0.20, 0.52, 0.46),
    barrel_color: Color::srgb(0.62, 0.92, 0.78),
    base_size: Vec2::new(32.0, 32.0),
    barrel_size: Vec2::new(10.0, 28.0),
    barrel_offset: 12.0,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::PassiveIncome, 1.0),
    ],
};
