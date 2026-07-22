use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Elemental Focus",
    &[
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::AirDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 2.0),
    ],
    9,
    Color::srgb(0.34, 0.60, 0.84),
);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct ElementalFocusPlugin;

impl Plugin for ElementalFocusPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
