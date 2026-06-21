use bevy::prelude::*;

use crate::components::{DamageFormula, TemporaryAttackSpeed};
use crate::game::game_is_running;
use crate::resources::{NewRoundEvent, PlayerStatKind, ShootEvent, TowerStatEffect};
use crate::towers::{progress_cooldown, reset_temporary_attack_speed};
use crate::tower_definitions::TowerKind;
use super::TowerDefinition;

pub struct GatlingPlugin;

impl Plugin for GatlingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (attach_gatling_tower, accelerate, reset).run_if(game_is_running),
        );
        app.add_systems(
            Update,
            decelerate
                .after(reset_temporary_attack_speed)
                .before(progress_cooldown)
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

const MAX_SHOTS: f32 = 8.0;
const SPEED_PER_SHOT: f32 = 0.2;
const SHOT_DECAY_RATE: f32 = 1.0;

#[derive(Component)]
struct GatlingTower;

#[derive(Component, Default)]
struct GatlingWindUp {
    shots: f32,
}

fn attach_gatling_tower(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Gatling {
            commands.entity(entity).insert((GatlingTower, GatlingWindUp::default()));
        }
    }
}

fn accelerate(
    mut events: EventReader<ShootEvent>,
    mut towers: Query<&mut GatlingWindUp, With<GatlingTower>>,
) {
    for event in events.read() {
        if let Ok(mut windup) = towers.get_mut(event.source_tower) {
            windup.shots = (windup.shots + 1.0).min(MAX_SHOTS);
        }
    }
}

fn decelerate(
    mut towers: Query<(&mut GatlingWindUp, &mut TemporaryAttackSpeed), With<GatlingTower>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (mut windup, mut temp_speed) in &mut towers {
        windup.shots = (windup.shots - delta * SHOT_DECAY_RATE).max(0.0);
        temp_speed.bonus = windup.shots * SPEED_PER_SHOT;
    }
}

fn reset(
    mut events: EventReader<NewRoundEvent>,
    mut towers: Query<&mut GatlingWindUp, With<GatlingTower>>,
) {
    if events.read().next().is_some() {
        for mut windup in &mut towers {
            windup.shots = 0.0;
        }
    }
}
