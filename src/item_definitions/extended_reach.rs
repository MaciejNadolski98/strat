use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::soul_harvester::{self, SoulHarvesterTower};
use super::{ItemDefinition, ItemKind};

const RANGE_MULTIPLIER: f32 = 1.25;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Extended Reach",
    &[],
    7,
    Color::srgb(0.46, 0.20, 0.66),
)
    .with_description("+25% Soul Harvester range")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct ExtendedReachPlugin;

impl Plugin for ExtendedReachPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(Update, apply_range_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_range_bonus(
    shop: Res<Shop>,
    mut towers: Query<&mut TemporaryRange, With<SoulHarvesterTower>>,
) {
    if shop.purchase_count(KIND) == 0 {
        return;
    }
    for mut range in &mut towers {
        range.multiplier *= RANGE_MULTIPLIER;
    }
}
