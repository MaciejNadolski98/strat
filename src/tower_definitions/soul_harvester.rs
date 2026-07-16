use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use crate::charges::try_emit_charge;
use crate::components::{CustomTooltip, DamageFormula, DefaultAim, DefaultFire, Tower};
use crate::game::game_is_running;
use crate::resources::{CurrentHp, EnemyKilledEvent};
use crate::tags::{self, Conduit};
use crate::tooltip::plain;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_PENTAGON_S, BARREL_NONE};

pub struct SoulHarvesterPlugin;

impl Plugin for SoulHarvesterPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_soul_harvester_marker.run_if(game_is_running));
        app.add_systems(Update, harvest_souls.run_if(game_is_running));
        app.add_systems(Update, update_soul_harvester_tooltip);
    }
}

pub const TOWER_SOUL_HARVESTER: TowerDefinition = TowerDefinition {
    name: "Soul Harvester",
    range: 110.0,
    cooldown: 999.0,
    damage_formula: DamageFormula {
        flat: 0,
        crit_multiplier: 1.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.0,
    spread: 0.0,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: Color::srgb(0.30, 0.08, 0.36),
    barrel_color: Color::srgb(0.30, 0.08, 0.36),
    base: BASE_PENTAGON_S,
    barrel: BARREL_NONE,
    stat_effects: &[],
    tooltip_config: TooltipConfig::UTILITY,
    tags: &[tags::INFERNAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_SOUL_HARVESTER);

/// Enemy deaths in range needed to trigger a harvest.
const KILLS_PER_HARVEST: u32 = 4;
/// Player HP restored per harvest.
const HEAL_PER_HARVEST: i32 = 1;

#[derive(Component, Default)]
struct SoulHarvesterTower {
    kill_progress: u32,
}

fn attach_soul_harvester_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((SoulHarvesterTower::default(), CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>();
        }
    }
}

/// Soul Harvester doesn't attack - it listens for any enemy death (from any
/// source) in its range. Every `KILLS_PER_HARVEST` such deaths, it heals the
/// player and shoots a charge at a random `Conduit` tower in range.
fn harvest_souls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut current_hp: ResMut<CurrentHp>,
    mut events: EventReader<EnemyKilledEvent>,
    mut harvesters: Query<(Entity, &Transform, &mut SoulHarvesterTower)>,
    conduits: Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
) {
    for event in events.read() {
        for (harvester_entity, harvester_transform, mut harvester) in &mut harvesters {
            let harvester_pos = harvester_transform.translation.truncate();
            if event.position.distance(harvester_pos) > TOWER_SOUL_HARVESTER.range {
                continue;
            }

            harvester.kill_progress += 1;
            if harvester.kill_progress >= KILLS_PER_HARVEST {
                harvester.kill_progress -= KILLS_PER_HARVEST;
                current_hp.amount += HEAL_PER_HARVEST;
                try_emit_charge(
                    &mut commands, &mut meshes, &mut materials, &conduits,
                    harvester_entity, harvester_pos, TOWER_SOUL_HARVESTER.range,
                );
            }
        }
    }
}

fn update_soul_harvester_tooltip(
    mut towers: Query<(&SoulHarvesterTower, &mut CustomTooltip)>,
) {
    for (harvester, mut tooltip) in &mut towers {
        let extras = format!(
            "Heals {HEAL_PER_HARVEST} HP every {KILLS_PER_HARVEST} enemy deaths in range\nProgress: {}/{KILLS_PER_HARVEST}",
            harvester.kill_progress,
        );
        tooltip.0 = vec![plain(extras)];
    }
}
