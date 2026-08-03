use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use noise::{NoiseFn, Perlin};

use crate::constants::HEX_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainKind {
    Plains,
    Mountain,
    WaterBody,
    Volcano,
}

impl TerrainKind {
    pub fn color(self) -> Color {
        match self {
            Self::Plains => Color::srgb(0.34, 0.46, 0.24),
            Self::Mountain => Color::srgb(0.44, 0.42, 0.40),
            Self::WaterBody => Color::srgb(0.16, 0.34, 0.50),
            Self::Volcano => Color::srgb(0.46, 0.16, 0.10),
        }
    }
}

const TERRAIN_SEED: u32 = 1337;
const NOISE_FREQUENCY: f64 = 1.0 / 200.0;

const MOUNTAIN_THRESHOLD: f64 = 0.20;

const VOLCANO_MIN_HEIGHT: f64 = 0.55;
const VOLCANO_SPAWN_CHANCE: f64 = 0.05;

const WATER_SEED_MIN_HEIGHT: f64 = 0.55;
const WATER_SEED_CHANCE: f64 = 0.03;
const MAX_RIVER_LENGTH: u32 = 400;
const RIVER_UPHILL_TOLERANCE: f64 = 0.3;

const AXIAL_NEIGHBOR_OFFSETS: [(i32, i32); 6] =
    [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

fn base_terrain_kind(height: f64) -> TerrainKind {
    if height > MOUNTAIN_THRESHOLD {
        TerrainKind::Mountain
    } else {
        TerrainKind::Plains
    }
}

pub fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    cells: &[(i32, i32, Vec2)],
) {
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

    let classified: Vec<(Vec2, TerrainKind)> = cells
        .iter()
        .map(|&(q, r, pos)| (pos, kinds[&(q, r)]))
        .collect();

    for kind in [
        TerrainKind::Plains,
        TerrainKind::Mountain,
        TerrainKind::WaterBody,
        TerrainKind::Volcano,
    ] {
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

fn build_hex_fill_mesh(centers: &[Vec2], radius: f32) -> Mesh {
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
