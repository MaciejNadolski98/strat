use bevy::prelude::*;

use crate::components::{AuraTower, CustomTooltip, DamageFormula, Enemy, FireCooldown, Health, TemporaryEnemySpeed};
use crate::effects::spawn_floating_text;
use crate::enemies::{move_enemies, reset_temporary_enemy_speed};
use crate::game::game_is_running;
use crate::resources::{EarthDamage, Money, PlayerStatKind, TowerStatEffect, WaterDamage};
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_HEX_M, BARREL_NONE, PALETTE_FOREST};

#[derive(Component)]
pub struct TreeTower;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(
            Update,
            attach_tree_marker.run_if(game_is_running),
        );
        app.add_systems(
            Update,
            apply_tree_aura
                .after(reset_temporary_enemy_speed)
                .before(move_enemies)
                .run_if(game_is_running),
        );
        app.add_systems(Update, update_income_bubbles.run_if(game_is_running));
        app.add_systems(Update, update_tree_tooltip);
    }
}

pub const TOWER_TREE: TowerDefinition = TowerDefinition {
    name: "Tree",
    range: 87.0,
    cooldown: 4.0,
    damage_formula: DamageFormula {
        flat: 0,
        crit_multiplier: 1.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.0,
    base_color: PALETTE_FOREST.base,
    barrel_color: PALETTE_FOREST.barrel,
    base: BASE_HEX_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::WaterDamage, 3.0)],
    tooltip_config: TooltipConfig::AURA
        .with_cooldown(true),
};

pub const KIND: TowerKind = TowerKind(&TOWER_TREE);

pub fn tree_slow_multiplier(earth: f32) -> f32 {
    0.3 + 0.7 / (1.0 + earth / 15.0)
}

pub fn tree_income_per_enemy(water: f32) -> f32 {
    1.0 + water * 0.015
}

fn update_tree_tooltip(
    earth_damage: Res<EarthDamage>,
    water_damage: Res<WaterDamage>,
    mut towers: Query<&mut CustomTooltip, With<TreeTower>>,
    mut tooltip_texts: ResMut<super::CustomTooltipTexts>,
) {
    let slow_pct = (1.0 - tree_slow_multiplier(earth_damage.value)) * 100.0;
    let income = tree_income_per_enemy(water_damage.value);
    let extras = format!(
        "Aura slow: {slow_pct:.0}% (→70% as earth→∞)\nIncome: ${income:.1} per enemy every 4s (1 + water × 0.015)",
    );
    tooltip_texts.0.insert(KIND, extras.clone());
    for mut tooltip in &mut towers {
        tooltip.0.clone_from(&extras);
    }
}

fn attach_tree_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity).insert((TreeTower, AuraTower, CustomTooltip::default()));
        }
    }
}

fn apply_tree_aura(
    mut commands: Commands,
    mut money: ResMut<Money>,
    earth_damage: Res<EarthDamage>,
    water_damage: Res<WaterDamage>,
    mut tree_towers: Query<(&Transform, &mut FireCooldown), With<TreeTower>>,
    mut enemies: Query<(&Transform, &Health, &mut TemporaryEnemySpeed), With<Enemy>>,
) {
    let slow_mult = tree_slow_multiplier(earth_damage.value);
    let income_per_enemy = tree_income_per_enemy(water_damage.value);

    for (tree_transform, mut cooldown) in &mut tree_towers {
        let tree_pos = tree_transform.translation.truncate();

        let income_due = cooldown.timer.just_finished();
        if income_due {
            cooldown.timer.reset();
        }

        let mut enemies_in_range: Vec<Vec2> = Vec::new();

        for (enemy_transform, health, mut temp_speed) in &mut enemies {
            if health.current <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            if enemy_pos.distance(tree_pos) > TOWER_TREE.range {
                continue;
            }

            temp_speed.multiplier *= slow_mult;
            enemies_in_range.push(enemy_pos);
        }

        if income_due && !enemies_in_range.is_empty() {
            for &enemy_pos in &enemies_in_range {
                spawn_income_bubble(&mut commands, enemy_pos, tree_pos);
            }
            let earned = ((enemies_in_range.len() as f32 * income_per_enemy).ceil() as i32)
                .max(1);
            money.amount += earned;
            spawn_floating_text(
                &mut commands,
                format!("+${earned}"),
                tree_pos + Vec2::new(20.0, 28.0),
                Color::srgb(0.40, 0.92, 0.36),
                18.0,
            );
        }
    }
}

#[derive(Component)]
struct IncomeBubble {
    origin: Vec2,
    target: Vec2,
    lifetime: Timer,
}

fn spawn_income_bubble(commands: &mut Commands, origin: Vec2, target: Vec2) {
    commands.spawn((
        Sprite::from_color(Color::srgba(0.28, 0.62, 0.96, 0.88), Vec2::splat(7.0)),
        Transform::from_translation(origin.extend(5.0)),
        IncomeBubble {
            origin,
            target,
            lifetime: Timer::from_seconds(0.65, TimerMode::Once),
        },
    ));
}

fn update_income_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut bubbles: Query<(Entity, &mut IncomeBubble, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut bubble, mut transform, mut sprite) in &mut bubbles {
        bubble.lifetime.tick(time.delta());
        let t = bubble.lifetime.fraction();
        let pos = bubble.origin.lerp(bubble.target, t);
        let arc_y = 48.0 * 4.0 * t * (1.0 - t);
        transform.translation = Vec3::new(pos.x, pos.y + arc_y, 5.0);
        let alpha = if t > 0.75 { 0.88 * (1.0 - (t - 0.75) / 0.25) } else { 0.88 };
        sprite.color = Color::srgba(0.28, 0.62, 0.96, alpha);
        if bubble.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
