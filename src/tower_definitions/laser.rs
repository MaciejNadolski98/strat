use bevy::prelude::*;

use crate::components::{BeamFire, ChargeConsumer, CustomTooltip, DamageFormula, DefaultFire, RangeBoost};
use crate::game::game_is_running;
use crate::resources::{ChargeConsumedEvent, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tooltip::plain;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_DIAMOND_M, BARREL_SNIPER, PALETTE_CRIMSON};

const CHARGED_RANGE_MULTIPLIER: f32 = 3.0;

#[derive(Component)]
pub struct LaserTower;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_laser_marker.run_if(game_is_running));
        app.add_systems(Update, consume_laser_charge.run_if(game_is_running));
        app.add_systems(Update, update_laser_tooltip);
    }
}

pub const TOWER_LASER: TowerDefinition = TowerDefinition {
    name: "Laser",
    range: 140.0,
    cooldown: 0.6,
    damage_formula: DamageFormula {
        flat: 12,
        crit_multiplier: 1.5,
        earth_multiplier: 0.0,
        fire_multiplier: 1.2,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 2.0,
    spread: 0.0,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: PALETTE_CRIMSON.base,
    barrel_color: PALETTE_CRIMSON.barrel,
    base: BASE_DIAMOND_M,
    barrel: BARREL_SNIPER,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0),
    ],
    tooltip_config: TooltipConfig::STANDARD.with_projectile(false),
    tags: &[tags::MECHANICAL, tags::INFERNAL, tags::CONDUIT],
};

pub const KIND: TowerKind = TowerKind(&TOWER_LASER);

fn attach_laser_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((
                    LaserTower,
                    BeamFire,
                    ChargeConsumer,
                    RangeBoost { multiplier: 1.0 },
                    CustomTooltip::default(),
                ))
                .remove::<DefaultFire>();
        }
    }
}

fn consume_laser_charge(
    mut events: EventReader<ChargeConsumedEvent>,
    mut towers: Query<&mut RangeBoost, With<LaserTower>>,
) {
    for event in events.read() {
        if let Ok(mut boost) = towers.get_mut(event.tower) {
            boost.multiplier = CHARGED_RANGE_MULTIPLIER;
        }
    }
}

fn update_laser_tooltip(
    mut towers: Query<(&RangeBoost, &mut CustomTooltip), With<LaserTower>>,
) {
    for (boost, mut tooltip) in &mut towers {
        tooltip.0 = if boost.multiplier > 1.0 {
            vec![plain(format!("Charged: {:.0}x range on next attack", boost.multiplier))]
        } else {
            Vec::new()
        };
    }
}
