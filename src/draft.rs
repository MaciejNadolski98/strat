use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy::sprite::ColorMaterial;

use crate::components::{
    AngularSpeed, DraftHeaderText, DraftPanel, DraftPreview, DraftSlot, DraftSlotBarrel,
    DraftSlotIcon, DraftSlotLabel, FireCooldown, TemporaryAttackSpeed, TemporaryDamageBonus,
    Tower, TowerPhantom, TowerPhantomBarrel, TowerRangeIndicator,
};
use crate::tower_definitions::{BarrelTemplate, TowerKind};
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
    draft_slots: Query<(&DraftSlot, &GlobalTransform)>,
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

        for (slot, global) in &draft_slots {
            let pos = global.translation().truncate();
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
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut queries: ParamSet<(
        Query<&mut Visibility, With<DraftPanel>>,
        Query<(&mut Text2d, &mut Visibility), With<DraftHeaderText>>,
        Query<(&GlobalTransform, &mut Sprite, &mut Visibility), With<DraftSlot>>,
        Query<(&DraftSlotIcon, &mut Mesh2d, &mut MeshMaterial2d<ColorMaterial>, &mut Visibility)>,
        Query<(&DraftSlotBarrel, &mut Sprite, &mut Visibility, &mut Transform)>,
        Query<(&DraftSlotLabel, &mut Text2d, &mut Visibility)>,
    )>,
) {
    let is_visible = draft.phase == TowerDraftPhase::Picking;

    let cursor_world = (|| -> Option<Vec2> {
        let window = windows.single().ok()?;
        let (cam, cam_t) = camera.single().ok()?;
        cam.viewport_to_world_2d(cam_t, window.cursor_position()?).ok()
    })();

    if let Ok(mut visibility) = queries.p0().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
    }

    if let Ok((mut text, mut visibility)) = queries.p1().single_mut() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            text.0 = if draft.phase == TowerDraftPhase::Picking {
                format!("Wave {} - Click a tower to pick it", wave_number.value)
            } else {
                "Click on the map to place your tower".to_string()
            };
        }
    }

    for (global, mut sprite, mut visibility) in &mut queries.p2() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            let pos: Vec2 = global.translation().truncate();
            let is_hovered = cursor_world
                .map(|wp| (wp.x - pos.x).abs() <= 65.0 && (wp.y - pos.y).abs() <= 70.0)
                .unwrap_or(false);
            sprite.color = if is_hovered {
                Color::srgb(0.28, 0.32, 0.22)
            } else {
                Color::srgb(0.15, 0.17, 0.16)
            };
        }
    }

    for (icon, mut mesh, mut mat, mut visibility) in &mut queries.p3() {
        *visibility = if is_visible { Visibility::Visible } else { Visibility::Hidden };
        if is_visible {
            let kind = draft.offers[icon.index];
            mesh.0 = meshes.add(kind.base_shape().into_mesh(kind.base_size()));
            *mat = MeshMaterial2d(materials.add(kind.base_color()));
        }
    }

    for (barrel, mut sprite, mut visibility, mut transform) in &mut queries.p4() {
        let kind = draft.offers[barrel.index];
        let barrel_template = kind.definition().barrel;
        let slot_x = -150.0 + barrel.index as f32 * 150.0;
        let (show, dx) = match barrel_template {
            BarrelTemplate::None => (false, 0.0),
            BarrelTemplate::Single { .. } => (barrel.sub_index == 0, 0.0),
            BarrelTemplate::Double { spacing, .. } => {
                let dx = if barrel.sub_index == 0 { -spacing * 0.5 } else { spacing * 0.5 };
                (true, dx)
            }
        };
        *visibility = if is_visible && show { Visibility::Visible } else { Visibility::Hidden };
        if is_visible && show {
            sprite.color = kind.barrel_color();
            sprite.custom_size = Some(barrel_template.size());
            transform.translation = Vec3::new(slot_x + dx, 50.0 + barrel_template.offset(), 13.0);
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
    if !is_buildable_cell(grid_position, &path_tiles)
        || towers
            .iter()
            .any(|t| t.translation.truncate().distance(grid_position) < GRID_SIZE * 0.5)
    {
        return;
    }

    let TowerDraftPhase::Placing(tower_kind) = draft.phase else { return; };
    apply_tower_effects(tower_kind, &mut stats);

    let mut spawner = commands.spawn((
        Transform::from_translation(grid_position.extend(2.0)),
        Tower,
        tower_kind,
        tower_kind.damage_formula(),
        FireCooldown {
            base_cooldown: tower_kind.cooldown(),
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
        TemporaryAttackSpeed::default(),
        TemporaryDamageBonus::default(),
    ));

    for tag in tower_kind.definition().tags {
        tag.insert(&mut spawner);
    }

    if tower_kind.base_shape().is_rectangle() {
        spawner.insert(tower_kind.body_sprite(1.0));
    } else {
        spawner.insert((
            Mesh2d(meshes.add(tower_kind.base_shape().into_mesh(tower_kind.base_size()))),
            MeshMaterial2d(materials.add(tower_kind.base_color())),
        ));
    }

    match tower_kind.definition().barrel {
        BarrelTemplate::None => {}
        BarrelTemplate::Single { .. } => {
            spawner.with_child((
                tower_kind.barrel_sprite(1.0),
                Transform::from_translation(Vec3::new(0.0, tower_kind.barrel_offset(), 1.0)),
            ));
        }
        BarrelTemplate::Double { spacing, .. } => {
            let offset = tower_kind.barrel_offset();
            spawner.with_children(|parent| {
                for dx in [-spacing * 0.5, spacing * 0.5] {
                    parent.spawn((
                        tower_kind.barrel_sprite(1.0),
                        Transform::from_translation(Vec3::new(dx, offset, 1.0)),
                    ));
                }
            });
        }
    }

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut phantom: Query<
        (&mut Mesh2d, &mut MeshMaterial2d<ColorMaterial>, &mut Transform, &mut Visibility),
        (With<TowerPhantom>, Without<Tower>, Without<TowerPhantomBarrel>),
    >,
    mut barrel: Query<
        (&TowerPhantomBarrel, &mut Transform, &mut Sprite, &mut Visibility),
        (Without<Tower>, Without<TowerPhantom>),
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
    let Ok((mut p_mesh, mut p_mat, mut p_transform, mut p_visibility)) = phantom.single_mut() else {
        return;
    };
    let Ok((mut i_transform, mut i_visibility)) = indicator.single_mut() else {
        return;
    };

    *p_visibility = Visibility::Hidden;
    for (_, _, _, mut b_visibility) in &mut barrel {
        *b_visibility = Visibility::Hidden;
    }

    let TowerDraftPhase::Placing(kind) = draft.phase else {
        return;
    };

    *i_visibility = Visibility::Hidden;

    let Ok(window) = windows.single() else { return; };
    let Ok((cam, cam_transform)) = camera.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else { return; };

    let grid_pos = snap_to_grid(world_pos);

    if !is_buildable_cell(grid_pos, &path_tiles)
        || towers
            .iter()
            .any(|t| t.translation.truncate().distance(grid_pos) < GRID_SIZE * 0.5)
    {
        return;
    }

    const ALPHA: f32 = 0.55;

    p_mesh.0 = meshes.add(kind.base_shape().into_mesh(kind.base_size()));
    *p_mat = MeshMaterial2d(materials.add(kind.base_color().with_alpha(ALPHA)));
    p_transform.translation = grid_pos.extend(3.0);
    *p_visibility = Visibility::Visible;

    let barrel_template = kind.definition().barrel;
    for (phantom_barrel, mut b_transform, mut b_sprite, mut b_visibility) in &mut barrel {
        let (show, dx) = match barrel_template {
            BarrelTemplate::None => (false, 0.0),
            BarrelTemplate::Single { .. } => (phantom_barrel.sub_index == 0, 0.0),
            BarrelTemplate::Double { spacing, .. } => {
                let dx = if phantom_barrel.sub_index == 0 { -spacing * 0.5 } else { spacing * 0.5 };
                (true, dx)
            }
        };
        if show {
            *b_sprite = kind.barrel_sprite(ALPHA);
            b_transform.translation = Vec3::new(
                grid_pos.x + dx,
                grid_pos.y + barrel_template.offset(),
                4.0,
            );
            *b_visibility = Visibility::Visible;
        }
    }

    i_transform.translation = grid_pos.extend(1.5);
    i_transform.scale = Vec3::splat(kind.range());
    *i_visibility = Visibility::Visible;
}

pub fn sync_draft_previews(
    draft: Res<TowerDraft>,
    mut commands: Commands,
    slots: Query<(Entity, &DraftSlot)>,
    previews: Query<Entity, With<DraftPreview>>,
    mut last_offers: Local<Vec<TowerKind>>,
) {
    if *last_offers == draft.offers {
        return;
    }

    for entity in &previews {
        commands.entity(entity).despawn();
    }

    for (slot_entity, slot) in &slots {
        let Some(&kind) = draft.offers.get(slot.index) else { continue; };
        commands.entity(slot_entity).with_children(|parent| {
            parent.spawn((kind, DraftPreview, Visibility::Hidden));
        });
    }

    *last_offers = draft.offers.clone();
}
