use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::window::PrimaryWindow;
use noise::{NoiseFn, Perlin};

use crate::components::ShopTooltip;
use crate::constants::HEX_SIZE;
use crate::pathing::world_to_axial_cell;
use crate::regions::RegionMap;
use crate::terrain_file::load_terrain_file;
use crate::tooltip::{plain, set_tooltip_segments};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerrainKind {
    Plains,
    Mountain,
    WaterBody,
    Volcano,
    Forest,
    PineForest,
    AlpineForest,
    Desert,
}

pub const ALL_TERRAIN_KINDS: [TerrainKind; 8] = [
    TerrainKind::Plains,
    TerrainKind::Mountain,
    TerrainKind::WaterBody,
    TerrainKind::Volcano,
    TerrainKind::Forest,
    TerrainKind::PineForest,
    TerrainKind::AlpineForest,
    TerrainKind::Desert,
];

impl TerrainKind {
    pub fn color(self) -> Color {
        match self {
            Self::Plains => Color::srgb(0.34, 0.46, 0.24),
            Self::Mountain => Color::srgb(0.44, 0.42, 0.40),
            Self::WaterBody => Color::srgb(0.16, 0.34, 0.50),
            Self::Volcano => Color::srgb(0.46, 0.16, 0.10),
            Self::Forest => Color::srgb(0.14, 0.42, 0.17),
            Self::PineForest => Color::srgb(0.09, 0.30, 0.12),
            Self::AlpineForest => Color::srgb(0.05, 0.20, 0.08),
            Self::Desert => Color::srgb(0.80, 0.68, 0.40),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Plains => "Plains",
            Self::Mountain => "Mountain",
            Self::WaterBody => "Water",
            Self::Volcano => "Volcano",
            Self::Forest => "Forest",
            Self::PineForest => "Pine Forest",
            Self::AlpineForest => "Alpine Forest",
            Self::Desert => "Desert",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Plains => "Plains",
            Self::Mountain => "Mountain",
            Self::WaterBody => "WaterBody",
            Self::Volcano => "Volcano",
            Self::Forest => "Forest",
            Self::PineForest => "PineForest",
            Self::AlpineForest => "AlpineForest",
            Self::Desert => "Desert",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            "Plains" => Self::Plains,
            "Mountain" => Self::Mountain,
            "WaterBody" => Self::WaterBody,
            "Volcano" => Self::Volcano,
            "Forest" => Self::Forest,
            "PineForest" => Self::PineForest,
            "AlpineForest" => Self::AlpineForest,
            "Desert" => Self::Desert,
            _ => return None,
        })
    }
}

pub const DEFAULT_TERRAIN_KIND: TerrainKind = TerrainKind::Plains;

#[derive(Component)]
pub struct TerrainTile {
    pub kind: TerrainKind,
}

#[derive(Resource, Default)]
pub struct TerrainTileIndex(pub HashMap<(i32, i32), Entity>);

const TERRAIN_SEED: u32 = 1337;
const NOISE_FREQUENCY: f64 = 1.0 / 200.0;

const MOUNTAIN_THRESHOLD: f64 = 0.20;

const VOLCANO_MIN_HEIGHT: f64 = 0.55;
const VOLCANO_SPAWN_CHANCE: f64 = 0.05;

const WATER_SEED_MIN_HEIGHT: f64 = 0.55;
const WATER_SEED_CHANCE: f64 = 0.03;
const MAX_RIVER_LENGTH: u32 = 400;
const RIVER_UPHILL_TOLERANCE: f64 = 0.3;

const VEGETATION_SEED: u32 = 2027;
const VEGETATION_NOISE_FREQUENCY: f64 = 1.0 / 130.0;
const VEGETATION_THRESHOLD: f64 = 0.10;
const DESERT_MAX_COVERAGE: f64 = -0.35;

const FOREST_MAX_HEIGHT: f64 = 0.05;
const PINE_FOREST_MAX_HEIGHT: f64 = 0.40;

const AXIAL_NEIGHBOR_OFFSETS: [(i32, i32); 6] =
    [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

fn base_terrain_kind(height: f64) -> TerrainKind {
    if height > MOUNTAIN_THRESHOLD {
        TerrainKind::Mountain
    } else {
        TerrainKind::Plains
    }
}

fn vegetation_kind_for_height(height: f64) -> TerrainKind {
    if height < FOREST_MAX_HEIGHT {
        TerrainKind::Forest
    } else if height < PINE_FOREST_MAX_HEIGHT {
        TerrainKind::PineForest
    } else {
        TerrainKind::AlpineForest
    }
}

pub fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    tile_index: &mut TerrainTileIndex,
    region_map: &mut RegionMap,
    cells: &[(i32, i32, Vec2)],
) {
    let data = load_terrain_file();
    let kinds: HashMap<(i32, i32), TerrainKind> = cells
        .iter()
        .map(|&(q, r, _)| {
            let kind = data.overrides.get(&(q, r)).copied().unwrap_or(DEFAULT_TERRAIN_KIND);
            ((q, r), kind)
        })
        .collect();

    *region_map = RegionMap::from_definitions(data.regions);
    spawn_terrain_kinds(commands, meshes, materials, tile_index, cells, &kinds);
}

pub fn spawn_terrain_kinds(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    tile_index: &mut TerrainTileIndex,
    cells: &[(i32, i32, Vec2)],
    kinds: &HashMap<(i32, i32), TerrainKind>,
) {
    let classified: Vec<(Vec2, TerrainKind)> = cells
        .iter()
        .map(|&(q, r, pos)| (pos, kinds[&(q, r)]))
        .collect();

    for &(q, r, _) in cells {
        let coord = (q, r);
        let entity = commands.spawn(TerrainTile { kind: kinds[&coord] }).id();
        tile_index.0.insert(coord, entity);
    }

    for kind in ALL_TERRAIN_KINDS {
        let kind_centers: Vec<Vec2> = classified
            .iter()
            .filter(|(_, k)| *k == kind)
            .map(|(pos, _)| *pos)
            .collect();
        if kind_centers.is_empty() {
            continue;
        }

        commands.spawn((
            Mesh2d(meshes.add(build_hex_fill_mesh(&kind_centers, HEX_SIZE))),
            MeshMaterial2d(materials.add(kind.color())),
            Transform::from_translation(Vec3::new(0.0, 0.0, -9.0)),
        ));
    }
}

pub fn generate_random_kinds(cells: &[(i32, i32, Vec2)]) -> HashMap<(i32, i32), TerrainKind> {
    let perlin = Perlin::new(TERRAIN_SEED);
    let heights: HashMap<(i32, i32), f64> = cells
        .iter()
        .map(|&(q, r, pos)| {
            let height = perlin.get([
                pos.x as f64 * NOISE_FREQUENCY,
                pos.y as f64 * NOISE_FREQUENCY,
            ]);
            ((q, r), height)
        })
        .collect();

    let mut kinds: HashMap<(i32, i32), TerrainKind> = heights
        .iter()
        .map(|(&coord, &height)| (coord, base_terrain_kind(height)))
        .collect();

    for (&coord, &height) in &heights {
        if height <= VOLCANO_MIN_HEIGHT || tile_roll(coord, Roll::VolcanoSpawn) >= VOLCANO_SPAWN_CHANCE {
            continue;
        }
        kinds.insert(coord, TerrainKind::Volcano);
    }

    for (&coord, &height) in &heights {
        if height <= WATER_SEED_MIN_HEIGHT || tile_roll(coord, Roll::WaterSeed) >= WATER_SEED_CHANCE {
            continue;
        }
        carve_river(coord, &heights, &mut kinds);
    }

    let vegetation_perlin = Perlin::new(VEGETATION_SEED);
    for &(q, r, pos) in cells {
        let coord = (q, r);
        let current_kind = kinds[&coord];
        if !matches!(current_kind, TerrainKind::Plains | TerrainKind::Mountain) {
            continue;
        }
        let coverage = vegetation_perlin.get([
            pos.x as f64 * VEGETATION_NOISE_FREQUENCY,
            pos.y as f64 * VEGETATION_NOISE_FREQUENCY,
        ]);
        if coverage > VEGETATION_THRESHOLD {
            kinds.insert(coord, vegetation_kind_for_height(heights[&coord]));
        } else if coverage < DESERT_MAX_COVERAGE && current_kind == TerrainKind::Plains {
            kinds.insert(coord, TerrainKind::Desert);
        }
    }

    kinds
}

#[derive(Clone, Copy, Hash)]
enum Roll {
    VolcanoSpawn,
    WaterSeed,
}

fn tile_roll(coord: (i32, i32), kind: Roll) -> f64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (TERRAIN_SEED, kind, coord).hash(&mut hasher);
    (hasher.finish() >> 11) as f64 / (1u64 << 53) as f64
}

fn carve_river(
    start: (i32, i32),
    heights: &HashMap<(i32, i32), f64>,
    kinds: &mut HashMap<(i32, i32), TerrainKind>,
) {
    let mut current = start;
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    visited.insert(current);

    for _ in 0..MAX_RIVER_LENGTH {
        kinds.insert(current, TerrainKind::WaterBody);
        let current_height = heights[&current];

        let lowest_neighbor = AXIAL_NEIGHBOR_OFFSETS
            .iter()
            .map(|&(dq, dr)| (current.0 + dq, current.1 + dr))
            .filter(|coord| !visited.contains(coord))
            .filter_map(|coord| heights.get(&coord).map(|&height| (coord, height)))
            .min_by(|(_, a), (_, b)| a.total_cmp(b));

        match lowest_neighbor {
            Some((coord, height)) if height < current_height + RIVER_UPHILL_TOLERANCE => {
                current = coord;
                visited.insert(current);
            }
            _ => break,
        }
    }
}

pub fn build_hex_fill_mesh(centers: &[Vec2], radius: f32) -> Mesh {
    let mut positions = Vec::with_capacity(centers.len() * 6);
    let mut indices = Vec::with_capacity(centers.len() * 12);

    for &center in centers {
        let base = positions.len() as u32;
        for k in 0..6 {
            let angle = (90.0 + 60.0 * k as f32).to_radians();
            let vertex = center + Vec2::from_angle(angle) * radius;
            positions.push([vertex.x, vertex.y, 0.0]);
        }
        for k in 1..5u32 {
            indices.extend_from_slice(&[base, base + k, base + k + 1]);
        }
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn update_terrain_tooltip(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    tile_index: Res<TerrainTileIndex>,
    tiles: Query<&TerrainTile>,
    mut tooltip: Query<(Entity, &mut Text, &mut Visibility), With<ShopTooltip>>,
    mut commands: Commands,
) {
    let Ok((tooltip_entity, mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };
    if *tooltip_visibility == Visibility::Visible {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let coord = world_to_axial_cell(world_position);
    let Some(&tile_entity) = tile_index.0.get(&coord) else {
        return;
    };
    let Ok(tile) = tiles.get(tile_entity) else {
        return;
    };

    set_tooltip_segments(&mut commands, tooltip_entity, &mut tooltip_text, vec![plain(tile.kind.name())]);
    *tooltip_visibility = Visibility::Visible;
}
