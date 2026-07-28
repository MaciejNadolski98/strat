use bevy::prelude::*;

use crate::components::{CustomTooltip, DamageFormula, TemporaryProjectileSpeed, TemporaryRange};
use crate::game::game_is_running;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tooltip::{colored, plain};
use crate::towers::EARTH_COLOR;
use crate::tower_definitions::TowerKind;
use crate::tower_definitions::templates::BASE_TRIANGLE_M;
use super::{TowerDefinition, TowerRegistry};
use super::templates::{BARREL_HEAVY, PALETTE_EARTH};

#[derive(Component)]
pub struct GolemTower;

pub struct GolemPlugin;

impl Plugin for GolemPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_golem_tower.run_if(game_is_running));
        app.add_systems(Update, update_golem_tooltip);
    }
}

pub static TOWER_GOLEM: TowerDefinition = TowerDefinition::new_attacking(
    "Golem",
    160.0,
    1.1,
    DamageFormula {
        flat: 4,
        crit_multiplier: 1.8,
        earth_multiplier: 0.2,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    PALETTE_EARTH.base,
    BASE_TRIANGLE_M,
    BARREL_HEAVY,
    350.0,
    1.2,
)
    .with_barrel_color(PALETTE_EARTH.barrel)
    .with_stat_effects(&[TowerStatEffect::new(PlayerStatKind::EarthDamage, 3.0)])
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL]);

pub static KIND: TowerKind = TowerKind(&TOWER_GOLEM);

fn attach_golem_tower(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands.entity(entity).insert((
                GolemTower,
                CustomTooltip::default(),
                TemporaryRange::default(),
                TemporaryProjectileSpeed::default(),
            ));
        }
    }
}

fn update_golem_tooltip(
    mut golems: Query<&mut CustomTooltip, With<GolemTower>>,
) {
    let extras = vec![
        plain("Deals small "),
        colored("earth", EARTH_COLOR),
        plain("-scaling damage on its own\nUpgraded by Golem's Hand/Mouth/Leg/Eye"),
    ];
    for mut tooltip in &mut golems {
        tooltip.0 = extras.clone();
    }
}
