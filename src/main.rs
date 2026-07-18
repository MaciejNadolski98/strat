mod charges;
mod components;
mod constants;
mod draft;
mod effects;
mod enemies;
mod game;
mod hud;
mod item_definitions;
mod pathing;
mod projectiles;
mod resources;
mod setup;
mod shop;
mod spell_definitions;
mod spells;
mod tags;
mod tooltip;
mod tower_definitions;
mod towers;
mod waves;

use bevy::prelude::*;

use charges::advance_charges;
use constants::{
    BASE_ATTACK_SPEED, BASE_CRITICAL_CHANCE, BASE_LOOT, BASE_PIERCING_DAMAGE, BASE_REGENERATION,
    PLAYER_BASE_MAX_HP, STARTING_MONEY, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use draft::{place_draft_tower, sync_draft_previews, update_draft_input, update_draft_ui, update_tower_phantom};
use effects::{update_beam_effects, update_explosion_effects, update_floating_text, update_pulses};
use enemies::{move_enemies, reset_temporary_enemy_speed, spawn_enemies, update_enemy_colors, update_enemy_health_bars};
use game::{game_is_running, pan_camera, restart_game, toggle_pause};
use item_definitions::{ItemPlugins, ItemPoolRestoreSet};
use spell_definitions::{SpellPlugins, SpellRegistry};
use tower_definitions::{TowerPlugins, TowerRegistry};
use hud::update_hud;
use pathing::{update_path_hints, update_path_input};
use projectiles::move_projectiles;
use resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage,
    EnemiesRemaining, EnemyKilledEvent, ExplosionSize, FireDamage, ForcedTowerOffers, GameOver,
    GameWon, KillCount, MaxHp, Money, NextWaveTimer, Loot, PathTiles, Paused, Piercing,
    PiercingDamage, Regeneration, Shop, SpawnTimer, Stat, SpellShop, TowerDraft, WaterDamage,
    WaveNumber, GamePhase, GameRestartEvent, reset_stat_temporaries,
};
use setup::setup;
use shop::{activate_shop_on_restart, update_shop_input, update_shop_text, update_shop_tooltip};
use spells::{
    update_burning_enemies, update_spell_input, update_spell_slots, update_spell_tooltip,
};
use towers::{
    aim_towers, fire_beam_towers, fire_towers, progress_cooldown, reset_temporary_attack_speed, reset_temporary_damage_bonus,
    reset_temporary_range, update_draft_tooltip, update_tower_range_indicator, update_tower_tooltip,
};
use waves::RunMode;

use crate::resources::{ChargeConsumedEvent, ItemPurchasedEvent, NewRoundEvent, ShootEvent};

fn initialize_draft(
    registry: Res<TowerRegistry>,
    mut draft: ResMut<TowerDraft>,
    mut forced_towers: ResMut<ForcedTowerOffers>,
) {
    draft.known_kinds = registry.kinds.clone();
    draft.activate(&mut forced_towers);
}

fn initialize_shop(mut shop: ResMut<Shop>) {
    shop.activate(1);
}

fn initialize_spell_shop(registry: Res<SpellRegistry>, mut spell_shop: ResMut<SpellShop>) {
    spell_shop.known_kinds = registry.kinds.clone();
}

fn main() {
    let run_mode = RunMode::from_args(std::env::args());

    let mut app = App::new();
    app
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .insert_resource(run_mode)
        .insert_resource(Money {
            amount: STARTING_MONEY,
        })
        .insert_resource(CurrentHp {
            amount: PLAYER_BASE_MAX_HP,
        })
        .insert_resource(MaxHp(Stat::new(PLAYER_BASE_MAX_HP as f32)))
        .insert_resource(KillCount { amount: 0 })
        .insert_resource(GameOver { value: false })
        .insert_resource(GameWon { value: false })
        .insert_resource(Paused { value: false })
        .insert_resource(Regeneration(Stat::new(BASE_REGENERATION as f32)))
        .insert_resource(AttackSpeed(Stat::new(BASE_ATTACK_SPEED)))
        .insert_resource(Loot(Stat::new(BASE_LOOT as f32)))
        .insert_resource(CriticalChance(Stat::new(BASE_CRITICAL_CHANCE)))
        .insert_resource(ExplosionSize(Stat::new(0.0)))
        .insert_resource(EarthDamage(Stat::new(0.0)))
        .insert_resource(FireDamage(Stat::new(0.0)))
        .insert_resource(AirDamage(Stat::new(0.0)))
        .insert_resource(WaterDamage(Stat::new(0.0)))
        .insert_resource(Piercing(Stat::new(0.0)))
        .insert_resource(PiercingDamage(Stat::new(BASE_PIERCING_DAMAGE)))
        .insert_resource(WaveNumber { value: 1 })
        .insert_resource(EnemiesRemaining { count: 0 })
        .insert_resource(SpellRegistry::default())
        .insert_resource(TowerRegistry::default())
        .insert_resource(TowerDraft::new_empty())
        .insert_resource(SpawnTimer::new())
        .insert_resource(NextWaveTimer {
            timer: Timer::from_seconds(2.5, TimerMode::Once),
        })
        .insert_resource(PathTiles::new())
        .insert_resource(Shop::new_empty())
        .insert_resource(SpellShop::new_empty())
        .add_event::<ItemPurchasedEvent>()
        .add_event::<EnemyKilledEvent>()
        .add_event::<ShootEvent>()
        .add_event::<ChargeConsumedEvent>()
        .add_event::<NewRoundEvent>()
        .add_event::<GameRestartEvent>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Tower Defense".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ItemPlugins)
        .add_plugins(SpellPlugins)
        .add_plugins(TowerPlugins);

    let forced_towers = ForcedTowerOffers::from_args(
        std::env::args(),
        &app.world().resource::<TowerRegistry>().kinds,
    );
    app.insert_resource(forced_towers);

    app
        .add_systems(Startup, (setup, initialize_draft, initialize_shop, initialize_spell_shop).chain())
        .add_systems(Update, toggle_pause)
        .configure_sets(
            Update,
            (
                GamePhase::ResetTemporaries,
                GamePhase::TemporaryStatEffects,
                GamePhase::TemporaryTowerEffects,
                GamePhase::Gameplay,
            )
                .chain()
                .run_if(game_is_running)
                .after(toggle_pause),
        )
        .add_systems(
            Update,
            (
                reset_stat_temporaries,
                reset_temporary_attack_speed,
                reset_temporary_damage_bonus,
                reset_temporary_range,
            )
                .in_set(GamePhase::ResetTemporaries),
        )
        .add_systems(
            Update,
            (
                progress_cooldown,
                place_draft_tower,
                update_draft_input,
                update_spell_input,
                spawn_enemies,
                reset_temporary_enemy_speed,
                move_enemies,
                update_burning_enemies,
                aim_towers,
                fire_towers,
                fire_beam_towers,
                advance_charges,
                move_projectiles,
                update_explosion_effects,
                update_pulses,
                update_beam_effects,
                update_floating_text,
            )
                .chain()
                .in_set(GamePhase::Gameplay),
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
                sync_draft_previews,
                update_shop_tooltip,
                update_spell_tooltip,
                update_tower_tooltip,
                update_draft_tooltip,
                update_tower_range_indicator,
                update_tower_phantom,
                update_draft_ui,
            )
                .chain()
                .after(update_spell_slots),
        )
        .add_systems(Update, pan_camera)
        .add_systems(Update, update_shop_input.after(toggle_pause))
        .add_systems(Update, update_path_input.after(toggle_pause))
        .add_systems(Update, update_path_hints)
        .add_systems(Update, restart_game)
        .configure_sets(Update, ItemPoolRestoreSet.after(restart_game))
        .add_systems(Update, activate_shop_on_restart.after(ItemPoolRestoreSet))
        .run();
}
