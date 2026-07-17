use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Splash",
    description: "",
    effects: &[TowerStatEffect::new(PlayerStatKind::ExplosionSize, 4.0)],
    cost: 4,
    icon_color: Color::srgb(0.82, 0.44, 0.18),
    tags: &[tags::MECHANICAL],
    max_purchases: None,
};

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
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
