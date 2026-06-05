use bevy::prelude::*;

use crate::components::{ALL_TOWER_KINDS, TowerKind};
use crate::constants::{
    GRID_SIZE, INITIAL_PATH, PATH_EXTENSION_BASE_COST, PATH_EXTENSION_COST_STEP, PRICE_GROWTH,
    SHOP_REROLL_COST,
};

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

#[derive(Resource)]
pub struct PathTiles {
    pub tiles: Vec<Vec2>,
}

impl PathTiles {
    pub fn new() -> Self {
        Self {
            tiles: INITIAL_PATH.to_vec(),
        }
    }

    pub fn reset(&mut self) {
        self.tiles = INITIAL_PATH.to_vec();
    }

    pub fn start(&self) -> Vec2 {
        self.tiles[0]
    }

    pub fn end(&self) -> Vec2 {
        self.tiles[self.tiles.len() - 1]
    }

    pub fn extension_cost(&self) -> i32 {
        let extensions = self.tiles.len().saturating_sub(INITIAL_PATH.len()) as i32;
        PATH_EXTENSION_BASE_COST + extensions * PATH_EXTENSION_COST_STEP
    }

    pub fn contains(&self, position: Vec2) -> bool {
        self.tiles
            .iter()
            .any(|tile| tile.distance_squared(position) < 1.0)
    }

    pub fn can_extend_to(&self, position: Vec2) -> bool {
        let distance_squared = position.distance_squared(self.end());
        !self.contains(position) && (distance_squared - GRID_SIZE.powi(2)).abs() < 1.0
    }

    pub fn extend_to(&mut self, position: Vec2) {
        self.tiles.push(position);
    }
}

#[derive(Clone, Copy)]
pub enum SpellKind {
    Ignite,
    ElementalSurge,
    Slow,
}

impl SpellKind {
    pub fn random() -> Self {
        match rand::random::<u8>() % 3 {
            0 => Self::Ignite,
            1 => Self::ElementalSurge,
            _ => Self::Slow,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Ignite => "Ignite",
            Self::ElementalSurge => "Surge",
            Self::Slow => "Slow",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Ignite => "Sets all enemies on fire",
            Self::ElementalSurge => "Doubles elemental damage until wave end",
            Self::Slow => "Slows all enemies until wave end",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Self::Ignite => 7,
            Self::ElementalSurge => 15,
            Self::Slow => 10,
        }
    }

    pub fn icon_color(self) -> Color {
        match self {
            Self::Ignite => Color::srgb(0.92, 0.26, 0.12),
            Self::ElementalSurge => Color::srgb(0.30, 0.62, 0.92),
            Self::Slow => Color::srgb(0.42, 0.82, 0.92),
        }
    }
}

#[derive(Resource)]
pub struct SpellShop {
    pub slots: [Option<SpellKind>; 3],
}

impl SpellShop {
    pub fn new() -> Self {
        Self {
            slots: [None, None, None],
        }
    }

    pub fn store_spell(&mut self, spell: SpellKind) -> bool {
        let Some(slot) = self.slots.iter_mut().find(|slot| slot.is_none()) else {
            return false;
        };
        *slot = Some(spell);
        true
    }

    pub fn take_spell(&mut self, index: usize) -> Option<SpellKind> {
        let spell = self.slots.get_mut(index)?;
        spell.take()
    }
}

#[derive(Resource)]
pub struct ActiveSpellEffects {
    pub elemental_multiplier: f32,
    pub enemy_speed_multiplier: f32,
}

impl ActiveSpellEffects {
    pub fn new() -> Self {
        Self {
            elemental_multiplier: 1.0,
            enemy_speed_multiplier: 1.0,
        }
    }

    pub fn reset_for_wave(&mut self) {
        self.elemental_multiplier = 1.0;
        self.enemy_speed_multiplier = 1.0;
    }
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

    pub fn cost(self) -> u32 {
        match self {
            Self::Regeneration => 2,
            Self::EarthDamage | Self::FireDamage | Self::AirDamage | Self::WaterDamage => 3,
            Self::ExplosionSize => 4,
            Self::MaxHp | Self::AttackSpeed | Self::CriticalChance => 5,
            Self::PassiveIncome => 10,
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

fn scale_price(base_price: u32, wave: u32) -> i32 {
    let wave_growth = (1.0 + PRICE_GROWTH).powi(wave.saturating_sub(1) as i32);
    (base_price as f32 * wave_growth).round() as i32
}

#[derive(Clone, Copy)]
pub enum ShopItem {
    Tower(TowerKind),
    StatUpgrade(StatUpgradeKind),
    Spell(SpellKind),
}

impl ShopItem {
    pub fn random_without_towers(unavailable_towers: &[TowerKind]) -> Self {
        let roll = rand::random::<f32>();
        if roll < 0.45 {
            let available_towers: Vec<TowerKind> = ALL_TOWER_KINDS
                .into_iter()
                .filter(|kind| !unavailable_towers.contains(kind))
                .collect();

            if !available_towers.is_empty() {
                let kind = available_towers[rand::random::<usize>() % available_towers.len()];
                return Self::Tower(kind);
            }
        }

        if roll < 0.80 {
            Self::StatUpgrade(StatUpgradeKind::random())
        } else {
            Self::Spell(SpellKind::random())
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Tower(kind) => kind.name(),
            Self::StatUpgrade(kind) => kind.name(),
            Self::Spell(kind) => kind.name(),
        }
    }

    pub fn tower_kind(self) -> Option<TowerKind> {
        match self {
            Self::Tower(kind) => Some(kind),
            Self::StatUpgrade(_) | Self::Spell(_) => None,
        }
    }

    pub fn stat_upgrade_kind(self) -> Option<StatUpgradeKind> {
        match self {
            Self::Tower(_) | Self::Spell(_) => None,
            Self::StatUpgrade(kind) => Some(kind),
        }
    }

    pub fn spell_kind(self) -> Option<SpellKind> {
        match self {
            Self::Tower(_) | Self::StatUpgrade(_) => None,
            Self::Spell(kind) => Some(kind),
        }
    }

    pub fn cost(self, wave: u32) -> i32 {
        match self {
            Self::Tower(kind) => scale_price(kind.cost(), wave),
            Self::StatUpgrade(kind) => scale_price(kind.cost(), wave),
            Self::Spell(kind) => scale_price(kind.cost(), wave),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ShopOffer {
    pub item: ShopItem,
    pub cost: i32,
}

impl ShopOffer {
    pub fn random_without_towers(wave: u32, unavailable_towers: &[TowerKind]) -> Self {
        let item = ShopItem::random_without_towers(unavailable_towers);
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
    purchased_towers: Vec<TowerKind>,
}

impl Shop {
    pub fn new(wave: u32) -> Self {
        Self {
            offers: Self::generate_offers(wave, &[]),
            selected: 0,
            reroll_cost: scale_price(SHOP_REROLL_COST, wave),
            purchased_towers: Vec::new(),
        }
    }

    pub fn reroll(&mut self, wave: u32) {
        self.offers = Self::generate_offers(wave, &self.purchased_towers);
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

        if let Some(tower_kind) = offer.and_then(|offer| offer.item.tower_kind()) {
            if !self.purchased_towers.contains(&tower_kind) {
                self.purchased_towers.push(tower_kind);
            }

            for slot in &mut self.offers {
                if slot
                    .and_then(|offer| offer.item.tower_kind())
                    .is_some_and(|kind| kind == tower_kind)
                {
                    *slot = None;
                }
            }
        }

        offer
    }

    fn generate_offers(wave: u32, purchased_towers: &[TowerKind]) -> [Option<ShopOffer>; 3] {
        let mut unavailable_towers = purchased_towers.to_vec();
        let mut offers = [None; 3];

        for offer in &mut offers {
            let generated_offer = ShopOffer::random_without_towers(wave, &unavailable_towers);
            if let Some(tower_kind) = generated_offer.item.tower_kind() {
                unavailable_towers.push(tower_kind);
            }
            *offer = Some(generated_offer);
        }

        offers
    }
}
