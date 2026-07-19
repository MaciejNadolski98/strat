use bevy::prelude::*;

use crate::components::{TemporaryProjectiles, TemporarySpread};
use crate::resources::{GamePhase, GameRestartEvent, ItemPurchasedEvent};
use crate::tags;
use crate::tower_definitions::sprayer::{self, SprayerTower};
use super::{unlock, ItemDefinition, ItemKind, ItemPoolRestoreSet};

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

#[derive(Resource, Default)]
struct NozzleStacks(u32);

pub struct NozzlePlugin;

impl Plugin for NozzlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NozzleStacks>();
        app.add_systems(
            Update,
            (
                unlock(Some(sprayer::KIND), KIND).in_set(ItemPoolRestoreSet),
                on_item_purchased,
                restart_stacks.in_set(ItemPoolRestoreSet),
            ),
        );
        app.add_systems(Update, apply_nozzle_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut stacks: ResMut<NozzleStacks>,
) {
    for event in events.read() {
        if event.kind == KIND {
            stacks.0 += 1;
        }
    }
}

fn restart_stacks(
    mut events: EventReader<GameRestartEvent>,
    mut stacks: ResMut<NozzleStacks>,
) {
    if events.read().next().is_some() {
        stacks.0 = 0;
    }
}

fn apply_nozzle_bonus(
    stacks: Res<NozzleStacks>,
    mut towers: Query<(&mut TemporaryProjectiles, &mut TemporarySpread), With<SprayerTower>>,
) {
    for (mut projectiles, mut spread) in &mut towers {
        projectiles.flat = stacks.0 as f32;
        spread.flat = stacks.0 as f32 * SPREAD_PER_STACK;
    }
}
