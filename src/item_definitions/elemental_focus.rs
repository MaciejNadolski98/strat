use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Elemental Focus",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::AirDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 2.0),
    ],
    cost: 9,
    icon_color: Color::srgb(0.34, 0.60, 0.84),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct ElementalFocusPlugin;

impl Plugin for ElementalFocusPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
