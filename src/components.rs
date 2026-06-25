use bevy::prelude::*;
use std::fmt;

use crate::{
    constants::MAX_HEALTH_GROWTH,
    resources::{AirDamage, EarthDamage, FireDamage, WaterDamage},
};

#[derive(Component)]
pub struct Tower;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Projectile;

#[derive(Component)]
pub struct PathTile;

#[derive(Component)]
pub struct PathEdge;

#[derive(Component)]
pub struct PathEndMarker;

#[derive(Component)]
pub struct PathExtensionHint;

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct ShopText;

#[derive(Component)]
pub struct ShopTooltip;

#[derive(Component)]
pub struct ShopSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopSlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct ShopSlotBarrel;

#[derive(Component)]
pub struct ShopSlotLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct SpellSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct SpellSlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct SpellSlotLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct FloatingText {
    pub lifetime: Timer,
    pub velocity: Vec3,
}

#[derive(Component)]
pub struct ExplosionEffect {
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct TowerRangeIndicator;

#[derive(Component)]
pub struct TowerKillCount {
    pub kills: u32,
}

#[derive(Component)]
pub struct TowerPhantom;

#[derive(Component)]
pub struct TowerPhantomBarrel {
    pub sub_index: usize,
}

#[derive(Component)]
pub struct DraftPanel;

#[derive(Component)]
pub struct DraftSlot {
    pub index: usize,
}

#[derive(Component)]
pub struct DraftSlotIcon {
    pub index: usize,
}

#[derive(Component)]
pub struct DraftSlotBarrel {
    pub index: usize,
    pub sub_index: usize,
}

#[derive(Component)]
pub struct DraftSlotLabel {
    pub index: usize,
}

#[derive(Component)]
pub struct DraftHeaderText;

#[derive(Component, Clone, Copy)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Brute,
    Armored,
    Titan,
}

impl EnemyKind {
    pub fn max_health(self, wave: u32) -> f32 {
        let base_health = match self {
            Self::Grunt => 71.0,
            Self::Runner => 48.0,
            Self::Brute => 130.0,
            Self::Armored => 102.0,
            Self::Titan => 450.0,
        };
        let wave_growth = (1.0 + MAX_HEALTH_GROWTH).powi(wave.saturating_sub(1) as i32);

        base_health * wave_growth
    }

    pub fn speed(self, wave: u32) -> f32 {
        match self {
            Self::Grunt => 58.0 + wave as f32 * 3.5,
            Self::Runner => 92.0 + wave as f32 * 5.0,
            Self::Brute => 38.0 + wave as f32 * 2.0,
            Self::Armored => 54.0 + wave as f32 * 2.5,
            Self::Titan => 20.0 + wave as f32 * 0.5,
        }
    }

    pub fn reward(self) -> i32 {
        match self {
            Self::Runner => 1,
            Self::Grunt => 2,
            Self::Armored => 4,
            Self::Brute => 5,
            Self::Titan => 15,
        }
    }

    pub fn size(self) -> Vec2 {
        match self {
            Self::Grunt => Vec2::new(26.0, 26.0),
            Self::Runner => Vec2::new(20.0, 20.0),
            Self::Brute => Vec2::new(34.0, 34.0),
            Self::Armored => Vec2::new(28.0, 28.0),
            Self::Titan => Vec2::new(50.0, 50.0),
        }
    }

    pub fn colors(self) -> ((f32, f32, f32), (f32, f32, f32)) {
        match self {
            Self::Grunt => ((0.95, 0.18, 0.16), (0.70, 0.76, 0.16)),
            Self::Runner => ((0.98, 0.45, 0.12), (0.94, 0.82, 0.24)),
            Self::Brute => ((0.45, 0.12, 0.11), (0.72, 0.22, 0.18)),
            Self::Armored => ((0.25, 0.28, 0.35), (0.42, 0.58, 0.72)),
            Self::Titan => ((0.22, 0.05, 0.30), (0.55, 0.15, 0.75)),
        }
    }
}

#[derive(Component)]
pub struct DropsSpell;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct AuraTower;

#[derive(Component)]
pub struct PulseEffect {
    pub lifetime: Timer,
    pub max_radius: f32,
}

#[derive(Component)]
pub struct FireCooldown {
    pub base_cooldown: f32,
    pub timer: Timer,
}

#[derive(Component, Default)]
pub struct TemporaryAttackSpeed {
    pub bonus: f32,
}

#[derive(Component)]
pub struct TemporaryEnemySpeed {
    pub multiplier: f32,
}

impl Default for TemporaryEnemySpeed {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

#[derive(Component)]
pub struct AngularSpeed {
    pub value: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Burning {
    pub timer: Timer,
    pub tick_timer: Timer,
    pub damage_per_tick: f32,
}

#[derive(Component)]
pub struct HealthBar {
    pub owner: Entity,
    pub width: f32,
    pub fill: bool,
}

#[derive(Component)]
pub struct Speed {
    pub value: f32,
}

#[derive(Component)]
pub struct Reward {
    pub amount: i32,
}

#[derive(Component)]
pub struct Waypoint {
    pub index: usize,
}

#[derive(Component)]
pub struct PathProgress {
    pub distance: f32,
}

#[derive(Component)]
pub struct Target {
    pub entity: Entity,
}

#[derive(Component)]
pub struct SourceTower {
    pub entity: Entity,
}

#[derive(Component)]
pub struct Damage {
    pub amount: f32,
}

#[derive(Component)]
pub struct DamageDealt {
    pub amount: f32,
}

#[derive(Component)]
pub struct IsCritical {
    pub value: bool,
}

#[derive(Component)]
pub struct ExplosionRadius {
    pub value: f32,
}

#[derive(Component, Clone, Copy)]
pub struct DamageFormula {
    pub flat: u32,
    pub crit_multiplier: f32,
    pub earth_multiplier: f32,
    pub fire_multiplier: f32,
    pub air_multiplier: f32,
    pub water_multiplier: f32,
}

impl DamageFormula {
    pub fn calculate_damage_with_elemental_multiplier(
        &self,
        earth_damage: &EarthDamage,
        fire_damage: &FireDamage,
        air_damage: &AirDamage,
        water_damage: &WaterDamage,
        crit: bool,
        elemental_multiplier: f32,
    ) -> u32 {
        let mut dmg = self.flat as f32;
        dmg += self.earth_multiplier * earth_damage.value * elemental_multiplier;
        dmg += self.air_multiplier * air_damage.value * elemental_multiplier;
        dmg += self.fire_multiplier * fire_damage.value * elemental_multiplier;
        dmg += self.water_multiplier * water_damage.value * elemental_multiplier;
        if crit {
            (dmg * self.crit_multiplier) as u32
        } else {
            dmg as u32
        }
    }
}

impl fmt::Display for DamageFormula {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.flat)?;
        if self.earth_multiplier != 0.0 {
            write!(formatter, " + {} earth", self.earth_multiplier)?;
        }
        if self.air_multiplier != 0.0 {
            write!(formatter, " + {} air", self.air_multiplier)?;
        }
        if self.fire_multiplier != 0.0 {
            write!(formatter, " + {} fire", self.fire_multiplier)?;
        }
        if self.water_multiplier != 0.0 {
            write!(formatter, " + {} water", self.water_multiplier)?;
        }
        Ok(())
    }
}
