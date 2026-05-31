use bevy::prelude::*;

use crate::components::{Enemy, Projectile};
use crate::resources::Game;

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile), Without<Enemy>>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    for (projectile_entity, mut projectile_transform, projectile) in &mut projectiles {
        let Ok((_, enemy_transform, enemy)) = enemies.get(projectile.target) else {
            commands.entity(projectile_entity).despawn();
            continue;
        };

        if enemy.health <= 0.0 {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        let projectile_position = projectile_transform.translation.truncate();
        let enemy_position = enemy_transform.translation.truncate();
        let to_enemy = enemy_position - projectile_position;
        let step = projectile.speed * time.delta_secs();

        if to_enemy.length() <= step + 10.0 {
            let impact_position = enemy_position;
            let mut killed = Vec::new();

            if let Ok((entity, _, mut enemy)) = enemies.get_mut(projectile.target) {
                enemy.health -= projectile.damage;
                if enemy.health <= 0.0 {
                    killed.push((entity, enemy.reward));
                }
            }

            if projectile.explosion_radius > 0.0 {
                for (entity, transform, mut enemy) in &mut enemies {
                    if entity == projectile.target || enemy.health <= 0.0 {
                        continue;
                    }

                    let distance = transform.translation.truncate().distance(impact_position);
                    if distance <= projectile.explosion_radius {
                        enemy.health -= projectile.damage * 0.5;
                        if enemy.health <= 0.0 {
                            killed.push((entity, enemy.reward));
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
