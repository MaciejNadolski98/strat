use bevy::prelude::*;

use crate::item_definitions::unlock;
use crate::resources::{FireDamage, GameRestartEvent, ItemPurchasedEvent, PlayerStatKind};
use crate::tags;
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Pyromania",
    &[],
    6,
    Color::srgb(0.96, 0.44, 0.08),
)
    .with_description("Whenever you buy fire, +1 Fire")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub const KIND: ItemKind = ItemKind(&ITEM);

#[derive(Resource, Default)]
struct PyromaniaStacks(u32);

pub struct PyromaniaPlugin;

impl Plugin for PyromaniaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, unlock(None, KIND).in_set(ItemPoolRestoreSet));
        app.init_resource::<PyromaniaStacks>();
        app.add_systems(Update, (on_item_purchased, reset_stacks.in_set(ItemPoolRestoreSet)));
    }
}

fn reset_stacks(
    mut events: EventReader<GameRestartEvent>,
    mut stacks: ResMut<PyromaniaStacks>,
) {
    if events.read().next().is_some() {
        stacks.0 = 0;
    }
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut stacks: ResMut<PyromaniaStacks>,
    mut fire_damage: ResMut<FireDamage>,
) {
    for event in events.read() {
        if event.kind == KIND {
            stacks.0 += 1;
        }
        let boosts_fire = event.kind.effects().iter().any(|e| {
            matches!(e.kind, PlayerStatKind::FireDamage) && e.amount > 0.0
        });
        if boosts_fire && stacks.0 > 0 {
            fire_damage.raw_value += stacks.0 as f32;
        }
    }
}
