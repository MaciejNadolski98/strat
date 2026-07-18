use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Offense",
    &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04),
    ],
    7,
    Color::srgb(0.82, 0.70, 0.24),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct OffensePlugin;

impl Plugin for OffensePlugin {
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
