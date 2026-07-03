use bevy::prelude::*;

use crate::components::{
    Damage, DamageDealt, DropsSpell, Enemy, ExplosionRadius, Health, IsCritical, Projectile,
    Reward, SourceTower, Speed, Target, Tower,
};
use crate::effects::{spawn_explosion_effect, spawn_floating_text};
use crate::resources::{EnemyKilledEvent, KillCount, Money, Loot, SpellShop};

pub fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut money: ResMut<Money>,
    mut kills: ResMut<KillCount>,
    loot: Res<Loot>,
    mut spell_shop: ResMut<SpellShop>,
    mut kill_events: EventWriter<EnemyKilledEvent>,
    mut projectiles: Query<
        (
            Entity,
            &mut Transform,
            &Target,
            &Speed,
            &Damage,
            &IsCritical,
            &ExplosionRadius,
            &SourceTower,
        ),
        (With<Projectile>, Without<Enemy>),
    >,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward, Option<&DropsSpell>), With<Enemy>>,
    mut towers: Query<&mut DamageDealt, With<Tower>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (
        projectile_entity,
        mut projectile_transform,
        target,
        projectile_speed,
        damage,
        is_critical,
        explosion_radius,
        source_tower,
    ) in &mut projectiles
    {
        let Ok((_, enemy_transform, health, _, _)) = enemies.get(target.entity) else {
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
            let mut killed: Vec<(Entity, i32, Vec2, bool)> = Vec::new();
            let mut total_damage_dealt = 0.0;

            if let Ok((entity, _, mut health, reward, drops_spell)) = enemies.get_mut(target.entity) {
                let hp_lost = damage.amount.min(health.current).max(0.0);
                health.current -= damage.amount;
                if hp_lost > 0.0 {
                    total_damage_dealt += hp_lost;
                    spawn_damage_text(&mut commands, impact_position, hp_lost, is_critical.value);
                }
                if health.current <= 0.0 {
                    killed.push((entity, reward.amount, impact_position, drops_spell.is_some()));
                }
            }

            if explosion_radius.value > 0.0 {
                spawn_explosion_effect(
                    &mut commands,
                    impact_position,
                    explosion_radius.value,
                    &mut meshes,
                    &mut materials,
                );
                for (entity, transform, mut health, reward, drops_spell) in &mut enemies {
                    if entity == target.entity || health.current <= 0.0 {
                        continue;
                    }

                    let distance = transform.translation.truncate().distance(impact_position);
                    if distance <= explosion_radius.value {
                        let splash_damage = damage.amount;
                        let hp_lost = splash_damage.min(health.current).max(0.0);
                        health.current -= splash_damage;
                        if hp_lost > 0.0 {
                            total_damage_dealt += hp_lost;
                            spawn_damage_text(
                                &mut commands,
                                transform.translation.truncate(),
                                hp_lost,
                                is_critical.value,
                            );
                        }
                        if health.current <= 0.0 {
                            killed.push((entity, reward.amount, transform.translation.truncate(), drops_spell.is_some()));
                        }
                    }
                }
            }

            commands.entity(projectile_entity).despawn();

            if total_damage_dealt > 0.0 {
                if let Ok(mut damage_dealt) = towers.get_mut(source_tower.entity) {
                    damage_dealt.amount += total_damage_dealt;
                }
            }

            for (entity, reward, position, drops_spell) in killed {
                let kill_loot = reward + loot.value().round() as i32;
                money.amount += kill_loot;
                kills.amount += 1;
                spawn_money_text(&mut commands, position + Vec2::new(34.0, 30.0), kill_loot);
                if drops_spell {
                    spell_shop.store_random_spell();
                    spawn_floating_text(
                        &mut commands,
                        "Spell!".to_string(),
                        position + Vec2::new(-20.0, 52.0),
                        Color::srgb(0.72, 0.30, 0.92),
                        22.0,
                    );
                }
                commands.entity(entity).despawn();
                kill_events.write(EnemyKilledEvent { source_tower: source_tower.entity });
            }
        } else {
            projectile_transform.translation += (to_enemy.normalize() * step).extend(0.0);
        }
    }
}

fn spawn_damage_text(commands: &mut Commands, position: Vec2, amount: f32, is_critical: bool) {
    spawn_floating_text(
        commands,
        format!("-{:.0}", amount),
        position + Vec2::new(0.0, 20.0),
        if is_critical {
            Color::srgb(1.0, 0.16, 0.12)
        } else {
            Color::srgb(1.0, 1.0, 1.0)
        },
        if is_critical { 23.0 } else { 20.0 },
    );
}

fn spawn_money_text(commands: &mut Commands, position: Vec2, amount: i32) {
    spawn_floating_text(
        commands,
        format!("+${amount}"),
        position,
        Color::srgb(1.0, 0.86, 0.20),
        19.0,
    );
}

pub fn projectile_color(is_critical: bool) -> Color {
    if is_critical {
        Color::srgb(1.0, 0.42, 0.16)
    } else {
        Color::srgb(0.96, 0.84, 0.28)
    }
}
