use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Offense",
    &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04),
    ],
    7,
    Color::srgb(0.82, 0.70, 0.24),
);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct OffensePlugin;

impl Plugin for OffensePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
