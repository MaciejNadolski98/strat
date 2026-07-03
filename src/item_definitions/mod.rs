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

#[derive(Clone, Copy)]
pub struct ItemDefinition {
    pub name: &'static str,
    pub effects: &'static [TowerStatEffect],
    pub cost: u32,
    pub icon_color: Color,
}

#[derive(Clone, Copy)]
pub struct ItemKind(pub &'static ItemDefinition);

impl ItemKind {
    pub fn name(self) -> &'static str {
        self.0.name
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

#[derive(Resource, Default)]
pub struct ItemRegistry {
    pub kinds: Vec<ItemKind>,
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
