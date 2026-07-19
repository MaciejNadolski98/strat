use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Offense",
    &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04),
    ],
    7,
    Color::srgb(0.82, 0.70, 0.24),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct OffensePlugin;

impl Plugin for OffensePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
