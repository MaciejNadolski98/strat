use bevy::prelude::*;

use crate::components::{CustomTooltip, DefaultAim, DefaultFire, TemporaryDamageBonus, Tower};
use crate::game::game_is_running;
use crate::resources::{FireDamage, GamePhase, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tooltip::{colored, plain};
use crate::towers::FIRE_COLOR;
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

pub const TOWER_PYRE: TowerDefinition = TowerDefinition::new_utility(
    "Pyre",
    82.0,
    Color::srgb(0.90, 0.32, 0.10),
    BASE_TRIANGLE_M,
    BARREL_NONE,
)
    .with_stat_effects(&[TowerStatEffect::new(PlayerStatKind::FireDamage, 3.0)])
    .with_tooltip_config(TooltipConfig::AURA)
    .with_tags(&[tags::INFERNAL]);

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
                .insert((PyreTower, CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>();
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
) {
    let bonus = pyre_damage_bonus(fire_damage.value());
    let extras = vec![
        plain("Boosts adjacent tower damage\n"),
        plain("+"),
        colored(format!("{bonus:.1}"), FIRE_COLOR),
        plain(" flat damage ("),
        colored("fire", FIRE_COLOR),
        plain(" x 0.5)"),
    ];
    for mut tooltip in &mut towers {
        tooltip.0 = extras.clone();
    }
}
