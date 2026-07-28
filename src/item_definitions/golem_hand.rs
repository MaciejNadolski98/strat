use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::golem::{self, GolemTower, TOWER_GOLEM};
use super::{golem_heart, unlock, ItemDefinition, ItemKind, UnlockCondition};

const EARTH_SCALING_PER_STACK: f32 = 1.0;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Golem's Hand",
    &[],
    6,
    Color::srgb(0.55, 0.42, 0.24),
)
    .with_description("+1 Golem earth scaling")
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL])
    .with_max_purchases(2);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct GolemHandPlugin;

impl Plugin for GolemHandPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(golem::KIND), KIND);
        app.add_systems(Update, apply_earth_scaling_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_earth_scaling_bonus(
    shop: Res<Shop>,
    mut towers: Query<&mut DamageFormula, With<GolemTower>>,
) {
    let stacks = shop.purchase_count(KIND) as f32;
    let multiplier = if shop.purchase_count(golem_heart::KIND) > 0 { 2.0 } else { 1.0 };
    let base_earth_multiplier = TOWER_GOLEM.damage_formula.unwrap().earth_multiplier;
    let earth_multiplier = base_earth_multiplier + stacks * EARTH_SCALING_PER_STACK * multiplier;

    for mut formula in &mut towers {
        formula.earth_multiplier = earth_multiplier;
    }
}
