use std::collections::HashMap;
use std::fs;

use crate::regions::RegionDefinition;
use crate::terrain::TerrainKind;

pub const TERRAIN_FILE_PATH: &str = "terrain_map.txt";

pub struct TerrainFileData {
    pub overrides: HashMap<(i32, i32), TerrainKind>,
    pub regions: Vec<RegionDefinition>,
}

pub fn load_terrain_file() -> TerrainFileData {
    let Ok(contents) = fs::read_to_string(TERRAIN_FILE_PATH) else {
        return TerrainFileData {
            overrides: HashMap::new(),
            regions: Vec::new(),
        };
    };

    let mut overrides = HashMap::new();
    let mut regions = Vec::new();
    let mut in_regions_section = false;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[regions]" {
            in_regions_section = true;
            continue;
        }
        if line.starts_with('[') {
            in_regions_section = false;
            continue;
        }

        if in_regions_section {
            if let Some(region) = parse_region_line(line) {
                regions.push(region);
            }
        } else {
            let mut parts = line.split_whitespace();
            let (Some(q), Some(r), Some(kind_key)) = (parts.next(), parts.next(), parts.next())
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
    }

    TerrainFileData { overrides, regions }
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

pub fn save_terrain_file(overrides: &HashMap<(i32, i32), TerrainKind>, regions: &[RegionDefinition]) {
    let mut entries: Vec<((i32, i32), TerrainKind)> =
        overrides.iter().map(|(&coord, &kind)| (coord, kind)).collect();
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

    if let Err(err) = fs::write(TERRAIN_FILE_PATH, contents) {
        eprintln!("Failed to save terrain map to {TERRAIN_FILE_PATH}: {err}");
    }
}

