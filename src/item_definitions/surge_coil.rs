use bevy::prelude::*;

use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::capacitor::{self, CapacitorBonusCapacity, CapacitorTower};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

const CAPACITY_PER_STACK: f32 = 5.0;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Surge Coil",
    &[],
    6,
    Color::srgb(0.20, 0.70, 0.62),
)
    .with_description("+5 Capacitor max energy")
    .with_tags(&[tags::MECHANICAL])
    .with_max_purchases(2);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct SurgeCoilPlugin;

impl Plugin for SurgeCoilPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(capacitor::KIND), KIND);
        app.add_systems(Update, apply_capacity_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_capacity_bonus(
    shop: Res<Shop>,
    mut towers: Query<&mut CapacitorBonusCapacity, With<CapacitorTower>>,
) {
    let stacks = shop.purchase_count(KIND) as f32;
    if stacks == 0.0 {
        return;
    }
    let bonus = stacks * CAPACITY_PER_STACK;
    for mut cap in &mut towers {
        cap.0 += bonus;
    }
}
