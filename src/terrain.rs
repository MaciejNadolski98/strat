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

const NOISE_SEED: u32 = 1337;
const NOISE_FREQUENCY: f64 = 1.0 / 260.0;

const WATER_THRESHOLD: f64 = -0.30;
const MOUNTAIN_THRESHOLD: f64 = 0.20;
const VOLCANO_THRESHOLD: f64 = 0.75;

fn terrain_at(perlin: &Perlin, position: Vec2) -> TerrainKind {
    let value = perlin.get([
        position.x as f64 * NOISE_FREQUENCY,
        position.y as f64 * NOISE_FREQUENCY,
    ]);

    if value < WATER_THRESHOLD {
        TerrainKind::WaterBody
    } else if value < MOUNTAIN_THRESHOLD {
        TerrainKind::Plains
    } else if value < VOLCANO_THRESHOLD {
        TerrainKind::Mountain
    } else {
        TerrainKind::Volcano
    }
}

pub fn spawn_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    centers: &[Vec2],
) {
    let perlin = Perlin::new(NOISE_SEED);
    let classified: Vec<(Vec2, TerrainKind)> = centers
        .iter()
        .map(|&center| (center, terrain_at(&perlin, center)))
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
