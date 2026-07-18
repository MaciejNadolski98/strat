use bevy::prelude::*;

use crate::components::{CustomTooltip, DamageFormula, DefaultAim, DefaultFire, TemporaryAttackSpeed, Tower};
use crate::game::game_is_running;
use crate::resources::{AirDamage, EarthDamage, GamePhase, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tooltip::{colored, plain};
use crate::tower_definitions::TowerKind;
use crate::towers::{AIR_COLOR, EARTH_COLOR};
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_CIRCLE_M, BARREL_NONE};

#[derive(Component)]
pub struct ZephyrTower;

pub struct ZephyrPlugin;

impl Plugin for ZephyrPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_zephyr_marker.run_if(game_is_running));
        app.add_systems(Update, apply_zephyr_aura.in_set(GamePhase::TemporaryTowerEffects));
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
    spread: 0.0,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: Color::srgb(0.72, 0.88, 0.96),
    barrel_color: Color::srgb(0.72, 0.88, 0.96),
    base: BASE_CIRCLE_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::AirDamage, 3.0)],
    tooltip_config: TooltipConfig::AURA,
    tags: &[tags::BIOTIC],
};

pub const KIND: TowerKind = TowerKind(&TOWER_ZEPHYR);

const AIR_SCALING: f32 = 0.04;
const EARTH_SCALING: f32 = -0.06;

pub fn zephyr_speed_bonus(air: f32, earth: f32) -> f32 {
    air * AIR_SCALING + earth * EARTH_SCALING
}

fn update_zephyr_tooltip(
    air_damage: Res<AirDamage>,
    earth_damage: Res<EarthDamage>,
    mut towers: Query<&mut CustomTooltip, With<ZephyrTower>>,
) {
    let eff_air = air_damage.value();
    let eff_earth = earth_damage.value();
    let bonus = zephyr_speed_bonus(eff_air, eff_earth);
    let extras = vec![
        plain("Boosts adjacent tower attack speed\n"),
        colored("Air", AIR_COLOR),
        plain(": "),
        colored(format!("{:.2}", eff_air * AIR_SCALING), AIR_COLOR),
        plain("  "),
        colored("Earth", EARTH_COLOR),
        plain(": "),
        colored(format!("{:.2}", eff_earth * EARTH_SCALING), EARTH_COLOR),
        plain("\n"),
        plain(format!("Total: {:+.2}x atk speed", bonus)),
    ];
    for mut tooltip in &mut towers {
        tooltip.0 = extras.clone();
    }
}

fn attach_zephyr_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity)
                .insert((ZephyrTower, CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>();
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
    let bonus = zephyr_speed_bonus(air_damage.value(), earth_damage.value());
    if bonus <= 0.0 {
        return;
    }

    for zephyr_transform in &zephyr_towers {
        let zephyr_pos = zephyr_transform.translation.truncate();
        for (tower_transform, mut temp_speed) in &mut adjacent_towers {
            if tower_transform.translation.truncate().distance(zephyr_pos) <= TOWER_ZEPHYR.range {
                temp_speed.flat += bonus;
            }
        }
    }
}
