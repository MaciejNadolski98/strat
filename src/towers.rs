use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    AngularSpeed, Damage, DamageFormula, Enemy, ExplosionRadius, FireCooldown, Health, IsCritical,
    PathProgress, Projectile, Speed, Target, Tower, TowerKind,
};
use crate::constants::GRID_SIZE;
use crate::effects::spawn_floating_text;
use crate::pathing::{is_buildable_cell, snap_to_grid};
use crate::projectiles::projectile_color;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, EarthDamage, ExplosionSize, FireDamage, GameOver,
    Money, Shop, WaterDamage,
};

pub fn place_tower(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<&Transform, With<Tower>>,
    mut money: ResMut<Money>,
    game_over: Res<GameOver>,
    attack_speed: Res<AttackSpeed>,
    mut shop: ResMut<Shop>,
) {
    let Some(offer) = shop.selected_offer() else {
        return;
    };

    if game_over.value || !mouse.just_pressed(MouseButton::Left) || money.amount < offer.cost {
        return;
    }

    let Some(tower_kind) = offer.item.tower_kind() else {
        return;
    };

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

    money.amount -= offer.cost;
    shop.take_selected_offer();

    spawn_floating_text(
        &mut commands,
        format!("-${}", offer.cost),
        grid_position + Vec2::new(-30.0, 32.0),
        Color::srgb(1.0, 0.86, 0.20),
        20.0,
    );

    commands
        .spawn((
            Sprite::from_color(tower_kind.base_color(), tower_kind.base_size()),
            Transform::from_translation(grid_position.extend(2.0)),
            Tower,
            tower_kind,
            tower_kind.damage_formula(),
            FireCooldown {
                timer: Timer::new(
                    Duration::from_secs_f32(tower_kind.cooldown() / attack_speed.value.max(0.1)),
                    TimerMode::Once,
                ),
            },
            AngularSpeed {
                value: tower_kind.rotational_speed(),
            },
        ))
        .with_child((
            Sprite::from_color(tower_kind.barrel_color(), tower_kind.barrel_size()),
            Transform::from_translation(Vec3::new(0.0, tower_kind.barrel_offset(), 1.0)),
        ));
}

pub fn progress_cooldown(
    mut towers: Query<(&TowerKind, &mut FireCooldown), With<Tower>>,
    time: Res<Time>,
    attack_speed: Res<AttackSpeed>,
) {
    let delta = time.delta();
    for (kind, mut cooldown) in &mut towers {
        cooldown.timer.set_duration(Duration::from_secs_f32(
            kind.cooldown() / attack_speed.value.max(0.1),
        ));
        cooldown.timer.tick(delta);
    }
}

pub fn aim_towers(
    mut commands: Commands,
    mut towers: Query<
        (
            &mut Transform,
            &TowerKind,
            &DamageFormula,
            &mut FireCooldown,
            &AngularSpeed,
        ),
        With<Tower>,
    >,
    enemies: Query<(Entity, &Transform, &Health, &PathProgress), (With<Enemy>, Without<Tower>)>,
    game_over: Res<GameOver>,
    time: Res<Time>,
    critical_chance: Res<CriticalChance>,
    explosion_size: Res<ExplosionSize>,
    earth_damage: Res<EarthDamage>,
    fire_damage: Res<FireDamage>,
    air_damage: Res<AirDamage>,
    water_damage: Res<WaterDamage>,
) {
    if game_over.value {
        return;
    }

    for (mut tower_transform, tower_kind, damage_formula, mut cooldown, rotation_speed) in
        &mut towers
    {
        let tower_position = tower_transform.translation.truncate();
        let Some((target, target_position)) = enemies
            .iter()
            .filter(|(_, _, health, _)| health.current > 0.0)
            .filter_map(|(entity, transform, _, progress)| {
                let enemy_position = transform.translation.truncate();
                let distance = enemy_position.distance(tower_position);
                (distance <= tower_kind.range()).then_some((
                    entity,
                    enemy_position,
                    progress.distance,
                ))
            })
            .max_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(entity, position, _)| (entity, position))
        else {
            continue;
        };

        let direction = target_position - tower_position;
        let target_rotation = Quat::from_rotation_z(direction.y.atan2(direction.x) - PI / 2.0);
        let current_rotation = tower_transform.rotation;
        let step = time.delta_secs() * rotation_speed.value;

        let ready_to_shoot = current_rotation.angle_between(target_rotation) <= step;
        tower_transform.rotation = tower_transform
            .rotation
            .rotate_towards(target_rotation, step);

        if ready_to_shoot && cooldown.timer.finished() {
            let is_critical = roll_critical_hit(critical_chance.value);
            let damage = damage_formula.calculate_damage(
                &earth_damage,
                &fire_damage,
                &air_damage,
                &water_damage,
                is_critical,
            ) as f32;

            cooldown.timer.reset();
            commands.spawn((
                Projectile,
                Sprite::from_color(
                    projectile_color(is_critical),
                    if is_critical {
                        Vec2::new(13.0, 13.0)
                    } else {
                        Vec2::new(10.0, 10.0)
                    },
                ),
                Transform::from_translation(tower_position.extend(4.0)),
                Target { entity: target },
                Speed {
                    value: tower_kind.projectile_speed(),
                },
                Damage { amount: damage },
                IsCritical { value: is_critical },
                ExplosionRadius {
                    value: explosion_size.value + tower_kind.explosion_radius(),
                },
            ));
        }
    }
}

fn roll_critical_hit(critical_chance: f32) -> bool {
    rand::random::<f32>() < critical_chance.clamp(0.0, 1.0)
}
