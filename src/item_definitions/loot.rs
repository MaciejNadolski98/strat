use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "loot",
    &[TowerStatEffect::new(PlayerStatKind::Loot, 1.0)],
    10,
    Color::srgb(0.95, 0.78, 0.24),
).with_max_purchases(3);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
