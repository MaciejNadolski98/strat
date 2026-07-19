use bevy::prelude::*;

use crate::item_definitions::unlock;
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Splash",
    &[TowerStatEffect::new(PlayerStatKind::ExplosionSize, 4.0)],
    4,
    Color::srgb(0.82, 0.44, 0.18),
).with_tags(&[tags::MECHANICAL]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
    }
}
