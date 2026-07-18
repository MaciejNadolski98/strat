use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Crit",
    &[TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04)],
    5,
    Color::srgb(0.70, 0.22, 0.22),
).with_tags(&[tags::MECHANICAL]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CritPlugin;

impl Plugin for CritPlugin {
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
