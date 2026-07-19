use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Water",
    &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, -4.0),
    ],
    3,
    Color::srgb(0.18, 0.42, 0.78),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
