use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Fire",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -4.0),
    ],
    cost: 3,
    icon_color: Color::srgb(0.86, 0.24, 0.12),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
