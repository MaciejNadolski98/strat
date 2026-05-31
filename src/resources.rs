use bevy::prelude::*;

use crate::components::TowerKind;
use crate::constants::{SHOP_REROLL_COST, TOWER_COST};

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
    pub timer: Timer,
}

#[derive(Resource)]
pub struct NextWaveTimer {
    pub timer: Timer,
}

#[derive(Clone, Copy)]
pub enum ShopItem {
    Tower(TowerKind),
}

impl ShopItem {
    pub fn random() -> Self {
        Self::Tower(TowerKind::random())
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Tower(kind) => kind.name(),
        }
    }

    pub fn tower_kind(self) -> Option<TowerKind> {
        match self {
            Self::Tower(kind) => Some(kind),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShopOffer {
    pub item: ShopItem,
    pub cost: i32,
}

impl ShopOffer {
    pub fn random() -> Self {
        Self {
            item: ShopItem::random(),
            cost: TOWER_COST,
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
    pub fn new() -> Self {
        Self {
            offers: [
                Some(ShopOffer::random()),
                Some(ShopOffer::random()),
                Some(ShopOffer::random()),
            ],
            selected: 0,
            reroll_cost: SHOP_REROLL_COST,
        }
    }

    pub fn reroll(&mut self) {
        self.offers = [
            Some(ShopOffer::random()),
            Some(ShopOffer::random()),
            Some(ShopOffer::random()),
        ];
        self.selected = self.selected.min(self.offers.len() - 1);
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
