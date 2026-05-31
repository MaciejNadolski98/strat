use bevy::prelude::*;

#[derive(Component)]
pub struct Tower {
    pub fire_cooldown: Timer,
    pub rotational_speed: f32,
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub waypoint: usize,
    pub progress: f32,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub reward: i32,
}

#[derive(Clone, Copy)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Brute,
    Armored,
}

impl EnemyKind {
    pub fn for_spawn(wave: u32, spawn_index: u32) -> Self {
        let sequence = spawn_index + 1;

        if wave >= 5 && sequence % 7 == 0 {
            Self::Armored
        } else if wave >= 3 && sequence % 5 == 0 {
            Self::Brute
        } else if wave >= 2 && sequence % 3 == 0 {
            Self::Runner
        } else {
            Self::Grunt
        }
    }

    pub fn max_health(self, wave: u32) -> f32 {
        match self {
            Self::Grunt => 55.0 + wave as f32 * 16.0,
            Self::Runner => 38.0 + wave as f32 * 10.0,
            Self::Brute => 105.0 + wave as f32 * 25.0,
            Self::Armored => 80.0 + wave as f32 * 22.0,
        }
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
pub struct Projectile {
    pub target: Entity,
    pub speed: f32,
    pub damage: f32,
    pub explosion_radius: f32,
}

#[derive(Component)]
pub struct HudText;
