use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::HudText;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, KillCount, MaxHp, Money, PassiveIncome, Paused, Regeneration, WaterDamage,
    WaveNumber,
};

#[derive(SystemParam)]
pub struct HudStats<'w> {
    money: Res<'w, Money>,
    hp: Res<'w, CurrentHp>,
    max_hp: Res<'w, MaxHp>,
    regeneration: Res<'w, Regeneration>,
    wave_number: Res<'w, WaveNumber>,
    kills: Res<'w, KillCount>,
    attack_speed: Res<'w, AttackSpeed>,
    passive_income: Res<'w, PassiveIncome>,
    critical_chance: Res<'w, CriticalChance>,
    explosion_size: Res<'w, ExplosionSize>,
    earth_damage: Res<'w, EarthDamage>,
    fire_damage: Res<'w, FireDamage>,
    air_damage: Res<'w, AirDamage>,
    water_damage: Res<'w, WaterDamage>,
    game_over: Res<'w, GameOver>,
    paused: Res<'w, Paused>,
}

pub fn update_hud(stats: HudStats, mut hud: Query<&mut Text, With<HudText>>) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let status = if stats.game_over.value {
        "Game over - press R to restart"
    } else if stats.paused.value {
        "Paused - press Space to resume"
    } else {
        "Left click: place selected shop item   Space: pause"
    };

    text.0 = format!(
        "Money: ${}   HP: {}/{}   Regen: {}   Wave: {}   Kills: {}\nAtk speed: {:.2}x   Income: ${}/s   Crit: {:.0}%   Explosion: {:.0}\nEarth: {:.0}   Fire: {:.0}   Air: {:.0}   Water: {:.0}\n{}",
        stats.money.amount,
        stats.hp.amount,
        stats.max_hp.amount,
        stats.regeneration.amount,
        stats.wave_number.value,
        stats.kills.amount,
        stats.attack_speed.value,
        stats.passive_income.amount,
        stats.critical_chance.value * 100.0,
        stats.explosion_size.value,
        stats.earth_damage.value,
        stats.fire_damage.value,
        stats.air_damage.value,
        stats.water_damage.value,
        status
    );
}
