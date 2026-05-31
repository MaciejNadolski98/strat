use bevy::prelude::*;

use crate::components::{Enemy, Projectile, Tower};
use crate::constants::STARTING_MONEY;
use crate::enemies::enemies_in_wave;
use crate::resources::{
    CurrentHp, EnemiesRemaining, GameOver, KillCount, MaxHp, Money, NextWaveTimer, Shop,
    SpawnTimer, WaveNumber,
};

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut money: ResMut<Money>,
    mut hp: ResMut<CurrentHp>,
    max_hp: Res<MaxHp>,
    mut kills: ResMut<KillCount>,
    mut game_over: ResMut<GameOver>,
    mut wave_number: ResMut<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut next_wave_timer: ResMut<NextWaveTimer>,
    mut shop: ResMut<Shop>,
    towers: Query<Entity, With<Tower>>,
    enemies: Query<Entity, With<Enemy>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in towers
        .iter()
        .chain(enemies.iter())
        .chain(projectiles.iter())
    {
        commands.entity(entity).despawn();
    }

    money.amount = STARTING_MONEY;
    hp.amount = max_hp.amount;
    kills.amount = 0;
    game_over.value = false;

    wave_number.value = 1;
    remaining.count = enemies_in_wave(1);
    spawn_timer.timer.reset();
    next_wave_timer.timer.reset();
    *shop = Shop::new();
}
