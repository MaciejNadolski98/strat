use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::resources::{GamePhase, GameRestartEvent};
use crate::tower_definitions::laser;
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

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

#[derive(Resource, Default)]
struct LensPurchased(bool);

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LensPurchased>();
        app.add_systems(
            Update,
            (
                unlock(Some(laser::KIND), KIND).in_set(ItemPoolRestoreSet),
                reset_stacks.in_set(ItemPoolRestoreSet),
            ),
        );
        app.add_systems(Update, apply_lens_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn reset_stacks(
    mut events: EventReader<GameRestartEvent>,
    mut purchased: ResMut<LensPurchased>,
) {
    if events.read().next().is_some() {
        purchased.0 = false;
    }
}

fn apply_lens_bonus(
    purchased: Res<LensPurchased>,
    mut towers: Query<&mut TemporaryRange, With<laser::LaserTower>>,
) {
    if !purchased.0 {
        return;
    }
    for mut range in &mut towers {
        range.flat += RANGE_BONUS;
    }
}
