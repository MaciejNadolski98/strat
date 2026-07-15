use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use crate::charges::try_emit_charge;
use crate::components::{CustomTooltip, DamageFormula, DefaultAim, DefaultFire, Tower, TemporaryAttackSpeed};
use crate::game::game_is_running;
use crate::resources::{GamePhase, ShootEvent};
use crate::tags::{self, Conduit};
use crate::tooltip::plain;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_CIRCLE_M, BARREL_NONE};

pub struct DynamoPlugin;

impl Plugin for DynamoPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_dynamo_marker.run_if(game_is_running));
        app.add_systems(Update, apply_dynamo_aura.in_set(GamePhase::TemporaryTowerEffects));
        app.add_systems(Update, accumulate_dynamo_charges.in_set(GamePhase::Gameplay));
        app.add_systems(Update, update_dynamo_charge_bar.after(accumulate_dynamo_charges));
        app.add_systems(Update, update_dynamo_tooltip);
    }
}

pub const TOWER_DYNAMO: TowerDefinition = TowerDefinition {
    name: "Dynamo",
    range: 100.0,
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
    base_color: Color::srgb(0.24, 0.46, 0.56),
    barrel_color: Color::srgb(0.24, 0.46, 0.56),
    base: BASE_CIRCLE_M,
    barrel: BARREL_NONE,
    stat_effects: &[],
    tooltip_config: TooltipConfig::AURA,
    tags: &[tags::MECHANICAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_DYNAMO);

const SPEED_PENALTY: f32 = 0.35;
const CHARGE_PER_SHOT: f32 = 1.0 / 200.0;

const BAR_WIDTH: f32 = 32.0;
const BAR_HEIGHT: f32 = 4.0;
const BAR_Y: f32 = -24.0;

#[derive(Component, Default)]
struct DynamoTower {
    charge_progress: f32,
}

#[derive(Component)]
struct DynamoChargeBar {
    owner: Entity,
    width: f32,
    fill: bool,
}

fn attach_dynamo_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((DynamoTower::default(), CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>()
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.08, 0.08, 0.12),
                            Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 2.0)),
                        DynamoChargeBar { owner: entity, width: BAR_WIDTH, fill: false },
                    ));
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.55, 0.90, 0.98),
                            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 3.0)),
                        DynamoChargeBar { owner: entity, width: BAR_WIDTH, fill: true },
                    ));
                });
        }
    }
}

fn update_dynamo_charge_bar(
    towers: Query<&DynamoTower>,
    mut bars: Query<(&DynamoChargeBar, &mut Transform)>,
) {
    for (bar, mut transform) in &mut bars {
        let Ok(dynamo) = towers.get(bar.owner) else { continue; };
        let ratio = dynamo.charge_progress.clamp(0.0, 1.0);
        if bar.fill {
            transform.scale.x = ratio;
            transform.translation.x = -bar.width * (1.0 - ratio) * 0.5;
        }
    }
}

fn apply_dynamo_aura(
    dynamos: Query<&Transform, With<DynamoTower>>,
    mut adjacent_towers: Query<
        (&Transform, &mut TemporaryAttackSpeed),
        (With<Tower>, Without<DynamoTower>),
    >,
) {
    for dynamo_transform in &dynamos {
        let dynamo_pos = dynamo_transform.translation.truncate();
        for (tower_transform, mut temp_speed) in &mut adjacent_towers {
            if tower_transform.translation.truncate().distance(dynamo_pos) <= TOWER_DYNAMO.range {
                temp_speed.bonus -= SPEED_PENALTY;
            }
        }
    }
}

fn accumulate_dynamo_charges(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut events: EventReader<ShootEvent>,
    shooters: Query<&Transform, With<Tower>>,
    mut dynamos: Query<(Entity, &Transform, &mut DynamoTower)>,
    conduits: Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
) {
    for event in events.read() {
        let Ok(shooter_transform) = shooters.get(event.source_tower) else { continue };
        let shooter_pos = shooter_transform.translation.truncate();

        for (dynamo_entity, dynamo_transform, mut dynamo) in &mut dynamos {
            if event.source_tower == dynamo_entity {
                continue;
            }
            let dynamo_pos = dynamo_transform.translation.truncate();
            if shooter_pos.distance(dynamo_pos) > TOWER_DYNAMO.range {
                continue;
            }

            dynamo.charge_progress += CHARGE_PER_SHOT;
            if dynamo.charge_progress >= 1.0 {
                dynamo.charge_progress -= 1.0;
                try_emit_charge(
                    &mut commands, &mut meshes, &mut materials, &conduits,
                    dynamo_entity, dynamo_pos, TOWER_DYNAMO.range,
                );
            }
        }
    }
}

fn update_dynamo_tooltip(
    mut towers: Query<(&DynamoTower, &mut CustomTooltip)>,
) {
    for (dynamo, mut tooltip) in &mut towers {
        let extras = format!(
            "Saps {SPEED_PENALTY:.2}x attack speed from nearby towers\nProduces a charge every {:.0} shots from nearby towers\nCharge: {:.0}%",
            1.0 / CHARGE_PER_SHOT,
            dynamo.charge_progress * 100.0,
        );
        tooltip.0 = vec![plain(extras)];
    }
}
