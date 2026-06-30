use bevy::prelude::*;

use crate::components::{CustomTooltip, DamageFormula, TemporaryAttackSpeed};
use crate::game::game_is_running;
use crate::resources::{AttackSpeed, NewRoundEvent, PlayerStatKind, ShootEvent, TowerStatEffect};
use crate::towers::{progress_cooldown, reset_temporary_attack_speed};
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig};
use super::templates::{BASE_STANDARD, BARREL_DOUBLE_LIGHT, PALETTE_BLUE};

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
        app.add_systems(
            Update,
            update_windup_bar.after(decelerate).run_if(game_is_running),
        );
        app.add_systems(Update, update_gatling_tooltip);
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
    base_color: PALETTE_BLUE.base,
    barrel_color: Color::srgb(0.10, 0.10, 0.10),
    base: BASE_STANDARD,
    barrel: BARREL_DOUBLE_LIGHT,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::AirDamage, 2.0),
    ],
    tooltip_config: TooltipConfig::STANDARD,
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

#[derive(Component)]
struct GatlingWindUpBar {
    owner: Entity,
    width: f32,
    fill: bool,
}

const BAR_WIDTH: f32 = 32.0;
const BAR_HEIGHT: f32 = 4.0;
const BAR_Y: f32 = -24.0;

fn attach_gatling_tower(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Gatling {
            commands.entity(entity)
                .insert((GatlingTower, GatlingWindUp::default(), CustomTooltip::default()))
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.08, 0.08, 0.12),
                            Vec2::new(BAR_WIDTH + 2.0, BAR_HEIGHT + 2.0),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 2.0)),
                        Visibility::Hidden,
                        GatlingWindUpBar { owner: entity, width: BAR_WIDTH, fill: false },
                    ));
                    parent.spawn((
                        Sprite::from_color(
                            Color::srgb(0.30, 0.65, 1.0),
                            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
                        ),
                        Transform::from_translation(Vec3::new(0.0, BAR_Y, 3.0)),
                        Visibility::Hidden,
                        GatlingWindUpBar { owner: entity, width: BAR_WIDTH, fill: true },
                    ));
                });
        }
    }
}

fn update_windup_bar(
    towers: Query<&GatlingWindUp, With<GatlingTower>>,
    mut bars: Query<(&GatlingWindUpBar, &mut Transform, &mut Visibility)>,
) {
    for (bar, mut transform, mut visibility) in &mut bars {
        let Ok(windup) = towers.get(bar.owner) else { continue; };
        let ratio = (windup.shots / MAX_SHOTS).clamp(0.0, 1.0);
        *visibility = if ratio > 0.0 { Visibility::Visible } else { Visibility::Hidden };
        if bar.fill {
            transform.scale.x = ratio;
            transform.translation.x = -bar.width * (1.0 - ratio) * 0.5;
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

fn update_gatling_tooltip(
    attack_speed: Res<AttackSpeed>,
    mut towers: Query<(&GatlingWindUp, &mut CustomTooltip), With<GatlingTower>>,
    mut tooltip_texts: ResMut<super::CustomTooltipTexts>,
) {
    let max_bonus = MAX_SHOTS * SPEED_PER_SHOT;
    let effective_speed = attack_speed.value.max(0.1);
    let base_cooldown = TOWER_GATLING.cooldown / effective_speed;
    let min_cooldown = TOWER_GATLING.cooldown / (effective_speed + max_bonus);

    let static_extras = format!(
        "Builds attack speed while firing, decays when idle\nMax wind-up: +{max_bonus:.1}x atk speed ({MAX_SHOTS:.0} shots)\nBase cooldown: {base_cooldown:.2}s  Min: {min_cooldown:.2}s\nDecay: {SHOT_DECAY_RATE:.0} shot/s",
    );
    tooltip_texts.0.insert(TowerKind::Gatling, static_extras.clone());

    for (windup, mut tooltip) in &mut towers {
        let current_bonus = windup.shots * SPEED_PER_SHOT;
        let current_cooldown = TOWER_GATLING.cooldown / (effective_speed + current_bonus);
        tooltip.0 = format!(
            "{static_extras}\nCurrent wind-up: +{current_bonus:.2}x ({:.0}/{MAX_SHOTS:.0} shots)  {current_cooldown:.2}s",
            windup.shots,
        );
    }
}
