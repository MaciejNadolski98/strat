use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Meds",
    &[
        TowerStatEffect::new(PlayerStatKind::Regeneration, 2.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.1),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -10.0),
    ],
    2,
    Color::srgb(0.22, 0.62, 0.30),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct MedsPlugin;

impl Plugin for MedsPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
    }
}
