use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Crit",
    description: "",
    effects: &[TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04)],
    cost: 5,
    icon_color: Color::srgb(0.70, 0.22, 0.22),
    tags: &[tags::MECHANICAL],
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CritPlugin;

impl Plugin for CritPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
