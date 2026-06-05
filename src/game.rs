use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::{Enemy, PathEndMarker, PathTile, Projectile, Tower};
use crate::constants::STARTING_MONEY;
use crate::pathing::spawn_path_tile;
use crate::resources::{
    ActiveSpellEffects, CurrentHp, EnemiesRemaining, GameOver, GameWon, KillCount, MaxHp, Money,
    NextWaveTimer, PathTiles, Paused, Shop, SpawnTimer, SpellShop, WaveNumber,
};
use crate::waves::enemies_in_wave;

#[derive(SystemParam)]
pub struct RestartState<'w> {
    money: ResMut<'w, Money>,
    hp: ResMut<'w, CurrentHp>,
    max_hp: Res<'w, MaxHp>,
    kills: ResMut<'w, KillCount>,
    game_over: ResMut<'w, GameOver>,
    game_won: ResMut<'w, GameWon>,
    wave_number: ResMut<'w, WaveNumber>,
    remaining: ResMut<'w, EnemiesRemaining>,
    spawn_timer: ResMut<'w, SpawnTimer>,
    next_wave_timer: ResMut<'w, NextWaveTimer>,
    shop: ResMut<'w, Shop>,
    spell_shop: ResMut<'w, SpellShop>,
    active_spell_effects: ResMut<'w, ActiveSpellEffects>,
    paused: ResMut<'w, Paused>,
    path_tiles: ResMut<'w, PathTiles>,
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_over: Res<GameOver>,
    game_won: Res<GameWon>,
    mut paused: ResMut<Paused>,
) {
    if game_over.value || game_won.value || !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    paused.value = !paused.value;
}

pub fn game_is_running(paused: Res<Paused>, game_won: Res<GameWon>) -> bool {
    !paused.value && !game_won.value
}

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: RestartState,
    mut cleanup: ParamSet<(
        Query<Entity, With<Tower>>,
        Query<Entity, With<Enemy>>,
        Query<Entity, With<Projectile>>,
        Query<Entity, With<PathTile>>,
    )>,
    mut end_marker: Query<&mut Transform, With<PathEndMarker>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in cleanup.p0().iter() {
        commands.entity(entity).despawn();
    }
    for entity in cleanup.p1().iter() {
        commands.entity(entity).despawn();
    }
    for entity in cleanup.p2().iter() {
        commands.entity(entity).despawn();
    }
    for entity in cleanup.p3().iter() {
        commands.entity(entity).despawn();
    }

    state.money.amount = STARTING_MONEY;
    state.hp.amount = state.max_hp.amount;
    state.kills.amount = 0;
    state.game_over.value = false;
    state.game_won.value = false;

    state.wave_number.value = 1;
    state.remaining.count = enemies_in_wave(1);
    state.spawn_timer.reset();
    state.next_wave_timer.timer.reset();
    *state.shop = Shop::new(1);
    *state.spell_shop = SpellShop::new();
    state.path_tiles.reset();
    for tile in &state.path_tiles.tiles {
        spawn_path_tile(&mut commands, *tile);
    }
    if let Ok(mut marker_transform) = end_marker.single_mut() {
        marker_transform.translation = state.path_tiles.end().extend(0.0);
    }
    state.active_spell_effects.reset_for_wave();
    state.paused.value = false;
}
