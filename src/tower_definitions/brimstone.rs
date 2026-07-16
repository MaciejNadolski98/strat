use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use bevy::window::PrimaryWindow;

use crate::components::{
    CustomTooltip, DamageFormula, DefaultAim, DefaultFire, DropsSpell, Enemy, FireCooldown,
    Health, Reward,
};
use crate::effects::{spawn_floating_text, spawn_pulse};
use crate::game::game_is_running;
use crate::resources::{
    CriticalChance, CurrentHp, EarthDamage, EnemyKilledEvent, FireDamage, AirDamage, GameOver,
    KillCount, Loot, Money, PlayerStatKind, SpellShop, TowerDraft, TowerDraftPhase,
    TowerStatEffect, WaterDamage,
};
use crate::tags;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_HEX_S, BARREL_NONE};

const HP_COST: i32 = 10;

#[derive(Component)]
struct BrimstoneTower;

pub struct BrimstonePlugin;

impl Plugin for BrimstonePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_brimstone_marker.run_if(game_is_running));
        app.add_systems(Update, trigger_brimstone.run_if(game_is_running));
        app.add_systems(Update, update_brimstone_tooltip);
    }
}

pub const TOWER_BRIMSTONE: TowerDefinition = TowerDefinition {
    name: "Brimstone",
    range: 170.0,
    cooldown: 10.0,
    damage_formula: DamageFormula {
        flat: 250,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 300.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.0,
    spread: 0.0,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: Color::srgb(0.16, 0.05, 0.04),
    barrel_color: Color::srgb(0.16, 0.05, 0.04),
    base: BASE_HEX_S,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0)],
    tooltip_config: TooltipConfig::UTILITY.with_damage(true),
    tags: &[tags::INFERNAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_BRIMSTONE);

#[derive(SystemParam)]
struct ElementalDamages<'w> {
    earth: Res<'w, EarthDamage>,
    fire: Res<'w, FireDamage>,
    air: Res<'w, AirDamage>,
    water: Res<'w, WaterDamage>,
}

#[derive(SystemParam)]
struct KillRewards<'w> {
    money: ResMut<'w, Money>,
    kills: ResMut<'w, KillCount>,
    loot: Res<'w, Loot>,
    spell_shop: ResMut<'w, SpellShop>,
    kill_events: EventWriter<'w, EnemyKilledEvent>,
}

fn attach_brimstone_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((BrimstoneTower, CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>();
        }
    }
}

fn trigger_brimstone(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    draft: Res<TowerDraft>,
    mut game_over: ResMut<GameOver>,
    mut current_hp: ResMut<CurrentHp>,
    critical_chance: Res<CriticalChance>,
    elemental: ElementalDamages,
    mut rewards: KillRewards,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut towers: Query<(Entity, &Transform, &TowerKind, &DamageFormula, &mut FireCooldown), With<BrimstoneTower>>,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward, Option<&DropsSpell>), With<Enemy>>,
) {
    if game_over.value
        || matches!(draft.phase, TowerDraftPhase::Placing(_))
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (tower_entity, tower_transform, tower_kind, formula, mut cooldown) in &mut towers {
        if !cooldown.timer.finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();
        let half_size = tower_kind.base_size() * 0.5;
        let clicked = (world_position.x - tower_pos.x).abs() <= half_size.x
            && (world_position.y - tower_pos.y).abs() <= half_size.y;
        if !clicked {
            continue;
        }

        cooldown.timer.reset();

        current_hp.amount -= HP_COST;
        if current_hp.amount <= 0 {
            game_over.value = true;
        }

        spawn_pulse(
            &mut commands,
            tower_pos,
            TOWER_BRIMSTONE.range,
            Color::srgba(0.95, 0.25, 0.05, 0.75),
            &mut meshes,
            &mut materials,
        );

        let is_critical = rand::random::<f32>() < critical_chance.value().clamp(0.0, 1.0);
        let dmg = formula.calculate_damage_with_elemental_multiplier(
            &elemental.earth, &elemental.fire, &elemental.air, &elemental.water, is_critical,
        ).max(1.0);

        let mut killed: Vec<(Entity, i32, Vec2, bool)> = Vec::new();

        for (enemy_entity, enemy_transform, mut health, reward, drops_spell) in &mut enemies {
            if health.current <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            if enemy_pos.distance(tower_pos) > TOWER_BRIMSTONE.range {
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
                        Color::srgb(1.0, 0.86, 0.20)
                    } else {
                        Color::srgb(1.0, 0.45, 0.12)
                    },
                    if is_critical { 26.0 } else { 22.0 },
                );
            }

            if health.current <= 0.0 {
                killed.push((enemy_entity, reward.amount, enemy_pos, drops_spell.is_some()));
            }
        }

        for (entity, reward_amount, position, drops_spell) in killed {
            let kill_loot = (reward_amount + rewards.loot.value().round() as i32).max(0);
            rewards.money.amount += kill_loot;
            rewards.kills.amount += 1;
            spawn_floating_text(
                &mut commands,
                format!("+${kill_loot}"),
                position + Vec2::new(34.0, 30.0),
                Color::srgb(1.0, 0.86, 0.20),
                19.0,
            );
            if drops_spell {
                rewards.spell_shop.store_random_spell();
                spawn_floating_text(
                    &mut commands,
                    "Spell!".to_string(),
                    position + Vec2::new(-20.0, 52.0),
                    Color::srgb(0.72, 0.30, 0.92),
                    22.0,
                );
            }
            commands.entity(entity).despawn();
            rewards.kill_events.write(EnemyKilledEvent { source_tower: tower_entity, position });
        }

        return;
    }
}

fn update_brimstone_tooltip(
    mut towers: Query<(Option<&FireCooldown>, &mut CustomTooltip), With<BrimstoneTower>>,
) {
    for (cooldown, mut tooltip) in &mut towers {
        let status = match cooldown {
            None => "".to_string(),
            Some(cooldown) => {
                if cooldown.timer.finished() {
                    "\nReady now".to_string()
                } else {
                    format!(
                        "\nRecharging ({:.0}s left)",
                        (cooldown.timer.duration().as_secs_f32() - cooldown.timer.elapsed_secs()).max(0.0),
                    )
                }
            },
        };

        let extras = format!(
            "Does not attack on its own. Click it to unleash a devastating explosion, at the cost of your own HP.\nCosts {HP_COST} HP{status}",
        );
        tooltip.0 = vec![crate::tooltip::plain(extras)];
    }
}
