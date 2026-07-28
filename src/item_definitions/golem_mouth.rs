use bevy::prelude::*;

use crate::resources::{EarthDamage, EnemyKilledEvent, Shop};
use crate::tags;
use crate::tower_definitions::golem::{self, GolemTower};
use super::{golem_heart, unlock, ItemDefinition, ItemKind, UnlockCondition};

const EARTH_PER_KILL: f32 = 1.0;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Golem's Mouth",
    &[],
    5,
    Color::srgb(0.72, 0.30, 0.20),
)
    .with_description("+1 Earth per Golem kill")
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct GolemMouthPlugin;

impl Plugin for GolemMouthPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(golem::KIND), KIND);
        app.add_systems(Update, apply_kill_bonus);
    }
}

fn apply_kill_bonus(
    shop: Res<Shop>,
    mut events: EventReader<EnemyKilledEvent>,
    golems: Query<(), With<GolemTower>>,
    mut earth_damage: ResMut<EarthDamage>,
) {
    if shop.purchase_count(KIND) == 0 {
        return;
    }
    let multiplier = if shop.purchase_count(golem_heart::KIND) > 0 { 2.0 } else { 1.0 };
    for event in events.read() {
        if golems.contains(event.source_tower) {
            earth_damage.raw_value += EARTH_PER_KILL * multiplier;
        }
    }
}
