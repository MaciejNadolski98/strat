use bevy::math::primitives::Circle;
use bevy::prelude::*;

use crate::components::{
    DraftHeaderText, DraftPanel, DraftSlot, DraftSlotBarrel, DraftSlotIcon, DraftSlotLabel,
    HudText, MainCamera, PathEndMarker, ShopSlot, ShopSlotBarrel, ShopSlotIcon, ShopSlotLabel,
    ShopText, ShopTooltip, SpellSlot, SpellSlotIcon, SpellSlotLabel, TowerPhantom,
    TowerPhantomBarrel, TowerRangeIndicator,
};
use crate::constants::{GRID_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::pathing::{snap_axis, spawn_path_visuals};
use crate::resources::PathTiles;

pub fn setup(
    mut commands: Commands,
    path_tiles: Res<PathTiles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = commands.spawn((Camera2d, MainCamera)).id();

    let background = commands.spawn((
        Sprite::from_color(
            Color::srgb(0.10, 0.15, 0.13),
            Vec2::new(WINDOW_WIDTH * 4.0, WINDOW_HEIGHT * 4.0),
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
    )).id();

    spawn_grid(&mut commands);

    spawn_path_visuals(&mut commands, &path_tiles, &[]);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.13, 0.12), Vec2::new(52.0, 52.0)),
        Transform::from_translation(path_tiles.start().extend(0.0)),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.35, 0.36), Vec2::new(58.0, 58.0)),
        Transform::from_translation(path_tiles.end().extend(0.0)),
        PathEndMarker,
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

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.94, 0.88)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            bottom: Val::Px(72.0),
            ..default()
        },
        ShopText,
    ));

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.94, 0.94, 0.86)),
        TextShadow::default(),
        BackgroundColor(Color::srgba(0.03, 0.04, 0.04, 0.92)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            bottom: Val::Px(152.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        Visibility::Hidden,
        ShopTooltip,
    ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(Color::srgba(0.85, 0.90, 0.88, 0.10))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.5)),
        Visibility::Hidden,
        TowerRangeIndicator,
    ));

    commands.spawn((
        Mesh2d(meshes.add(bevy::math::primitives::Rectangle::new(40.0, 40.0))),
        MeshMaterial2d(materials.add(Color::srgba(0.0, 0.0, 0.0, 0.0))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        Visibility::Hidden,
        TowerPhantom,
    ));

    for sub_index in 0..2 {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.0), Vec2::new(9.0, 32.0)),
            Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
            Visibility::Hidden,
            TowerPhantomBarrel { sub_index },
        ));
    }

    let shop_ui = spawn_shop_slots(&mut commands);
    let spell_ui = spawn_spell_slots(&mut commands);
    let draft_ui = spawn_draft_ui(&mut commands, &mut meshes, &mut materials);

    commands.entity(camera).add_child(background);
    for e in shop_ui.into_iter().chain(spell_ui).chain(draft_ui) {
        commands.entity(camera).add_child(e);
    }
}

fn spawn_grid(commands: &mut Commands) {
    let grid_color = Color::srgba(0.68, 0.76, 0.70, 0.12);
    let extent_x = WINDOW_WIDTH * 5.0;
    let extent_y = WINDOW_HEIGHT * 5.0;

    let mut x = snap_axis(-extent_x);
    while x <= extent_x {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(1.0, extent_y * 2.0)),
            Transform::from_translation(Vec3::new(x, 0.0, -8.0)),
        ));
        x += GRID_SIZE;
    }

    let mut y = snap_axis(-extent_y);
    while y <= extent_y {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(extent_x * 2.0, 1.0)),
            Transform::from_translation(Vec3::new(0.0, y, -8.0)),
        ));
        y += GRID_SIZE;
    }
}

fn spawn_shop_slots(commands: &mut Commands) -> Vec<Entity> {
    let y = -WINDOW_HEIGHT * 0.5 + 44.0;
    let start_x = -170.0;
    let spacing = 116.0;
    let mut entities = Vec::new();

    for index in 0..3 {
        let x = start_x + index as f32 * spacing;
        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.15, 0.17, 0.16), Vec2::new(96.0, 72.0)),
            Transform::from_translation(Vec3::new(x, y, 6.0)),
            ShopSlot { index },
        )).id());

        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.20, 0.22, 0.21), Vec2::new(36.0, 36.0)),
            Transform::from_translation(Vec3::new(x, y + 8.0, 7.0)),
            Visibility::Hidden,
            ShopSlotIcon { index },
        )).id());

        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.72, 0.78, 0.76), Vec2::new(9.0, 32.0)),
            Transform::from_translation(Vec3::new(x, y + 22.0, 8.0)),
            Visibility::Hidden,
            ShopSlotBarrel,
        )).id());

        entities.push(commands.spawn((
            Text2d::new(""),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.88, 0.80)),
            TextShadow::default(),
            Transform::from_translation(Vec3::new(x, y - 28.0, 9.0)),
            ShopSlotLabel { index },
        )).id());
    }

    entities
}

fn spawn_spell_slots(commands: &mut Commands) -> Vec<Entity> {
    let x = WINDOW_WIDTH * 0.5 - 58.0;
    let start_y = 84.0;
    let spacing = 92.0;
    let mut entities = Vec::new();

    for index in 0..3 {
        let y = start_y - index as f32 * spacing;
        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.12, 0.13, 0.14), Vec2::new(92.0, 76.0)),
            Transform::from_translation(Vec3::new(x, y, 6.0)),
            SpellSlot { index },
        )).id());

        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.20, 0.22, 0.24), Vec2::new(38.0, 38.0)),
            Transform::from_translation(Vec3::new(x, y + 10.0, 7.0)),
            Visibility::Hidden,
            SpellSlotIcon { index },
        )).id());

        entities.push(commands.spawn((
            Text2d::new(""),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.88, 0.80)),
            TextShadow::default(),
            Transform::from_translation(Vec3::new(x, y - 28.0, 9.0)),
            SpellSlotLabel { index },
        )).id());
    }

    entities
}

fn spawn_draft_ui(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Vec<Entity> {
    let y = 30.0;
    let start_x = -150.0;
    let spacing = 150.0;
    let mut entities = Vec::new();

    entities.push(commands.spawn((
        Sprite::from_color(Color::srgba(0.05, 0.07, 0.09, 0.96), Vec2::new(490.0, 210.0)),
        Transform::from_translation(Vec3::new(0.0, y, 10.0)),
        Visibility::Hidden,
        DraftPanel,
    )).id());

    entities.push(commands.spawn((
        Text2d::new(""),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.92, 0.84)),
        TextShadow::default(),
        Transform::from_translation(Vec3::new(0.0, y + 98.0, 14.0)),
        Visibility::Hidden,
        DraftHeaderText,
    )).id());

    for index in 0..3 {
        let x = start_x + index as f32 * spacing;

        entities.push(commands.spawn((
            Sprite::from_color(Color::srgb(0.15, 0.17, 0.16), Vec2::new(130.0, 140.0)),
            Transform::from_translation(Vec3::new(x, y, 11.0)),
            Visibility::Hidden,
            DraftSlot { index },
        )).id());

        entities.push(commands.spawn((
            Mesh2d(meshes.add(bevy::math::primitives::Rectangle::new(36.0, 36.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
            Transform::from_translation(Vec3::new(x, y + 20.0, 12.0)),
            Visibility::Hidden,
            DraftSlotIcon { index },
        )).id());

        for sub_index in 0..2 {
            entities.push(commands.spawn((
                Sprite::from_color(Color::srgb(0.72, 0.78, 0.76), Vec2::new(9.0, 32.0)),
                Transform::from_translation(Vec3::new(x, y + 36.0, 13.0)),
                Visibility::Hidden,
                DraftSlotBarrel { index, sub_index },
            )).id());
        }

        entities.push(commands.spawn((
            Text2d::new(""),
            TextFont {
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.88, 0.80)),
            TextShadow::default(),
            Transform::from_translation(Vec3::new(x, y - 56.0, 14.0)),
            Visibility::Hidden,
            DraftSlotLabel { index },
        )).id());
    }

    entities
}
