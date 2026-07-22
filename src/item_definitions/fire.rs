use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Fire",
    &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -4.0),
    ],
    3,
    Color::srgb(0.86, 0.24, 0.12),
);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
