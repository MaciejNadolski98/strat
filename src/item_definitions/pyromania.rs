use bevy::prelude::*;

use crate::resources::{FireDamage, ItemPurchasedEvent, PlayerStatKind};
use super::{ItemDefinition, ItemKind, ItemRegistry};

pub const ITEM: ItemDefinition = ItemDefinition {
    name: "Pyromania",
    effects: &[],
    cost: 6,
    icon_color: Color::srgb(0.96, 0.44, 0.08),
};

pub const KIND: ItemKind = ItemKind(&ITEM);

#[derive(Resource, Default)]
struct PyromaniaStacks(u32);

pub struct PyromaniaPlugin;

impl Plugin for PyromaniaPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<ItemRegistry>().kinds.push(KIND);
        app.init_resource::<PyromaniaStacks>();
        app.add_systems(Update, on_item_purchased);
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
            fire_damage.value += stacks.0 as f32;
        }
    }
}
