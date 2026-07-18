use bevy::prelude::*;

use crate::components::{BeamFire, ChargeConsumer, CustomTooltip, DamageFormula, DefaultFire, TemporaryRange};
use crate::game::game_is_running;
use crate::resources::{ChargeConsumedEvent, GamePhase, PlayerStatKind, ShootEvent, TowerStatEffect};
use crate::tags;
use crate::tooltip::plain;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_DIAMOND_M, BARREL_SNIPER, PALETTE_CRIMSON};

const CHARGE_MULTIPLIER_BONUS: f32 = 2.0;

#[derive(Component)]
pub struct LaserTower;

#[derive(Component, Default)]
struct LaserCharge {
    charged: bool,
}

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_laser_marker.run_if(game_is_running));
        app.add_systems(Update, (consume_laser_charge, discharge_on_shot).run_if(game_is_running));
        app.add_systems(Update, apply_laser_charge.in_set(GamePhase::TemporaryTowerEffects));
        app.add_systems(Update, update_laser_tooltip);
    }
}

pub static TOWER_LASER: TowerDefinition = TowerDefinition {
    name: "Laser",
    range: 150.0,
    cooldown: 3.0,
    damage_formula: DamageFormula {
        flat: 60,
        crit_multiplier: 1.5,
        earth_multiplier: 0.0,
        fire_multiplier: 6.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.5,
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

pub static KIND: TowerKind = TowerKind(&TOWER_LASER);

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
                    LaserCharge::default(),
                    TemporaryRange::default(),
                    CustomTooltip::default(),
                ))
                .remove::<DefaultFire>();
        }
    }
}

fn consume_laser_charge(
    mut events: EventReader<ChargeConsumedEvent>,
    mut towers: Query<&mut LaserCharge, With<LaserTower>>,
) {
    for event in events.read() {
        if let Ok(mut charge) = towers.get_mut(event.tower) {
            charge.charged = true;
        }
    }
}

fn discharge_on_shot(
    mut events: EventReader<ShootEvent>,
    mut towers: Query<&mut LaserCharge, With<LaserTower>>,
) {
    for event in events.read() {
        if let Ok(mut charge) = towers.get_mut(event.source_tower) {
            charge.charged = false;
        }
    }
}

fn apply_laser_charge(
    mut towers: Query<(&LaserCharge, &mut TemporaryRange), With<LaserTower>>,
) {
    for (charge, mut boost) in &mut towers {
        if charge.charged {
            boost.multiplier += CHARGE_MULTIPLIER_BONUS;
        }
    }
}

fn update_laser_tooltip(
    mut towers: Query<(&LaserCharge, &mut CustomTooltip), With<LaserTower>>,
) {
    for (charge, mut tooltip) in &mut towers {
        tooltip.0 = if charge.charged {
            vec![plain(format!("Charged: {:.0}x range on next attack", 1.0 + CHARGE_MULTIPLIER_BONUS))]
        } else {
            Vec::new()
        };
    }
}
