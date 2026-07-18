use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Potato",
    &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 4.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 1.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -1.0),
    ],
    5,
    Color::srgb(0.74, 0.18, 0.18),
).with_tags(&[tags::BIOTIC]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct PotatoPlugin;

impl Plugin for PotatoPlugin {
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
