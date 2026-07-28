use bevy::prelude::*;

use crate::tags;
use super::{golem_eye, golem_hand, golem_leg, golem_mouth, unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Golem's Heart",
    &[],
    10,
    Color::srgb(0.86, 0.20, 0.24),
)
    .with_description("Doubles the bonuses from Golem's Hand/Mouth/Leg/Eye")
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

static REQUIRED_ITEMS: [ItemKind; 4] =
    [golem_hand::KIND, golem_mouth::KIND, golem_leg::KIND, golem_eye::KIND];

pub struct GolemHeartPlugin;

impl Plugin for GolemHeartPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::AllMaxed(&REQUIRED_ITEMS), KIND);
    }
}
