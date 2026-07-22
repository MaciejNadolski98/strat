use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Vitality",
    &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 5.0),
        TowerStatEffect::new(PlayerStatKind::Regeneration, 1.0),
    ],
    6,
    Color::srgb(0.72, 0.34, 0.34),
).with_tags(&[tags::BIOTIC]);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct VitalityPlugin;

impl Plugin for VitalityPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
