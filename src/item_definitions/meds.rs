use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Meds",
    &[
        TowerStatEffect::new(PlayerStatKind::Regeneration, 2.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.1),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -10.0),
    ],
    2,
    Color::srgb(0.22, 0.62, 0.30),
);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct MedsPlugin;

impl Plugin for MedsPlugin {
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
