use crate::resources::{PlayerStatKind, TowerStatEffect};
use bevy::prelude::*;


#[derive(Clone, Copy)]
pub struct StatUpgradeDefinition {
    pub name: &'static str,
    pub effects: &'static [TowerStatEffect],
    pub cost: u32,
    pub icon_color: Color,
}

pub const ITEM_POTATO: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Potato",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 4.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 1.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -1.0),
    ],
    cost: 5,
    icon_color: Color::srgb(0.74, 0.18, 0.18),
};

pub const ITEM_MEDS: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Meds",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::Regeneration, 2.0),
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, -0.1),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -10.0),
    ],
    cost: 2,
    icon_color: Color::srgb(0.22, 0.62, 0.30),
};

pub const ITEM_COFFEE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Coffee",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::MaxHp, -1.0)
    ],
    cost: 5,
    icon_color: Color::srgb(0.86, 0.72, 0.24),
};

pub const ITEM_PASSIVE_INCOME: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Income",
    effects: &[TowerStatEffect::new(PlayerStatKind::PassiveIncome, 1.0)],
    cost: 10,
    icon_color: Color::srgb(0.95, 0.78, 0.24),
};

pub const ITEM_CRITICAL_CHANCE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Crit",
    effects: &[TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04)],
    cost: 5,
    icon_color: Color::srgb(0.70, 0.22, 0.22),
};

pub const ITEM_EXPLOSION_SIZE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Splash",
    effects: &[TowerStatEffect::new(PlayerStatKind::ExplosionSize, 4.0)],
    cost: 4,
    icon_color: Color::srgb(0.82, 0.44, 0.18),
};

pub const ITEM_EARTH_DAMAGE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Earth",
    effects: &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 4.0)],
    cost: 3,
    icon_color: Color::srgb(0.46, 0.34, 0.22),
};

pub const ITEM_FIRE_DAMAGE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Fire",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::FireDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, -4.0)
    ],
    cost: 3,
    icon_color: Color::srgb(0.86, 0.24, 0.12),
};

pub const ITEM_AIR_DAMAGE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Air",
    effects: &[TowerStatEffect::new(PlayerStatKind::AirDamage, 4.0)],
    cost: 3,
    icon_color: Color::srgb(0.58, 0.72, 0.92),
};

pub const ITEM_WATER_DAMAGE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Water",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 4.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, -4.0),
    ],
    cost: 3,
    icon_color: Color::srgb(0.18, 0.42, 0.78),
};

pub const ITEM_VITALITY: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Vitality",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::MaxHp, 5.0),
        TowerStatEffect::new(PlayerStatKind::Regeneration, 1.0),
    ],
    cost: 6,
    icon_color: Color::srgb(0.72, 0.34, 0.34),
};

pub const ITEM_OFFENSE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Offense",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::AttackSpeed, 0.12),
        TowerStatEffect::new(PlayerStatKind::CriticalChance, 0.04),
    ],
    cost: 7,
    icon_color: Color::srgb(0.82, 0.70, 0.24),
};

pub const ITEM_ELEMENTAL_FOCUS: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Elemental Focus",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::AirDamage, 2.0),
        TowerStatEffect::new(PlayerStatKind::WaterDamage, 2.0),
    ],
    cost: 9,
    icon_color: Color::srgb(0.34, 0.60, 0.84),
};

pub const ITEM_SIEGE: StatUpgradeDefinition = StatUpgradeDefinition {
    name: "Siege",
    effects: &[
        TowerStatEffect::new(PlayerStatKind::ExplosionSize, 3.0),
        TowerStatEffect::new(PlayerStatKind::EarthDamage, 2.0),
    ],
    cost: 8,
    icon_color: Color::srgb(0.74, 0.46, 0.20),
};
