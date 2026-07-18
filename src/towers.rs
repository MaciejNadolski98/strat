use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::system::SystemParam;
use bevy::math::primitives::Ellipse;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy::window::PrimaryWindow;

use crate::components::{
    Aim, AngularSpeed, BeamFire, CustomTooltip, Damage, DamageFormula, DefaultAim, DefaultFire,
    Direction, DraftPreview, DraftSlot, DropsSpell, Enemy, ExplosionRadius, FireCooldown, Health,
    IsCritical, PathProgress, Pierce, Pierced, PiercingFalloff, Projectile, TemporaryRange,
    RemainingRange, Reward, ShopTooltip, SourceTower, Speed, TemporaryAttackSpeed,
    TemporaryDamageBonus, Tower, TowerRangeIndicator,
};
use crate::effects::{spawn_beam_effect, spawn_floating_text};
use crate::projectiles::projectile_color;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, EarthDamage, EnemyKilledEvent, ExplosionSize,
    FireDamage, GameOver, KillCount, Loot, Money, Piercing, PiercingDamage, PlayerStatKind,
    ShootEvent, SpellShop, TowerDraft, TowerDraftPhase, WaterDamage,
};
use crate::shop::PlayerStatsMut;
use crate::tooltip::{colored, plain, tag_segments, Segment};
use crate::tower_definitions::TowerKind;

const BEAM_HALF_WIDTH: f32 = 10.0;

#[derive(SystemParam)]
pub struct TowerTooltipStats<'w> {
    attack_speed: Res<'w, AttackSpeed>,
    critical_chance: Res<'w, CriticalChance>,
    explosion_size: Res<'w, ExplosionSize>,
    earth_damage: Res<'w, EarthDamage>,
    fire_damage: Res<'w, FireDamage>,
    air_damage: Res<'w, AirDamage>,
    water_damage: Res<'w, WaterDamage>,
    piercing: Res<'w, Piercing>,
    piercing_damage: Res<'w, PiercingDamage>,
}

fn effective_piercing(tower_piercing: u32, global_piercing: f32) -> u32 {
    (tower_piercing as f32 + global_piercing).max(0.0).round() as u32
}

fn effective_piercing_falloff(tower_piercing_damage: f32, global_piercing_damage: f32) -> f32 {
    (tower_piercing_damage + global_piercing_damage).clamp(-1.0, 0.0)
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
        Option<&TemporaryRange>,
    ), With<Tower>>,
    stats: TowerTooltipStats,
    mut tooltip: Query<(Entity, &mut Text, &mut Visibility), With<ShopTooltip>>,
    mut commands: Commands,
) {
    let Ok((tooltip_entity, mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
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
        .filter_map(|(kind, damage_formula, transform, custom, temp_speed, temp_damage, temp_range)| {
            let tower_position = transform.translation.truncate();
            let half_size = kind.base_size() * 0.5;
            let inside_tower = (world_position.x - tower_position.x).abs() <= half_size.x
                && (world_position.y - tower_position.y).abs() <= half_size.y;

            inside_tower.then_some((kind, damage_formula, transform.translation.z, custom, temp_speed, temp_damage, temp_range))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2));

    let Some((kind, damage_formula, _, custom, temp_speed, temp_damage, temp_range)) = hovered_tower else {
        return;
    };

    let temp_speed_val = temp_speed.map(|ts| ts.flat).unwrap_or(0.0);
    let temp_damage_val = temp_damage.map(|td| td.flat).unwrap_or(0.0);
    let range_multiplier = temp_range.map(|tr| tr.multiplier).unwrap_or(1.0);

    let mut segments = tower_tooltip(*kind, &Some(*damage_formula), &stats, temp_damage_val, temp_speed_val, range_multiplier);
    if let Some(ct) = custom {
        if !ct.0.is_empty() {
            segments.push(plain("\n"));
            segments.extend(ct.0.iter().cloned());
        }
    }

    let tags = kind.tags();
    if !tags.is_empty() {
        segments.push(plain("\nTags: "));
        segments.extend(tag_segments(tags));
    }
    crate::tooltip::set_tooltip_segments(&mut commands, tooltip_entity, &mut tooltip_text, segments);
    *tooltip_visibility = Visibility::Visible;
}

pub const EARTH_COLOR: Color = Color::srgb(0.62, 0.46, 0.28);
pub const FIRE_COLOR: Color = Color::srgb(0.95, 0.45, 0.15);
pub const AIR_COLOR: Color = Color::srgb(0.62, 0.82, 0.95);
pub const WATER_COLOR: Color = Color::srgb(0.30, 0.55, 0.92);

fn damage_formula_segments(formula: &DamageFormula) -> Vec<Segment> {
    let mut segs = vec![plain(formula.flat.to_string())];
    if formula.earth_multiplier != 0.0 {
        segs.push(plain(" + "));
        segs.push(colored(format!("{} earth", formula.earth_multiplier), EARTH_COLOR));
    }
    if formula.air_multiplier != 0.0 {
        segs.push(plain(" + "));
        segs.push(colored(format!("{} air", formula.air_multiplier), AIR_COLOR));
    }
    if formula.fire_multiplier != 0.0 {
        segs.push(plain(" + "));
        segs.push(colored(format!("{} fire", formula.fire_multiplier), FIRE_COLOR));
    }
    if formula.water_multiplier != 0.0 {
        segs.push(plain(" + "));
        segs.push(colored(format!("{} water", formula.water_multiplier), WATER_COLOR));
    }
    segs
}

fn push_line(segments: &mut Vec<Segment>, first: &mut bool, line: Vec<Segment>) {
    if !*first {
        segments.push(plain("\n"));
    }
    segments.extend(line);
    *first = false;
}

pub fn tower_tooltip(
    kind: TowerKind,
    damage_formula: &Option<DamageFormula>,
    stats: &TowerTooltipStats,
    temp_flat_damage: f32,
    temp_speed_bonus: f32,
    range_multiplier: f32,
) -> Vec<Segment> {
    let config = kind.definition().tooltip_config;
    let effective_speed = (stats.attack_speed.value() + temp_speed_bonus).max(0.1);

    let mut segments = Vec::new();
    let mut first = true;
    push_line(&mut segments, &mut first, vec![plain(kind.name().to_string())]);

    if config.show_damage {
        let regular_damage = (damage_formula.unwrap().calculate_damage_with_elemental_multiplier(
            &stats.earth_damage, &stats.fire_damage, &stats.air_damage, &stats.water_damage,
            false,
        ) + temp_flat_damage).max(1.0);
        let critical_damage = (damage_formula.unwrap().calculate_damage_with_elemental_multiplier(
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

        push_line(&mut segments, &mut first, vec![plain(format!("Damage: {regular_damage:.0} ({critical_damage:.0} crit)"))]);

        let mut formula_line = vec![plain("Formula: ")];
        formula_line.extend(damage_formula_segments(&damage_formula.unwrap()));
        formula_line.push(plain(active_suffix));
        push_line(&mut segments, &mut first, formula_line);
    }

    if config.show_range {
        push_line(&mut segments, &mut first, vec![plain(format!("Range: {:.0}", kind.range() * range_multiplier))]);
    }
    if config.show_cooldown {
        push_line(&mut segments, &mut first, vec![plain(format!("Cooldown: {:.2}s", kind.cooldown() / effective_speed))]);
    }
    if config.show_crit {
        push_line(&mut segments, &mut first, vec![plain(format!("Crit: {:.0}%", stats.critical_chance.value() * 100.0))]);
    }
    if config.show_projectile {
        push_line(&mut segments, &mut first, vec![plain(format!("Projectile: {:.0}/s", kind.projectile_speed()))]);
    }
    if config.show_splash {
        push_line(&mut segments, &mut first, vec![plain(format!("Splash: {:.0}", kind.upgraded_explosion_radius(stats.explosion_size.value().max(0.0))))]);
    }
    if config.show_turn_speed {
        push_line(&mut segments, &mut first, vec![plain(format!("Turn speed: {:.1}", kind.angular_speed()))]);
    }

    let piercing_total = effective_piercing(kind.definition().piercing, stats.piercing.value());
    if piercing_total > 0 {
        let falloff = effective_piercing_falloff(kind.definition().piercing_damage, stats.piercing_damage.value());
        push_line(
            &mut segments,
            &mut first,
            vec![plain(format!("Piercing: {piercing_total} (each hit -{:.0}% dmg)", -falloff * 100.0))],
        );
    }

    let stat_effects = kind.stat_effects();
    if !stat_effects.is_empty() {
        push_line(&mut segments, &mut first, vec![plain("Stat effects:")]);
        for effect in stat_effects {
            let line = match element_color(effect.kind) {
                Some(color) => vec![colored(effect.effect_text(), color)],
                None => vec![plain(effect.effect_text())],
            };
            push_line(&mut segments, &mut first, line);
        }
    }

    segments
}

pub fn element_color(kind: PlayerStatKind) -> Option<Color> {
    match kind {
        PlayerStatKind::EarthDamage => Some(EARTH_COLOR),
        PlayerStatKind::FireDamage => Some(FIRE_COLOR),
        PlayerStatKind::AirDamage => Some(AIR_COLOR),
        PlayerStatKind::WaterDamage => Some(WATER_COLOR),
        _ => None,
    }
}

pub fn reset_temporary_attack_speed(mut towers: Query<&mut TemporaryAttackSpeed, With<Tower>>) {
    for mut temp in &mut towers {
        temp.reset();
    }
}

pub fn reset_temporary_damage_bonus(mut towers: Query<&mut TemporaryDamageBonus, With<Tower>>) {
    for mut temp in &mut towers {
        temp.reset();
    }
}

pub fn reset_temporary_range(mut towers: Query<&mut TemporaryRange, With<Tower>>) {
    for mut temp in &mut towers {
        temp.reset();
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
        let effective_speed = (attack_speed.value() + temp_speed.flat).max(0.1);
        cooldown.timer.set_duration(Duration::from_secs_f32(
            base_cooldown / effective_speed,
        ));
        cooldown.timer.tick(delta);
    }
}

pub fn aim_towers(
    mut towers: Query<
        (&mut Transform, &TowerKind, &AngularSpeed, &mut Aim, Option<&TemporaryRange>),
        (With<Tower>, With<DefaultAim>),
    >,
    enemies: Query<(Entity, &Transform, &Health, &PathProgress), (With<Enemy>, Without<Tower>)>,
    game_over: Res<GameOver>,
    time: Res<Time>,
) {
    if game_over.value {
        return;
    }

    for (mut tower_transform, tower_kind, rotation_speed, mut aim, range_boost) in &mut towers {
        let tower_position = tower_transform.translation.truncate();
        let effective_range = range_boost.map_or(tower_kind.range(), |b| b.apply(tower_kind.range()));
        let Some(target_position) = enemies
            .iter()
            .filter(|(_, _, health, _)| health.current > 0.0)
            .filter_map(|(_, transform, _, progress)| {
                let enemy_position = transform.translation.truncate();
                let distance = enemy_position.distance(tower_position);
                (distance <= effective_range).then_some((enemy_position, progress.distance))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(position, _)| position)
        else {
            aim.ready = false;
            aim.direction = Vec2::ZERO;
            continue;
        };

        let direction = target_position - tower_position;
        let target_rotation = Quat::from_rotation_z(direction.y.atan2(direction.x) - PI / 2.0);
        let current_rotation = tower_transform.rotation;
        let step = time.delta_secs() * rotation_speed.value;

        aim.ready = current_rotation.angle_between(target_rotation) <= step;
        aim.direction = direction.normalize_or_zero();
        tower_transform.rotation = tower_transform
            .rotation
            .rotate_towards(target_rotation, step);
    }
}

pub fn fire_towers(
    mut commands: Commands,
    mut towers: Query<
        (
            Entity,
            &Transform,
            &TowerKind,
            &DamageFormula,
            &mut FireCooldown,
            &TemporaryDamageBonus,
            &Aim,
        ),
        (With<Tower>, With<DefaultFire>),
    >,
    game_over: Res<GameOver>,
    critical_chance: Res<CriticalChance>,
    explosion_size: Res<ExplosionSize>,
    earth_damage: Res<EarthDamage>,
    fire_damage: Res<FireDamage>,
    air_damage: Res<AirDamage>,
    water_damage: Res<WaterDamage>,
    piercing: Res<Piercing>,
    piercing_damage: Res<PiercingDamage>,
    mut shoot_events: EventWriter<ShootEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_over.value {
        return;
    }

    for (
        tower_entity,
        tower_transform,
        tower_kind,
        damage_formula,
        mut cooldown,
        damage_bonus,
        aim,
    ) in &mut towers
    {
        if !(aim.ready && cooldown.timer.finished()) {
            continue;
        }

        let tower_position = tower_transform.translation.truncate();

        cooldown.timer.reset();
        shoot_events.write(ShootEvent { source_tower: tower_entity });

        let spread = tower_kind.definition().spread;
        let piercing_total = effective_piercing(tower_kind.definition().piercing, piercing.value());
        let piercing_falloff = effective_piercing_falloff(
            tower_kind.definition().piercing_damage,
            piercing_damage.value(),
        );

        for _ in 0..tower_kind.definition().projectiles_per_shot {
            let is_critical = roll_critical_hit(critical_chance.value());
            let damage = (damage_formula.calculate_damage_with_elemental_multiplier(
                &earth_damage,
                &fire_damage,
                &air_damage,
                &water_damage,
                is_critical,
            ) + damage_bonus.flat).max(1.0);

            let fire_direction = if spread > 0.0 {
                let angle_offset = (rand::random::<f32>() - 0.5) * spread;
                Vec2::from_angle(angle_offset).rotate(aim.direction)
            } else {
                aim.direction
            };

            let (proj_length, proj_width) = if is_critical { (18.0, 8.0) } else { (14.0, 6.0) };
            let proj_angle = fire_direction.y.atan2(fire_direction.x);

            commands.spawn((
                Projectile,
                Mesh2d(meshes.add(Ellipse::new(proj_length * 0.5, proj_width * 0.5))),
                MeshMaterial2d(materials.add(projectile_color(is_critical))),
                Transform::from_translation(tower_position.extend(4.0))
                    .with_rotation(Quat::from_rotation_z(proj_angle)),
                Direction { value: fire_direction },
                RemainingRange { value: tower_kind.range() },
                Pierce { remaining: piercing_total },
                PiercingFalloff { value: piercing_falloff },
                Pierced::default(),
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

pub fn fire_beam_towers(
    mut commands: Commands,
    game_over: Res<GameOver>,
    critical_chance: Res<CriticalChance>,
    earth_damage: Res<EarthDamage>,
    fire_damage: Res<FireDamage>,
    air_damage: Res<AirDamage>,
    water_damage: Res<WaterDamage>,
    mut money: ResMut<Money>,
    mut kills: ResMut<KillCount>,
    loot: Res<Loot>,
    mut spell_shop: ResMut<SpellShop>,
    mut shoot_events: EventWriter<ShootEvent>,
    mut kill_events: EventWriter<EnemyKilledEvent>,
    mut towers: Query<
        (
            Entity,
            &Transform,
            &TowerKind,
            &DamageFormula,
            &mut FireCooldown,
            &TemporaryDamageBonus,
            &Aim,
            Option<&TemporaryRange>,
        ),
        (With<Tower>, With<BeamFire>),
    >,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward, Option<&DropsSpell>), With<Enemy>>,
) {
    if game_over.value {
        return;
    }

    for (
        tower_entity,
        tower_transform,
        tower_kind,
        formula,
        mut cooldown,
        damage_bonus,
        aim,
        range_boost,
    ) in &mut towers
    {
        if !(aim.ready && cooldown.timer.finished()) {
            continue;
        }

        cooldown.timer.reset();
        shoot_events.write(ShootEvent { source_tower: tower_entity });

        let effective_range = range_boost.map_or(tower_kind.range(), |b| b.apply(tower_kind.range()));

        let tower_pos = tower_transform.translation.truncate();
        let beam_end = tower_pos + aim.direction * effective_range;

        let is_critical = roll_critical_hit(critical_chance.value());

        spawn_beam_effect(
            &mut commands,
            tower_pos,
            beam_end,
            BEAM_HALF_WIDTH * 2.0,
            projectile_color(is_critical),
        );

        let base = formula.calculate_damage_with_elemental_multiplier(
            &earth_damage, &fire_damage, &air_damage, &water_damage, is_critical,
        );
        let dmg = (base + damage_bonus.flat).max(1.0);

        let segment = beam_end - tower_pos;
        let segment_len_sq = segment.length_squared();

        let mut killed: Vec<(Entity, i32, Vec2, bool)> = Vec::new();

        for (enemy_entity, enemy_transform, mut health, reward, drops_spell) in &mut enemies {
            if health.current <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let t = if segment_len_sq > f32::EPSILON {
                ((enemy_pos - tower_pos).dot(segment) / segment_len_sq).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let closest = tower_pos + segment * t;
            if enemy_pos.distance(closest) > BEAM_HALF_WIDTH {
                continue;
            }

            let hp_lost = dmg.min(health.current).max(0.0);
            health.current -= dmg;

            if hp_lost > 0.0 {
                spawn_floating_text(
                    &mut commands,
                    format!("-{:.0}", hp_lost),
                    enemy_pos + Vec2::new(0.0, 20.0),
                    if is_critical {
                        Color::srgb(1.0, 0.16, 0.12)
                    } else {
                        Color::srgb(1.0, 1.0, 1.0)
                    },
                    if is_critical { 23.0 } else { 20.0 },
                );
            }

            if health.current <= 0.0 {
                killed.push((enemy_entity, reward.amount, enemy_pos, drops_spell.is_some()));
            }
        }

        for (entity, reward_amount, position, drops_spell) in killed {
            let kill_loot = (reward_amount + loot.value().round() as i32).max(0);
            money.amount += kill_loot;
            kills.amount += 1;
            spawn_floating_text(
                &mut commands,
                format!("+${kill_loot}"),
                position + Vec2::new(34.0, 30.0),
                Color::srgb(1.0, 0.86, 0.20),
                19.0,
            );
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
            kill_events.write(EnemyKilledEvent { source_tower: tower_entity, position });
        }
    }
}

fn roll_critical_hit(critical_chance: f32) -> bool {
    rand::random::<f32>() < critical_chance.clamp(0.0, 1.0)
}

pub fn update_tower_range_indicator(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<(&TowerKind, &Transform, Option<&TemporaryRange>), With<Tower>>,
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
        .filter_map(|(kind, transform, range_boost)| {
            let tower_position = transform.translation.truncate();
            let half_size = kind.base_size() * 0.5;
            let inside_tower = (world_position.x - tower_position.x).abs() <= half_size.x
                && (world_position.y - tower_position.y).abs() <= half_size.y;
            inside_tower.then_some((kind, transform, range_boost))
        })
        .max_by(|a, b| a.1.translation.z.total_cmp(&b.1.translation.z));

    let Some((kind, tower_transform, range_boost)) = hovered_tower else {
        return;
    };

    let effective_range = range_boost.map_or(kind.range(), |b| b.apply(kind.range()));
    indicator_transform.translation = tower_transform.translation.truncate().extend(1.5);
    indicator_transform.scale = Vec3::splat(effective_range);
    *indicator_visibility = Visibility::Visible;
}

pub fn update_draft_tooltip(
    draft: Res<TowerDraft>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    slots: Query<(Entity, &DraftSlot, &GlobalTransform)>,
    slot_children: Query<&Children>,
    previews: Query<Option<&CustomTooltip>, With<DraftPreview>>,
    stats: TowerTooltipStats,
    mut tooltip: Query<(Entity, &mut Text, &mut Visibility), With<ShopTooltip>>,
    mut commands: Commands,
) {
    if draft.phase != TowerDraftPhase::Picking {
        return;
    }

    let Ok((tooltip_entity, mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
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

    for (slot_entity, slot, transform) in &slots {
        let pos = transform.translation().truncate();
        if (cursor_world.x - pos.x).abs() <= 65.0 && (cursor_world.y - pos.y).abs() <= 70.0 {
            let kind = draft.offers[slot.index];
            let damage_formula = kind.damage_formula();
            let mut segments = tower_tooltip(kind, &damage_formula, &stats, 0.0, 0.0, 1.0);
            let extras = slot_children
                .get(slot_entity)
                .ok()
                .and_then(|children| children.iter().find_map(|child| previews.get(child).ok()))
                .flatten();
            if let Some(ct) = extras {
                if !ct.0.is_empty() {
                    segments.push(plain("\n"));
                    segments.extend(ct.0.iter().cloned());
                }
            }
            let tags = kind.tags();
            if !tags.is_empty() {
                segments.push(plain("\nTags: "));
                segments.extend(tag_segments(tags));
            }
            crate::tooltip::set_tooltip_segments(&mut commands, tooltip_entity, &mut tooltip_text, segments);
            *tooltip_visibility = Visibility::Visible;
            return;
        }
    }
}
