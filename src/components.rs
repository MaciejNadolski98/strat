use bevy::prelude::*;
use std::fmt;

use crate::{
    constants::MAX_HEALTH_GROWTH,
    resources::{AirDamage, EarthDamage, FireDamage, TowerStatEffect, WaterDamage},
    tower_definitions::*,
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
pub struct ShopSlotBarrel {
    pub index: usize,
}

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

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum TowerKind {
    Ballista,
    Cannon,
    Sprayer,
    Sniper,
}

impl TowerKind {
    pub fn definition(self) -> &'static TowerDefinition {
        match self {
            Self::Ballista => &TOWER_BALLISTA,
            Self::Cannon => &TOWER_CANNON,
            Self::Sprayer => &TOWER_SPRAYER,
            Self::Sniper => &TOWER_SNIPER,
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

    pub fn explosion_radius(self) -> f32 {
        self.definition().explosion_radius
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

    pub fn cost(self) -> u32 {
        self.definition().cost
    }

    pub fn stat_effects(self) -> &'static [TowerStatEffect] {
        self.definition().stat_effects
    }
}

#[derive(Component, Clone, Copy)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Brute,
    Armored,
}

impl EnemyKind {
    pub fn max_health(self, wave: u32) -> f32 {
        let base_health = match self {
            Self::Grunt => 71.0,
            Self::Runner => 48.0,
            Self::Brute => 130.0,
            Self::Armored => 102.0,
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
        }
    }

    pub fn reward(self) -> i32 {
        match self {
            Self::Runner => 1,
            Self::Grunt => 2,
            Self::Armored => 4,
            Self::Brute => 5,
        }
    }

    pub fn size(self) -> Vec2 {
        match self {
            Self::Grunt => Vec2::new(26.0, 26.0),
            Self::Runner => Vec2::new(20.0, 20.0),
            Self::Brute => Vec2::new(34.0, 34.0),
            Self::Armored => Vec2::new(28.0, 28.0),
        }
    }

    pub fn colors(self) -> ((f32, f32, f32), (f32, f32, f32)) {
        match self {
            Self::Grunt => ((0.95, 0.18, 0.16), (0.70, 0.76, 0.16)),
            Self::Runner => ((0.98, 0.45, 0.12), (0.94, 0.82, 0.24)),
            Self::Brute => ((0.45, 0.12, 0.11), (0.72, 0.22, 0.18)),
            Self::Armored => ((0.25, 0.28, 0.35), (0.42, 0.58, 0.72)),
        }
    }
}

#[derive(Component)]
pub struct FireCooldown {
    pub timer: Timer,
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
