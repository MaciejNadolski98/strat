use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Air",
    &[TowerStatEffect::new(PlayerStatKind::AirDamage, 4.0)],
    3,
    Color::srgb(0.58, 0.72, 0.92),
);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct AirPlugin;

impl Plugin for AirPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
