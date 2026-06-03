use bevy::prelude::*;

use crate::components::TowerKind;
use crate::constants::{PRICE_GROWTH, SHOP_REROLL_COST, TOWER_COST};

#[derive(Resource)]
pub struct Money {
    pub amount: i32,
}

#[derive(Resource)]
pub struct CurrentHp {
    pub amount: i32,
}

#[derive(Resource)]
pub struct MaxHp {
    pub amount: i32,
}

#[derive(Resource)]
pub struct KillCount {
    pub amount: u32,
}

#[derive(Resource)]
pub struct GameOver {
    pub value: bool,
}

#[derive(Resource)]
pub struct GameWon {
    pub value: bool,
}

#[derive(Resource)]
pub struct Paused {
    pub value: bool,
}

#[derive(Resource)]
pub struct Regeneration {
    pub amount: i32,
}

#[derive(Resource)]
pub struct AttackSpeed {
    pub value: f32,
}

#[derive(Resource)]
pub struct PassiveIncome {
    pub amount: i32,
}

#[derive(Resource)]
pub struct CriticalChance {
    pub value: f32,
}

#[derive(Resource)]
pub struct ExplosionSize {
    pub value: f32,
}

#[derive(Resource)]
pub struct EarthDamage {
    pub value: f32,
}

#[derive(Resource)]
pub struct FireDamage {
    pub value: f32,
}

#[derive(Resource)]
pub struct AirDamage {
    pub value: f32,
}

#[derive(Resource)]
pub struct WaterDamage {
    pub value: f32,
}

#[derive(Resource)]
pub struct WaveNumber {
    pub value: u32,
}

#[derive(Resource)]
pub struct EnemiesRemaining {
    pub count: u32,
}

#[derive(Resource)]
pub struct SpawnTimer {
    pub elapsed: f32,
    pub spawned_by_group: Vec<u32>,
}

impl SpawnTimer {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            spawned_by_group: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.spawned_by_group.clear();
    }

    pub fn spawned_in_group(&self, index: usize) -> u32 {
        self.spawned_by_group.get(index).copied().unwrap_or(0)
    }

    pub fn set_spawned_in_group(&mut self, index: usize, count: u32) {
        if self.spawned_by_group.len() <= index {
            self.spawned_by_group.resize(index + 1, 0);
        }
        self.spawned_by_group[index] = count;
    }
}

#[derive(Resource)]
pub struct NextWaveTimer {
    pub timer: Timer,
}

#[derive(Clone, Copy)]
pub enum StatUpgradeKind {
    MaxHp,
    Regeneration,
    AttackSpeed,
    PassiveIncome,
    CriticalChance,
    ExplosionSize,
    EarthDamage,
    FireDamage,
    AirDamage,
    WaterDamage,
}

impl StatUpgradeKind {
    pub fn random() -> Self {
        match rand::random::<u8>() % 10 {
            0 => Self::MaxHp,
            1 => Self::Regeneration,
            2 => Self::AttackSpeed,
            3 => Self::PassiveIncome,
            4 => Self::CriticalChance,
            5 => Self::ExplosionSize,
            6 => Self::EarthDamage,
            7 => Self::FireDamage,
            8 => Self::AirDamage,
            _ => Self::WaterDamage,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MaxHp => "Max HP",
            Self::Regeneration => "Regen",
            Self::AttackSpeed => "Atk Speed",
            Self::PassiveIncome => "Income",
            Self::CriticalChance => "Crit",
            Self::ExplosionSize => "Splash",
            Self::EarthDamage => "Earth",
            Self::FireDamage => "Fire",
            Self::AirDamage => "Air",
            Self::WaterDamage => "Water",
        }
    }

    pub fn effect_text(self) -> &'static str {
        match self {
            Self::MaxHp => "+5 max HP",
            Self::Regeneration => "+1 HP at wave start",
            Self::AttackSpeed => "+12% tower attack speed",
            Self::PassiveIncome => "+$1 per kill",
            Self::CriticalChance => "+4% critical chance",
            Self::ExplosionSize => "+12 splash size",
            Self::EarthDamage => "+4 earth damage",
            Self::FireDamage => "+4 fire damage",
            Self::AirDamage => "+4 air damage",
            Self::WaterDamage => "+4 water damage",
        }
    }

    pub fn cost(self) -> i32 {
        match self {
            Self::MaxHp => 35,
            Self::Regeneration => 30,
            Self::AttackSpeed => 45,
            Self::PassiveIncome => 40,
            Self::CriticalChance => 45,
            Self::ExplosionSize => 35,
            Self::EarthDamage | Self::FireDamage | Self::AirDamage | Self::WaterDamage => 35,
        }
    }

    pub fn icon_color(self) -> Color {
        match self {
            Self::MaxHp => Color::srgb(0.74, 0.18, 0.18),
            Self::Regeneration => Color::srgb(0.22, 0.62, 0.30),
            Self::AttackSpeed => Color::srgb(0.86, 0.72, 0.24),
            Self::PassiveIncome => Color::srgb(0.95, 0.78, 0.24),
            Self::CriticalChance => Color::srgb(0.70, 0.22, 0.22),
            Self::ExplosionSize => Color::srgb(0.82, 0.44, 0.18),
            Self::EarthDamage => Color::srgb(0.46, 0.34, 0.22),
            Self::FireDamage => Color::srgb(0.86, 0.24, 0.12),
            Self::AirDamage => Color::srgb(0.58, 0.72, 0.92),
            Self::WaterDamage => Color::srgb(0.18, 0.42, 0.78),
        }
    }
}

fn scale_price(base_price: i32, wave: u32) -> i32 {
    let wave_growth = (1.0 + PRICE_GROWTH).powi(wave.saturating_sub(1) as i32);
    (base_price as f32 * wave_growth).round() as i32
}

#[derive(Clone, Copy)]
pub enum ShopItem {
    Tower(TowerKind),
    StatUpgrade(StatUpgradeKind),
}

impl ShopItem {
    pub fn random() -> Self {
        if rand::random::<f32>() < 0.55 {
            Self::Tower(TowerKind::random())
        } else {
            Self::StatUpgrade(StatUpgradeKind::random())
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Tower(kind) => kind.name(),
            Self::StatUpgrade(kind) => kind.name(),
        }
    }

    pub fn tower_kind(self) -> Option<TowerKind> {
        match self {
            Self::Tower(kind) => Some(kind),
            Self::StatUpgrade(_) => None,
        }
    }

    pub fn stat_upgrade_kind(self) -> Option<StatUpgradeKind> {
        match self {
            Self::Tower(_) => None,
            Self::StatUpgrade(kind) => Some(kind),
        }
    }

    pub fn cost(self, wave: u32) -> i32 {
        match self {
            Self::Tower(_) => scale_price(TOWER_COST, wave),
            Self::StatUpgrade(kind) => scale_price(kind.cost(), wave),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShopOffer {
    pub item: ShopItem,
    pub cost: i32,
}

impl ShopOffer {
    pub fn random(wave: u32) -> Self {
        let item = ShopItem::random();
        Self {
            item,
            cost: item.cost(wave),
        }
    }
}

#[derive(Resource)]
pub struct Shop {
    pub offers: [Option<ShopOffer>; 3],
    pub selected: usize,
    pub reroll_cost: i32,
}

impl Shop {
    pub fn new(wave: u32) -> Self {
        Self {
            offers: [
                Some(ShopOffer::random(wave)),
                Some(ShopOffer::random(wave)),
                Some(ShopOffer::random(wave)),
            ],
            selected: 0,
            reroll_cost: scale_price(SHOP_REROLL_COST, wave),
        }
    }

    pub fn reroll(&mut self, wave: u32) {
        self.offers = [
            Some(ShopOffer::random(wave)),
            Some(ShopOffer::random(wave)),
            Some(ShopOffer::random(wave)),
        ];
        self.reroll_cost = scale_price(SHOP_REROLL_COST, wave);
        self.selected = self.selected.min(self.offers.len() - 1);
    }

    pub fn update_prices_for_wave(&mut self, wave: u32) {
        self.reroll_cost = scale_price(SHOP_REROLL_COST, wave);
    }

    pub fn selected_offer(&self) -> Option<ShopOffer> {
        self.offers[self.selected]
    }

    pub fn take_selected_offer(&mut self) -> Option<ShopOffer> {
        let offer = self.offers[self.selected];
        self.offers[self.selected] = None;
        offer
    }
}
