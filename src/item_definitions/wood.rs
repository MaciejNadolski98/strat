use bevy::prelude::*;

use crate::components::Tower;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{FireDamage, ItemPurchasedEvent};
use crate::tags;
use crate::tower_definitions::tree::{self, TreeTower};
use super::{ItemDefinition, ItemKind};

const BONUS_PER_TREE: u32 = 2;

pub const ITEM: ItemDefinition = ItemDefinition::new(
    "Wood",
    &[],
    5,
    Color::srgb(0.59, 0.44, 0.20),
)
    .with_tags(&[tags::BIOTIC])
    .with_description("+2 Fire for each Tree on board");

pub const KIND: ItemKind = ItemKind(&ITEM);

pub struct WoodPlugin;

impl Plugin for WoodPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(tree::KIND), KIND);
        app.add_systems(Update, apply_effect);
    }
}

fn apply_effect(
    mut fire: ResMut<FireDamage>,
    mut events: EventReader<ItemPurchasedEvent>,
    trees: Query<(), (With<TreeTower>, With<Tower>)>,
) {
    for event in events.read() {
        if event.kind == KIND {
            fire.0.raw_value += (trees.iter().count() as u32 * BONUS_PER_TREE) as f32;
        }
    }
}
