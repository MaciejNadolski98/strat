mod components;
mod constants;
mod effects;
mod enemies;
mod game;
mod hud;
mod pathing;
mod projectiles;
mod resources;
mod setup;
mod towers;

use bevy::prelude::*;

use constants::{PLAYER_BASE_MAX_HP, STARTING_MONEY, WINDOW_HEIGHT, WINDOW_WIDTH};
use effects::update_floating_text;
use enemies::{enemies_in_wave, move_enemies, spawn_enemies, update_enemy_colors};
use game::restart_game;
use hud::update_hud;
use projectiles::move_projectiles;
use resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, EnemiesRemaining,
    ExplosionSize, FireDamage, GameOver, KillCount, MaxHp, Money, NextWaveTimer, PassiveIncome,
    Regeneration, SpawnTimer, WaterDamage, WaveNumber,
};
use setup::setup;
use towers::{aim_towers, place_tower, progress_cooldown};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .insert_resource(Money {
            amount: STARTING_MONEY,
        })
        .insert_resource(CurrentHp {
            amount: PLAYER_BASE_MAX_HP,
        })
        .insert_resource(MaxHp {
            amount: PLAYER_BASE_MAX_HP,
        })
        .insert_resource(KillCount { amount: 0 })
        .insert_resource(GameOver { value: false })
        .insert_resource(Regeneration { amount: 1 })
        .insert_resource(AttackSpeed { value: 1.0 })
        .insert_resource(PassiveIncome { amount: 2 })
        .insert_resource(CriticalChance { value: 0.12 })
        .insert_resource(ExplosionSize { value: 0.0 })
        .insert_resource(EarthDamage { value: 0.0 })
        .insert_resource(FireDamage { value: 0.0 })
        .insert_resource(AirDamage { value: 0.0 })
        .insert_resource(WaterDamage { value: 0.0 })
        .insert_resource(WaveNumber { value: 1 })
        .insert_resource(EnemiesRemaining {
            count: enemies_in_wave(1),
        })
        .insert_resource(SpawnTimer {
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        })
        .insert_resource(NextWaveTimer {
            timer: Timer::from_seconds(2.5, TimerMode::Once),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Tower Defense".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                progress_cooldown,
                place_tower,
                spawn_enemies,
                move_enemies,
                aim_towers,
                move_projectiles,
                update_enemy_colors,
                update_floating_text,
                update_hud,
                restart_game,
            )
                .chain(),
        )
        .run();
}
