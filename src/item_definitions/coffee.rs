use bevy::prelude::*;

use crate::item_definitions::unlock;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Coffee",
    &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -1.0),
    ],
    5,
    Color::srgb(0.86, 0.72, 0.24),
).with_tags(&[tags::MECHANICAL]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CoffeePlugin;

impl Plugin for CoffeePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
