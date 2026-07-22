use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{FireDamage, ItemPurchasedEvent, PlayerStatKind, Shop};
use crate::tags;
use super::{ItemDefinition, ItemKind};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Pyromania",
    &[],
    6,
    Color::srgb(0.96, 0.44, 0.08),
)
    .with_description("Whenever you buy fire, +1 Fire")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct PyromaniaPlugin;

impl Plugin for PyromaniaPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Always, KIND);
        app.add_systems(Update, on_item_purchased);
    }
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    shop: Res<Shop>,
    mut fire_damage: ResMut<FireDamage>,
) {
    let stacks = shop.purchase_count(KIND);
    if stacks == 0 {
        return;
    }
    for event in events.read() {
        let boosts_fire = event.kind.effects().iter().any(|e| {
            matches!(e.kind, PlayerStatKind::FireDamage) && e.amount > 0.0
        });
        if boosts_fire {
            fire_damage.raw_value += stacks as f32;
        }
    }
}
