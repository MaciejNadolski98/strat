use bevy::prelude::*;

use crate::components::{Enemy, EnemyKind};
use crate::constants::PATH;
use crate::resources::{Game, PlayerStats, Wave};

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave: ResMut<Wave>,
    mut game: ResMut<Game>,
    stats: Res<PlayerStats>,
    enemies: Query<(), With<Enemy>>,
) {
    if game.game_over {
        return;
    }

    if wave.remaining == 0 {
        if enemies.is_empty() {
            wave.next_wave_timer.tick(time.delta());
            if wave.next_wave_timer.just_finished() {
                wave.number += 1;
                wave.remaining = enemies_in_wave(wave.number);
                wave.next_wave_timer.reset();
                apply_wave_start_stats(&mut game, &stats);
            }
        }
        return;
    }

    wave.spawn_timer.tick(time.delta());
    if !wave.spawn_timer.just_finished() {
        return;
    }

    let spawn_index = enemies_in_wave(wave.number) - wave.remaining;
    wave.remaining -= 1;

    let kind = EnemyKind::for_spawn(wave.number, spawn_index);
    let max_health = kind.max_health(wave.number);
    commands.spawn((
        Sprite::from_color(enemy_color(kind, 1.0), kind.size()),
        Transform::from_translation(PATH[0].extend(3.0)),
        Enemy {
            kind,
            waypoint: 1,
            progress: 0.0,
            health: max_health,
            max_health,
            speed: kind.speed(wave.number),
            reward: kind.reward(wave.number),
        },
    ));
}

pub fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
) {
    if game.game_over {
        return;
    }

    for (entity, mut transform, mut enemy) in &mut enemies {
        if enemy.health <= 0.0 {
            continue;
        }

        let target = PATH[enemy.waypoint];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let step = enemy.speed * time.delta_secs();
        enemy.progress += step;

        if to_target.length() <= step {
            transform.translation = target.extend(3.0);
            enemy.waypoint += 1;
            if enemy.waypoint >= PATH.len() {
                commands.entity(entity).despawn();
                game.lives -= 1;
                if game.lives <= 0 {
                    game.game_over = true;
                }
            }
        } else {
            transform.translation += (to_target.normalize() * step).extend(0.0);
        }
    }
}

pub fn update_enemy_colors(mut enemies: Query<(&Enemy, &mut Sprite)>) {
    for (enemy, mut sprite) in &mut enemies {
        let health_ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
        sprite.color = enemy_color(enemy.kind, health_ratio);
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

fn apply_wave_start_stats(game: &mut Game, stats: &PlayerStats) {
    if stats.regeneration > 0 {
        game.lives = (game.lives + stats.regeneration).min(stats.max_hp);
    }

    if stats.passive_income > 0 {
        game.money += stats.passive_income;
    }
}
