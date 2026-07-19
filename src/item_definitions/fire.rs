use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Fire",
    &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -4.0),
    ],
    3,
    Color::srgb(0.86, 0.24, 0.12),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
