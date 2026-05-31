use bevy::prelude::*;

use crate::constants::PLAYER_BASE_MAX_HP;

#[derive(Resource)]
pub struct Game {
    pub money: i32,
    pub lives: i32,
    pub kills: u32,
    pub game_over: bool,
}

#[derive(Resource)]
pub struct PlayerStats {
    pub max_hp: i32,
    pub regeneration: i32,
    pub attack_speed: f32,
    pub passive_income: i32,
    pub critical_chance: f32,
    pub explosion_size: f32,
    pub earth_damage: f32,
    pub fire_damage: f32,
    pub air_damage: f32,
    pub water_damage: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            max_hp: PLAYER_BASE_MAX_HP,
            regeneration: 1,
            attack_speed: 1.0,
            passive_income: 2,
            critical_chance: 0.12,
            explosion_size: 0.0,
            earth_damage: 0.0,
            fire_damage: 0.0,
            air_damage: 0.0,
            water_damage: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct Wave {
    pub number: u32,
    pub remaining: u32,
    pub spawn_timer: Timer,
    pub next_wave_timer: Timer,
}
