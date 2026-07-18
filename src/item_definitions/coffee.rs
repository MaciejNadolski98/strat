use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Coffee",
    &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -1.0),
    ],
    5,
    Color::srgb(0.86, 0.72, 0.24),
).with_tags(&[tags::MECHANICAL]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct CoffeePlugin;

impl Plugin for CoffeePlugin {
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
