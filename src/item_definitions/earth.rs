use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Earth",
    effects: &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 4.0)],
    cost: 3,
    icon_color: Color::srgb(0.46, 0.34, 0.22),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct EarthPlugin;

impl Plugin for EarthPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
