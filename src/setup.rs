use bevy::prelude::*;

use crate::components::HudText;
use crate::constants::{GRID_SIZE, PATH, PATH_HALF_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::pathing::snap_axis;

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.10, 0.15, 0.13),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
    ));

    spawn_grid(&mut commands);

    for segment in PATH.windows(2) {
        spawn_path_segment(&mut commands, segment[0], segment[1]);
    }

    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.13, 0.12), Vec2::new(52.0, 52.0)),
        Transform::from_translation(PATH[0].extend(0.0)),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.35, 0.36), Vec2::new(58.0, 58.0)),
        Transform::from_translation(PATH[PATH.len() - 1].extend(0.0)),
    ));

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.94, 0.88)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
        HudText,
    ));
}

fn spawn_grid(commands: &mut Commands) {
    let grid_color = Color::srgba(0.68, 0.76, 0.70, 0.12);
    let min_x = -WINDOW_WIDTH * 0.5;
    let max_x = WINDOW_WIDTH * 0.5;
    let min_y = -WINDOW_HEIGHT * 0.5;
    let max_y = WINDOW_HEIGHT * 0.5;

    let mut x = snap_axis(min_x);
    while x <= max_x {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(1.0, WINDOW_HEIGHT)),
            Transform::from_translation(Vec3::new(x, 0.0, -8.0)),
        ));
        x += GRID_SIZE;
    }

    let mut y = snap_axis(min_y);
    while y <= max_y {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(WINDOW_WIDTH, 1.0)),
            Transform::from_translation(Vec3::new(0.0, y, -8.0)),
        ));
        y += GRID_SIZE;
    }
}

fn spawn_path_segment(commands: &mut Commands, start: Vec2, end: Vec2) {
    let delta = end - start;
    let midpoint = (start + end) * 0.5;
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.43, 0.39, 0.31),
            Vec2::new(
                delta.length() + PATH_HALF_WIDTH * 2.0,
                PATH_HALF_WIDTH * 2.0,
            ),
        ),
        Transform {
            translation: midpoint.extend(-2.0),
            rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
            ..default()
        },
    ));
}
