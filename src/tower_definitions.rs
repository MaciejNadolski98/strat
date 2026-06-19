use crate::{components::{DamageFormula, TowerKind}, resources::{PlayerStatKind, TowerStatEffect}};
use bevy::prelude::*;


pub const ALL_TOWER_KINDS: [TowerKind; 4] = [
    TowerKind::Ballista,
    TowerKind::Cannon,
    TowerKind::Sprayer,
    TowerKind::Sniper,
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
    pub cost: u32,
    pub stat_effects: &'static [TowerStatEffect],
}

pub const TOWER_BALLISTA: TowerDefinition = TowerDefinition {
    name: "Ballista",
    range: 185.0,
    cooldown: 0.73,
    damage_formula: DamageFormula {
        flat: 24,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.75,
        water_multiplier: 0.75,
    },
    projectile_speed: 430.0,
    explosion_radius: 0.0,
    angular_speed: 1.6,
    base_color: Color::srgb(0.22, 0.42, 0.74),
    barrel_color: Color::srgb(0.67, 0.83, 0.96),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::new(12.0, 38.0),
    barrel_offset: 16.0,
    cost: 10,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.03),
    ],
};

pub const TOWER_CANNON: TowerDefinition = TowerDefinition {
    name: "Cannon",
    range: 150.0,
    cooldown: 1.45,
    damage_formula: DamageFormula {
        flat: 34,
        crit_multiplier: 1.5,
        earth_multiplier: 1.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 320.0,
    explosion_radius: 64.0,
    angular_speed: 1.0,
    base_color: Color::srgb(0.42, 0.36, 0.30),
    barrel_color: Color::srgb(0.74, 0.66, 0.54),
    base_size: Vec2::new(40.0, 40.0),
    barrel_size: Vec2::new(18.0, 30.0),
    barrel_offset: 13.0,
    cost: 16,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 12.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.08),
    ],
};

pub const TOWER_SPRAYER: TowerDefinition = TowerDefinition {
    name: "Sprayer",
    range: 125.0,
    cooldown: 0.32,
    damage_formula: DamageFormula {
        flat: 11,
        crit_multiplier: 2.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 1.0,
    },
    projectile_speed: 520.0,
    explosion_radius: 0.0,
    angular_speed: 4.2,
    base_color: Color::srgb(0.20, 0.52, 0.46),
    barrel_color: Color::srgb(0.62, 0.92, 0.78),
    base_size: Vec2::new(32.0, 32.0),
    barrel_size: Vec2::new(10.0, 28.0),
    barrel_offset: 12.0,
    cost: 18,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::PassiveIncome, 1.0),
    ],
};

pub const TOWER_SNIPER: TowerDefinition = TowerDefinition {
    name: "Sniper",
    range: 260.0,
    cooldown: 1.75,
    damage_formula: DamageFormula {
        flat: 55,
        crit_multiplier: 5.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 1.9,
    },
    projectile_speed: 720.0,
    explosion_radius: 0.0,
    angular_speed: 0.9,
    base_color: Color::srgb(0.34, 0.28, 0.56),
    barrel_color: Color::srgb(0.82, 0.76, 0.98),
    base_size: Vec2::new(36.0, 36.0),
    barrel_size: Vec2::new(8.0, 48.0),
    barrel_offset: 20.0,
    cost: 15,
    stat_effects: &[
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.08),
        TowerStatEffect::new(PlayerStatKind::Regeneration, -1.0),
    ],
};
