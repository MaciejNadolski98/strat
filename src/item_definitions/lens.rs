use bevy::prelude::*;

use crate::components::TemporaryRange;
use crate::resources::{GamePhase, GameRestartEvent, Shop};
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

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                unlock_on_first_laser,
                on_restart.in_set(ItemPoolRestoreSet),
            ),
        );
        app.add_systems(Update, apply_lens_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn unlock_on_first_laser(
    new_towers: Query<&TowerKind, Added<TowerKind>>,
    mut shop: ResMut<Shop>,
) {
    if new_towers.iter().any(|kind| *kind == laser::KIND) {
        shop.add_to_pool(KIND);
    }
}

fn on_restart(mut events: EventReader<GameRestartEvent>, mut shop: ResMut<Shop>) {
    if events.read().next().is_some() {
        shop.remove_from_pool(KIND);
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
