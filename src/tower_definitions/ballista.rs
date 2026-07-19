use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TowerKind, TowerRegistry};
use super::templates::{BASE_TRIANGLE_M, BARREL_LIGHT, PALETTE_BLUE};

pub struct BallistaPlugin;

impl Plugin for BallistaPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
}

pub static TOWER_BALLISTA: TowerDefinition = TowerDefinition::new_attacking(
    "Ballista",
    185.0,
    0.73,
    DamageFormula {
        flat: 24,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.75,
        water_multiplier: 0.75,
    },
    PALETTE_BLUE.base,
    BASE_TRIANGLE_M,
    BARREL_LIGHT,
    430.0,
    1.6,
)
    .with_barrel_color(PALETTE_BLUE.barrel)
    .with_piercing(1)
    .with_piercing_damage(-0.20)
    .with_stat_effects(&[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.03),
    ])
    .with_tags(&[tags::MECHANICAL]);

pub static KIND: TowerKind = TowerKind(&TOWER_BALLISTA);
