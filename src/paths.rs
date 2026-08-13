use std::collections::{HashMap, HashSet};

use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;

use crate::components::EnemyKind;
use crate::constants::{HEX_SIZE, PATH_EDGE_COLOR, PATH_FILLED_Z, PATH_FILL_COLOR, PATH_LINE_COLOR, PATH_LINE_Z};
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

        let exclusions = compute_exclusions(&self.paths);
        if exclusions.iter().any(|&(a, b)| {
            (a == id && self.placed.contains(&b)) || (b == id && self.placed.contains(&a))
        }) {
            return false;
        }

        if id == PathId(0) {
            return true;
        }

        let parents = compute_parents(&self.paths);
        parents
            .get(&id)
            .map_or(false, |ps| ps.iter().any(|p| self.placed.contains(p)))
    }

    pub fn placeable_paths(&self) -> Vec<PathId> {
        self.paths.iter()
            .filter(|p| self.is_placeable(p.id))
            .map(|p| p.id)
            .collect()
    }

    pub fn total_enemy_count(&self) -> u32 {
        let terminals = self.terminal_paths();
        let mut total = 0;
        for path in &self.paths {
            if terminals.contains(&path.id) {
                for group in &path.enemies {
                    total += group.count;
                }
            }
        }
        total
    }

    pub fn terminal_paths(&self) -> Vec<PathId> {
        let mut child_starts: HashSet<(i32, i32)> = HashSet::new();
        for path in &self.paths {
            if self.placed.contains(&path.id) {
                if let Some(&start) = path.tiles.first() {
                    child_starts.insert(start);
                }
            }
        }

        self.paths.iter()
            .filter(|p| {
                if !self.placed.contains(&p.id) {
                    return false;
                }
                let Some(&end) = p.tiles.last() else { return false };
                !self.paths.iter().any(|other| {
                    other.id != p.id
                        && self.placed.contains(&other.id)
                        && other.tiles.first() == Some(&end)
                })
            })
            .map(|p| p.id)
            .collect()
    }

    pub fn full_route_tiles(&self, terminal_id: PathId) -> Vec<Vec2> {
        let parents = compute_parents(&self.paths);
        let mut chain = Vec::new();
        let mut current = terminal_id;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(current) {
                break;
            }
            chain.push(current);
            match parents.get(&current).and_then(|ps| ps.iter().find(|p| self.placed.contains(p))) {
                Some(&parent) => current = parent,
                None => break,
            }
        }
        chain.reverse();

        let mut tiles = Vec::new();
        for (i, &path_id) in chain.iter().enumerate() {
            let path = self.paths.iter().find(|p| p.id == path_id).unwrap();
            let world: Vec<Vec2> = path.tiles.iter()
                .map(|&(q, r)| axial_to_world(q as f32, r as f32))
                .collect();
            if i == 0 {
                tiles.extend(world);
            } else {
                tiles.extend(world.into_iter().skip(1));
            }
        }
        tiles
    }

    pub fn reset_placed(&mut self) {
        self.placed.clear();
        if self.paths.iter().any(|p| p.id == PathId(0)) {
            self.placed.insert(PathId(0));
        }
    }
}

// --- Validation ---

const HEX_NEIGHBOR_OFFSETS: [(i32, i32); 6] = [
    (1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1),
];

pub fn are_hex_adjacent(a: (i32, i32), b: (i32, i32)) -> bool {
    let dq = b.0 - a.0;
    let dr = b.1 - a.1;
    HEX_NEIGHBOR_OFFSETS.contains(&(dq, dr))
}

pub fn compute_parents(paths: &[PathDefinition]) -> HashMap<PathId, Vec<PathId>> {
    let mut parents: HashMap<PathId, Vec<PathId>> = HashMap::new();
    for child in paths {
        let Some(&child_start) = child.tiles.first() else { continue };
        for parent in paths {
            if parent.id == child.id {
                continue;
            }
            let Some(&parent_end) = parent.tiles.last() else { continue };
            if parent_end == child_start {
                parents.entry(child.id).or_default().push(parent.id);
            }
        }
    }
    parents
}

pub fn compute_exclusions(paths: &[PathDefinition]) -> Vec<(PathId, PathId)> {
    let mut result = Vec::new();
    for (i, a) in paths.iter().enumerate() {
        let a_tiles: HashSet<(i32, i32)> = a.tiles.iter().copied().collect();
        for b in &paths[i + 1..] {
            let mut junction = HashSet::new();
            if a.tiles.last() == b.tiles.first() {
                junction.insert(*a.tiles.last().unwrap());
            }
            if b.tiles.last() == a.tiles.first() {
                junction.insert(*b.tiles.last().unwrap());
            }
            if a.tiles.first() == b.tiles.first() {
                if let Some(&start) = a.tiles.first() {
                    junction.insert(start);
                }
            }
            let crosses = b.tiles.iter().any(|t| a_tiles.contains(t) && !junction.contains(t));
            if crosses {
                result.push((a.id, b.id));
            }
        }
    }
    result
}

pub enum PathError {
    EmptyPath(PathId),
    NonAdjacentTile { path_id: PathId, tile_index: usize },
    OrphanPath(PathId),
    CycleDetected,
}

impl PathError {
    pub fn message(&self, paths: &[PathDefinition]) -> String {
        let name = |id: &PathId| -> &str {
            paths.iter().find(|p| p.id == *id).map(|p| p.name.as_str()).unwrap_or("?")
        };
        match self {
            PathError::EmptyPath(id) => format!("'{}': no tiles", name(id)),
            PathError::NonAdjacentTile { path_id, tile_index } => {
                format!("'{}': tile {} not adjacent to tile {}", name(path_id), tile_index, tile_index - 1)
            }
            PathError::OrphanPath(id) => {
                format!("'{}': no parent (must start where another path ends)", name(id))
            }
            PathError::CycleDetected => "Unlock graph has a cycle".to_string(),
        }
    }
}

pub fn validate_paths(paths: &[PathDefinition]) -> Vec<PathError> {
    let mut errors = Vec::new();

    for path in paths {
        if path.tiles.is_empty() {
            errors.push(PathError::EmptyPath(path.id));
            continue;
        }
        for i in 1..path.tiles.len() {
            if !are_hex_adjacent(path.tiles[i - 1], path.tiles[i]) {
                errors.push(PathError::NonAdjacentTile { path_id: path.id, tile_index: i });
            }
        }
    }

    let parents = compute_parents(paths);
    for path in paths {
        if path.id == PathId(0) {
            continue;
        }
        if !parents.contains_key(&path.id) {
            errors.push(PathError::OrphanPath(path.id));
        }
    }

    if has_cycle(paths, &parents) {
        errors.push(PathError::CycleDetected);
    }

    errors
}

fn has_cycle(paths: &[PathDefinition], parents: &HashMap<PathId, Vec<PathId>>) -> bool {
    let mut children: HashMap<PathId, Vec<PathId>> = HashMap::new();
    for (child, parent_list) in parents {
        for parent in parent_list {
            children.entry(*parent).or_default().push(*child);
        }
    }

    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    fn dfs(
        id: PathId,
        children: &HashMap<PathId, Vec<PathId>>,
        visited: &mut HashSet<PathId>,
        in_stack: &mut HashSet<PathId>,
    ) -> bool {
        if in_stack.contains(&id) {
            return true;
        }
        if visited.contains(&id) {
            return false;
        }
        visited.insert(id);
        in_stack.insert(id);
        if let Some(kids) = children.get(&id) {
            for &kid in kids {
                if dfs(kid, children, visited, in_stack) {
                    return true;
                }
            }
        }
        in_stack.remove(&id);
        false
    }

    for path in paths {
        if dfs(path.id, &children, &mut visited, &mut in_stack) {
            return true;
        }
    }
    false
}

// --- Rendering ---

const LINE_WIDTH: f32 = 4.0;
const EDGE_SIZE: Vec2 = Vec2::new(4.0, HEX_SIZE + 4.0);
const HEX_EDGE_DIRECTIONS: [(f32, (i32, i32)); 6] = [
    (0.0, (1, 0)),
    (60.0, (0, 1)),
    (120.0, (-1, 1)),
    (180.0, (-1, 0)),
    (240.0, (0, -1)),
    (300.0, (1, -1)),
];

pub fn collect_connected_tiles(path_map: &PathMap) -> HashSet<((i32, i32), (i32, i32))> {
    let mut connected = HashSet::new();
    let placed: Vec<&PathDefinition> = path_map.paths.iter()
        .filter(|p| path_map.is_placed(p.id))
        .collect();

    for path in &placed {
        for pair in path.tiles.windows(2) {
            connected.insert((pair[0], pair[1]));
            connected.insert((pair[1], pair[0]));
        }
    }

    for (i, a) in placed.iter().enumerate() {
        for b in &placed[i + 1..] {
            if a.tiles.last() == b.tiles.first() {
                let edge = (*a.tiles.last().unwrap(), *b.tiles.first().unwrap());
                connected.insert(edge);
                connected.insert((edge.1, edge.0));
            }
            if b.tiles.last() == a.tiles.first() {
                let edge = (*b.tiles.last().unwrap(), *a.tiles.first().unwrap());
                connected.insert(edge);
                connected.insert((edge.1, edge.0));
            }
        }
    }

    connected
}

pub fn spawn_all_path_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    path_map: &PathMap,
) {
    let connected = collect_connected_tiles(path_map);

    for path in &path_map.paths {
        let world_tiles = path_map.path_world_tiles(path.id);
        if path_map.is_placed(path.id) {
            spawn_path_filled(commands, meshes, materials, &path.tiles, &world_tiles, &connected, PATH_FILL_COLOR, PATH_EDGE_COLOR, PATH_FILLED_Z);
        } else {
            spawn_path_line(commands, meshes, materials, &world_tiles, PATH_LINE_COLOR, PATH_LINE_Z);
        }
    }
}

pub fn spawn_path_filled(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    axial_tiles: &[(i32, i32)],
    world_tiles: &[Vec2],
    connected: &HashSet<((i32, i32), (i32, i32))>,
    fill_color: Color,
    edge_color: Color,
    z: f32,
) {
    let apothem = HEX_SIZE * 0.8660254;

    for (tile_idx, &position) in world_tiles.iter().enumerate() {
        let axial = axial_tiles[tile_idx];

        commands.spawn((
            Mesh2d(meshes.add(RegularPolygon::new(HEX_SIZE, 6))),
            MeshMaterial2d(materials.add(fill_color)),
            Transform::from_translation(position.extend(z)),
            PathVisual,
        ));

        for &(angle_deg, offset) in &HEX_EDGE_DIRECTIONS {
            let neighbor_axial = (axial.0 + offset.0, axial.1 + offset.1);

            if connected.contains(&(axial, neighbor_axial)) {
                continue;
            }

            let angle = angle_deg.to_radians();
            let direction = Vec2::from_angle(angle);

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
