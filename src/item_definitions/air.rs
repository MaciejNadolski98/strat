use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Air",
    &[TowerStatEffect::new(PlayerStatKind::AirDamage, 4.0)],
    3,
    Color::srgb(0.58, 0.72, 0.92),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct AirPlugin;

impl Plugin for AirPlugin {
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
