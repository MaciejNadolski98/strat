use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Earth",
    &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 4.0)],
    3,
    Color::srgb(0.46, 0.34, 0.22),
);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
