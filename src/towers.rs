use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    AngularSpeed, Damage, DamageFormula, DraftSlot, Enemy, ExplosionRadius, FireCooldown, Health,
    IsCritical, PathProgress, Projectile, ShopTooltip, SourceTower, Speed, Target, Tower,
    TowerKind, TowerRangeIndicator,
};
use crate::projectiles::projectile_color;
use crate::resources::{
    ActiveSpellEffects, AirDamage, AttackSpeed, CriticalChance, EarthDamage, ExplosionSize,
    FireDamage, GameOver, TowerDraft, TowerDraftPhase, WaterDamage,
};
use crate::shop::PlayerStatsMut;

#[derive(SystemParam)]
pub struct TowerTooltipStats<'w> {
    attack_speed: Res<'w, AttackSpeed>,
    critical_chance: Res<'w, CriticalChance>,
    explosion_size: Res<'w, ExplosionSize>,
    earth_damage: Res<'w, EarthDamage>,
    fire_damage: Res<'w, FireDamage>,
    air_damage: Res<'w, AirDamage>,
    water_damage: Res<'w, WaterDamage>,
    active_spell_effects: Res<'w, ActiveSpellEffects>,
}


pub fn apply_tower_effects(kind: TowerKind, stats: &mut PlayerStatsMut) {
    for effect in kind.stat_effects() {
        stats.apply_tower_effect(*effect);
    }
}

pub fn update_tower_tooltip(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<(&TowerKind, &DamageFormula, &Transform), With<Tower>>,
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
        .filter_map(|(kind, damage_formula, transform)| {
            let tower_position = transform.translation.truncate();
            let half_size = kind.base_size() * 0.5;
            let inside_tower = (world_position.x - tower_position.x).abs() <= half_size.x
                && (world_position.y - tower_position.y).abs() <= half_size.y;

            inside_tower.then_some((kind, damage_formula, transform.translation.z))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2));

    let Some((kind, damage_formula, _)) = hovered_tower else {
        return;
    };

    tooltip_text.0 = tower_tooltip(*kind, damage_formula, &stats);
    *tooltip_visibility = Visibility::Visible;
}

pub fn tower_tooltip(
    kind: TowerKind,
    damage_formula: &DamageFormula,
    stats: &TowerTooltipStats,
) -> String {
    let elemental_multiplier = stats.active_spell_effects.elemental_multiplier;
    let regular_damage = damage_formula.calculate_damage_with_elemental_multiplier(
        &stats.earth_damage,
        &stats.fire_damage,
        &stats.air_damage,
        &stats.water_damage,
        false,
        elemental_multiplier,
    );
    let critical_damage = damage_formula.calculate_damage_with_elemental_multiplier(
        &stats.earth_damage,
        &stats.fire_damage,
        &stats.air_damage,
        &stats.water_damage,
        true,
        elemental_multiplier,
    );
    let effective_cooldown = kind.cooldown() / stats.attack_speed.value.max(0.1);
    let stat_effects = kind.stat_effects();
    let effect_text = if stat_effects.is_empty() {
        "None".to_string()
    } else {
        stat_effects
            .iter()
            .map(|effect| effect.effect_text())
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "{}\nDamage: {} ({} crit)\nFormula: {}\nRange: {:.0}\nCooldown: {:.2}s\nCrit: {:.0}%\nProjectile: {:.0}/s\nSplash: {:.0}\nTurn speed: {:.1}\nStat effects:\n{}",
        kind.name(),
        regular_damage,
        critical_damage,
        damage_formula,
        kind.range(),
        effective_cooldown,
        stats.critical_chance.value * 100.0,
        kind.projectile_speed(),
        kind.upgraded_explosion_radius(stats.explosion_size.value),
        kind.angular_speed(),
        effect_text,
    )
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
            Entity,
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
    active_spell_effects: Res<ActiveSpellEffects>,
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
            let is_critical = roll_critical_hit(critical_chance.value);
            let damage = damage_formula.calculate_damage_with_elemental_multiplier(
                &earth_damage,
                &fire_damage,
                &air_damage,
                &water_damage,
                is_critical,
                active_spell_effects.elemental_multiplier,
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
                SourceTower {
                    entity: tower_entity,
                },
                Speed {
                    value: tower_kind.projectile_speed(),
                },
                Damage { amount: damage },
                IsCritical { value: is_critical },
                ExplosionRadius {
                    value: tower_kind.upgraded_explosion_radius(explosion_size.value),
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
            tooltip_text.0 = tower_tooltip(kind, &damage_formula, &stats);
            *tooltip_visibility = Visibility::Visible;
            return;
        }
    }
}
