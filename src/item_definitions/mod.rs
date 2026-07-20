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

use bevy::prelude::*;

use crate::components::DraftPreview;
use crate::resources::{GameRestartEvent, Shop, TowerStatEffect};
use crate::tags::TagInfo;
use crate::tower_definitions::TowerKind;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct ItemPoolRestoreSet;

pub fn unlock(
    tower: Option<TowerKind>,
    item: ItemKind,
) -> impl Fn(
    Query<&TowerKind, (Added<TowerKind>, Without<DraftPreview>)>,
    Local<bool>,
    ResMut<Shop>,
    EventReader<GameRestartEvent>,
) {
    move |new_towers, mut unlocked, mut shop, mut restart_events| {
        if restart_events.read().next().is_some() {
            *unlocked = false;
            shop.remove_from_pool(item);
            return;
        }

        let should_unlock = match tower {
            Some(tower) => new_towers.iter().any(|kind| *kind == tower),
            None => true,
        };

        if !*unlocked && should_unlock {
            *unlocked = true;
            shop.add_to_pool(item);
        }
    }
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
        ));
    }
}
