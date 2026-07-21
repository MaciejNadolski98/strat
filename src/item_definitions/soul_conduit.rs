use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use crate::charges::try_emit_charge;
use crate::components::Tower;
use crate::game::GameState;
use crate::item_definitions::{unlock, UnlockCondition};
use crate::resources::ItemPurchasedEvent;
use crate::tags::{self, Conduit};
use crate::tower_definitions::soul_harvester::{self, SoulHarvestEvent};
use super::{ItemDefinition, ItemKind};

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Soul Conduit",
    &[],
    7,
    Color::srgb(0.62, 0.16, 0.72),
)
    .with_description("Soul Harvester produces a charge on every harvest")
    .with_tags(&[tags::INFERNAL])
    .with_max_purchases(1);

pub static KIND: ItemKind = ItemKind(&ITEM);

#[derive(Resource, Default)]
struct SoulConduitPurchased(bool);

pub struct SoulConduitPlugin;

impl Plugin for SoulConduitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoulConduitPurchased>();
        unlock(app, UnlockCondition::Tower(soul_harvester::KIND), KIND);
        app.add_systems(OnEnter(GameState::Playing), reset_purchased);
        app.add_systems(Update, on_item_purchased);
        app.add_systems(Update, emit_charge_on_harvest);
    }
}

fn reset_purchased(mut purchased: ResMut<SoulConduitPurchased>) {
    purchased.0 = false;
}

fn on_item_purchased(
    mut events: EventReader<ItemPurchasedEvent>,
    mut purchased: ResMut<SoulConduitPurchased>,
) {
    for event in events.read() {
        if event.kind == KIND {
            purchased.0 = true;
        }
    }
}

fn emit_charge_on_harvest(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    purchased: Res<SoulConduitPurchased>,
    mut events: EventReader<SoulHarvestEvent>,
    conduits: Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
) {
    for event in events.read() {
        if !purchased.0 {
            continue;
        }
        try_emit_charge(
            &mut commands, &mut meshes, &mut materials, &conduits,
            event.tower, event.position, soul_harvester::TOWER_SOUL_HARVESTER.range,
        );
    }
}
