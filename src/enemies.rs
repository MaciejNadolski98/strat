use bevy::prelude::*;

use crate::components::{
    Enemy, EnemyKind, Health, HealthBar, PathProgress, Reward, Speed, Waypoint,
};
use crate::constants::PATH;
use crate::resources::{
    ActiveSpellEffects, CurrentHp, EnemiesRemaining, GameOver, GameWon, MaxHp, NextWaveTimer,
    Regeneration, SpawnTimer, WaveNumber,
};
use crate::waves::{RunMode, enemies_in_wave, wave};

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_number: ResMut<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut next_wave_timer: ResMut<NextWaveTimer>,
    game_over: Res<GameOver>,
    mut game_won: ResMut<GameWon>,
    mut hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
    regeneration: Res<Regeneration>,
    run_mode: Res<RunMode>,
    mut active_spell_effects: ResMut<ActiveSpellEffects>,
    enemies: Query<(), With<Enemy>>,
) {
    if game_over.value || game_won.value {
        return;
    }

    if remaining.count == 0 {
        if enemies.is_empty() {
            active_spell_effects.reset_for_wave();

            if wave_number.value >= run_mode.final_wave() {
                game_won.value = true;
                return;
            }

            next_wave_timer.timer.tick(time.delta());
            if next_wave_timer.timer.just_finished() {
                wave_number.value += 1;
                remaining.count = enemies_in_wave(wave_number.value);
                spawn_timer.reset();
                next_wave_timer.timer.reset();
                apply_wave_start_stats(&mut hp, &max_hp, &regeneration);
            }
        }
        return;
    }

    let Some(current_wave) = wave(wave_number.value) else {
        remaining.count = 0;
        return;
    };

    spawn_timer.elapsed += time.delta_secs();

    for (group_index, group) in current_wave.groups.iter().enumerate() {
        let mut spawned = spawn_timer.spawned_in_group(group_index);
        while spawned < group.count && spawn_timer.elapsed >= group.spawn_time(spawned) {
            spawn_enemy(&mut commands, group.kind, wave_number.value);
            spawned += 1;
            remaining.count -= 1;
        }
        spawn_timer.set_spawned_in_group(group_index, spawned);
    }
}

fn spawn_enemy(commands: &mut Commands, kind: EnemyKind, wave_number: u32) {
    let max_health = kind.max_health(wave_number);
    let size = kind.size();
    let enemy = commands
        .spawn((
            Sprite::from_color(enemy_color(kind, 1.0), size),
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
                value: kind.speed(wave_number),
            },
            Reward {
                amount: kind.reward(wave_number),
            },
        ))
        .id();

    spawn_health_bar(commands, enemy, size);
}

pub fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut hp: ResMut<CurrentHp>,
    mut game_over: ResMut<GameOver>,
    active_spell_effects: Res<ActiveSpellEffects>,
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
        let step = speed.value * active_spell_effects.enemy_speed_multiplier * time.delta_secs();
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

pub fn update_enemy_health_bars(
    enemies: Query<&Health, With<Enemy>>,
    mut bars: Query<(&HealthBar, &mut Transform, &mut Sprite, &mut Visibility)>,
) {
    for (bar, mut transform, mut sprite, mut visibility) in &mut bars {
        let Ok(health) = enemies.get(bar.owner) else {
            continue;
        };

        let health_ratio = (health.current / health.max).clamp(0.0, 1.0);
        *visibility = if health_ratio >= 1.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };

        if bar.fill {
            transform.scale.x = health_ratio;
            transform.translation.x = -bar.width * (1.0 - health_ratio) * 0.5;
            sprite.color = Color::srgb(1.0 - health_ratio * 0.2, 0.18 + health_ratio * 0.78, 0.16);
        }
    }
}

fn enemy_color(kind: EnemyKind, health_ratio: f32) -> Color {
    let (damaged, healthy) = kind.colors();
    Color::srgb(
        damaged.0 + (healthy.0 - damaged.0) * health_ratio,
        damaged.1 + (healthy.1 - damaged.1) * health_ratio,
        damaged.2 + (healthy.2 - damaged.2) * health_ratio,
    )
}

fn spawn_health_bar(commands: &mut Commands, enemy: Entity, enemy_size: Vec2) {
    let width = enemy_size.x + 12.0;
    let height = 5.0;
    let y = enemy_size.y * 0.5 + 8.0;

    commands.entity(enemy).with_children(|parent| {
        parent.spawn((
            Sprite::from_color(
                Color::srgb(0.05, 0.06, 0.06),
                Vec2::new(width + 2.0, height + 2.0),
            ),
            Transform::from_translation(Vec3::new(0.0, y, 1.0)),
            Visibility::Hidden,
            HealthBar {
                owner: enemy,
                width,
                fill: false,
            },
        ));
        parent.spawn((
            Sprite::from_color(Color::srgb(0.80, 0.96, 0.16), Vec2::new(width, height)),
            Transform::from_translation(Vec3::new(0.0, y, 2.0)),
            Visibility::Hidden,
            HealthBar {
                owner: enemy,
                width,
                fill: true,
            },
        ));
    });
}

fn apply_wave_start_stats(hp: &mut CurrentHp, max_hp: &MaxHp, regeneration: &Regeneration) {
    if regeneration.amount > 0 {
        hp.amount = (hp.amount + regeneration.amount).min(max_hp.amount);
    }
}
