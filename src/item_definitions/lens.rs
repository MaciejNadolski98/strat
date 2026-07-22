use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::resources::{GamePhase, Shop};
use crate::tower_definitions::laser;
use super::{unlock, ItemDefinition, ItemKind, UnlockCondition};

const RANGE_FLAT_FRACTION: f32 = 0.5;
static RANGE_BONUS: f32 = laser::TOWER_LASER.range * RANGE_FLAT_FRACTION;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Lens",
    &[],
    8,
    Color::srgb(0.70, 0.92, 0.98),
)
    .with_description("+50% Laser tower range")
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(laser::KIND), KIND);
        app.add_systems(Update, apply_lens_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_lens_bonus(
    shop: Res<Shop>,
    mut towers: Query<&mut TemporaryRange, With<laser::LaserTower>>,
) {
    if shop.purchase_count(KIND) == 0 {
        return;
    }
    for mut range in &mut towers {
        range.flat += RANGE_BONUS;
    }
}
