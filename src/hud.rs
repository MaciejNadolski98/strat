use bevy::prelude::*;

use crate::components::HudText;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, KillCount, MaxHp, Money, PassiveIncome, Regeneration, WaterDamage, WaveNumber,
};

pub fn update_hud(
    money: Res<Money>,
    hp: Res<CurrentHp>,
    max_hp: Res<MaxHp>,
    regeneration: Res<Regeneration>,
    wave_number: Res<WaveNumber>,
    kills: Res<KillCount>,
    attack_speed: Res<AttackSpeed>,
    passive_income: Res<PassiveIncome>,
    critical_chance: Res<CriticalChance>,
    explosion_size: Res<ExplosionSize>,
    earth_damage: Res<EarthDamage>,
    fire_damage: Res<FireDamage>,
    air_damage: Res<AirDamage>,
    water_damage: Res<WaterDamage>,
    game_over: Res<GameOver>,
    mut hud: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let status = if game_over.value {
        "Game over - press R to restart"
    } else {
        "Left click: place tower ($40)"
    };

    text.0 = format!(
        "Money: ${}   HP: {}/{}   Regen: {}   Wave: {}   Kills: {}\nAtk speed: {:.2}x   Income: ${}/s   Crit: {:.0}%   Explosion: {:.0}\nEarth: {:.0}   Fire: {:.0}   Air: {:.0}   Water: {:.0}\n{}",
        money.amount,
        hp.amount,
        max_hp.amount,
        regeneration.amount,
        wave_number.value,
        kills.amount,
        attack_speed.value,
        passive_income.amount,
        critical_chance.value * 100.0,
        explosion_size.value,
        earth_damage.value,
        fire_damage.value,
        air_damage.value,
        water_damage.value,
        status
    );
}
