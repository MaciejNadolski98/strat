use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;

pub struct BallistaPlugin;

impl Plugin for BallistaPlugin {
    fn build(&self, _app: &mut App) {}
}

pub const TOWER_BALLISTA: TowerDefinition = TowerDefinition {
    name: "Ballista",
    range: 185.0,
    cooldown: 0.73,
    damage_formula: DamageFormula {
        flat: 24,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.75,
        water_multiplier: 0.75,
    },
    projectile_speed: 430.0,
    explosion_radius: 0.0,
    angular_speed: 1.6,
    base_color: Color::srgb(0.22, 0.42, 0.74),
    barrel_color: Color::srgb(0.67, 0.83, 0.96),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::new(12.0, 38.0),
    barrel_offset: 16.0,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.03),
    ],
    custom_tooltip: None,
};
