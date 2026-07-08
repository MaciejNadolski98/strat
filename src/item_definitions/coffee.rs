use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Coffee",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -1.0),
    ],
    cost: 5,
    icon_color: Color::srgb(0.86, 0.72, 0.24),
    tags: &[tags::MECHANICAL],
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CoffeePlugin;

impl Plugin for CoffeePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
