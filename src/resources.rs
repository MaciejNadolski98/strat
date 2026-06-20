use bevy::prelude::*;

use crate::constants::{
    GRID_SIZE, INITIAL_PATH, PATH_EXTENSION_BASE_COST, PATH_EXTENSION_COST_STEP, PRICE_GROWTH,
    SHOP_REROLL_COST,
};
use crate::item_definitions::*;
use crate::tower_definitions::{ALL_TOWER_KINDS, TowerKind};

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

#[derive(Clone, Copy)]
pub struct SpellDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: u32,
    pub icon_color: Color,
}

const IGNITE_SPELL_DEFINITION: SpellDefinition = SpellDefinition {
    name: "Ignite",
    description: "Sets all enemies on fire, scaling with fire damage",
    cost: 7,
    icon_color: Color::srgb(0.92, 0.26, 0.12),
};

const ELEMENTAL_SURGE_SPELL_DEFINITION: SpellDefinition = SpellDefinition {
    name: "Surge",
    description: "Doubles elemental damage until wave end",
    cost: 15,
    icon_color: Color::srgb(0.30, 0.62, 0.92),
};

const SLOW_SPELL_DEFINITION: SpellDefinition = SpellDefinition {
    name: "Slow",
    description: "Slows all enemies until wave end",
    cost: 10,
    icon_color: Color::srgb(0.42, 0.82, 0.92),
};

impl SpellKind {
    pub fn random() -> Self {
        match rand::random::<u8>() % 3 {
            0 => Self::Ignite,
            1 => Self::ElementalSurge,
            _ => Self::Slow,
        }
    }

    pub fn definition(self) -> &'static SpellDefinition {
        match self {
            Self::Ignite => &IGNITE_SPELL_DEFINITION,
            Self::ElementalSurge => &ELEMENTAL_SURGE_SPELL_DEFINITION,
            Self::Slow => &SLOW_SPELL_DEFINITION,
        }
    }

    pub fn name(self) -> &'static str {
        self.definition().name
    }

    pub fn description(self) -> &'static str {
        self.definition().description
    }

    pub fn cost(self) -> u32 {
        self.definition().cost
    }

    pub fn icon_color(self) -> Color {
        self.definition().icon_color
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
pub enum PlayerStatKind {
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

impl PlayerStatKind {
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

    fn is_percent(self) -> bool {
        matches!(self, Self::AttackSpeed | Self::CriticalChance)
    }
}

#[derive(Clone, Copy)]
pub struct TowerStatEffect {
    pub kind: PlayerStatKind,
    pub amount: f32,
}

impl TowerStatEffect {
    pub const fn new(kind: PlayerStatKind, amount: f32) -> Self {
        Self { kind, amount }
    }

    pub fn effect_text(self) -> String {
        let amount = if self.kind.is_percent() {
            self.amount.abs() * 100.0
        } else {
            self.amount.abs()
        };
        let sign = if self.amount >= 0.0 { "+" } else { "-" };
        let amount_text = if (amount - amount.round()).abs() < 0.0001 {
            format!("{}", amount.round() as i32)
        } else {
            format!("{amount:.1}")
        };
        let suffix = if self.kind.is_percent() { "%" } else { "" };

        format!("{} {}{}{}", self.kind.name(), sign, amount_text, suffix)
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
    Vitality,
    Offense,
    ElementalFocus,
    Siege,
}

impl StatUpgradeKind {
    pub fn random() -> Self {
        match rand::random::<u8>() % 14 {
            0 => Self::MaxHp,
            1 => Self::Regeneration,
            2 => Self::AttackSpeed,
            3 => Self::PassiveIncome,
            4 => Self::CriticalChance,
            5 => Self::ExplosionSize,
            6 => Self::EarthDamage,
            7 => Self::FireDamage,
            8 => Self::AirDamage,
            9 => Self::WaterDamage,
            10 => Self::Vitality,
            11 => Self::Offense,
            12 => Self::ElementalFocus,
            _ => Self::Siege,
        }
    }

    pub fn definition(self) -> &'static StatUpgradeDefinition {
        match self {
            Self::MaxHp => &ITEM_POTATO,
            Self::Regeneration => &ITEM_MEDS,
            Self::AttackSpeed => &ITEM_COFFEE,
            Self::PassiveIncome => &ITEM_PASSIVE_INCOME,
            Self::CriticalChance => &ITEM_CRITICAL_CHANCE,
            Self::ExplosionSize => &ITEM_EXPLOSION_SIZE,
            Self::EarthDamage => &ITEM_EARTH_DAMAGE,
            Self::FireDamage => &ITEM_FIRE_DAMAGE,
            Self::AirDamage => &ITEM_AIR_DAMAGE,
            Self::WaterDamage => &ITEM_WATER_DAMAGE,
            Self::Vitality => &ITEM_VITALITY,
            Self::Offense => &ITEM_OFFENSE,
            Self::ElementalFocus => &ITEM_ELEMENTAL_FOCUS,
            Self::Siege => &ITEM_SIEGE,
        }
    }

    pub fn name(self) -> &'static str {
        self.definition().name
    }

    pub fn effects(self) -> &'static [TowerStatEffect] {
        self.definition().effects
    }

    pub fn effect_text(self) -> String {
        self.effects()
            .iter()
            .map(|effect| effect.effect_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn cost(self) -> u32 {
        self.definition().cost
    }

    pub fn icon_color(self) -> Color {
        self.definition().icon_color
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TowerDraftPhase {
    WaveRunning,
    Picking,
    Placing(TowerKind),
}

#[derive(Resource)]
pub struct TowerDraft {
    pub offers: [TowerKind; 3],
    pub phase: TowerDraftPhase,
}

impl TowerDraft {
    pub fn new() -> Self {
        Self {
            offers: Self::generate_offers(),
            phase: TowerDraftPhase::Picking,
        }
    }

    pub fn activate(&mut self) {
        self.offers = Self::generate_offers();
        self.phase = TowerDraftPhase::Picking;
    }

    fn generate_offers() -> [TowerKind; 3] {
        let mut kinds = ALL_TOWER_KINDS.to_vec();
        let n = kinds.len();
        for i in 0..3.min(n) {
            let j = i + rand::random::<usize>() % (n - i);
            kinds.swap(i, j);
        }
        [kinds[0], kinds[1], kinds[2]]
    }
}

fn scale_price(base_price: u32, wave: u32) -> i32 {
    let wave_growth = (1.0 + PRICE_GROWTH).powi(wave.saturating_sub(1) as i32);
    (base_price as f32 * wave_growth).round() as i32
}

#[derive(Clone, Copy)]
pub enum ShopItem {
    StatUpgrade(StatUpgradeKind),
    Spell(SpellKind),
}

impl ShopItem {
    pub fn random() -> Self {
        if rand::random::<f32>() < 0.70 {
            Self::StatUpgrade(StatUpgradeKind::random())
        } else {
            Self::Spell(SpellKind::random())
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::StatUpgrade(kind) => kind.name(),
            Self::Spell(kind) => kind.name(),
        }
    }

    pub fn stat_upgrade_kind(self) -> Option<StatUpgradeKind> {
        match self {
            Self::Spell(_) => None,
            Self::StatUpgrade(kind) => Some(kind),
        }
    }

    pub fn spell_kind(self) -> Option<SpellKind> {
        match self {
            Self::StatUpgrade(_) => None,
            Self::Spell(kind) => Some(kind),
        }
    }

    pub fn cost(self, wave: u32) -> i32 {
        match self {
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
    pub fn random(wave: u32) -> Self {
        let item = ShopItem::random();
        Self {
            item,
            cost: item.cost(wave),
        }
    }
}

#[derive(Event)]
pub struct EnemyKilledEvent {
    pub source_tower: Entity,
}

#[derive(Event)]
pub struct ShootEvent {
    pub source_tower: Entity,
}

#[derive(Event)]
pub struct NewRoundEvent;

#[derive(Resource)]
pub struct Shop {
    pub offers: [Option<ShopOffer>; 3],
    pub reroll_cost: i32,
}

impl Shop {
    pub fn new(wave: u32) -> Self {
        Self {
            offers: Self::generate_offers(wave),
            reroll_cost: scale_price(SHOP_REROLL_COST, wave),
        }
    }

    pub fn reroll(&mut self, wave: u32) {
        self.offers = Self::generate_offers(wave);
        self.reroll_cost = scale_price(SHOP_REROLL_COST, wave);
    }

    pub fn update_prices_for_wave(&mut self, wave: u32) {
        self.reroll_cost = scale_price(SHOP_REROLL_COST, wave);
    }

    pub fn take_offer(&mut self, selected: usize) -> Option<ShopOffer> {
        let offer = self.offers[selected];
        self.offers[selected] = None;
        offer
    }

    fn generate_offers(wave: u32) -> [Option<ShopOffer>; 3] {
        let mut offers = [None; 3];
        for offer in &mut offers {
            *offer = Some(ShopOffer::random(wave));
        }
        offers
    }
}
