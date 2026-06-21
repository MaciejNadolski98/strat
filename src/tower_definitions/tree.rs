use bevy::prelude::*;

use crate::components::{DamageFormula, Enemy, FireCooldown, Health, Speed};
use crate::effects::spawn_floating_text;
use crate::enemies::{move_enemies, reset_enemy_speeds};
use crate::game::game_is_running;
use crate::resources::{EarthDamage, Loot, Money, PlayerStatKind, TowerStatEffect, WaterDamage};
use crate::tower_definitions::TowerKind;
use super::TowerDefinition;

#[derive(Component)]
pub struct TreeTower;

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            attach_tree_marker.run_if(game_is_running),
        );
        app.add_systems(
            Update,
            apply_tree_aura
                .after(reset_enemy_speeds)
                .before(move_enemies)
                .run_if(game_is_running),
        );
    }
}

pub const TOWER_TREE: TowerDefinition = TowerDefinition {
    name: "Tree",
    range: 180.0,
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
    base_color: Color::srgb(0.20, 0.55, 0.18),
    barrel_color: Color::srgb(0.20, 0.55, 0.18),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::ZERO,
    barrel_offset: 0.0,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::WaterDamage, 3.0)],
    custom_tooltip: Some(tree_custom_tooltip),
};

pub fn tree_slow_multiplier(earth: f32) -> f32 {
    0.3 + 0.7 / (1.0 + earth / 15.0)
}

pub fn tree_income_per_enemy(water: f32) -> f32 {
    1.0 + water * 0.15
}

pub fn tree_custom_tooltip(earth: f32, water: f32) -> String {
    let slow_pct = (1.0 - tree_slow_multiplier(earth)) * 100.0;
    let income = tree_income_per_enemy(water);
    format!(
        "Tree\nAura slow: {slow_pct:.0}% (→70% as earth→∞)\nIncome: ${income:.1} per enemy every 4s\nRange: {:.0}\nStat effects:\n+3 water",
        TOWER_TREE.range,
    )
}

fn attach_tree_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Tree {
            commands.entity(entity).insert(TreeTower);
        }
    }
}

fn apply_tree_aura(
    mut commands: Commands,
    mut money: ResMut<Money>,
    earth_damage: Res<EarthDamage>,
    water_damage: Res<WaterDamage>,
    loot: Res<Loot>,
    mut tree_towers: Query<(&Transform, &mut FireCooldown), With<TreeTower>>,
    mut enemies: Query<(&Transform, &Health, &mut Speed), With<Enemy>>,
) {
    let slow_mult = tree_slow_multiplier(earth_damage.value);
    let income_per_enemy = tree_income_per_enemy(water_damage.value);

    for (tree_transform, mut cooldown) in &mut tree_towers {
        let tree_pos = tree_transform.translation.truncate();

        let income_due = cooldown.timer.just_finished();
        if income_due {
            cooldown.timer.reset();
        }

        let mut enemies_in_range: u32 = 0;

        for (enemy_transform, health, mut speed) in &mut enemies {
            if health.current <= 0.0 {
                continue;
            }
            let dist = enemy_transform.translation.truncate().distance(tree_pos);
            if dist > TOWER_TREE.range {
                continue;
            }

            speed.value *= slow_mult;
            enemies_in_range += 1;
        }

        if income_due && enemies_in_range > 0 {
            let earned = ((enemies_in_range as f32 * income_per_enemy).ceil() as i32)
                .max(1)
                + loot.amount;
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
