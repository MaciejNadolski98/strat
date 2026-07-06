use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Splash",
    description: "",
    effects: &[TowerStatEffect::new(PlayerStatKind::ExplosionSize, 4.0)],
    cost: 4,
    icon_color: Color::srgb(0.82, 0.44, 0.18),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
