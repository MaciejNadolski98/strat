use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Earth",
    &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 4.0)],
    3,
    Color::srgb(0.46, 0.34, 0.22),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
