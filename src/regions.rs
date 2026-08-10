use std::collections::HashMap;

use bevy::prelude::*;

use crate::components::MainCamera;
use crate::pathing::world_to_axial_cell;

pub struct RegionDefinition {
    pub name: String,
    pub tiles: Vec<(i32, i32)>,
}

#[derive(Resource, Default)]
pub struct RegionMap {
    pub regions: Vec<Region>,
    pub tile_to_region: HashMap<(i32, i32), usize>,
}

pub struct Region {
    pub name: String,
}

impl RegionMap {
    pub fn from_definitions(definitions: Vec<RegionDefinition>) -> Self {
        let mut tile_to_region = HashMap::new();
        let mut regions = Vec::new();
        for def in definitions {
            let index = regions.len();
            regions.push(Region {
                name: def.name,
            });
            for tile in def.tiles {
                tile_to_region.insert(tile, index);
            }
        }
        Self {
            regions,
            tile_to_region,
        }
    }

    pub fn region_at(&self, coord: (i32, i32)) -> Option<&Region> {
        self.tile_to_region.get(&coord).map(|&i| &self.regions[i])
    }
}

#[derive(Resource, Default)]
pub struct CurrentRegion {
    pub name: Option<String>,
}

#[derive(Component)]
pub struct RegionTitle {
    pub timer: Timer,
    pub phase: RegionTitlePhase,
}

pub enum RegionTitlePhase {
    FadeIn,
    Hold,
    FadeOut,
}

const FADE_IN_DURATION: f32 = 0.4;
const HOLD_DURATION: f32 = 1.8;
const FADE_OUT_DURATION: f32 = 0.6;

pub fn track_camera_region(
    mut commands: Commands,
    camera: Query<&Transform, With<MainCamera>>,
    region_map: Res<RegionMap>,
    mut current: ResMut<CurrentRegion>,
    existing_titles: Query<Entity, With<RegionTitle>>,
) {
    let Ok(camera_transform) = camera.single() else {
        return;
    };
    let cam_pos = camera_transform.translation.truncate();
    let coord = world_to_axial_cell(cam_pos);

    let new_name = region_map.region_at(coord).map(|r| r.name.clone());

    if new_name == current.name {
        return;
    }
    current.name = new_name.clone();

    for entity in &existing_titles {
        commands.entity(entity).despawn();
    }

    let Some(name) = new_name else {
        return;
    };

    commands.spawn((
        Text::new(name),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgba(0.92, 0.90, 0.78, 0.0)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            width: Val::Percent(100.0),
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        RegionTitle {
            timer: Timer::from_seconds(FADE_IN_DURATION, TimerMode::Once),
            phase: RegionTitlePhase::FadeIn,
        },
    ));
}

pub fn update_region_titles(
    mut commands: Commands,
    time: Res<Time>,
    mut titles: Query<(Entity, &mut RegionTitle, &mut TextColor)>,
) {
    for (entity, mut title, mut color) in &mut titles {
        title.timer.tick(time.delta());

        match title.phase {
            RegionTitlePhase::FadeIn => {
                let alpha = title.timer.fraction();
                color.0 = Color::srgba(0.92, 0.90, 0.78, alpha);
                if title.timer.finished() {
                    title.phase = RegionTitlePhase::Hold;
                    title.timer = Timer::from_seconds(HOLD_DURATION, TimerMode::Once);
                }
            }
            RegionTitlePhase::Hold => {
                color.0 = Color::srgba(0.92, 0.90, 0.78, 1.0);
                if title.timer.finished() {
                    title.phase = RegionTitlePhase::FadeOut;
                    title.timer = Timer::from_seconds(FADE_OUT_DURATION, TimerMode::Once);
                }
            }
            RegionTitlePhase::FadeOut => {
                let alpha = 1.0 - title.timer.fraction();
                color.0 = Color::srgba(0.92, 0.90, 0.78, alpha);
                if title.timer.finished() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
