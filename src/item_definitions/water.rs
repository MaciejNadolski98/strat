use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Water",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, -4.0),
    ],
    cost: 3,
    icon_color: Color::srgb(0.18, 0.42, 0.78),
    tags: &[],
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
