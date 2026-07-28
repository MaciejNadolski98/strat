use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::constants::HEX_SPACING;
use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::golem::{self, GolemTower};
use super::{golem_heart, unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Golem's Eye",
    &[],
    5,
    Color::srgb(0.36, 0.58, 0.44),
)
    .with_description("+1 tile Golem range")
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL])
    .with_max_purchases(2);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct GolemEyePlugin;

impl Plugin for GolemEyePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(golem::KIND), KIND);
        app.add_systems(Update, apply_range_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_range_bonus(
    shop: Res<Shop>,
    mut towers: Query<&mut TemporaryRange, With<GolemTower>>,
) {
    let stacks = shop.purchase_count(KIND) as f32;
    let multiplier = if shop.purchase_count(golem_heart::KIND) > 0 { 2.0 } else { 1.0 };
    let range_bonus = stacks * HEX_SPACING * multiplier;

    for mut range in &mut towers {
        range.flat += range_bonus;
    }
}
