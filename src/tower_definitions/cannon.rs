use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, _app: &mut App) {}
}

pub const TOWER_CANNON: TowerDefinition = TowerDefinition {
    name: "Cannon",
    range: 150.0,
    cooldown: 1.45,
    damage_formula: DamageFormula {
        flat: 34,
        crit_multiplier: 1.5,
        earth_multiplier: 1.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 320.0,
    explosion_radius: 64.0,
    angular_speed: 1.0,
    base_color: Color::srgb(0.42, 0.36, 0.30),
    barrel_color: Color::srgb(0.74, 0.66, 0.54),
    base_size: Vec2::new(40.0, 40.0),
    barrel_size: Vec2::new(18.0, 30.0),
    barrel_offset: 13.0,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 12.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.08),
    ],
    custom_tooltip: None,
};
