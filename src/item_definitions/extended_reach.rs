use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::game::GameState;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{GamePhase, ItemPurchasedEvent};
use crate::tags;
use crate::tower_definitions::soul_harvester::{self, SoulHarvesterTower};
use super::{ItemDefinition, ItemKind};

const RANGE_MULTIPLIER: f32 = 1.25;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Extended Reach",
    &[],
    7,
    Color::srgb(0.46, 0.20, 0.66),
)
    .with_description("+25% Soul Harvester range")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

#[derive(Resource, Default)]
struct ExtendedReachPurchased(bool);

pub struct ExtendedReachPlugin;

impl Plugin for ExtendedReachPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtendedReachPurchased>();
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(OnEnter(GameState::Playing), reset_purchased);
        app.add_systems(Update, on_item_purchased);
        app.add_systems(Update, apply_range_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn reset_purchased(mut purchased: ResMut<ExtendedReachPurchased>) {
    purchased.0 = false;
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut purchased: ResMut<ExtendedReachPurchased>,
) {
    for event in events.read() {
        if event.kind == KIND {
            purchased.0 = true;
        }
    }
}

fn apply_range_bonus(
    purchased: Res<ExtendedReachPurchased>,
    mut towers: Query<&mut TemporaryRange, With<SoulHarvesterTower>>,
) {
    if !purchased.0 {
        return;
    }
    for mut range in &mut towers {
        range.multiplier *= RANGE_MULTIPLIER;
    }
}
