use bevy::prelude::*;

use crate::components::{AuraTower, CustomTooltip, DamageFormula, TemporaryAttackSpeed, Tower};
use crate::game::game_is_running;
use crate::resources::{AirDamage, EarthDamage, PlayerStatKind, TowerStatEffect};
use crate::tower_definitions::TowerKind;
use crate::towers::{progress_cooldown, reset_temporary_attack_speed};
use super::{TowerDefinition, TooltipConfig};
use super::templates::{BASE_CIRCLE_M, BARREL_NONE};

#[derive(Component)]
pub struct ZephyrTower;

pub struct ZephyrPlugin;

impl Plugin for ZephyrPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_zephyr_marker.run_if(game_is_running));
        app.add_systems(
            Update,
            apply_zephyr_aura
                .after(reset_temporary_attack_speed)
                .before(progress_cooldown)
                .run_if(game_is_running),
        );
        app.add_systems(Update, update_zephyr_tooltip);
    }
}

pub const TOWER_ZEPHYR: TowerDefinition = TowerDefinition {
    name: "Zephyr",
    range: 82.0,
    cooldown: 999.0,
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
    base_color: Color::srgb(0.72, 0.88, 0.96),
    barrel_color: Color::srgb(0.72, 0.88, 0.96),
    base: BASE_CIRCLE_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::AirDamage, 3.0)],
    tooltip_config: TooltipConfig::AURA,
};

pub fn zephyr_speed_bonus(air: f32, earth: f32) -> f32 {
    air * 0.04 - earth * 0.06
}

fn update_zephyr_tooltip(
    air_damage: Res<AirDamage>,
    earth_damage: Res<EarthDamage>,
    mut towers: Query<&mut CustomTooltip, With<ZephyrTower>>,
    mut tooltip_texts: ResMut<super::CustomTooltipTexts>,
) {
    let bonus = zephyr_speed_bonus(air_damage.value, earth_damage.value);
    let extras = format!(
        "Boosts adjacent tower attack speed\nAir: +{:.2}  Earth: -{:.2}\nTotal: {:+.2}x atk speed",
        air_damage.value * 0.04,
        earth_damage.value * 0.06,
        bonus,
    );
    tooltip_texts.0.insert(TowerKind::Zephyr, extras.clone());
    for mut tooltip in &mut towers {
        tooltip.0.clone_from(&extras);
    }
}

fn attach_zephyr_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Zephyr {
            commands.entity(entity).insert((ZephyrTower, AuraTower, CustomTooltip::default()));
        }
    }
}

fn apply_zephyr_aura(
    zephyr_towers: Query<&Transform, With<ZephyrTower>>,
    mut adjacent_towers: Query<
        (&Transform, &mut TemporaryAttackSpeed),
        (With<Tower>, Without<ZephyrTower>),
    >,
    air_damage: Res<AirDamage>,
    earth_damage: Res<EarthDamage>,
) {
    let bonus = zephyr_speed_bonus(air_damage.value, earth_damage.value);
    if bonus <= 0.0 {
        return;
    }

    for zephyr_transform in &zephyr_towers {
        let zephyr_pos = zephyr_transform.translation.truncate();
        for (tower_transform, mut temp_speed) in &mut adjacent_towers {
            if tower_transform.translation.truncate().distance(zephyr_pos) <= TOWER_ZEPHYR.range {
                temp_speed.bonus += bonus;
            }
        }
    }
}
