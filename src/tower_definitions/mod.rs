pub mod templates;
pub use templates::{BarrelTemplate, BaseTemplate, TowerShape};

pub mod ballista;
pub mod brimstone;
pub mod catalyst;
pub mod cyclone;
pub mod dynamo;
pub mod pyre;
pub mod zephyr;
pub mod gatling;
pub mod cannon;
pub mod golem;
pub mod laser;
pub mod sniper;
pub mod soul_harvester;
pub mod sprayer;
pub mod tree;

use bevy::prelude::*;

use crate::components::DamageFormula;
use crate::resources::TowerStatEffect;
use crate::tags::TagInfo;

#[derive(Component, Clone, Copy)]
pub struct TowerKind(pub &'static TowerDefinition);

impl TowerKind {
    pub fn definition(self) -> &'static TowerDefinition {
        self.0
    }

    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn range(self) -> f32 {
        self.0.range
    }

    pub fn cooldown(self) -> f32 {
        self.0.cooldown
    }

    pub fn damage_formula(self) -> Option<DamageFormula> {
        self.0.damage_formula
    }

    pub fn projectile_speed(self) -> f32 {
        self.0.projectile_speed
    }

    pub fn upgraded_explosion_radius(self, explosion_size: f32) -> f32 {
        let base_radius = self.0.explosion_radius;
        if base_radius > 0.0 {
            base_radius + explosion_size
        } else {
            0.0
        }
    }

    pub fn angular_speed(self) -> f32 {
        self.0.angular_speed
    }

    pub fn base_color(self) -> Color {
        self.0.base_color
    }

    pub fn barrel_color(self) -> Color {
        self.0.barrel_color
    }

    pub fn base_size(self) -> Vec2 {
        self.0.base.size
    }

    pub fn base_shape(self) -> TowerShape {
        self.0.base.shape
    }

    pub fn barrel_size(self) -> Vec2 {
        self.0.barrel.size()
    }

    pub fn barrel_offset(self) -> f32 {
        self.0.barrel.offset()
    }

    pub fn stat_effects(self) -> &'static [TowerStatEffect] {
        self.0.stat_effects
    }

    pub fn tags(self) -> &'static [TagInfo] {
        self.0.tags
    }

    pub fn body_sprite(self, alpha: f32) -> Sprite {
        Sprite::from_color(self.base_color().with_alpha(alpha), self.base_size())
    }

    pub fn barrel_sprite(self, alpha: f32) -> Sprite {
        Sprite::from_color(self.barrel_color().with_alpha(alpha), self.barrel_size())
    }
}

impl PartialEq for TowerKind {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for TowerKind {}

impl std::hash::Hash for TowerKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const TowerDefinition).hash(state);
    }
}

pub struct TowerPlugins;

impl Plugin for TowerPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ballista::BallistaPlugin, catalyst::CatalystPlugin, cyclone::CyclonePlugin,
            pyre::PyrePlugin, zephyr::ZephyrPlugin, gatling::GatlingPlugin,
            cannon::CannonPlugin, sprayer::SprayerPlugin, sniper::SniperPlugin,
            golem::GolemPlugin, tree::TreePlugin, laser::LaserPlugin,
            brimstone::BrimstonePlugin, dynamo::DynamoPlugin,
            soul_harvester::SoulHarvesterPlugin,
        ));
    }
}

#[derive(Resource, Default)]
pub struct TowerRegistry {
    pub kinds: Vec<TowerKind>,
}

#[derive(Clone, Copy)]
pub struct TooltipConfig {
    pub show_damage: bool,
    pub show_range: bool,
    pub show_cooldown: bool,
    pub show_crit: bool,
    pub show_projectile: bool,
    pub show_splash: bool,
    pub show_turn_speed: bool,
}

#[allow(dead_code)]
impl TooltipConfig {
    pub const STANDARD: Self = Self {
        show_damage: true,
        show_range: true,
        show_cooldown: true,
        show_crit: true,
        show_projectile: true,
        show_splash: false,
        show_turn_speed: true,
    };

    pub const AURA: Self = Self {
        show_damage: false,
        show_range: true,
        show_cooldown: false,
        show_crit: false,
        show_projectile: false,
        show_splash: false,
        show_turn_speed: false,
    };

    pub const UTILITY: Self = Self {
        show_damage: false,
        show_range: false,
        show_cooldown: false,
        show_crit: false,
        show_projectile: false,
        show_splash: false,
        show_turn_speed: false,
    };

    pub const fn with_damage(self, value: bool) -> Self {
        Self { show_damage: value, ..self }
    }

    pub const fn with_range(self, value: bool) -> Self {
        Self { show_range: value, ..self }
    }

    pub const fn with_cooldown(self, value: bool) -> Self {
        Self { show_cooldown: value, ..self }
    }

    pub const fn with_crit(self, value: bool) -> Self {
        Self { show_crit: value, ..self }
    }

    pub const fn with_projectile(self, value: bool) -> Self {
        Self { show_projectile: value, ..self }
    }

    pub const fn with_splash(self, value: bool) -> Self {
        Self { show_splash: value, ..self }
    }

    pub const fn with_turn_speed(self, value: bool) -> Self {
        Self { show_turn_speed: value, ..self }
    }
}

#[derive(Clone, Copy)]
pub struct TowerDefinition {
    pub name: &'static str,
    pub range: f32,
    pub cooldown: f32,
    pub damage_formula: Option<DamageFormula>,
    pub projectile_speed: f32,
    pub explosion_radius: f32,
    pub angular_speed: f32,
    pub spread: f32,
    pub piercing: u32,
    pub piercing_damage: f32,
    pub projectiles_per_shot: u32,
    pub base_color: Color,
    pub barrel_color: Color,
    pub base: BaseTemplate,
    pub barrel: BarrelTemplate,
    pub stat_effects: &'static [TowerStatEffect],
    pub tooltip_config: TooltipConfig,
    pub tags: &'static [TagInfo],
}

impl TowerDefinition {
    const fn new(
        name: &'static str,
        range: f32,
        cooldown: f32,
        damage_formula: Option<DamageFormula>,
        base_color: Color,
        base: BaseTemplate,
        barrel: BarrelTemplate,
    ) -> Self {
        Self {
            name,
            range,
            cooldown,
            damage_formula,
            projectile_speed: 0.0,
            explosion_radius: 0.0,
            angular_speed: 0.0,
            spread: 0.0,
            piercing: 0,
            piercing_damage: 0.0,
            projectiles_per_shot: 1,
            base_color,
            barrel_color: base_color,
            base,
            barrel,
            stat_effects: &[],
            tooltip_config: TooltipConfig::STANDARD,
            tags: &[],
        }
    }

    pub const fn new_attacking(
        name: &'static str,
        range: f32,
        cooldown: f32,
        damage_formula: DamageFormula,
        base_color: Color,
        base: BaseTemplate,
        barrel: BarrelTemplate,
        projectile_speed: f32,
        angular_speed: f32,
    ) -> Self {
        Self::new(name, range, cooldown, Some(damage_formula), base_color, base, barrel)
            .with_projectile_speed(projectile_speed)
            .with_angular_speed(angular_speed)
    }

    pub const fn new_utility(
        name: &'static str,
        range: f32,
        base_color: Color,
        base: BaseTemplate,
        barrel: BarrelTemplate,
    ) -> Self {
        Self::new(name, range, 0.0, None, base_color, base, barrel)
    }

    const fn with_projectile_speed(self, projectile_speed: f32) -> Self {
        Self { projectile_speed, ..self }
    }

    pub const fn with_explosion_radius(self, explosion_radius: f32) -> Self {
        Self { explosion_radius, ..self }
    }

    const fn with_angular_speed(self, angular_speed: f32) -> Self {
        Self { angular_speed, ..self }
    }

    pub const fn with_spread(self, spread: f32) -> Self {
        Self { spread, ..self }
    }

    pub const fn with_piercing(self, piercing: u32) -> Self {
        Self { piercing, ..self }
    }

    pub const fn with_piercing_damage(self, piercing_damage: f32) -> Self {
        Self { piercing_damage, ..self }
    }

    pub const fn with_projectiles_per_shot(self, projectiles_per_shot: u32) -> Self {
        Self { projectiles_per_shot, ..self }
    }

    pub const fn with_barrel_color(self, barrel_color: Color) -> Self {
        Self { barrel_color, ..self }
    }

    pub const fn with_stat_effects(self, stat_effects: &'static [TowerStatEffect]) -> Self {
        Self { stat_effects, ..self }
    }

    pub const fn with_tooltip_config(self, tooltip_config: TooltipConfig) -> Self {
        Self { tooltip_config, ..self }
    }

    pub const fn with_tags(self, tags: &'static [TagInfo]) -> Self {
        Self { tags, ..self }
    }

    pub const fn with_cooldown(self, cooldown: f32) -> Self {
        Self { cooldown, ..self }
    }
}
