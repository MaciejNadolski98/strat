use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TowerKind, TowerRegistry};
use super::templates::{BASE_LIGHT, BARREL_LIGHT, PALETTE_TEAL};

pub struct SprayerPlugin;

impl Plugin for SprayerPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
}

pub const TOWER_SPRAYER: TowerDefinition = TowerDefinition::new_attacking(
    "Sprayer",
    100.0,
    0.32,
    DamageFormula {
        flat: 11,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.3,
    },
    PALETTE_TEAL.base,
    BASE_LIGHT,
    BARREL_LIGHT,
    260.0,
    4.2,
)
    .with_barrel_color(PALETTE_TEAL.barrel)
    .with_spread(0.7)
    .with_projectiles_per_shot(2)
    .with_stat_effects(&[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::Loot, 1.0),
    ])
    .with_tags(&[tags::MECHANICAL]);

pub const KIND: TowerKind = TowerKind(&TOWER_SPRAYER);
