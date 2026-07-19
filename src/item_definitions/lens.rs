use bevy::prelude::*;

use crate::components::{DraftPreview, TemporaryRange};
use crate::resources::{GamePhase, GameRestartEvent, ItemPurchasedEvent, Shop};
use crate::tags;
use crate::tower_definitions::{laser, TowerKind};
use super::{ItemDefinition, ItemKind, ItemPoolRestoreSet};

const RANGE_FLAT_FRACTION: f32 = 0.5;
static RANGE_BONUS: f32 = laser::TOWER_LASER.range * RANGE_FLAT_FRACTION;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Lens",
    &[],
    8,
    Color::srgb(0.70, 0.92, 0.98),
)
    .with_description("+50% Laser tower range")
    .with_tags(&[tags::CONDUIT])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

// `Shop::purchase_count` resets every wave (see `Shop::activate`), so it can't
// tell us whether Lens was ever bought this run - only a dedicated resource
// that's reset solely on `GameRestartEvent` can.
#[derive(Resource, Default)]
struct LensPurchased(bool);

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LensPurchased>();
        app.add_systems(
            Update,
            (
                unlock_on_first_laser,
                on_item_purchased,
                on_restart.in_set(ItemPoolRestoreSet),
            ),
        );
        app.add_systems(Update, apply_lens_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn unlock_on_first_laser(
    new_towers: Query<&TowerKind, (Added<TowerKind>, Without<DraftPreview>)>,
    purchased: Res<LensPurchased>,
    mut shop: ResMut<Shop>,
) {
    if !purchased.0 && new_towers.iter().any(|kind| *kind == laser::KIND) {
        shop.add_to_pool(KIND);
    }
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

fn on_restart(
    mut events: EventReader<GameRestartEvent>,
    mut shop: ResMut<Shop>,
    mut purchased: ResMut<LensPurchased>,
) {
    if events.read().next().is_some() {
        shop.remove_from_pool(KIND);
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
