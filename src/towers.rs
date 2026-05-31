use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{Enemy, Projectile, Tower};
use crate::constants::{GRID_SIZE, TOWER_COST, TOWER_RANGE};
use crate::pathing::{is_buildable_cell, snap_to_grid};
use crate::projectiles::projectile_color;
use crate::resources::{Game, PassiveIncomeClock, PlayerStats};

pub fn place_tower(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<&Transform, With<Tower>>,
    mut game: ResMut<Game>,
    stats: Res<PlayerStats>,
) {
    if game.game_over || !mouse.just_pressed(MouseButton::Left) || game.money < TOWER_COST {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };
    let grid_position = snap_to_grid(world_position);

    if !is_buildable_cell(grid_position)
        || towers
            .iter()
            .any(|tower| tower.translation.truncate().distance(grid_position) < GRID_SIZE * 0.5)
    {
        return;
    }

    game.money -= TOWER_COST;
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.22, 0.42, 0.74), Vec2::new(36.0, 36.0)),
            Transform::from_translation(grid_position.extend(2.0)),
            Tower {
                fire_cooldown: Timer::new(stats.tower_cooldown(), TimerMode::Once),
                rotational_speed: 1.5,
            },
        ))
        .with_child((
            Sprite::from_color(Color::srgb(0.67, 0.83, 0.96), Vec2::new(12.0, 38.0)),
            Transform::from_translation(Vec3::new(0.0, 16.0, 1.0)),
        ));
}

pub fn progress_cooldown(towers: Query<&mut Tower>, time: Res<Time>) {
    let delta = time.delta();
    for mut tower in towers {
        tower.fire_cooldown.tick(delta);
    }
}

pub fn apply_passive_income(
    time: Res<Time>,
    stats: Res<PlayerStats>,
    mut game: ResMut<Game>,
    mut income: ResMut<PassiveIncomeClock>,
) {
    if game.game_over || stats.passive_income <= 0 {
        return;
    }

    income.timer.tick(time.delta());
    if income.timer.just_finished() {
        game.money += stats.passive_income;
    }
}

pub fn aim_towers(
    mut commands: Commands,
    mut towers: Query<(&mut Transform, &mut Tower)>,
    enemies: Query<(Entity, &Transform, &Enemy), Without<Tower>>,
    game: Res<Game>,
    time: Res<Time>,
    stats: Res<PlayerStats>,
) {
    if game.game_over {
        return;
    }

    for (mut tower_transform, mut tower) in &mut towers {
        let tower_position = tower_transform.translation.truncate();
        let Some((target, target_position)) = enemies
            .iter()
            .filter(|(_, _, enemy)| enemy.health > 0.0)
            .filter_map(|(entity, transform, enemy)| {
                let enemy_position = transform.translation.truncate();
                let distance = enemy_position.distance(tower_position);
                let progress = enemy.progress;
                (distance <= TOWER_RANGE).then_some((entity, enemy_position, progress))
            })
            .max_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(entity, position, _)| (entity, position))
        else {
            continue;
        };

        let direction = target_position - tower_position;
        let target_rotation = Quat::from_rotation_z(direction.y.atan2(direction.x) - PI / 2.0);
        let current_rotation = tower_transform.rotation;
        let step = time.delta_secs() * tower.rotational_speed;

        let ready_to_shoot = current_rotation.angle_between(target_rotation) <= step;
        tower_transform.rotation = tower_transform
            .rotation
            .rotate_towards(target_rotation, step);

        if ready_to_shoot && tower.fire_cooldown.finished() {
            let is_critical = roll_critical_hit(stats.critical_chance);
            let damage = if is_critical {
                stats.projectile_damage() * 2.0
            } else {
                stats.projectile_damage()
            };

            tower.fire_cooldown.set_duration(stats.tower_cooldown());
            tower.fire_cooldown.reset();
            commands.spawn((
                Sprite::from_color(
                    projectile_color(is_critical),
                    if is_critical {
                        Vec2::new(13.0, 13.0)
                    } else {
                        Vec2::new(10.0, 10.0)
                    },
                ),
                Transform::from_translation(tower_position.extend(4.0)),
                Projectile {
                    target,
                    speed: 430.0,
                    damage,
                    explosion_radius: stats.explosion_size,
                },
            ));
        }
    }
}

fn roll_critical_hit(critical_chance: f32) -> bool {
    rand::random::<f32>() < critical_chance.clamp(0.0, 1.0)
}
