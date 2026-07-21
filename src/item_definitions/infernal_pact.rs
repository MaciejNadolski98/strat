use bevy::prelude::*;

use crate::game::GameState;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{FireDamage, ItemPurchasedEvent, PlayerStatKind, TowerStatEffect};
use crate::tags;
use crate::tower_definitions::soul_harvester::{self, SoulHarvestEvent};
use super::{ItemDefinition, ItemKind};

const FIRE_PER_HARVEST: f32 = 1.0;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Infernal Pact",
    &[TowerStatEffect::new(PlayerStatKind::FireDamage, 1.0)],
    7,
    Color::srgb(0.88, 0.22, 0.10),
)
    .with_description("+1 Fire per Soul Harvester harvest")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

#[derive(Resource, Default)]
struct InfernalPactPurchased(bool);

pub struct InfernalPactPlugin;

impl Plugin for InfernalPactPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfernalPactPurchased>();
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(OnEnter(GameState::Playing), reset_purchased);
        app.add_systems(Update, on_item_purchased);
        app.add_systems(Update, apply_harvest_bonus);
    }
}

fn reset_purchased(mut purchased: ResMut<InfernalPactPurchased>) {
    purchased.0 = false;
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut purchased: ResMut<InfernalPactPurchased>,
) {
    for event in events.read() {
        if event.kind == KIND {
            purchased.0 = true;
        }
    }
}

fn apply_harvest_bonus(
    purchased: Res<InfernalPactPurchased>,
    mut events: EventReader<SoulHarvestEvent>,
    mut fire_damage: ResMut<FireDamage>,
) {
    for _ in events.read() {
        if purchased.0 {
            fire_damage.raw_value += FIRE_PER_HARVEST;
        }
    }
}
