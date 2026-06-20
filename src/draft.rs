use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    AngularSpeed, DamageDealt, DraftHeaderText, DraftPanel, DraftSlot, DraftSlotBarrel,
    DraftSlotIcon, DraftSlotLabel, FireCooldown, Tower, TowerPhantom, TowerPhantomBarrel,
    TowerRangeIndicator,
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
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    draft_slots: Query<(&DraftSlot, &Transform)>,
    mut draft: ResMut<TowerDraft>,
    game_over: Res<GameOver>,
) {
    if game_over.value || draft.phase != TowerDraftPhase::Picking {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return; };
        let Ok((cam, cam_transform)) = camera.single() else { return; };
        let Some(cursor_pos) = window.cursor_position() else { return; };
        let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

        for (slot, transform) in &draft_slots {
            let pos = transform.translation.truncate();
            if (world_pos.x - pos.x).abs() <= 65.0 && (world_pos.y - pos.y).abs() <= 70.0 {
                draft.phase = TowerDraftPhase::Placing(draft.offers[slot.index]);
                return;
            }
        }
    }
}

pub fn update_draft_ui(
    draft: Res<TowerDraft>,
    wave_number: Res<WaveNumber>,
    mut queries: ParamSet<(
        Query<&mut Visibility, With<DraftPanel>>,
        Query<(&mut Text2d, &mut Visibility), With<DraftHeaderText>>,
        Query<(&mut Sprite, &mut Visibility), With<DraftSlot>>,
        Query<(&DraftSlotIcon, &mut Sprite, &mut Visibility)>,
        Query<(&DraftSlotBarrel, &mut Sprite, &mut Visibility)>,
        Query<(&DraftSlotLabel, &mut Text2d, &mut Visibility)>,
    )>,
) {
    let is_visible = draft.phase == TowerDraftPhase::Picking;

    if let Ok(mut visibility) = queries.p0().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }

    if let Ok((mut text, mut visibility)) = queries.p1().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            text.0 = if draft.phase == TowerDraftPhase::Picking {
                format!("Wave {} — Click a tower to pick it", wave_number.value)
            } else {
                "Click on the map to place your tower".to_string()
            };
        }
    }

    for (mut sprite, mut visibility) in &mut queries.p2() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            sprite.color = Color::srgb(0.15, 0.17, 0.16)
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
    if game_over.value || !matches!(draft.phase, TowerDraftPhase::Placing(_)) {
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

    let TowerDraftPhase::Placing(tower_kind) = draft.phase else { return; };
    apply_tower_effects(tower_kind, &mut stats);

    commands
        .spawn((
            tower_kind.body_sprite(1.0),
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
            tower_kind.barrel_sprite(1.0),
            Transform::from_translation(Vec3::new(0.0, tower_kind.barrel_offset(), 1.0)),
        ));

    draft.phase = TowerDraftPhase::WaveRunning;
    remaining.count = enemies_in_wave(wave_number.value);
    spawn_timer.reset();
}

pub fn update_tower_phantom(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<&Transform, With<Tower>>,
    path_tiles: Res<PathTiles>,
    draft: Res<TowerDraft>,
    mut phantom: Query<
        (&mut Transform, &mut Sprite, &mut Visibility),
        (With<TowerPhantom>, Without<Tower>, Without<TowerPhantomBarrel>),
    >,
    mut barrel: Query<
        (&mut Transform, &mut Sprite, &mut Visibility),
        (With<TowerPhantomBarrel>, Without<Tower>, Without<TowerPhantom>),
    >,
    mut indicator: Query<
        (&mut Transform, &mut Visibility),
        (
            With<TowerRangeIndicator>,
            Without<Tower>,
            Without<TowerPhantom>,
            Without<TowerPhantomBarrel>,
        ),
    >,
) {
    let Ok((mut p_transform, mut p_sprite, mut p_visibility)) = phantom.single_mut() else {
        return;
    };
    let Ok((mut b_transform, mut b_sprite, mut b_visibility)) = barrel.single_mut() else {
        return;
    };
    let Ok((mut i_transform, mut i_visibility)) = indicator.single_mut() else {
        return;
    };

    *p_visibility = Visibility::Hidden;
    *b_visibility = Visibility::Hidden;
    *i_visibility = Visibility::Hidden;

    let TowerDraftPhase::Placing(kind) = draft.phase else {
        return;
    };

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

    let grid_pos = snap_to_grid(world_pos);

    if path_tiles.can_extend_to(grid_pos)
        || !is_buildable_cell(grid_pos, &path_tiles)
        || towers
            .iter()
            .any(|t| t.translation.truncate().distance(grid_pos) < GRID_SIZE * 0.5)
    {
        return;
    }

    const ALPHA: f32 = 0.55;

    *p_sprite = kind.body_sprite(ALPHA);
    p_transform.translation = grid_pos.extend(3.0);
    *p_visibility = Visibility::Visible;

    *b_sprite = kind.barrel_sprite(ALPHA);
    b_transform.translation = Vec3::new(grid_pos.x, grid_pos.y + kind.barrel_offset(), 4.0);
    *b_visibility = Visibility::Visible;

    i_transform.translation = grid_pos.extend(1.5);
    i_transform.scale = Vec3::splat(kind.range());
    *i_visibility = Visibility::Visible;
}
