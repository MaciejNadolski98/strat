use bevy::prelude::*;

use crate::components::{
    Damage, Enemy, ExplosionRadius, Health, Projectile, Reward, Speed, Target,
};
use crate::resources::Game;

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &Target,
            &Speed,
            &Damage,
            &ExplosionRadius,
        ),
        (With<Projectile>, Without<Enemy>),
    >,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward), With<Enemy>>,
) {
    for (
        projectile_entity,
        mut projectile_transform,
        target,
        projectile_speed,
        damage,
        explosion_radius,
    ) in &mut projectiles
    {
        let Ok((_, enemy_transform, health, _)) = enemies.get(target.entity) else {
            commands.entity(projectile_entity).despawn();
            continue;
        };

        if health.current <= 0.0 {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        let projectile_position = projectile_transform.translation.truncate();
        let enemy_position = enemy_transform.translation.truncate();
        let to_enemy = enemy_position - projectile_position;
        let step = projectile_speed.value * time.delta_secs();

        if to_enemy.length() <= step + 10.0 {
            let impact_position = enemy_position;
            let mut killed = Vec::new();

            if let Ok((entity, _, mut health, reward)) = enemies.get_mut(target.entity) {
                health.current -= damage.amount;
                if health.current <= 0.0 {
                    killed.push((entity, reward.amount));
                }
            }

            if explosion_radius.value > 0.0 {
                for (entity, transform, mut health, reward) in &mut enemies {
                    if entity == target.entity || health.current <= 0.0 {
                        continue;
                    }

                    let distance = transform.translation.truncate().distance(impact_position);
                    if distance <= explosion_radius.value {
                        health.current -= damage.amount * 0.5;
                        if health.current <= 0.0 {
                            killed.push((entity, reward.amount));
                        }
                    }
                }
            }

            commands.entity(projectile_entity).despawn();

            for (entity, reward) in killed {
                game.money += reward;
                game.kills += 1;
                commands.entity(entity).despawn();
            }
        } else {
            projectile_transform.translation += (to_enemy.normalize() * step).extend(0.0);
        }
    }
}

pub fn projectile_color(is_critical: bool) -> Color {
    if is_critical {
        Color::srgb(1.0, 0.42, 0.16)
    } else {
        Color::srgb(0.96, 0.84, 0.28)
    }
}
