use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tower_definitions::tree;
use super::{ItemDefinition, ItemKind};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Apple",
    &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 3.0),
        TowerStatEffect::new(PlayerStatKind::Regeneration, 2.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 1.0),
    ],
    5,
    Color::srgb(0.92, 0.12, 0.13),
).with_tags(&[tags::BIOTIC]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct ApplePlugin;

impl Plugin for ApplePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(tree::KIND), KIND);
    }
}
