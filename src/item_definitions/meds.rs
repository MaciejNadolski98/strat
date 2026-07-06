use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Meds",
    description: "",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::Regeneration, 2.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.1),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -10.0),
    ],
    cost: 2,
    icon_color: Color::srgb(0.22, 0.62, 0.30),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct MedsPlugin;

impl Plugin for MedsPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
