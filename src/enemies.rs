use bevy::prelude::*;

use crate::components::{Enemy, EnemyKind, Health, PathProgress, Reward, Speed, Waypoint};
use crate::constants::{PATH, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{
    CurrentHp, EnemiesRemaining, GameOver, MaxHp, Money, NextWaveTimer, PassiveIncome,
    Regeneration, SpawnTimer, WaveNumber,
};

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_number: ResMut<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut next_wave_timer: ResMut<NextWaveTimer>,
    game_over: Res<GameOver>,
    mut money: ResMut<Money>,
    mut hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
    regeneration: Res<Regeneration>,
    passive_income: Res<PassiveIncome>,
    enemies: Query<(), With<Enemy>>,
) {
    if game_over.value {
        return;
    }

    if remaining.count == 0 {
        if enemies.is_empty() {
            next_wave_timer.timer.tick(time.delta());
            if next_wave_timer.timer.just_finished() {
                wave_number.value += 1;
                remaining.count = enemies_in_wave(wave_number.value);
                next_wave_timer.timer.reset();
                apply_wave_start_stats(
                    &mut commands,
                    &mut money,
                    &mut hp,
                    &max_hp,
                    &regeneration,
                    &passive_income,
                );
            }
        }
        return;
    }

    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() {
        return;
    }

    let spawn_index = enemies_in_wave(wave_number.value) - remaining.count;
    remaining.count -= 1;

    let kind = EnemyKind::for_spawn(wave_number.value, spawn_index);
    let max_health = kind.max_health(wave_number.value);
    commands.spawn((
        Sprite::from_color(enemy_color(kind, 1.0), kind.size()),
        Transform::from_translation(PATH[0].extend(3.0)),
        Enemy,
        kind,
        Waypoint { index: 1 },
        PathProgress { distance: 0.0 },
        Health {
            current: max_health,
            max: max_health,
        },
        Speed {
            value: kind.speed(wave_number.value),
        },
        Reward {
            amount: kind.reward(wave_number.value),
        },
    ));
}

pub fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut hp: ResMut<CurrentHp>,
    mut game_over: ResMut<GameOver>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Waypoint,
            &mut PathProgress,
            &Health,
            &Speed,
        ),
        With<Enemy>,
    >,
) {
    if game_over.value {
        return;
    }

    for (entity, mut transform, mut waypoint, mut progress, health, speed) in &mut enemies {
        if health.current <= 0.0 {
            continue;
        }

        let target = PATH[waypoint.index];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let step = speed.value * time.delta_secs();
        progress.distance += step;

        if to_target.length() <= step {
            transform.translation = target.extend(3.0);
            waypoint.index += 1;
            if waypoint.index >= PATH.len() {
                commands.entity(entity).despawn();
                hp.amount -= 1;
                if hp.amount <= 0 {
                    game_over.value = true;
                }
            }
        } else {
            transform.translation += (to_target.normalize() * step).extend(0.0);
        }
    }
}

pub fn update_enemy_colors(mut enemies: Query<(&EnemyKind, &Health, &mut Sprite), With<Enemy>>) {
    for (kind, health, mut sprite) in &mut enemies {
        let health_ratio = (health.current / health.max).clamp(0.0, 1.0);
        sprite.color = enemy_color(*kind, health_ratio);
    }
}

pub fn enemies_in_wave(wave: u32) -> u32 {
    8 + wave * 3
}

fn enemy_color(kind: EnemyKind, health_ratio: f32) -> Color {
    let (damaged, healthy) = kind.colors();
    Color::srgb(
        damaged.0 + (healthy.0 - damaged.0) * health_ratio,
        damaged.1 + (healthy.1 - damaged.1) * health_ratio,
        damaged.2 + (healthy.2 - damaged.2) * health_ratio,
    )
}

fn apply_wave_start_stats(
    commands: &mut Commands,
    money: &mut Money,
    hp: &mut CurrentHp,
    max_hp: &MaxHp,
    regeneration: &Regeneration,
    passive_income: &PassiveIncome,
) {
    if regeneration.amount > 0 {
        hp.amount = (hp.amount + regeneration.amount).min(max_hp.amount);
    }

    if passive_income.amount > 0 {
        money.amount += passive_income.amount;
        spawn_floating_text(
            commands,
            format!("+${}", passive_income.amount),
            Vec2::new(-WINDOW_WIDTH * 0.5 + 150.0, WINDOW_HEIGHT * 0.5 - 104.0),
            Color::srgb(1.0, 0.86, 0.20),
            22.0,
        );
    }
}
