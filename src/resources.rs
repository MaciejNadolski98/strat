use bevy::prelude::*;

use crate::constants::{
    GRID_SIZE, INITIAL_PATH, PATH_EXTENSION_BASE_COST, PATH_EXTENSION_COST_STEP, PRICE_GROWTH,
    SHOP_REROLL_COST,
};
use crate::item_definitions::ItemKind;
pub use crate::spell_definitions::SpellKind;
use crate::tower_definitions::TowerKind;

#[derive(Resource)]
pub struct Money {
    pub amount: i32,
}

#[derive(Resource)]
pub struct CurrentHp {
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

pub struct Stat {
    pub raw_value: f32,
    pub permanent_multiplier: f32,
    pub temporary_boost: f32,
    pub temporary_multiplier: f32,
}

impl Stat {
    pub fn new(raw_value: f32) -> Self {
        Self {
            raw_value,
            permanent_multiplier: 1.0,
            temporary_boost: 0.0,
            temporary_multiplier: 0.0,
        }
    }

    pub fn value(&self) -> f32 {
        (self.raw_value + self.temporary_boost) * (self.permanent_multiplier + self.temporary_multiplier)
    }

    pub fn reset_temporary(&mut self) {
        self.temporary_boost = 0.0;
        self.temporary_multiplier = 0.0;
    }
}

macro_rules! stat_resource {
    ($name:ident) => {
        #[derive(Resource)]
        pub struct $name(pub Stat);

        impl std::ops::Deref for $name {
            type Target = Stat;
            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
        }
    };
}

stat_resource!(Regeneration);
stat_resource!(AttackSpeed);
stat_resource!(Loot);
stat_resource!(CriticalChance);
stat_resource!(ExplosionSize);
stat_resource!(EarthDamage);
stat_resource!(FireDamage);
stat_resource!(AirDamage);
stat_resource!(WaterDamage);
stat_resource!(MaxHp);
stat_resource!(Piercing);
stat_resource!(PiercingDamage);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GamePhase {
    ResetTemporaries,
    TemporaryStatEffects,
    TemporaryTowerEffects,
    Gameplay,
}

pub fn reset_stat_temporaries(
    mut earth: ResMut<EarthDamage>,
    mut fire: ResMut<FireDamage>,
    mut air: ResMut<AirDamage>,
    mut water: ResMut<WaterDamage>,
    mut attack_speed: ResMut<AttackSpeed>,
    mut loot: ResMut<Loot>,
    mut critical_chance: ResMut<CriticalChance>,
    mut explosion_size: ResMut<ExplosionSize>,
    mut regeneration: ResMut<Regeneration>,
    mut max_hp: ResMut<MaxHp>,
    mut piercing: ResMut<Piercing>,
    mut piercing_damage: ResMut<PiercingDamage>,
) {
    earth.reset_temporary();
    fire.reset_temporary();
    air.reset_temporary();
    water.reset_temporary();
    attack_speed.reset_temporary();
    loot.reset_temporary();
    critical_chance.reset_temporary();
    explosion_size.reset_temporary();
    regeneration.reset_temporary();
    max_hp.reset_temporary();
    piercing.reset_temporary();
    piercing_damage.reset_temporary();
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

#[derive(Resource)]
pub struct SpellShop {
    pub slots: [Option<SpellKind>; 3],
    pub(crate) known_kinds: Vec<SpellKind>,
}

impl SpellShop {
    pub fn new_empty() -> Self {
        Self {
            slots: [None; 3],
            known_kinds: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.slots = [None; 3];
    }

    pub fn store_spell(&mut self, spell: SpellKind) -> bool {
        let Some(slot) = self.slots.iter_mut().find(|slot| slot.is_none()) else {
            return false;
        };
        *slot = Some(spell);
        true
    }

    pub fn store_random_spell(&mut self) -> bool {
        if self.known_kinds.is_empty() {
            return false;
        }
        let kind = self.known_kinds[rand::random::<usize>() % self.known_kinds.len()];
        self.store_spell(kind)
    }

    pub fn take_spell(&mut self, index: usize) -> Option<SpellKind> {
        self.slots.get_mut(index)?.take()
    }
}


#[derive(Clone, Copy)]
pub enum PlayerStatKind {
    MaxHp,
    Regeneration,
    AttackSpeed,
    Loot,
    CriticalChance,
    ExplosionSize,
    EarthDamage,
    FireDamage,
    AirDamage,
    WaterDamage,
    Piercing,
    PiercingDamage,
}

impl PlayerStatKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::MaxHp => "Max HP",
            Self::Regeneration => "Regen",
            Self::AttackSpeed => "Atk Speed",
            Self::Loot => "loot",
            Self::CriticalChance => "Crit",
            Self::ExplosionSize => "Splash",
            Self::EarthDamage => "Earth",
            Self::FireDamage => "Fire",
            Self::AirDamage => "Air",
            Self::WaterDamage => "Water",
            Self::Piercing => "Piercing",
            Self::PiercingDamage => "Piercing Damage",
        }
    }

    fn is_percent(self) -> bool {
        matches!(self, Self::AttackSpeed | Self::CriticalChance | Self::PiercingDamage)
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


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TowerDraftPhase {
    WaveRunning,
    Picking,
    Placing(TowerKind),
}

#[derive(Resource)]
pub struct TowerDraft {
    pub offers: Vec<TowerKind>,
    pub phase: TowerDraftPhase,
    pub(crate) known_kinds: Vec<TowerKind>,
}

impl TowerDraft {
    pub fn new_empty() -> Self {
        Self {
            offers: Vec::new(),
            phase: TowerDraftPhase::Picking,
            known_kinds: Vec::new(),
        }
    }

    pub fn activate(&mut self, forced: &mut ForcedTowerOffers) {
        self.offers = Self::generate_offers(&self.known_kinds, forced);
        self.phase = TowerDraftPhase::Picking;
    }

    fn generate_offers(kinds: &[TowerKind], forced: &mut ForcedTowerOffers) -> Vec<TowerKind> {
        let mut kinds = kinds.to_vec();
        let n = kinds.len();

        let mut start = 0;
        if n > 0 && !forced.queue.is_empty() {
            let forced_kind = forced.queue.remove(0);
            if let Some(idx) = kinds.iter().position(|k| *k == forced_kind) {
                kinds.swap(0, idx);
                start = 1;
            }
        }

        for i in start..3.min(n) {
            let j = i + rand::random::<usize>() % (n - i);
            kinds.swap(i, j);
        }
        kinds[..3].to_vec()
    }
}

/// Queue of towers (from `--towers=a,b,c`) to force into slot 0 of the next
/// rounds' draft offers, one consumed per round, oldest first. Names are
/// resolved against the tower registry at startup so a typo fails fast
/// instead of silently never triggering.
#[derive(Resource, Default)]
pub struct ForcedTowerOffers {
    pub queue: Vec<TowerKind>,
    original: Vec<TowerKind>,
}

impl ForcedTowerOffers {
    pub fn from_args(args: impl IntoIterator<Item = String>, known_kinds: &[TowerKind]) -> Self {
        for arg in args {
            if let Some(list) = arg.strip_prefix("--towers=") {
                let queue: Vec<TowerKind> = list
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(|name| {
                        known_kinds
                            .iter()
                            .find(|kind| kind.name().eq_ignore_ascii_case(name))
                            .copied()
                            .unwrap_or_else(|| panic!("Unknown tower name in --towers: {name}"))
                    })
                    .collect();
                return Self { queue: queue.clone(), original: queue };
            }
        }
        Self::default()
    }

    pub fn reset(&mut self) {
        self.queue = self.original.clone();
    }
}

fn scale_price(base_price: u32, wave: u32) -> i32 {
    let wave_growth = (1.0 + PRICE_GROWTH).powi(wave.saturating_sub(1) as i32);
    (base_price as f32 * wave_growth).round() as i32
}

#[derive(Clone, Copy)]
pub struct ShopOffer {
    pub item: ItemKind,
    pub cost: i32,
}

#[derive(Event)]
pub struct ItemPurchasedEvent {
    pub kind: ItemKind,
}

#[derive(Event)]
pub struct EnemyKilledEvent {
    pub source_tower: Entity,
}

#[derive(Event)]
pub struct ShootEvent {
    pub source_tower: Entity,
}

/// Fired when a `Charge` reaches a tower with `ChargeConsumer`, so that
/// tower's own systems can react with whatever effect it provides.
#[derive(Event)]
pub struct ChargeConsumedEvent {
    pub tower: Entity,
}

#[derive(Event)]
pub struct NewRoundEvent;

#[derive(Event)]
pub struct GameRestartEvent;

#[derive(Resource)]
pub struct Shop {
    pub offers: [Option<ShopOffer>; 3],
    pub reroll_cost: i32,
    pub(crate) known_kinds: Vec<ItemKind>,
}

impl Shop {
    pub fn new_empty() -> Self {
        Self {
            offers: [None; 3],
            reroll_cost: 0,
            known_kinds: Vec::new(),
        }
    }

    pub fn activate(&mut self, wave: u32) {
        self.offers = Self::generate_offers(&self.known_kinds, wave);
        self.reroll_cost = scale_price(SHOP_REROLL_COST, wave);
    }

    pub fn reroll(&mut self, wave: u32) {
        self.offers = Self::generate_offers(&self.known_kinds, wave);
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

    fn generate_offers(kinds: &[ItemKind], wave: u32) -> [Option<ShopOffer>; 3] {
        let mut offers = [None; 3];
        for offer in &mut offers {
            if kinds.is_empty() {
                break;
            }
            let kind = kinds[rand::random::<usize>() % kinds.len()];
            *offer = Some(ShopOffer {
                item: kind,
                cost: scale_price(kind.cost(), wave),
            });
        }
        offers
    }
}
