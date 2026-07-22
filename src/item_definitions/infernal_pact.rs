use bevy::prelude::*;

use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::{FireDamage, PlayerStatKind, Shop, TowerStatEffect};
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

pub struct InfernalPactPlugin;

impl Plugin for InfernalPactPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(Update, apply_harvest_bonus);
    }
}

fn apply_harvest_bonus(
    shop: Res<Shop>,
    mut events: EventReader<SoulHarvestEvent>,
    mut fire_damage: ResMut<FireDamage>,
) {
    let purchased = shop.purchase_count(KIND) > 0;
    for _ in events.read() {
        if purchased {
            fire_damage.raw_value += FIRE_PER_HARVEST;
        }
    }
}
