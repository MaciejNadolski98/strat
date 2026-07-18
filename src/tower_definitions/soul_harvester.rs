use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use crate::charges::try_emit_charge;
use crate::components::{CustomTooltip, DamageFormula, DefaultAim, DefaultFire, Tower};
use crate::game::game_is_running;
use crate::resources::{CurrentHp, EnemyKilledEvent, MaxHp};
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
        app.add_systems(Update, update_soul_harvester_progress_bar.after(harvest_souls));
        app.add_systems(Update, update_soul_harvester_tooltip);
    }
}

pub const TOWER_SOUL_HARVESTER: TowerDefinition = TowerDefinition::new_utility(
    "Soul Harvester",
    110.0,
    Color::srgb(0.30, 0.08, 0.36),
    BASE_PENTAGON_S,
    BARREL_NONE,
)
    .with_tooltip_config(TooltipConfig::UTILITY)
    .with_tags(&[tags::INFERNAL]);

pub const KIND: TowerKind = TowerKind(&TOWER_SOUL_HARVESTER);

const KILLS_PER_HARVEST: u32 = 4;
const HEAL_PER_HARVEST: i32 = 1;

const BAR_WIDTH: f32 = 32.0;
const BAR_HEIGHT: f32 = 4.0;
const BAR_Y: f32 = -24.0;

#[derive(Component, Default)]
struct SoulHarvesterTower {
    kill_progress: u32,
}

#[derive(Component)]
struct SoulHarvesterProgressBar {
    owner: Entity,
    width: f32,
    fill: bool,
}

fn attach_soul_harvester_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((SoulHarvesterTower::default(), CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>()
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.08, 0.08, 0.12),
                            Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 2.0)),
                        SoulHarvesterProgressBar { owner: entity, width: BAR_WIDTH, fill: false },
                    ));
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.78, 0.24, 0.90),
                            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 3.0)),
                        SoulHarvesterProgressBar { owner: entity, width: BAR_WIDTH, fill: true },
                    ));
                });
        }
    }
}

fn update_soul_harvester_progress_bar(
    towers: Query<&SoulHarvesterTower>,
    mut bars: Query<(&SoulHarvesterProgressBar, &mut Transform)>,
) {
    for (bar, mut transform) in &mut bars {
        let Ok(harvester) = towers.get(bar.owner) else { continue; };
        let ratio = (harvester.kill_progress as f32 / KILLS_PER_HARVEST as f32).clamp(0.0, 1.0);
        if bar.fill {
            transform.scale.x = ratio;
            transform.translation.x = -bar.width * (1.0 - ratio) * 0.5;
        }
    }
}

fn harvest_souls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut current_hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
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
                current_hp.amount = (current_hp.amount + HEAL_PER_HARVEST).min(max_hp.value().round() as i32);
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
            "Heals {HEAL_PER_HARVEST} HP and produces a charge every {KILLS_PER_HARVEST} enemy deaths in range\nProgress: {}/{KILLS_PER_HARVEST}",
            harvester.kill_progress,
        );
        tooltip.0 = vec![plain(extras)];
    }
}
