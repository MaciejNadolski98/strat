use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;

pub struct SniperPlugin;

impl Plugin for SniperPlugin {
    fn build(&self, _app: &mut App) {}
}

pub const TOWER_SNIPER: TowerDefinition = TowerDefinition {
    name: "Sniper",
    range: 260.0,
    cooldown: 1.75,
    damage_formula: DamageFormula {
        flat: 55,
        crit_multiplier: 5.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 1.9,
    },
    projectile_speed: 720.0,
    explosion_radius: 0.0,
    angular_speed: 0.9,
    base_color: Color::srgb(0.34, 0.28, 0.56),
    barrel_color: Color::srgb(0.82, 0.76, 0.98),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::new(8.0, 48.0),
    barrel_offset: 20.0,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.08),
        TowerStatEffect::new(PlayerStatKind::Regeneration, -1.0),
    ],
};
