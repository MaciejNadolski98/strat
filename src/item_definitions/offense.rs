use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Offense",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04),
    ],
    cost: 7,
    icon_color: Color::srgb(0.82, 0.70, 0.24),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct OffensePlugin;

impl Plugin for OffensePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
