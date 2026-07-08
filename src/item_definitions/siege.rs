use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Siege",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 3.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
    ],
    cost: 8,
    icon_color: Color::srgb(0.74, 0.46, 0.20),
    tags: &[tags::MECHANICAL, tags::BIOTIC],
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct SiegePlugin;

impl Plugin for SiegePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
