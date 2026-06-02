use bevy::prelude::*;

use crate::components::{Enemy, Projectile, Tower};
use crate::constants::STARTING_MONEY;
use crate::resources::{
    CurrentHp, EnemiesRemaining, GameOver, GameWon, KillCount, MaxHp, Money, NextWaveTimer, Paused,
    Shop, SpawnTimer, WaveNumber,
};
use crate::waves::enemies_in_wave;

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
    mut money: ResMut<Money>,
    mut hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
    mut kills: ResMut<KillCount>,
    mut game_over: ResMut<GameOver>,
    mut game_won: ResMut<GameWon>,
    mut wave_number: ResMut<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut next_wave_timer: ResMut<NextWaveTimer>,
    mut shop: ResMut<Shop>,
    mut paused: ResMut<Paused>,
    mut cleanup: ParamSet<(
        Query<Entity, With<Tower>>,
        Query<Entity, With<Enemy>>,
        Query<Entity, With<Projectile>>,
    )>,
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

    money.amount = STARTING_MONEY;
    hp.amount = max_hp.amount;
    kills.amount = 0;
    game_over.value = false;
    game_won.value = false;

    wave_number.value = 1;
    remaining.count = enemies_in_wave(1);
    spawn_timer.reset();
    next_wave_timer.timer.reset();
    *shop = Shop::new(1);
    paused.value = false;
}
