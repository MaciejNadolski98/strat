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
pub struct FloatingText {
    pub lifetime: Timer,
    pub velocity: Vec3,
}

#[derive(Component, Clone, Copy)]
pub enum TowerKind {
    Ballista,
    Cannon,
    Sprayer,
    Sniper,
}

impl TowerKind {
    pub fn random() -> Self {
        match rand::random::<u8>() % 4 {
            0 => Self::Ballista,
            1 => Self::Cannon,
            2 => Self::Sprayer,
            _ => Self::Sniper,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Ballista => "Ballista",
            Self::Cannon => "Cannon",
            Self::Sprayer => "Sprayer",
            Self::Sniper => "Sniper",
        }
    }

    pub fn range(self) -> f32 {
        match self {
            Self::Ballista => 185.0,
            Self::Cannon => 150.0,
            Self::Sprayer => 125.0,
            Self::Sniper => 260.0,
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Self::Ballista => 0.73,
            Self::Cannon => 1.45,
            Self::Sprayer => 0.32,
            Self::Sniper => 1.75,
        }
    }

    pub fn damage_formula(self) -> DamageFormula {
        match self {
            Self::Ballista => DamageFormula {
                flat: 24,
                crit_multiplier: 2.0,
                earth_multiplier: 0.0,
                fire_multiplier: 0.0,
                air_multiplier: 0.75,
                water_multiplier: 0.75,
            },
            Self::Cannon => DamageFormula {
                flat: 34,
                crit_multiplier: 1.5,
                earth_multiplier: 1.0,
                fire_multiplier: 0.0,
                air_multiplier: 0.0,
                water_multiplier: 0.0,
            },
            Self::Sprayer => DamageFormula {
                flat: 11,
                crit_multiplier: 2.0,
                earth_multiplier: 0.0,
                fire_multiplier: 0.0,
                air_multiplier: 0.0,
                water_multiplier: 1.0,
            },
            Self::Sniper => DamageFormula {
                flat: 55,
                crit_multiplier: 5.0,
                earth_multiplier: 0.0,
                fire_multiplier: 0.0,
                air_multiplier: 0.0,
                water_multiplier: 1.0,
            },
        }
    }

    pub fn projectile_speed(self) -> f32 {
        match self {
            Self::Ballista => 430.0,
            Self::Cannon => 320.0,
            Self::Sprayer => 520.0,
            Self::Sniper => 720.0,
        }
    }

    pub fn explosion_radius(self) -> f32 {
        match self {
            Self::Cannon => 64.0,
            _ => 0.0,
        }
    }

    pub fn angular_speed(self) -> f32 {
        match self {
            Self::Ballista => 1.6,
            Self::Cannon => 1.0,
            Self::Sprayer => 4.2,
            Self::Sniper => 0.9,
        }
    }

    pub fn base_color(self) -> Color {
        match self {
            Self::Ballista => Color::srgb(0.22, 0.42, 0.74),
            Self::Cannon => Color::srgb(0.42, 0.36, 0.30),
            Self::Sprayer => Color::srgb(0.20, 0.52, 0.46),
            Self::Sniper => Color::srgb(0.34, 0.28, 0.56),
        }
    }

    pub fn barrel_color(self) -> Color {
        match self {
            Self::Ballista => Color::srgb(0.67, 0.83, 0.96),
            Self::Cannon => Color::srgb(0.74, 0.66, 0.54),
            Self::Sprayer => Color::srgb(0.62, 0.92, 0.78),
            Self::Sniper => Color::srgb(0.82, 0.76, 0.98),
        }
    }

    pub fn base_size(self) -> Vec2 {
        match self {
            Self::Cannon => Vec2::new(40.0, 40.0),
            Self::Sprayer => Vec2::new(32.0, 32.0),
            _ => Vec2::new(36.0, 36.0),
        }
    }

    pub fn barrel_size(self) -> Vec2 {
        match self {
            Self::Ballista => Vec2::new(12.0, 38.0),
            Self::Cannon => Vec2::new(18.0, 30.0),
            Self::Sprayer => Vec2::new(10.0, 28.0),
            Self::Sniper => Vec2::new(8.0, 48.0),
        }
    }

    pub fn barrel_offset(self) -> f32 {
        match self {
            Self::Sniper => 20.0,
            Self::Cannon => 13.0,
            Self::Sprayer => 12.0,
            Self::Ballista => 16.0,
        }
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

    pub fn reward(self, wave: u32) -> i32 {
        match self {
            Self::Grunt => 12 + wave as i32,
            Self::Runner => 10 + wave as i32,
            Self::Brute => 28 + wave as i32 * 2,
            Self::Armored => 22 + wave as i32 * 2,
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
pub struct Damage {
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
    pub fn calculate_damage(
        &self,
        earth_damage: &EarthDamage,
        fire_damage: &FireDamage,
        air_damage: &AirDamage,
        water_damage: &WaterDamage,
        crit: bool,
    ) -> u32 {
        let mut dmg = self.flat as f32;
        dmg += self.earth_multiplier * earth_damage.value;
        dmg += self.air_multiplier * air_damage.value;
        dmg += self.fire_multiplier * fire_damage.value;
        dmg += self.water_multiplier * water_damage.value;
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
