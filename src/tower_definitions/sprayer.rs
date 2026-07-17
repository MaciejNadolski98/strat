use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TooltipConfig, TowerKind, TowerRegistry};
use super::templates::{BASE_LIGHT, BARREL_LIGHT, PALETTE_TEAL};

pub struct SprayerPlugin;

impl Plugin for SprayerPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
}

pub const TOWER_SPRAYER: TowerDefinition = TowerDefinition {
    name: "Sprayer",
    range: 100.0,
    cooldown: 0.16,
    damage_formula: DamageFormula {
        flat: 11,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.3,
    },
    projectile_speed: 260.0,
    explosion_radius: 0.0,
    angular_speed: 4.2,
    spread: 0.7,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: PALETTE_TEAL.base,
    barrel_color: PALETTE_TEAL.barrel,
    base: BASE_LIGHT,
    barrel: BARREL_LIGHT,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::Loot, 1.0),
    ],
    tooltip_config: TooltipConfig::STANDARD,
    tags: &[tags::MECHANICAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_SPRAYER);
