use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TowerKind, TowerRegistry};
use super::templates::{BASE_STANDARD, BARREL_SNIPER, PALETTE_VIOLET};

pub struct SniperPlugin;

impl Plugin for SniperPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
}

pub const TOWER_SNIPER: TowerDefinition = TowerDefinition::new_attacking(
    "Sniper",
    260.0,
    1.75,
    DamageFormula {
        flat: 55,
        crit_multiplier: 5.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 1.9,
    },
    PALETTE_VIOLET.base,
    BASE_STANDARD,
    BARREL_SNIPER,
    720.0,
    0.9,
)
    .with_barrel_color(PALETTE_VIOLET.barrel)
    .with_piercing(2)
    .with_stat_effects(&[
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.08),
        TowerStatEffect::new(PlayerStatKind::Regeneration, -1.0),
    ])
    .with_tags(&[tags::MECHANICAL]);

pub const KIND: TowerKind = TowerKind(&TOWER_SNIPER);
