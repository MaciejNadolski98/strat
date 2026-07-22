use bevy::prelude::*;

use crate::effects::spawn_floating_text;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{Money, Shop};
use crate::tags;
use crate::tower_definitions::soul_harvester::{self, SoulHarvestEvent};
use super::{ItemDefinition, ItemKind};

const MONEY_PER_HARVEST: i32 = 5;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Soul Toll",
    &[],
    4,
    Color::srgb(0.78, 0.68, 0.24),
)
    .with_description("+$5 per Soul Harvester harvest")
    .with_tags(&[tags::INFERNAL]);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct SoulTollPlugin;

impl Plugin for SoulTollPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(Update, apply_harvest_bonus);
    }
}

fn apply_harvest_bonus(
    mut commands: Commands,
    shop: Res<Shop>,
    mut events: EventReader<SoulHarvestEvent>,
    mut money: ResMut<Money>,
) {
    let stacks = shop.purchase_count(KIND);
    for SoulHarvestEvent { tower: _, position } in events.read() {
        if stacks > 0 {
            let gain = MONEY_PER_HARVEST * stacks as i32;
            money.amount += gain;
            spawn_floating_text(
                &mut commands,
                format!("+${gain}"),
                position + Vec2::new(20.0, 28.0),
                Color::srgb(0.40, 0.92, 0.36),
                18.0,
            );
        }
    }
}
