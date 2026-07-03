use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    AngularSpeed, AuraTower, CustomTooltip, Damage, DamageFormula, DraftSlot, Enemy,
    ExplosionRadius, FireCooldown, Health, IsCritical, PathProgress, Projectile, ShopTooltip,
    SourceTower, Speed, Target, TemporaryAttackSpeed, TemporaryDamageBonus, Tower,
    TowerRangeIndicator,
};
use crate::projectiles::projectile_color;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, EarthDamage, ExplosionSize,
    FireDamage, GameOver, ShootEvent, TowerDraft, TowerDraftPhase, WaterDamage,
};
use crate::shop::PlayerStatsMut;
use crate::tower_definitions::TowerKind;

#[derive(SystemParam)]
pub struct TowerTooltipStats<'w> {
    attack_speed: Res<'w, AttackSpeed>,
    critical_chance: Res<'w, CriticalChance>,
    explosion_size: Res<'w, ExplosionSize>,
    earth_damage: Res<'w, EarthDamage>,
    fire_damage: Res<'w, FireDamage>,
    air_damage: Res<'w, AirDamage>,
    water_damage: Res<'w, WaterDamage>,
}


pub fn apply_tower_effects(kind: TowerKind, stats: &mut PlayerStatsMut) {
    for effect in kind.stat_effects() {
        stats.apply_tower_effect(*effect);
    }
}

pub fn update_tower_tooltip(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<(
        &TowerKind,
        &DamageFormula,
        &Transform,
        Option<&CustomTooltip>,
        Option<&TemporaryAttackSpeed>,
        Option<&TemporaryDamageBonus>,
    ), With<Tower>>,
    stats: TowerTooltipStats,
    mut tooltip: Query<(&mut Text, &mut Visibility), With<ShopTooltip>>,
) {
    let Ok((mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
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

    let hovered_tower = towers
        .iter()
        .filter_map(|(kind, damage_formula, transform, custom, temp_speed, temp_damage)| {
            let tower_position = transform.translation.truncate();
            let half_size = kind.base_size() * 0.5;
            let inside_tower = (world_position.x - tower_position.x).abs() <= half_size.x
                && (world_position.y - tower_position.y).abs() <= half_size.y;

            inside_tower.then_some((kind, damage_formula, transform.translation.z, custom, temp_speed, temp_damage))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2));

    let Some((kind, damage_formula, _, custom, temp_speed, temp_damage)) = hovered_tower else {
        return;
    };

    let temp_speed_val = temp_speed.map(|ts| ts.bonus).unwrap_or(0.0);
    let temp_damage_val = temp_damage.map(|td| td.flat).unwrap_or(0.0);

    let base = tower_tooltip(*kind, damage_formula, &stats, temp_damage_val, temp_speed_val);
    let text = match custom {
        Some(ct) if !ct.0.is_empty() => format!("{base}\n{}", ct.0),
        _ => base,
    };

    tooltip_text.0 = text;
    *tooltip_visibility = Visibility::Visible;
}

pub fn tower_tooltip(
    kind: TowerKind,
    damage_formula: &DamageFormula,
    stats: &TowerTooltipStats,
    temp_flat_damage: f32,
    temp_speed_bonus: f32,
) -> String {
    let config = kind.definition().tooltip_config;
    let effective_speed = (stats.attack_speed.value() + temp_speed_bonus).max(0.1);

    let mut lines = vec![kind.name().to_string()];

    if config.show_damage {
        let regular_damage = (damage_formula.calculate_damage_with_elemental_multiplier(
            &stats.earth_damage, &stats.fire_damage, &stats.air_damage, &stats.water_damage,
            false,
        ) + temp_flat_damage).max(1.0);
        let critical_damage = (damage_formula.calculate_damage_with_elemental_multiplier(
            &stats.earth_damage, &stats.fire_damage, &stats.air_damage, &stats.water_damage,
            true,
        ) + temp_flat_damage).max(1.0);

        let mut active_parts: Vec<String> = Vec::new();
        if temp_flat_damage > 0.01 { active_parts.push(format!("+{temp_flat_damage:.1} dmg")); }
        if temp_speed_bonus > 0.01 { active_parts.push(format!("+{temp_speed_bonus:.2}x spd")); }
        let active_suffix = if active_parts.is_empty() {
            String::new()
        } else {
            format!(" [Active: {}]", active_parts.join(", "))
        };

        lines.push(format!("Damage: {regular_damage:.0} ({critical_damage:.0} crit)"));
        lines.push(format!("Formula: {damage_formula}{active_suffix}"));
    }

    if config.show_range {
        lines.push(format!("Range: {:.0}", kind.range()));
    }
    if config.show_cooldown {
        lines.push(format!("Cooldown: {:.2}s", kind.cooldown() / effective_speed));
    }
    if config.show_crit {
        lines.push(format!("Crit: {:.0}%", stats.critical_chance.value() * 100.0));
    }
    if config.show_projectile {
        lines.push(format!("Projectile: {:.0}/s", kind.projectile_speed()));
    }
    if config.show_splash {
        lines.push(format!("Splash: {:.0}", kind.upgraded_explosion_radius(stats.explosion_size.value().max(0.0))));
    }
    if config.show_turn_speed {
        lines.push(format!("Turn speed: {:.1}", kind.angular_speed()));
    }

    let stat_effects = kind.stat_effects();
    if !stat_effects.is_empty() {
        lines.push("Stat effects:".to_string());
        for effect in stat_effects {
            lines.push(effect.effect_text());
        }
    }

    lines.join("\n")
}

pub fn reset_temporary_attack_speed(mut towers: Query<&mut TemporaryAttackSpeed, With<Tower>>) {
    for mut temp in &mut towers {
        temp.bonus = 0.0;
    }
}

pub fn reset_temporary_damage_bonus(mut towers: Query<&mut TemporaryDamageBonus, With<Tower>>) {
    for mut temp in &mut towers {
        temp.flat = 0.0;
    }
}

pub fn progress_cooldown(
    mut towers: Query<(&mut FireCooldown, &TemporaryAttackSpeed), With<Tower>>,
    time: Res<Time>,
    attack_speed: Res<AttackSpeed>,
) {
    let delta = time.delta();
    for (mut cooldown, temp_speed) in &mut towers {
        let base_cooldown = cooldown.base_cooldown;
        let effective_speed = (attack_speed.value() + temp_speed.bonus).max(0.1);
        cooldown.timer.set_duration(Duration::from_secs_f32(
            base_cooldown / effective_speed,
        ));
        cooldown.timer.tick(delta);
    }
}

pub fn aim_towers(
    mut commands: Commands,
    mut towers: Query<
        (
            Entity,
            &mut Transform,
            &TowerKind,
            &DamageFormula,
            &mut FireCooldown,
            &AngularSpeed,
            &TemporaryDamageBonus,
        ),
        (With<Tower>, Without<AuraTower>),
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
    mut shoot_events: EventWriter<ShootEvent>,
) {
    if game_over.value {
        return;
    }

    for (
        tower_entity,
        mut tower_transform,
        tower_kind,
        damage_formula,
        mut cooldown,
        rotation_speed,
        damage_bonus,
    ) in &mut towers
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
            let is_critical = roll_critical_hit(critical_chance.value());
            let damage = (damage_formula.calculate_damage_with_elemental_multiplier(
                &earth_damage,
                &fire_damage,
                &air_damage,
                &water_damage,
                is_critical,
            ) + damage_bonus.flat).max(1.0);

            cooldown.timer.reset();
            shoot_events.write(ShootEvent { source_tower: tower_entity });
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
                SourceTower {
                    entity: tower_entity,
                },
                Speed {
                    value: tower_kind.projectile_speed(),
                },
                Damage { amount: damage },
                IsCritical { value: is_critical },
                ExplosionRadius {
                    value: tower_kind.upgraded_explosion_radius(explosion_size.value().max(0.0)),
                },
            ));
        }
    }
}

fn roll_critical_hit(critical_chance: f32) -> bool {
    rand::random::<f32>() < critical_chance.clamp(0.0, 1.0)
}

pub fn update_tower_range_indicator(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<(&TowerKind, &Transform), With<Tower>>,
    mut indicator: Query<
        (&mut Transform, &mut Visibility),
        (With<TowerRangeIndicator>, Without<Tower>),
    >,
) {
    let Ok((mut indicator_transform, mut indicator_visibility)) = indicator.single_mut() else {
        return;
    };

    *indicator_visibility = Visibility::Hidden;

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

    let hovered_tower = towers
        .iter()
        .filter_map(|(kind, transform)| {
            let tower_position = transform.translation.truncate();
            let half_size = kind.base_size() * 0.5;
            let inside_tower = (world_position.x - tower_position.x).abs() <= half_size.x
                && (world_position.y - tower_position.y).abs() <= half_size.y;
            inside_tower.then_some((kind, transform))
        })
        .max_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));

    let Some((kind, tower_transform)) = hovered_tower else {
        return;
    };

    indicator_transform.translation = tower_transform.translation.truncate().extend(1.5);
    indicator_transform.scale = Vec3::splat(kind.range());
    *indicator_visibility = Visibility::Visible;
}

pub fn update_draft_tooltip(
    draft: Res<TowerDraft>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    slots: Query<(&DraftSlot, &Transform)>,
    stats: TowerTooltipStats,
    custom_tooltips: Res<crate::tower_definitions::CustomTooltipTexts>,
    mut tooltip: Query<(&mut Text, &mut Visibility), With<ShopTooltip>>,
) {
    if draft.phase != TowerDraftPhase::Picking {
        return;
    }

    let Ok((mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };
    *tooltip_visibility = Visibility::Hidden;

    let Some(cursor_world) = (|| -> Option<Vec2> {
        let window = windows.single().ok()?;
        let (cam, cam_t) = camera.single().ok()?;
        cam.viewport_to_world_2d(cam_t, window.cursor_position()?).ok()
    })() else {
        return;
    };

    for (slot, transform) in &slots {
        let pos = transform.translation.truncate();
        if (cursor_world.x - pos.x).abs() <= 65.0 && (cursor_world.y - pos.y).abs() <= 70.0 {
            let kind = draft.offers[slot.index];
            let damage_formula = kind.damage_formula();
            let base = tower_tooltip(kind, &damage_formula, &stats, 0.0, 0.0);
            tooltip_text.0 = match custom_tooltips.0.get(&kind).filter(|s| !s.is_empty()) {
                Some(extras) => format!("{base}\n{extras}"),
                None => base,
            };
            *tooltip_visibility = Visibility::Visible;
            return;
        }
    }
}
