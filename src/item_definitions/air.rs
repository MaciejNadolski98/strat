use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Air",
    &[TowerStatEffect::new(PlayerStatKind::AirDamage, 4.0)],
    3,
    Color::srgb(0.58, 0.72, 0.92),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct AirPlugin;

impl Plugin for AirPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
