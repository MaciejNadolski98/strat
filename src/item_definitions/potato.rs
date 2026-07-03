use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Potato",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 4.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 1.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -1.0),
    ],
    cost: 5,
    icon_color: Color::srgb(0.74, 0.18, 0.18),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
