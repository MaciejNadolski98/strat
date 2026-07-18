use bevy::prelude::*;

use crate::components::{
    Damage, DamageDealt, Direction, DropsSpell, Enemy, ExplosionRadius, Health, IsCritical,
    Pierce, Pierced, PiercingFalloff, Projectile, RemainingRange, Reward, SourceTower, Speed,
    Tower,
};
use crate::effects::{spawn_explosion_effect, spawn_floating_text};
use crate::resources::{EnemyKilledEvent, KillCount, Money, Loot, SpellShop};

const HIT_RADIUS: f32 = 14.0;

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
            &Direction,
            &mut RemainingRange,
            &mut Pierce,
            &PiercingFalloff,
            &mut Pierced,
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
        direction,
        mut remaining_range,
        mut pierce,
        falloff,
        mut pierced,
        projectile_speed,
        damage,
        is_critical,
        explosion_radius,
        source_tower,
    ) in &mut projectiles
    {
        let step = projectile_speed.value * time.delta_secs();
        let projectile_position = projectile_transform.translation.truncate();

        let hit = enemies
            .iter()
            .filter(|(entity, _, health, _, _)| {
                health.current > 0.0 && !pierced.entities.contains(entity)
            })
            .map(|(entity, transform, _, _, _)| {
                let position = transform.translation.truncate();
                (entity, position, position.distance(projectile_position))
            })
            .filter(|(_, _, distance)| *distance <= HIT_RADIUS + step)
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(entity, position, _)| (entity, position));

        let (direct_hit, impact_position, mut should_despawn) = match hit {
            Some((hit_entity, position)) => (Some(hit_entity), position, false),
            None => {
                remaining_range.value -= step;
                if remaining_range.value <= 0.0 {
                    if explosion_radius.value <= 0.0 {
                        commands.entity(projectile_entity).despawn();
                        continue;
                    }
                    (None, projectile_position, true)
                } else {
                    projectile_transform.translation += (direction.value * step).extend(0.0);
                    continue;
                }
            }
        };

        let hits_before_this_one = pierced.entities.len() as i32;
        let hit_damage = damage.amount * (1.0 + falloff.value).max(0.0).powi(hits_before_this_one);

        let mut killed: Vec<(Entity, i32, Vec2, bool)> = Vec::new();
        let mut total_damage_dealt = 0.0;

        if let Some(hit_entity) = direct_hit {
            pierced.entities.push(hit_entity);
            if let Ok((entity, _, mut health, reward, drops_spell)) = enemies.get_mut(hit_entity) {
                let hp_lost = hit_damage.min(health.current).max(0.0);
                health.current -= hit_damage;
                if hp_lost > 0.0 {
                    total_damage_dealt += hp_lost;
                    spawn_damage_text(&mut commands, impact_position, hp_lost, is_critical.value);
                }
                if health.current <= 0.0 {
                    killed.push((entity, reward.amount, impact_position, drops_spell.is_some()));
                }
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
                if Some(entity) == direct_hit || health.current <= 0.0 {
                    continue;
                }

                let distance = transform.translation.truncate().distance(impact_position);
                if distance <= explosion_radius.value {
                    let splash_damage = hit_damage;
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

        if direct_hit.is_some() {
            if pierce.remaining > 0 {
                pierce.remaining -= 1;
            } else {
                should_despawn = true;
            }
        }

        if should_despawn {
            commands.entity(projectile_entity).despawn();
        }

        if total_damage_dealt > 0.0 {
            if let Ok(mut damage_dealt) = towers.get_mut(source_tower.entity) {
                damage_dealt.amount += total_damage_dealt;
            }
        }

        for (entity, reward, position, drops_spell) in killed {
            let kill_loot = (reward + loot.value().round() as i32).max(0);
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
            kill_events.write(EnemyKilledEvent { source_tower: source_tower.entity, position });
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
