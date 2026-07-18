use bevy::prelude::*;

use crate::resources::{PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::resources::{GameRestartEvent, Shop};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Vitality",
    &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 5.0),
        TowerStatEffect::new(PlayerStatKind::Regeneration, 1.0),
    ],
    6,
    Color::srgb(0.72, 0.34, 0.34),
).with_tags(&[tags::BIOTIC]);

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct VitalityPlugin;

impl Plugin for VitalityPlugin {
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
