use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;
use super::templates::{BASE_SIEGE, BARREL_CANNON, PALETTE_BRONZE};

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
    base_color: PALETTE_BRONZE.base,
    barrel_color: PALETTE_BRONZE.barrel,
    base: BASE_SIEGE,
    barrel: BARREL_CANNON,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 12.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.08),
    ],
};
