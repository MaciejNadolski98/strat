use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{TowerDefinition, TooltipConfig};
use super::templates::{BASE_STANDARD, BARREL_SNIPER, PALETTE_VIOLET};

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
    base_color: PALETTE_VIOLET.base,
    barrel_color: PALETTE_VIOLET.barrel,
    base: BASE_STANDARD,
    barrel: BARREL_SNIPER,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.08),
        TowerStatEffect::new(PlayerStatKind::Regeneration, -1.0),
    ],
    tooltip_config: TooltipConfig::STANDARD,
};
