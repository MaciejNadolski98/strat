use bevy::prelude::*;

use crate::components::{Enemy, Projectile, Tower};
use crate::constants::STARTING_MONEY;
use crate::enemies::enemies_in_wave;
use crate::resources::{Game, PlayerStats, Wave};

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut wave: ResMut<Wave>,
    stats: Res<PlayerStats>,
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

    game.money = STARTING_MONEY;
    game.lives = stats.max_hp;
    game.kills = 0;
    game.game_over = false;

    wave.number = 1;
    wave.remaining = enemies_in_wave(1);
    wave.spawn_timer.reset();
    wave.next_wave_timer.reset();
}
