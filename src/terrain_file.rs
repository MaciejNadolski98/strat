use std::collections::HashMap;
use std::fs;

use crate::terrain::TerrainKind;

pub const TERRAIN_FILE_PATH: &str = "terrain_map.txt";

pub fn load_terrain_overrides() -> HashMap<(i32, i32), TerrainKind> {
    let Ok(contents) = fs::read_to_string(TERRAIN_FILE_PATH) else {
        return HashMap::new();
    };

    let mut overrides = HashMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(q), Some(r), Some(kind_key)) = (parts.next(), parts.next(), parts.next()) else {
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
    overrides
}

pub fn save_terrain_overrides(overrides: &HashMap<(i32, i32), TerrainKind>) {
    let mut entries: Vec<((i32, i32), TerrainKind)> =
        overrides.iter().map(|(&coord, &kind)| (coord, kind)).collect();
    entries.sort_by_key(|&(coord, _)| coord);

    let mut contents = String::new();
    for ((q, r), kind) in entries {
        contents.push_str(&format!("{q} {r} {}\n", kind.key()));
    }

    if let Err(err) = fs::write(TERRAIN_FILE_PATH, contents) {
        eprintln!("Failed to save terrain map to {TERRAIN_FILE_PATH}: {err}");
    }
}
