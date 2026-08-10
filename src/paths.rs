use std::collections::HashSet;

use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;

use crate::components::EnemyKind;
use crate::constants::{HEX_SIZE, HEX_SPACING};
use crate::pathing::axial_to_world;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PathId(pub usize);

#[derive(Clone)]
pub struct PathEnemyGroup {
    pub kind: EnemyKind,
    pub count: u32,
    pub cooldown: f32,
}

#[derive(Clone)]
pub struct PathDefinition {
    pub id: PathId,
    pub name: String,
    pub tiles: Vec<(i32, i32)>,
    pub enemies: Vec<PathEnemyGroup>,
    pub level: u8,
    pub unlocks: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Component)]
pub struct PathVisual;

#[derive(Resource, Default)]
pub struct PathMap {
    pub paths: Vec<PathDefinition>,
    pub placed: HashSet<PathId>,
}

impl PathMap {
    pub fn from_definitions(definitions: Vec<PathDefinition>) -> Self {
        Self {
            paths: definitions,
            placed: HashSet::new(),
        }
    }

    pub fn path_world_tiles(&self, id: PathId) -> Vec<Vec2> {
        let path = self.paths.iter().find(|p| p.id == id).expect("unknown PathId");
        path.tiles
            .iter()
            .map(|&(q, r)| axial_to_world(q as f32, r as f32))
            .collect()
    }

    pub fn is_placed(&self, id: PathId) -> bool {
        self.placed.contains(&id)
    }

    pub fn contains_world(&self, position: Vec2) -> bool {
        for path in &self.paths {
            if !self.placed.contains(&path.id) {
                continue;
            }
            for &(q, r) in &path.tiles {
                let tile_pos = axial_to_world(q as f32, r as f32);
                if tile_pos.distance_squared(position) < 1.0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_placeable(&self, id: PathId) -> bool {
        if self.placed.contains(&id) {
            return false;
        }

        let path = self.paths.iter().find(|p| p.id == id).expect("unknown PathId");
        for exclude_name in &path.excludes {
            if self.paths.iter().any(|p| p.name == *exclude_name && self.placed.contains(&p.id)) {
                return false;
            }
        }

        if path.level == 0 {
            return true;
        }

        for other in &self.paths {
            if self.placed.contains(&other.id) && other.unlocks.iter().any(|u| *u == path.name) {
                return true;
            }
        }

        false
    }

    pub fn total_enemy_count(&self) -> u32 {
        let mut total = 0;
        for path in &self.paths {
            if self.placed.contains(&path.id) {
                for group in &path.enemies {
                    total += group.count;
                }
            }
        }
        total
    }

    pub fn reset_placed(&mut self) {
        self.placed.clear();
        for path in &self.paths {
            if path.level == 0 {
                self.placed.insert(path.id);
            }
        }
    }
}

const FILL_COLOR: Color = Color::srgb(0.43, 0.39, 0.31);
const EDGE_COLOR: Color = Color::srgb(0.24, 0.21, 0.16);
const UNPLACED_LINE_COLOR: Color = Color::srgb(0.43, 0.39, 0.31);
const LINE_WIDTH: f32 = 4.0;
const EDGE_SIZE: Vec2 = Vec2::new(4.0, HEX_SIZE + 4.0);
const HEX_EDGE_ANGLES_DEG: [f32; 6] = [0.0, 60.0, 120.0, 180.0, 240.0, 300.0];

pub fn spawn_all_path_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    path_map: &PathMap,
) {
    for path in &path_map.paths {
        let world_tiles = path_map.path_world_tiles(path.id);
        if path_map.is_placed(path.id) {
            spawn_path_filled(commands, meshes, materials, &world_tiles, FILL_COLOR, EDGE_COLOR, -2.0);
        } else {
            spawn_path_line(commands, meshes, materials, &world_tiles, UNPLACED_LINE_COLOR, -1.5);
        }
    }
}

pub fn spawn_path_filled(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    tiles: &[Vec2],
    fill_color: Color,
    edge_color: Color,
    z: f32,
) {
    let apothem = HEX_SIZE * 0.8660254;

    for (index, &position) in tiles.iter().enumerate() {
        commands.spawn((
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
            MeshMaterial2d(materials.add(fill_color)),
            Transform::from_translation(position.extend(z)),
            PathVisual,
        ));

        for angle_deg in HEX_EDGE_ANGLES_DEG {
            let angle = angle_deg.to_radians();
            let direction = Vec2::from_angle(angle);
            let neighbor_pos = position + direction * HEX_SPACING;

            let is_path_neighbor = index.checked_sub(1)
                .and_then(|i| tiles.get(i))
                .into_iter()
                .chain(tiles.get(index + 1))
                .any(|t| t.distance(neighbor_pos) < 1.0);

            if is_path_neighbor {
                continue;
            }

            commands.spawn((
                Sprite::from_color(edge_color, EDGE_SIZE),
                Transform::from_translation((position + direction * apothem).extend(z + 1.0))
                    .with_rotation(Quat::from_rotation_z(angle)),
                PathVisual,
            ));
        }
    }
}

pub fn spawn_path_line(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    tiles: &[Vec2],
    color: Color,
    z: f32,
) {
    for window in tiles.windows(2) {
        let from = window[0];
        let to = window[1];
        let mid = (from + to) * 0.5;
        let diff = to - from;
        let length = diff.length();
        let angle = diff.y.atan2(diff.x);

        commands.spawn((
            Sprite::from_color(color, Vec2::new(length, LINE_WIDTH)),
            Transform::from_translation(mid.extend(z))
                .with_rotation(Quat::from_rotation_z(angle)),
            PathVisual,
        ));
    }

    if let Some(&last) = tiles.last() {
        commands.spawn((
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE * 0.4, 6))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(last.extend(z)),
            PathVisual,
        ));
    }
}

pub fn despawn_path_visuals(commands: &mut Commands, visuals: &Query<Entity, With<PathVisual>>) {
    for entity in visuals.iter() {
        commands.entity(entity).despawn();
    }
}
