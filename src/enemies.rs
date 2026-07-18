use bevy::prelude::*;

use crate::components::{
    DropsSpell, Enemy, EnemyKind, Health, HealthBar, PathProgress, Reward, Speed,
    TemporaryEnemySpeed, Waypoint,
};
use crate::resources::{
    CurrentHp, EnemiesRemaining, ForcedTowerOffers, GameOver, GameWon, MaxHp, NewRoundEvent,
    PathTiles, Regeneration, SpawnTimer, TowerDraft, TowerDraftPhase, WaveNumber
};
use crate::spell_definitions::SlowActive;
use crate::waves::{RunMode, wave};

pub fn reset_temporary_enemy_speed(mut enemies: Query<&mut TemporaryEnemySpeed, With<Enemy>>) {
    for mut temp in &mut enemies {
        temp.reset();
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave_number: ResMut<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_over: Res<GameOver>,
    mut game_won: ResMut<GameWon>,
    mut hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
    regeneration: Res<Regeneration>,
    run_mode: Res<RunMode>,
    path_tiles: Res<PathTiles>,
    mut draft: ResMut<TowerDraft>,
    mut forced_towers: ResMut<ForcedTowerOffers>,
    enemies: Query<(), With<Enemy>>,
    mut new_round_events: EventWriter<NewRoundEvent>,
) {
    if game_over.value || game_won.value {
        return;
    }

    if remaining.count == 0 {
        if enemies.is_empty() && draft.phase == TowerDraftPhase::WaveRunning {
            new_round_events.write(NewRoundEvent);

            if wave_number.value >= run_mode.final_wave() {
                game_won.value = true;
                return;
            }

            wave_number.value += 1;
            let regen = regeneration.value().round() as i32;
            if regen > 0 {
                hp.amount = (hp.amount + regen).min(max_hp.value().round() as i32);
            }
            draft.activate(&mut forced_towers);
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
            spawn_enemy(&mut commands, group.kind, wave_number.value, &path_tiles);
            spawned += 1;
            remaining.count -= 1;
        }
        spawn_timer.set_spawned_in_group(group_index, spawned);
    }
}

fn spawn_enemy(commands: &mut Commands, kind: EnemyKind, wave_number: u32, path_tiles: &PathTiles) {
    let max_health = kind.max_health(wave_number);
    let size = kind.size();
    let mut spawner = commands.spawn((
        Sprite::from_color(enemy_color(kind, 1.0), size),
        Transform::from_translation(path_tiles.start().extend(3.0)),
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
            amount: kind.reward(),
        },
        TemporaryEnemySpeed::default(),
    ));

    if matches!(kind, EnemyKind::Titan) {
        spawner.insert(DropsSpell);
    }

    let enemy = spawner.id();
    spawn_health_bar(commands, enemy, size);
}

pub fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut hp: ResMut<CurrentHp>,
    mut game_over: ResMut<GameOver>,
    slow: Res<SlowActive>,
    path_tiles: Res<PathTiles>,
    mut enemies: Query<
        (
            Entity,
            &mut Transform,
            &mut Waypoint,
            &mut PathProgress,
            &Health,
            &Speed,
            &TemporaryEnemySpeed,
        ),
        With<Enemy>,
    >,
) {
    if game_over.value {
        return;
    }

    for (entity, mut transform, mut waypoint, mut progress, health, speed, temp_speed) in &mut enemies {
        if health.current <= 0.0 {
            continue;
        }

        let Some(target) = path_tiles.tiles.get(waypoint.index).copied() else {
            commands.entity(entity).despawn();
            hp.amount -= 1;
            if hp.amount <= 0 {
                game_over.value = true;
            }
            continue;
        };
        let position = transform.translation.truncate();
        let to_target = target - position;
        let step = speed.value * temp_speed.multiplier * slow.multiplier * time.delta_secs();
        progress.distance += step;

        if to_target.length() <= step {
            transform.translation = target.extend(3.0);
            waypoint.index += 1;
            if waypoint.index >= path_tiles.tiles.len() {
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

