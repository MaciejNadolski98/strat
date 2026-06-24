use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::{DamageDealt, HudText, Tower};
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, GameWon, KillCount, MaxHp, Money, Loot, PathTiles, Paused, Regeneration,
    TowerDraft, TowerDraftPhase, WaterDamage, WaveNumber,
};
use crate::tower_definitions::TowerKind;
use crate::waves::RunMode;

#[derive(SystemParam)]
pub struct HudStats<'w> {
    draft: Res<'w, TowerDraft>,
    money: Res<'w, Money>,
    hp: Res<'w, CurrentHp>,
    max_hp: Res<'w, MaxHp>,
    regeneration: Res<'w, Regeneration>,
    wave_number: Res<'w, WaveNumber>,
    kills: Res<'w, KillCount>,
    attack_speed: Res<'w, AttackSpeed>,
    loot: Res<'w, Loot>,
    critical_chance: Res<'w, CriticalChance>,
    explosion_size: Res<'w, ExplosionSize>,
    earth_damage: Res<'w, EarthDamage>,
    fire_damage: Res<'w, FireDamage>,
    air_damage: Res<'w, AirDamage>,
    water_damage: Res<'w, WaterDamage>,
    game_over: Res<'w, GameOver>,
    game_won: Res<'w, GameWon>,
    paused: Res<'w, Paused>,
    run_mode: Res<'w, RunMode>,
    path_tiles: Res<'w, PathTiles>,
}

pub fn update_hud(
    stats: HudStats,
    mut hud: Query<&mut Text, With<HudText>>,
    towers: Query<(&TowerKind, &DamageDealt), With<Tower>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let status = if stats.game_won.value {
        Some("Victory — press R to restart")
    } else if stats.game_over.value {
        Some("Game over — press R to restart")
    } else if stats.paused.value {
        Some("Paused — press Space to resume")
    } else if matches!(stats.draft.phase, TowerDraftPhase::Placing(_)) {
        Some("Left click on the map to place your tower")
    } else {
        None
    };

    let mut hud_text = format!(
        "Money: ${}   HP: {}/{}   Regen: {}   Wave: {}/{}   Kills: {}   Mode: {}   Path tile: ${}\nAtk speed: {:.2}x   loot: +${}/kill   Crit: {:.0}%   Explosion: {:.0}\nEarth: {:.0}   Fire: {:.0}   Air: {:.0}   Water: {:.0}",
        stats.money.amount,
        stats.hp.amount,
        stats.max_hp.amount,
        stats.regeneration.amount,
        stats.wave_number.value,
        stats.run_mode.final_wave(),
        stats.kills.amount,
        stats.run_mode.label(),
        stats.path_tiles.extension_cost(),
        stats.attack_speed.value,
        stats.loot.amount,
        stats.critical_chance.value * 100.0,
        stats.explosion_size.value,
        stats.earth_damage.value,
        stats.fire_damage.value,
        stats.air_damage.value,
        stats.water_damage.value,
    );
    if let Some(s) = status {
        hud_text.push('\n');
        hud_text.push_str(s);
    }

    if stats.game_over.value || stats.game_won.value {
        hud_text.push_str(&tower_damage_summary(&towers));
    }

    text.0 = hud_text;
}

fn tower_damage_summary(towers: &Query<(&TowerKind, &DamageDealt), With<Tower>>) -> String {
    let mut entries = towers
        .iter()
        .enumerate()
        .map(|(index, (kind, damage_dealt))| (index + 1, kind.name(), damage_dealt.amount))
        .collect::<Vec<_>>();

    if entries.is_empty() {
        return "\nTower damage: none".to_string();
    }

    entries.sort_by(|a, b| b.2.total_cmp(&a.2));

    let mut summary = "\nTower damage:".to_string();
    for (rank, (_, name, amount)) in entries.iter().enumerate() {
        summary.push_str(&format!("\n#{} {name}: {:.0}", rank + 1, amount));
    }

    summary
}
