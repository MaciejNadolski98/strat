pub mod ballista;
pub mod gatling;
pub mod cannon;
pub mod golem;
pub mod sniper;
pub mod sprayer;
pub mod tree;

pub use ballista::TOWER_BALLISTA;
pub use gatling::TOWER_GATLING;
pub use cannon::TOWER_CANNON;
pub use golem::TOWER_GOLEM;
pub use sniper::TOWER_SNIPER;
pub use sprayer::TOWER_SPRAYER;
pub use tree::TOWER_TREE;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::TowerStatEffect;


#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TowerKind {
    Ballista,
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
        self.definition().base_size
    }

    pub fn barrel_size(self) -> Vec2 {
        self.definition().barrel_size
    }

    pub fn barrel_offset(self) -> f32 {
        self.definition().barrel_offset
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

pub const ALL_TOWER_KINDS: [TowerKind; 7] = [
    TowerKind::Ballista,
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
    pub base_size: Vec2,
    pub barrel_size: Vec2,
    pub barrel_offset: f32,
    pub stat_effects: &'static [TowerStatEffect],
    pub custom_tooltip: Option<fn(f32, f32) -> String>,
}

pub struct TowerDefinitionPlugins;

impl PluginGroup for TowerDefinitionPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ballista::BallistaPlugin)
            .add(gatling::GatlingPlugin)
            .add(cannon::CannonPlugin)
            .add(sprayer::SprayerPlugin)
            .add(sniper::SniperPlugin)
            .add(golem::GolemPlugin)
            .add(tree::TreePlugin)
    }
}
