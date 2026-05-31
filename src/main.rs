mod components;
mod constants;
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
use enemies::{enemies_in_wave, move_enemies, spawn_enemies, update_enemy_colors};
use game::restart_game;
use hud::update_hud;
use projectiles::move_projectiles;
use resources::{Game, PassiveIncomeClock, PlayerStats, Wave};
use setup::setup;
use towers::{aim_towers, apply_passive_income, place_tower, progress_cooldown};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .insert_resource(Game {
            money: STARTING_MONEY,
            lives: PLAYER_BASE_MAX_HP,
            kills: 0,
            game_over: false,
        })
        .insert_resource(PlayerStats::default())
        .insert_resource(PassiveIncomeClock {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        })
        .insert_resource(Wave {
            number: 1,
            remaining: enemies_in_wave(1),
            spawn_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            next_wave_timer: Timer::from_seconds(2.5, TimerMode::Once),
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
                apply_passive_income,
                spawn_enemies,
                move_enemies,
                aim_towers,
                move_projectiles,
                update_enemy_colors,
                update_hud,
                restart_game,
            )
                .chain(),
        )
        .run();
}
