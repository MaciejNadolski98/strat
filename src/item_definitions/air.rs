use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Air",
    effects: &[TowerStatEffect::new(PlayerStatKind::AirDamage, 4.0)],
    cost: 3,
    icon_color: Color::srgb(0.58, 0.72, 0.92),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct AirPlugin;

impl Plugin for AirPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
