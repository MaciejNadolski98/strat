use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Vitality",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 5.0),
        TowerStatEffect::new(PlayerStatKind::Regeneration, 1.0),
    ],
    cost: 6,
    icon_color: Color::srgb(0.72, 0.34, 0.34),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct VitalityPlugin;

impl Plugin for VitalityPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
    }
}
