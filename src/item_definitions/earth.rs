use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Earth",
    description: "",
    effects: &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 4.0)],
    cost: 3,
    icon_color: Color::srgb(0.46, 0.34, 0.22),
    tags: &[],
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
