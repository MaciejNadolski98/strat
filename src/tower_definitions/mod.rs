pub mod templates;
pub use templates::{BarrelTemplate, BaseTemplate, TowerShape};

pub mod ballista;
pub mod catalyst;
pub mod cyclone;
pub mod pyre;
pub mod zephyr;
pub mod gatling;
pub mod cannon;
pub mod golem;
pub mod sniper;
pub mod sprayer;
pub mod tree;

pub use ballista::TOWER_BALLISTA;
pub use catalyst::TOWER_CATALYST;
pub use cyclone::TOWER_CYCLONE;
pub use pyre::TOWER_PYRE;
pub use zephyr::TOWER_ZEPHYR;
pub use gatling::TOWER_GATLING;
pub use cannon::TOWER_CANNON;
pub use golem::TOWER_GOLEM;
pub use sniper::TOWER_SNIPER;
pub use sprayer::TOWER_SPRAYER;
pub use tree::TOWER_TREE;

use std::collections::HashMap;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::TowerStatEffect;

#[derive(Resource, Default)]
pub struct CustomTooltipTexts(pub HashMap<TowerKind, String>);


#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerKind {
    Ballista,
    Catalyst,
    Cyclone,
    Pyre,
    Zephyr,
    Gatling,
    Cannon,
    Sprayer,
    Sniper,
    Golem,
    Tree,
}

impl TowerKind {
    pub fn definition(self) -> &'static TowerDefinition {
        match self {
            Self::Ballista => &TOWER_BALLISTA,
            Self::Catalyst => &TOWER_CATALYST,
            Self::Cyclone => &TOWER_CYCLONE,
            Self::Pyre => &TOWER_PYRE,
            Self::Zephyr => &TOWER_ZEPHYR,
            Self::Gatling => &TOWER_GATLING,
            Self::Cannon => &TOWER_CANNON,
            Self::Sprayer => &TOWER_SPRAYER,
            Self::Sniper => &TOWER_SNIPER,
            Self::Golem => &TOWER_GOLEM,
            Self::Tree => &TOWER_TREE,
        }
    }

    pub fn name(self) -> &'static str {
        self.definition().name
    }

    pub fn range(self) -> f32 {
        self.definition().range
    }

    pub fn cooldown(self) -> f32 {
        self.definition().cooldown
    }

    pub fn damage_formula(self) -> DamageFormula {
        self.definition().damage_formula
    }

    pub fn projectile_speed(self) -> f32 {
        self.definition().projectile_speed
    }

    pub fn upgraded_explosion_radius(self, explosion_size: f32) -> f32 {
        let base_radius = self.definition().explosion_radius;
        if base_radius > 0.0 {
            base_radius + explosion_size
        } else {
            0.0
        }
    }

    pub fn angular_speed(self) -> f32 {
        self.definition().angular_speed
    }

    pub fn base_color(self) -> Color {
        self.definition().base_color
    }

    pub fn barrel_color(self) -> Color {
        self.definition().barrel_color
    }

    pub fn base_size(self) -> Vec2 {
        self.definition().base.size
    }

    pub fn base_shape(self) -> TowerShape {
        self.definition().base.shape
    }

    pub fn barrel_size(self) -> Vec2 {
        self.definition().barrel.size()
    }

    pub fn barrel_offset(self) -> f32 {
        self.definition().barrel.offset()
    }

    pub fn stat_effects(self) -> &'static [TowerStatEffect] {
        self.definition().stat_effects
    }

    pub fn body_sprite(self, alpha: f32) -> Sprite {
        Sprite::from_color(self.base_color().with_alpha(alpha), self.base_size())
    }

    pub fn barrel_sprite(self, alpha: f32) -> Sprite {
        Sprite::from_color(self.barrel_color().with_alpha(alpha), self.barrel_size())
    }
}

pub const ALL_TOWER_KINDS: [TowerKind; 11] = [
    TowerKind::Ballista,
    TowerKind::Catalyst,
    TowerKind::Cyclone,
    TowerKind::Pyre,
    TowerKind::Zephyr,
    TowerKind::Gatling,
    TowerKind::Cannon,
    TowerKind::Sprayer,
    TowerKind::Sniper,
    TowerKind::Golem,
    TowerKind::Tree,
];

#[derive(Clone, Copy)]
pub struct TowerDefinition {
    pub name: &'static str,
    pub range: f32,
    pub cooldown: f32,
    pub damage_formula: DamageFormula,
    pub projectile_speed: f32,
    pub explosion_radius: f32,
    pub angular_speed: f32,
    pub base_color: Color,
    pub barrel_color: Color,
    pub base: BaseTemplate,
    pub barrel: BarrelTemplate,
    pub stat_effects: &'static [TowerStatEffect],
}

pub struct TowerDefinitionPlugins;

impl PluginGroup for TowerDefinitionPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ballista::BallistaPlugin)
            .add(catalyst::CatalystPlugin)
            .add(cyclone::CyclonePlugin)
            .add(pyre::PyrePlugin)
            .add(zephyr::ZephyrPlugin)
            .add(gatling::GatlingPlugin)
            .add(cannon::CannonPlugin)
            .add(sprayer::SprayerPlugin)
            .add(sniper::SniperPlugin)
            .add(golem::GolemPlugin)
            .add(tree::TreePlugin)
    }
}
