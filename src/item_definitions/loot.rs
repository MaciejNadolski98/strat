use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "loot",
    &[TowerStatEffect::new(PlayerStatKind::Loot, 1.0)],
    10,
    Color::srgb(0.95, 0.78, 0.24),
).with_max_purchases(3);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct LootPlugin;

impl Plugin for LootPlugin {
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
