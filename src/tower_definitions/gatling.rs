use bevy::prelude::*;

use crate::components::{DamageFormula, FireCooldown};
use crate::game::game_is_running;
use crate::resources::{NewRoundEvent, PlayerStatKind, ShootEvent, TowerStatEffect};
use crate::tower_definitions::TowerKind;
use super::TowerDefinition;

pub struct GatlingPlugin;

impl Plugin for GatlingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_gatling_tower,
                accelerate,
                decelerate,
                reset,
            )
                .run_if(game_is_running),
        );
    }
}

pub const TOWER_GATLING: TowerDefinition = TowerDefinition {
    name: "Gatling",
    range: 92.0,
    cooldown: 1.3,
    damage_formula: DamageFormula {
        flat: 10,
        crit_multiplier: 1.2,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.60,
        water_multiplier: 0.60,
    },
    projectile_speed: 890.0,
    explosion_radius: 0.0,
    angular_speed: 2.3,
    base_color: Color::srgb(0.22, 0.42, 0.74),
    barrel_color: Color::srgb(0.10, 0.10, 0.10),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::new(12.0, 38.0),
    barrel_offset: 16.0,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::AirDamage, 2.0),
    ],
    custom_tooltip: None,
};

#[derive(Component)]
struct GatlingTower;

fn attach_gatling_tower(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Gatling {
            commands.entity(entity).insert(GatlingTower);
        }
    }
}

fn accelerate(
    mut events: EventReader<ShootEvent>,
    mut towers: Query<&mut FireCooldown, With<GatlingTower>>,
) {
    for event in events.read() {
        if let Ok(mut cooldown) = towers.get_mut(event.source_tower) {
            let current_cooldown = cooldown.base_cooldown;
            cooldown.base_cooldown = (current_cooldown - 0.2).max(0.1);
        }
    }
}

fn decelerate(
    towers: Query<&mut FireCooldown, With<GatlingTower>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for mut cooldown in towers {
        let current_cooldown = cooldown.base_cooldown;
        cooldown.base_cooldown = (current_cooldown + delta / 3.0).min(TOWER_GATLING.cooldown);
    }
}

fn reset(
    mut events: EventReader<NewRoundEvent>,
    towers: Query<&mut FireCooldown, With<GatlingTower>>,
) {
    if events.read().next().is_some() {
        for mut cooldown in towers {
            cooldown.base_cooldown = TOWER_GATLING.cooldown;
        }
    }
}
