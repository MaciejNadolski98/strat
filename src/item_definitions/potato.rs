use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Potato",
    &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 4.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 1.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -1.0),
    ],
    5,
    Color::srgb(0.74, 0.18, 0.18),
).with_tags(&[tags::BIOTIC]);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
