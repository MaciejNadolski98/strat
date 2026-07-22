use bevy::prelude::*;

use crate::components::{TemporaryProjectiles, TemporarySpread};
use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::sprayer::{self, SprayerTower};
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

const SPREAD_PER_STACK: f32 = 0.15;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Extra Nozzle",
    &[],
    6,
    Color::srgb(0.35, 0.80, 0.72),
)
    .with_description("+1 Sprayer projectile per shot, wider spray")
    .with_tags(&[tags::MECHANICAL])
    .with_max_purchases(3);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct NozzlePlugin;

impl Plugin for NozzlePlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(sprayer::KIND), KIND);
        app.add_systems(Update, apply_nozzle_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_nozzle_bonus(
    shop: Res<Shop>,
    mut towers: Query<(&mut TemporaryProjectiles, &mut TemporarySpread), With<SprayerTower>>,
) {
    let stacks = shop.purchase_count(KIND);
    for (mut projectiles, mut spread) in &mut towers {
        projectiles.flat = stacks as f32;
        spread.flat = stacks as f32 * SPREAD_PER_STACK;
    }
}
