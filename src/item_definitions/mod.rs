pub mod potato;
pub mod meds;
pub mod pyromania;
pub mod coffee;
pub mod loot;
pub mod crit;
pub mod splash;
pub mod earth;
pub mod fire;
pub mod air;
pub mod water;
pub mod vitality;
pub mod offense;
pub mod elemental_focus;
pub mod siege;
pub mod lens;
pub mod nozzle;
pub mod apple;
pub mod wood;
pub mod soul_conduit;
pub mod extended_reach;
pub mod soul_toll;
pub mod infernal_pact;

use std::collections::HashSet;

use bevy::prelude::*;

use crate::components::DraftPreview;
use crate::game::GameState;
use crate::resources::{ItemPurchasedEvent, Shop, TowerStatEffect};
use crate::tags::TagInfo;
use crate::tower_definitions::TowerKind;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct ItemPoolRestoreSet;

#[derive(Resource, Default)]
pub(crate) struct UnlockedItems(HashSet<ItemKind>);

pub enum UnlockCondition {
    Always,
    Tower(TowerKind),
    Item(ItemKind),
}

pub fn unlock(app: &mut App, condition: UnlockCondition, item: ItemKind) {
    match condition {
        UnlockCondition::Always => {
            let unlock_system = move |mut shop: ResMut<Shop>, mut unlocked: ResMut<UnlockedItems>| {
                if unlocked.0.insert(item) {
                    shop.add_to_pool(item);
                }
            };
            app.add_systems(OnEnter(GameState::Playing), unlock_system.in_set(ItemPoolRestoreSet));
        }
        UnlockCondition::Tower(tower) => {
            let unlock_system = move |
                new_towers: Query<&TowerKind, (Added<TowerKind>, Without<DraftPreview>)>,
                mut shop: ResMut<Shop>,
                mut unlocked: ResMut<UnlockedItems>,
            | {
                if unlocked.0.contains(&item) {
                    return;
                }
                if new_towers.iter().any(|kind| *kind == tower) {
                    unlocked.0.insert(item);
                    shop.add_to_pool(item);
                }
            };
            app.add_systems(Update, unlock_system.in_set(ItemPoolRestoreSet));
        }
        UnlockCondition::Item(required_item) => {
            let unlock_system = move |
                mut events: EventReader<ItemPurchasedEvent>,
                mut shop: ResMut<Shop>,
                mut unlocked: ResMut<UnlockedItems>,
            | {
                if unlocked.0.contains(&item) {
                    return;
                }
                if events.read().any(|event| event.kind == required_item) {
                    unlocked.0.insert(item);
                    shop.add_to_pool(item);
                }
            };
            app.add_systems(Update, unlock_system.in_set(ItemPoolRestoreSet));
        }
    }

    let lock_system = move |mut shop: ResMut<Shop>, mut unlocked: ResMut<UnlockedItems>| {
        unlocked.0.remove(&item);
        shop.remove_from_pool(item);
    };
    app.add_systems(OnEnter(GameState::Playing), lock_system.in_set(ItemPoolRestoreSet));
}

#[derive(Clone, Copy)]
pub struct ItemDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub effects: &'static [TowerStatEffect],
    pub cost: u32,
    pub icon_color: Color,
    pub tags: &'static [TagInfo],
    pub max_purchases: Option<u32>,
}

impl ItemDefinition {
    pub const fn new(
        name: &'static str,
        effects: &'static [TowerStatEffect],
        cost: u32,
        icon_color: Color,
    ) -> Self {
        Self {
            name,
            description: "",
            effects,
            cost,
            icon_color,
            tags: &[],
            max_purchases: None,
        }
    }

    pub const fn with_description(self, description: &'static str) -> Self {
        Self { description, ..self }
    }

    pub const fn with_tags(self, tags: &'static [TagInfo]) -> Self {
        Self { tags, ..self }
    }

    pub const fn with_max_purchases(self, max_purchases: u32) -> Self {
        Self { max_purchases: Some(max_purchases), ..self }
    }
}

#[derive(Clone, Copy)]
pub struct ItemKind(pub &'static ItemDefinition);

impl ItemKind {
    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn description(self) -> &'static str {
        self.0.description
    }

    pub fn effects(self) -> &'static [TowerStatEffect] {
        self.0.effects
    }

    pub fn cost(self) -> u32 {
        self.0.cost
    }

    pub fn icon_color(self) -> Color {
        self.0.icon_color
    }

    pub fn effect_text(self) -> String {
        self.effects()
            .iter()
            .map(|e| e.effect_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn tags(self) -> &'static [TagInfo] {
        self.0.tags
    }

    pub fn max_purchases(self) -> Option<u32> {
        self.0.max_purchases
    }
}

impl PartialEq for ItemKind {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for ItemKind {}

impl std::hash::Hash for ItemKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const ItemDefinition).hash(state);
    }
}

pub struct ItemPlugins;

impl Plugin for ItemPlugins {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnlockedItems>();
        app.add_plugins((
            potato::PotatoPlugin,
            meds::MedsPlugin,
            pyromania::PyromaniaPlugin,
            coffee::CoffeePlugin,
            loot::LootPlugin,
            crit::CritPlugin,
            splash::SplashPlugin,
            earth::EarthPlugin,
            fire::FirePlugin,
            air::AirPlugin,
            water::WaterPlugin,
            vitality::VitalityPlugin,
            offense::OffensePlugin,
            elemental_focus::ElementalFocusPlugin,
            siege::SiegePlugin,
        ));
        app.add_plugins((
            lens::LensPlugin,
            nozzle::NozzlePlugin,
            apple::ApplePlugin,
            wood::WoodPlugin,
            soul_conduit::SoulConduitPlugin,
            extended_reach::ExtendedReachPlugin,
            soul_toll::SoulTollPlugin,
            infernal_pact::InfernalPactPlugin,
        ));
    }
}
