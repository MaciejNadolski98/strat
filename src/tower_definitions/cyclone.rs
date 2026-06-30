use bevy::prelude::*;

use crate::components::{
    AuraTower, CustomTooltip, DamageFormula, DropsSpell, Enemy, FireCooldown, Health, Reward,
};
use crate::effects::{spawn_floating_text, spawn_pulse};
use crate::game::game_is_running;
use crate::resources::{
    ActiveSpellEffects, AirDamage, CriticalChance, EnemyKilledEvent, GameOver, KillCount, Loot,
    Money, PlayerStatKind, SpellKind, SpellShop, TowerStatEffect,
};
use crate::towers::progress_cooldown;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig};
use super::templates::{BASE_PENTAGON_M, BARREL_NONE};

#[derive(Component)]
pub struct CycloneTower;

pub struct CyclonePlugin;

impl Plugin for CyclonePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_cyclone_tower.run_if(game_is_running));
        app.add_systems(
            Update,
            apply_cyclone_burst
                .after(progress_cooldown)
                .run_if(game_is_running),
        );
        app.add_systems(Update, update_cyclone_tooltip);
    }
}

pub const TOWER_CYCLONE: TowerDefinition = TowerDefinition {
    name: "Cyclone",
    range: 80.0,
    cooldown: 2.0,
    damage_formula: DamageFormula {
        flat: 16,
        crit_multiplier: 1.5,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.85,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.0,
    base_color: Color::srgb(0.42, 0.68, 0.90),
    barrel_color: Color::srgb(0.42, 0.68, 0.90),
    base: BASE_PENTAGON_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::AirDamage, 3.0)],
    tooltip_config: TooltipConfig::STANDARD
        .with_turn_speed(false)
        .with_projectile(false),
};

fn attach_cyclone_tower(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Cyclone {
            commands.entity(entity).insert((CycloneTower, AuraTower, CustomTooltip::default()));
        }
    }
}

fn apply_cyclone_burst(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut money: ResMut<Money>,
    mut kills: ResMut<KillCount>,
    loot: Res<Loot>,
    mut spell_shop: ResMut<SpellShop>,
    game_over: Res<GameOver>,
    air_damage: Res<AirDamage>,
    critical_chance: Res<CriticalChance>,
    active_spell_effects: Res<ActiveSpellEffects>,
    mut kill_events: EventWriter<EnemyKilledEvent>,
    mut cyclone_towers: Query<(Entity, &Transform, &mut FireCooldown, &DamageFormula), With<CycloneTower>>,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward, Option<&DropsSpell>), With<Enemy>>,
) {
    if game_over.value {
        return;
    }

    for (tower_entity, tower_transform, mut cooldown, formula) in &mut cyclone_towers {
        if !cooldown.timer.finished() {
            continue;
        }

        let tower_pos = tower_transform.translation.truncate();

        let has_targets = enemies.iter().any(|(_, t, h, _, _)| {
            h.current > 0.0 && t.translation.truncate().distance(tower_pos) <= TOWER_CYCLONE.range
        });
        if !has_targets {
            continue;
        }

        cooldown.timer.reset();

        spawn_pulse(
            &mut commands,
            tower_pos,
            TOWER_CYCLONE.range,
            Color::srgba(0.48, 0.84, 1.0, 0.65),
            &mut meshes,
            &mut materials,
        );

        let is_critical = rand::random::<f32>() < critical_chance.value.clamp(0.0, 1.0);
        let base = formula.flat as f32
            + formula.air_multiplier * air_damage.value * active_spell_effects.elemental_multiplier;
        let dmg = if is_critical { base * formula.crit_multiplier } else { base };

        let mut killed: Vec<(Entity, i32, Vec2, bool)> = Vec::new();

        for (enemy_entity, enemy_transform, mut health, reward, drops_spell) in &mut enemies {
            if health.current <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            if enemy_pos.distance(tower_pos) > TOWER_CYCLONE.range {
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
                        Color::srgb(0.62, 0.92, 1.0)
                    },
                    if is_critical { 23.0 } else { 20.0 },
                );
            }

            if health.current <= 0.0 {
                killed.push((enemy_entity, reward.amount, enemy_pos, drops_spell.is_some()));
            }
        }

        for (entity, reward_amount, position, drops_spell) in killed {
            let kill_loot = reward_amount + loot.amount;
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
                spell_shop.store_spell(SpellKind::random());
                spawn_floating_text(
                    &mut commands,
                    "Spell!".to_string(),
                    position + Vec2::new(-20.0, 52.0),
                    Color::srgb(0.72, 0.30, 0.92),
                    22.0,
                );
            }
            commands.entity(entity).despawn();
            kill_events.write(EnemyKilledEvent { source_tower: tower_entity });
        }
    }
}

fn update_cyclone_tooltip(
    mut towers: Query<&mut CustomTooltip, With<CycloneTower>>,
    mut tooltip_texts: ResMut<super::CustomTooltipTexts>,
) {
    let extras = "Hits all enemies in range simultaneously".to_string();
    tooltip_texts.0.insert(TowerKind::Cyclone, extras.clone());
    for mut tooltip in &mut towers {
        tooltip.0.clone_from(&extras);
    }
}
