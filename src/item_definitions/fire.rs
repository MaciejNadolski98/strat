use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Fire",
    &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -4.0),
    ],
    3,
    Color::srgb(0.86, 0.24, 0.12),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<Shop>().add_to_pool(KIND);
        app.add_systems(Update, on_restart.in_set(ItemPoolRestoreSet));
    }
}

fn on_restart(mut events: EventReader<GameRestartEvent>, mut shop: ResMut<Shop>) {
    if events.read().next().is_some() {
        shop.add_to_pool(KIND);
    }
}
