use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Siege",
    &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 3.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
    ],
    8,
    Color::srgb(0.74, 0.46, 0.20),
).with_tags(&[tags::MECHANICAL, tags::BIOTIC]);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct SiegePlugin;

impl Plugin for SiegePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
