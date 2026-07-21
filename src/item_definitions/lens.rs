use bevy::prelude::*;

use crate::{components::TemporaryRange, resources::ItemPurchasedEvent};
use crate::game::GameState;
use crate::resources::GamePhase;
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

#[derive(Resource, Default)]
struct LensPurchased(bool);

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LensPurchased>();
        unlock(app, UnlockCondition::Tower(laser::KIND), KIND);
        app.add_systems(OnEnter(GameState::Playing), reset_stacks);
        app.add_systems(Update, on_item_purchased);
        app.add_systems(Update, apply_lens_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn reset_stacks(mut purchased: ResMut<LensPurchased>) {
    purchased.0 = false;
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut purchased: ResMut<LensPurchased>,
) {
    for event in events.read() {
        if event.kind == KIND {
            purchased.0 = true;
        }
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
