use bevy::prelude::*;

use crate::components::{BeamFire, CustomTooltip, DamageFormula, DefaultFire};
use crate::game::game_is_running;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_DIAMOND_M, BARREL_SNIPER, PALETTE_CRIMSON};

#[derive(Component)]
pub struct LaserTower;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_laser_marker.run_if(game_is_running));
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
    tags: &[tags::MECHANICAL, tags::INFERNAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_LASER);

/// Laser keeps the default aiming/rotation (`DefaultAim` stays) and opts
/// into `towers::fire_beam_towers` instead of the default firing.
fn attach_laser_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((LaserTower, BeamFire, CustomTooltip::default()))
                .remove::<DefaultFire>();
        }
    }
}
