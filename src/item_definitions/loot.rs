use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "loot",
    effects: &[TowerStatEffect::new(PlayerStatKind::Loot, 1.0)],
    cost: 10,
    icon_color: Color::srgb(0.95, 0.78, 0.24),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
