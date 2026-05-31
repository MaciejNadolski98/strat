use bevy::prelude::*;

use crate::components::HudText;
use crate::resources::{Game, PlayerStats, Wave};

pub fn update_hud(
    game: Res<Game>,
    wave: Res<Wave>,
    stats: Res<PlayerStats>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let status = if game.game_over {
        "Game over - press R to restart"
    } else {
        "Left click: place tower ($40)"
    };

    text.0 = format!(
        "Money: ${}   HP: {}/{}   Regen: {}   Wave: {}   Kills: {}\nAtk speed: {:.2}x   Income: ${}/s   Crit: {:.0}%   Explosion: {:.0}\nEarth: {:.0}   Fire: {:.0}   Air: {:.0}   Water: {:.0}\n{}",
        game.money,
        game.lives,
        stats.max_hp,
        stats.regeneration,
        wave.number,
        game.kills,
        stats.attack_speed,
        stats.passive_income,
        stats.critical_chance * 100.0,
        stats.explosion_size,
        stats.earth_damage,
        stats.fire_damage,
        stats.air_damage,
        stats.water_damage,
        status
    );
}
