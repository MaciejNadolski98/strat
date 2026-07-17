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

use bevy::prelude::*;

use crate::resources::TowerStatEffect;
use crate::tags::TagInfo;

/// Systems that (re-)add an item's [`ItemKind`] to the [`crate::resources::Shop`]
/// pool in response to [`crate::resources::GameRestartEvent`]. Ordered to run
/// after `restart_game` and before the shop regenerates its offers, so every
/// item plugin's kind is back in the pool by the time offers are rolled.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct ItemPoolRestoreSet;

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
    }
}
