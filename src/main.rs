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
mod shop;
mod spells;
mod towers;
mod waves;

use bevy::prelude::*;

use constants::{PLAYER_BASE_MAX_HP, STARTING_MONEY, WINDOW_HEIGHT, WINDOW_WIDTH};
use effects::{update_explosion_effects, update_floating_text};
use enemies::{move_enemies, spawn_enemies, update_enemy_colors, update_enemy_health_bars};
use game::{game_is_running, restart_game, toggle_pause};
use hud::update_hud;
use pathing::update_path_input;
use projectiles::move_projectiles;
use resources::{
    ActiveSpellEffects, AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage,
    EnemiesRemaining, ExplosionSize, FireDamage, GameOver, GameWon, KillCount, MaxHp, Money,
    NextWaveTimer, PassiveIncome, PathTiles, Paused, Regeneration, Shop, SpawnTimer, SpellShop,
    WaterDamage, WaveNumber,
};
use setup::setup;
use shop::{update_shop_input, update_shop_text, update_shop_tooltip};
use spells::{
    update_burning_enemies, update_spell_input, update_spell_slots, update_spell_tooltip,
};
use towers::{aim_towers, place_tower, progress_cooldown, update_tower_tooltip};
use waves::{RunMode, enemies_in_wave};

fn main() {
    let run_mode = RunMode::from_args(std::env::args());

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .insert_resource(run_mode)
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
        .insert_resource(GameWon { value: false })
        .insert_resource(Paused { value: false })
        .insert_resource(Regeneration { amount: 1 })
        .insert_resource(AttackSpeed { value: 1.0 })
        .insert_resource(PassiveIncome { amount: 2 })
        .insert_resource(CriticalChance { value: 0.12 })
        .insert_resource(ExplosionSize { value: 0.0 })
        .insert_resource(EarthDamage { value: 0.0 })
        .insert_resource(FireDamage { value: 0.0 })
        .insert_resource(AirDamage { value: 0.0 })
        .insert_resource(WaterDamage { value: 0.0 })
        .insert_resource(ActiveSpellEffects::new())
        .insert_resource(WaveNumber { value: 1 })
        .insert_resource(EnemiesRemaining {
            count: enemies_in_wave(1),
        })
        .insert_resource(SpawnTimer::new())
        .insert_resource(NextWaveTimer {
            timer: Timer::from_seconds(2.5, TimerMode::Once),
        })
        .insert_resource(PathTiles::new())
        .insert_resource(Shop::new(1))
        .insert_resource(SpellShop::new())
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
        .add_systems(Update, toggle_pause)
        .add_systems(
            Update,
            (
                progress_cooldown,
                update_shop_input,
                update_spell_input,
                update_path_input,
                place_tower,
                spawn_enemies,
                move_enemies,
                update_burning_enemies,
                aim_towers,
                move_projectiles,
                update_explosion_effects,
                update_floating_text,
            )
                .chain()
                .run_if(game_is_running)
                .after(toggle_pause),
        )
        .add_systems(
            Update,
            (
                update_enemy_colors,
                update_enemy_health_bars,
                update_hud,
                update_shop_text,
                update_spell_slots,
            )
                .chain()
                .after(update_floating_text),
        )
        .add_systems(
            Update,
            (
                update_shop_tooltip,
                update_spell_tooltip,
                update_tower_tooltip,
            )
                .chain()
                .after(update_spell_slots),
        )
        .add_systems(Update, restart_game)
        .run();
}
