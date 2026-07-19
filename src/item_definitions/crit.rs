use bevy::prelude::*;

use crate::item_definitions::unlock;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Crit",
    &[TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04)],
    5,
    Color::srgb(0.70, 0.22, 0.22),
).with_tags(&[tags::MECHANICAL]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CritPlugin;

impl Plugin for CritPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
