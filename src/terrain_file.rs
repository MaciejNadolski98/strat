use std::collections::HashMap;
use std::fs;

use crate::components::EnemyKind;
use crate::paths::{PathDefinition, PathEnemyGroup, PathId};
use crate::regions::RegionDefinition;
use crate::terrain::TerrainKind;

pub const TERRAIN_FILE_PATH: &str = "terrain_map.txt";

pub struct TerrainFileData {
    pub overrides: HashMap<(i32, i32), TerrainKind>,
    pub regions: Vec<RegionDefinition>,
    pub paths: Vec<PathDefinition>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Tiles,
    Regions,
    Paths,
}

pub fn load_terrain_file() -> TerrainFileData {
    let Ok(contents) = fs::read_to_string(TERRAIN_FILE_PATH) else {
        return TerrainFileData {
            overrides: HashMap::new(),
            regions: Vec::new(),
            paths: Vec::new(),
        };
    };

    let mut overrides = HashMap::new();
    let mut regions = Vec::new();
    let mut paths = Vec::new();
    let mut section = Section::Tiles;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[regions]" {
            section = Section::Regions;
            continue;
        }
        if line == "[paths]" {
            section = Section::Paths;
            continue;
        }
        if line.starts_with('[') {
            section = Section::Tiles;
            continue;
        }

        match section {
            Section::Tiles => {
                let mut parts = line.split_whitespace();
                let (Some(q), Some(r), Some(kind_key)) =
                    (parts.next(), parts.next(), parts.next())
                else {
                    continue;
                };
                let (Ok(q), Ok(r)) = (q.parse::<i32>(), r.parse::<i32>()) else {
                    continue;
                };
                let Some(kind) = TerrainKind::from_key(kind_key) else {
                    continue;
                };
                overrides.insert((q, r), kind);
            }
            Section::Regions => {
                if let Some(region) = parse_region_line(line) {
                    regions.push(region);
                }
            }
            Section::Paths => {
                if let Some(path) = parse_path_line(line, PathId(paths.len())) {
                    paths.push(path);
                }
            }
        }
    }

    TerrainFileData {
        overrides,
        regions,
        paths,
    }
}

fn parse_region_line(line: &str) -> Option<RegionDefinition> {
    let (name, coords_str) = line.split_once(':')?;
    let name = name.trim().to_string();
    let mut tiles = Vec::new();
    for pair in coords_str.split_whitespace() {
        let (q_str, r_str) = pair.split_once(',')?;
        let q = q_str.parse::<i32>().ok()?;
        let r = r_str.parse::<i32>().ok()?;
        tiles.push((q, r));
    }
    if tiles.is_empty() {
        return None;
    }
    Some(RegionDefinition { name, tiles })
}

fn parse_path_line(line: &str, id: PathId) -> Option<PathDefinition> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() < 4 {
        return None;
    }

    let name = parts[0].trim().to_string();

    let mut tiles = Vec::new();
    for pair in parts[1].trim().split_whitespace() {
        let (q_str, r_str) = pair.split_once(',')?;
        let q = q_str.parse::<i32>().ok()?;
        let r = r_str.parse::<i32>().ok()?;
        tiles.push((q, r));
    }

    let mut enemies = Vec::new();
    for group_str in parts[2].trim().split_whitespace() {
        if let Some(group) = parse_enemy_group(group_str) {
            enemies.push(group);
        }
    }

    let level = parts[3].trim().parse::<u8>().unwrap_or(0);

    let unlocks = if parts.len() > 4 {
        parts[4]
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    let excludes = if parts.len() > 5 {
        parts[5]
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    Some(PathDefinition {
        id,
        name,
        tiles,
        enemies,
        level,
        unlocks,
        excludes,
    })
}

fn parse_enemy_group(s: &str) -> Option<PathEnemyGroup> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let kind = match parts[0] {
        "Grunt" => EnemyKind::Grunt,
        "Runner" => EnemyKind::Runner,
        "Brute" => EnemyKind::Brute,
        "Armored" => EnemyKind::Armored,
        "Titan" => EnemyKind::Titan,
        _ => return None,
    };
    let count = parts[1].parse().ok()?;
    let cooldown = parts[2].parse().ok()?;
    Some(PathEnemyGroup {
        kind,
        count,
        cooldown,
    })
}

fn enemy_kind_key(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Grunt => "Grunt",
        EnemyKind::Runner => "Runner",
        EnemyKind::Brute => "Brute",
        EnemyKind::Armored => "Armored",
        EnemyKind::Titan => "Titan",
    }
}

pub fn save_terrain_file(
    overrides: &HashMap<(i32, i32), TerrainKind>,
    regions: &[RegionDefinition],
    paths: &[PathDefinition],
) {
    let mut entries: Vec<((i32, i32), TerrainKind)> = overrides
        .iter()
        .map(|(&coord, &kind)| (coord, kind))
        .collect();
    entries.sort_by_key(|&(coord, _)| coord);

    let mut contents = String::new();
    for ((q, r), kind) in entries {
        contents.push_str(&format!("{q} {r} {}\n", kind.key()));
    }

    if !regions.is_empty() {
        contents.push_str("\n[regions]\n");
        for region in regions {
            contents.push_str(&region.name);
            contents.push(':');
            for (i, &(q, r)) in region.tiles.iter().enumerate() {
                if i > 0 {
                    contents.push(' ');
                }
                contents.push_str(&format!("{q},{r}"));
            }
            contents.push('\n');
        }
    }

    if !paths.is_empty() {
        contents.push_str("\n[paths]\n");
        for path in paths {
            contents.push_str(&path.name);
            contents.push_str(" | ");
            for (i, &(q, r)) in path.tiles.iter().enumerate() {
                if i > 0 {
                    contents.push(' ');
                }
                contents.push_str(&format!("{q},{r}"));
            }
            contents.push_str(" | ");
            for (i, group) in path.enemies.iter().enumerate() {
                if i > 0 {
                    contents.push(' ');
                }
                contents.push_str(&format!(
                    "{}:{}:{}",
                    enemy_kind_key(group.kind),
                    group.count,
                    group.cooldown
                ));
            }
            contents.push_str(&format!(" | {}", path.level));
            contents.push_str(" | ");
            contents.push_str(&path.unlocks.join(","));
            contents.push_str(" | ");
            contents.push_str(&path.excludes.join(","));
            contents.push('\n');
        }
    }

    if let Err(err) = fs::write(TERRAIN_FILE_PATH, contents) {
        eprintln!("Failed to save terrain map to {TERRAIN_FILE_PATH}: {err}");
    }
}
