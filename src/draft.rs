use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    AngularSpeed, DamageDealt, DraftHeaderText, DraftPanel, DraftSlot, DraftSlotBarrel,
    DraftSlotIcon, DraftSlotLabel, FireCooldown, Tower,
};
use crate::constants::GRID_SIZE;
use crate::pathing::{is_buildable_cell, snap_to_grid};
use crate::resources::{
    EnemiesRemaining, GameOver, PathTiles, SpawnTimer, TowerDraft, TowerDraftPhase, WaveNumber,
};
use crate::shop::PlayerStatsMut;
use crate::towers::apply_tower_effects;
use crate::waves::enemies_in_wave;

pub fn update_draft_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut draft: ResMut<TowerDraft>,
    game_over: Res<GameOver>,
) {
    if game_over.value || draft.phase != TowerDraftPhase::Picking {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        draft.selected = 0;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        draft.selected = 1;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        draft.selected = 2;
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        draft.phase = TowerDraftPhase::Placing;
    }
}

pub fn update_draft_ui(
    draft: Res<TowerDraft>,
    wave_number: Res<WaveNumber>,
    mut queries: ParamSet<(
        Query<&mut Visibility, With<DraftPanel>>,
        Query<(&mut Text2d, &mut Visibility), With<DraftHeaderText>>,
        Query<(&DraftSlot, &mut Sprite, &mut Visibility)>,
        Query<(&DraftSlotIcon, &mut Sprite, &mut Visibility)>,
        Query<(&DraftSlotBarrel, &mut Sprite, &mut Visibility)>,
        Query<(&DraftSlotLabel, &mut Text2d, &mut Visibility)>,
    )>,
) {
    let is_visible = matches!(draft.phase, TowerDraftPhase::Picking | TowerDraftPhase::Placing);

    if let Ok(mut visibility) = queries.p0().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }

    if let Ok((mut text, mut visibility)) = queries.p1().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            text.0 = if draft.phase == TowerDraftPhase::Picking {
                format!(
                    "Wave {} — Pick a tower    1  2  3 to select    Enter to confirm",
                    wave_number.value
                )
            } else {
                "Click on the map to place your tower".to_string()
            };
        }
    }

    for (slot, mut sprite, mut visibility) in &mut queries.p2() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            sprite.color = if slot.index == draft.selected && draft.phase == TowerDraftPhase::Picking {
                Color::srgb(0.32, 0.34, 0.24)
            } else {
                Color::srgb(0.15, 0.17, 0.16)
            };
        }
    }

    for (icon, mut sprite, mut visibility) in &mut queries.p3() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            sprite.color = draft.offers[icon.index].base_color();
        }
    }

    for (barrel, mut sprite, mut visibility) in &mut queries.p4() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            sprite.color = draft.offers[barrel.index].barrel_color();
        }
    }

    for (label, mut text, mut visibility) in &mut queries.p5() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            text.0 = draft.offers[label.index].name().to_string();
        }
    }
}

pub fn place_draft_tower(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<&Transform, With<Tower>>,
    game_over: Res<GameOver>,
    path_tiles: Res<PathTiles>,
    mut draft: ResMut<TowerDraft>,
    wave_number: Res<WaveNumber>,
    mut remaining: ResMut<EnemiesRemaining>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut stats: PlayerStatsMut,
) {
    if game_over.value || draft.phase != TowerDraftPhase::Placing {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let grid_position = snap_to_grid(world_position);
    if path_tiles.can_extend_to(grid_position)
        || !is_buildable_cell(grid_position, &path_tiles)
        || towers
            .iter()
            .any(|t| t.translation.truncate().distance(grid_position) < GRID_SIZE * 0.5)
    {
        return;
    }

    let tower_kind = draft.selected_kind();
    apply_tower_effects(tower_kind, &mut stats);

    commands
        .spawn((
            Sprite::from_color(tower_kind.base_color(), tower_kind.base_size()),
            Transform::from_translation(grid_position.extend(2.0)),
            Tower,
            tower_kind,
            DamageDealt { amount: 0.0 },
            tower_kind.damage_formula(),
            FireCooldown {
                timer: Timer::new(
                    Duration::from_secs_f32(
                        tower_kind.cooldown() / stats.attack_speed_value().max(0.1),
                    ),
                    TimerMode::Once,
                ),
            },
            AngularSpeed {
                value: tower_kind.angular_speed(),
            },
        ))
        .with_child((
            Sprite::from_color(tower_kind.barrel_color(), tower_kind.barrel_size()),
            Transform::from_translation(Vec3::new(0.0, tower_kind.barrel_offset(), 1.0)),
        ));

    draft.phase = TowerDraftPhase::WaveRunning;
    remaining.count = enemies_in_wave(wave_number.value);
    spawn_timer.reset();
}
