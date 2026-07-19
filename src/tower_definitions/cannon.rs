use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{TowerDefinition, TooltipConfig, TowerKind, TowerRegistry};
use super::templates::{BASE_SIEGE, BARREL_CANNON, PALETTE_BRONZE};

pub struct CannonPlugin;

impl Plugin for CannonPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
    }
}

pub static TOWER_CANNON: TowerDefinition = TowerDefinition::new_attacking(
    "Cannon",
    150.0,
    1.45,
    DamageFormula {
        flat: 34,
        crit_multiplier: 1.5,
        earth_multiplier: 1.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    PALETTE_BRONZE.base,
    BASE_SIEGE,
    BARREL_CANNON,
    320.0,
    1.0,
)
    .with_barrel_color(PALETTE_BRONZE.barrel)
    .with_explosion_radius(64.0)
    .with_stat_effects(&[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 12.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.08),
    ])
    .with_tooltip_config(TooltipConfig::STANDARD.with_splash(true))
    .with_tags(&[tags::MECHANICAL]);

pub static KIND: TowerKind = TowerKind(&TOWER_CANNON);
