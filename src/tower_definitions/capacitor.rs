use bevy::prelude::*;

use crate::components::{ChargeConsumer, CustomTooltip, DamageFormula, TemporaryAttackSpeed};
use crate::game::game_is_running;
use crate::resources::{AttackSpeed, ChargeConsumedEvent, GamePhase, NewRoundEvent, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tooltip::plain;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TowerRegistry};
use super::templates::{BASE_PENTAGON_S, BARREL_STANDARD, PALETTE_TEAL};

pub struct CapacitorPlugin;

impl Plugin for CapacitorPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(
            Update,
            (attach_capacitor_marker, consume_charge, reset).run_if(game_is_running),
        );
        app.add_systems(Update, reset_bonus_capacity.in_set(GamePhase::ResetTemporaries));
        app.add_systems(Update, decay_energy.in_set(GamePhase::TemporaryTowerEffects));
        app.add_systems(Update, update_energy_bar.in_set(GamePhase::Gameplay));
        app.add_systems(Update, update_capacitor_tooltip);
    }
}

pub static TOWER_CAPACITOR: TowerDefinition = TowerDefinition::new_attacking(
    "Capacitor",
    105.0,
    1.4,
    DamageFormula {
        flat: 25,
        crit_multiplier: 1.3,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 1.3,
        water_multiplier: 0.0,
    },
    PALETTE_TEAL.base,
    BASE_PENTAGON_S,
    BARREL_STANDARD,
    720.0,
    1.8,
)
    .with_barrel_color(PALETTE_TEAL.barrel)
    .with_stat_effects(&[
        TowerStatEffect::new(PlayerStatKind::AirDamage, 1.5),
    ])
    .with_tags(&[tags::MECHANICAL, tags::CONDUIT]);

pub static KIND: TowerKind = TowerKind(&TOWER_CAPACITOR);

pub const BASE_MAX_ENERGY: f32 = 10.0;
const ENERGY_PER_CHARGE: f32 = 5.0;
pub const SPEED_PER_ENERGY: f32 = 0.9;
const ENERGY_DECAY_RATE: f32 = 1.2;

#[derive(Component)]
pub struct CapacitorTower;

#[derive(Component, Default)]
struct CapacitorEnergy {
    energy: f32,
}

#[derive(Component, Default)]
pub struct CapacitorBonusCapacity(pub f32);

#[derive(Component)]
struct CapacitorEnergyBar {
    owner: Entity,
    width: f32,
    fill: bool,
}

const BAR_WIDTH: f32 = 32.0;
const BAR_HEIGHT: f32 = 4.0;
const BAR_Y: f32 = -24.0;

fn attach_capacitor_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((
                    CapacitorTower, CapacitorEnergy::default(),
                    CapacitorBonusCapacity::default(), ChargeConsumer,
                    CustomTooltip::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.08, 0.08, 0.12),
                            Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 2.0)),
                        Visibility::Hidden,
                        CapacitorEnergyBar { owner: entity, width: BAR_WIDTH, fill: false },
                    ));
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.20, 0.80, 0.70),
                            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 3.0)),
                        Visibility::Hidden,
                        CapacitorEnergyBar { owner: entity, width: BAR_WIDTH, fill: true },
                    ));
                });
        }
    }
}

fn reset_bonus_capacity(
    mut towers: Query<&mut CapacitorBonusCapacity, With<CapacitorTower>>,
) {
    for mut bonus in &mut towers {
        bonus.0 = 0.0;
    }
}

fn update_energy_bar(
    towers: Query<(&CapacitorEnergy, &CapacitorBonusCapacity), With<CapacitorTower>>,
    mut bars: Query<(&CapacitorEnergyBar, &mut Transform, &mut Visibility)>,
) {
    for (bar, mut transform, mut visibility) in &mut bars {
        let Ok((energy, bonus)) = towers.get(bar.owner) else { continue; };
        let max = BASE_MAX_ENERGY + bonus.0;
        let ratio = (energy.energy / max).clamp(0.0, 1.0);
        *visibility = if ratio > 0.0 { Visibility::Visible } else { Visibility::Hidden };
        if bar.fill {
            transform.scale.x = ratio;
            transform.translation.x = -bar.width * (1.0 - ratio) * 0.5;
        }
    }
}

fn consume_charge(
    mut events: EventReader<ChargeConsumedEvent>,
    mut towers: Query<(&mut CapacitorEnergy, &CapacitorBonusCapacity), With<CapacitorTower>>,
) {
    for event in events.read() {
        if let Ok((mut cap, bonus)) = towers.get_mut(event.tower) {
            let max = BASE_MAX_ENERGY + bonus.0;
            cap.energy = (cap.energy + ENERGY_PER_CHARGE).min(max);
        }
    }
}

fn decay_energy(
    mut towers: Query<(&mut CapacitorEnergy, &mut TemporaryAttackSpeed), With<CapacitorTower>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (mut cap, mut temp_speed) in &mut towers {
        cap.energy = (cap.energy - delta * ENERGY_DECAY_RATE).max(0.0);
        temp_speed.flat = cap.energy * SPEED_PER_ENERGY;
    }
}

fn reset(
    mut events: EventReader<NewRoundEvent>,
    mut towers: Query<&mut CapacitorEnergy, With<CapacitorTower>>,
) {
    if events.read().next().is_some() {
        for mut cap in &mut towers {
            cap.energy = 0.0;
        }
    }
}

fn update_capacitor_tooltip(
    attack_speed: Res<AttackSpeed>,
    mut towers: Query<(&CapacitorEnergy, &CapacitorBonusCapacity, &mut CustomTooltip), With<CapacitorTower>>,
) {
    let effective_speed = attack_speed.value().max(0.1);

    for (cap, bonus, mut tooltip) in &mut towers {
        let max = BASE_MAX_ENERGY + bonus.0;
        let max_bonus = max * SPEED_PER_ENERGY;
        let base_cooldown = TOWER_CAPACITOR.cooldown / effective_speed;
        let min_cooldown = TOWER_CAPACITOR.cooldown / (effective_speed + max_bonus);
        let current_bonus = cap.energy * SPEED_PER_ENERGY;
        let current_cooldown = TOWER_CAPACITOR.cooldown / (effective_speed + current_bonus);

        tooltip.0 = vec![plain(format!(
            "Gains attack speed from charges, decays when idle\n\
             Max energy: +{max_bonus:.1}x atk speed ({max:.0} energy)\n\
             Base cooldown: {base_cooldown:.2}s  Min: {min_cooldown:.2}s\n\
             Decay: {ENERGY_DECAY_RATE:.1}/s\n\
             Each charge adds {ENERGY_PER_CHARGE:.0} energy\n\
             Current energy: +{current_bonus:.2}x ({:.1}/{max:.0})  {current_cooldown:.2}s",
            cap.energy,
        ))];
    }
}
