use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::components::{Enemy, MainCamera, PathEdge, PathEndMarker, PathExtensionHint, PathTile, Projectile, Tower};
use crate::constants::{
    BASE_ATTACK_SPEED, BASE_CRITICAL_CHANCE, BASE_LOOT, BASE_REGENERATION, PLAYER_BASE_MAX_HP,
    STARTING_MONEY,
};
use crate::pathing::spawn_path_visuals;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage,
    EnemiesRemaining, ExplosionSize, FireDamage, ForcedTowerOffers, GameOver, GameRestartEvent,
    GameWon, KillCount, Loot, MaxHp, Money, NewRoundEvent, NextWaveTimer, PathTiles, Paused,
    Regeneration, SpawnTimer, SpellShop, TowerDraft, WaterDamage, WaveNumber,
};

#[derive(SystemParam)]
pub struct RestartState<'w> {
    money: ResMut<'w, Money>,
    hp: ResMut<'w, CurrentHp>,
    max_hp: ResMut<'w, MaxHp>,
    kills: ResMut<'w, KillCount>,
    game_over: ResMut<'w, GameOver>,
    game_won: ResMut<'w, GameWon>,
    wave_number: ResMut<'w, WaveNumber>,
    remaining: ResMut<'w, EnemiesRemaining>,
    spawn_timer: ResMut<'w, SpawnTimer>,
    next_wave_timer: ResMut<'w, NextWaveTimer>,
    spell_shop: ResMut<'w, SpellShop>,
    draft: ResMut<'w, TowerDraft>,
    forced_towers: ResMut<'w, ForcedTowerOffers>,
    paused: ResMut<'w, Paused>,
    path_tiles: ResMut<'w, PathTiles>,
    regeneration: ResMut<'w, Regeneration>,
    attack_speed: ResMut<'w, AttackSpeed>,
    loot: ResMut<'w, Loot>,
    critical_chance: ResMut<'w, CriticalChance>,
    explosion_size: ResMut<'w, ExplosionSize>,
    earth_damage: ResMut<'w, EarthDamage>,
    fire_damage: ResMut<'w, FireDamage>,
    air_damage: ResMut<'w, AirDamage>,
    water_damage: ResMut<'w, WaterDamage>,
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

const PAN_SPEED: f32 = 320.0;

pub fn pan_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else { return; };
    let mut dir = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) { dir.y += 1.0; }
    if keyboard.pressed(KeyCode::KeyS) { dir.y -= 1.0; }
    if keyboard.pressed(KeyCode::KeyA) { dir.x -= 1.0; }
    if keyboard.pressed(KeyCode::KeyD) { dir.x += 1.0; }
    transform.translation += dir.normalize_or_zero().extend(0.0) * PAN_SPEED * time.delta_secs();
}

pub fn game_is_running(paused: Res<Paused>, game_won: Res<GameWon>) -> bool {
    !paused.value && !game_won.value
}

pub fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: RestartState,
    mut new_round: EventWriter<NewRoundEvent>,
    mut game_restart: EventWriter<GameRestartEvent>,
    mut cleanup: ParamSet<(
        Query<Entity, With<Tower>>,
        Query<Entity, With<Enemy>>,
        Query<Entity, With<Projectile>>,
        Query<Entity, With<PathTile>>,
        Query<Entity, With<PathEdge>>,
    )>,
    hints: Query<Entity, With<PathExtensionHint>>,
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
    for entity in cleanup.p4().iter() {
        commands.entity(entity).despawn();
    }
    for entity in &hints {
        commands.entity(entity).despawn();
    }

    state.max_hp.raw_value = PLAYER_BASE_MAX_HP as f32;
    state.money.amount = STARTING_MONEY;
    state.hp.amount = PLAYER_BASE_MAX_HP;
    state.kills.amount = 0;
    state.game_over.value = false;
    state.game_won.value = false;
    state.regeneration.raw_value = BASE_REGENERATION as f32;
    state.attack_speed.raw_value = BASE_ATTACK_SPEED;
    state.loot.raw_value = BASE_LOOT as f32;
    state.critical_chance.raw_value = BASE_CRITICAL_CHANCE;
    state.explosion_size.raw_value = 0.0;
    state.earth_damage.raw_value = 0.0;
    state.fire_damage.raw_value = 0.0;
    state.air_damage.raw_value = 0.0;
    state.water_damage.raw_value = 0.0;

    state.wave_number.value = 1;
    state.remaining.count = 0;
    state.spawn_timer.reset();
    state.next_wave_timer.timer.reset();
    state.spell_shop.reset();
    state.forced_towers.reset();
    state.draft.activate(&mut state.forced_towers);
    state.path_tiles.reset();
    spawn_path_visuals(&mut commands, &state.path_tiles, &[]);
    if let Ok(mut marker_transform) = end_marker.single_mut() {
        marker_transform.translation = state.path_tiles.end().extend(0.0);
    }
    new_round.write(NewRoundEvent);
    game_restart.write(GameRestartEvent);
    state.paused.value = false;
}
