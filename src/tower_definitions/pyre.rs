use bevy::prelude::*;

use crate::components::{AuraTower, CustomTooltip, DamageFormula, TemporaryDamageBonus, Tower};
use crate::game::game_is_running;
use crate::resources::{FireDamage, GamePhase, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_TRIANGLE_M, BARREL_NONE};

#[derive(Component)]
pub struct PyreTower;

pub struct PyrePlugin;

impl Plugin for PyrePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_pyre_marker.run_if(game_is_running));
        app.add_systems(Update, apply_pyre_aura.in_set(GamePhase::TemporaryTowerEffects));
        app.add_systems(Update, update_pyre_tooltip);
    }
}

pub const TOWER_PYRE: TowerDefinition = TowerDefinition {
    name: "Pyre",
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
    base_color: Color::srgb(0.90, 0.32, 0.10),
    barrel_color: Color::srgb(0.90, 0.32, 0.10),
    base: BASE_TRIANGLE_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::FireDamage, 3.0)],
    tooltip_config: TooltipConfig::AURA,
    tags: &[tags::INFERNAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_PYRE);

pub fn pyre_damage_bonus(fire: f32) -> f32 {
    fire * 0.5
}

fn attach_pyre_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands
                .entity(entity)
                .insert((PyreTower, AuraTower, CustomTooltip::default()));
        }
    }
}

fn apply_pyre_aura(
    pyre_towers: Query<&Transform, With<PyreTower>>,
    mut adjacent_towers: Query<
        (&Transform, &mut TemporaryDamageBonus),
        (With<Tower>, Without<PyreTower>),
    >,
    fire_damage: Res<FireDamage>,
) {
    let bonus = pyre_damage_bonus(fire_damage.value());
    if bonus <= 0.0 {
        return;
    }

    for pyre_transform in &pyre_towers {
        let pyre_pos = pyre_transform.translation.truncate();
        for (tower_transform, mut temp_damage) in &mut adjacent_towers {
            if tower_transform.translation.truncate().distance(pyre_pos) <= TOWER_PYRE.range {
                temp_damage.flat += bonus;
            }
        }
    }
}

fn update_pyre_tooltip(
    fire_damage: Res<FireDamage>,
    mut towers: Query<&mut CustomTooltip, With<PyreTower>>,
    mut tooltip_texts: ResMut<super::CustomTooltipTexts>,
) {
    let bonus = pyre_damage_bonus(fire_damage.value());
    let extras = format!("Boosts adjacent tower damage\n+{bonus:.1} flat damage (fire × 0.5)");
    tooltip_texts.0.insert(KIND, extras.clone());
    for mut tooltip in &mut towers {
        tooltip.0.clone_from(&extras);
    }
}
