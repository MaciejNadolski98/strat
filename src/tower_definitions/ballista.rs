use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TooltipConfig, TowerKind, TowerRegistry};
use super::templates::{BASE_TRIANGLE_M, BARREL_LIGHT, PALETTE_BLUE};

pub struct BallistaPlugin;

impl Plugin for BallistaPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
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
    spread: 0.0,
    piercing: 1,
    piercing_damage: 0.0,
    base_color: PALETTE_BLUE.base,
    barrel_color: PALETTE_BLUE.barrel,
    base: BASE_TRIANGLE_M,
    barrel: BARREL_LIGHT,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.03),
    ],
    tooltip_config: TooltipConfig::STANDARD,
    tags: &[tags::MECHANICAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_BALLISTA);
