use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::TowerDefinition;
use super::templates::{BASE_LIGHT, BARREL_LIGHT, PALETTE_TEAL};

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
    base_color: PALETTE_TEAL.base,
    barrel_color: PALETTE_TEAL.barrel,
    base: BASE_LIGHT,
    barrel: BARREL_LIGHT,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::Loot, 1.0),
    ],
    custom_tooltip: None,
};
