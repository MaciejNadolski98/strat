use bevy::prelude::*;

use crate::tags::TagInfo;

const FONT_SIZE: f32 = 16.0;
const DEFAULT_COLOR: Color = Color::srgb(0.94, 0.94, 0.86);

#[derive(Clone)]
pub struct Segment {
    text: String,
    color: Option<Color>,
}

pub fn plain(text: impl Into<String>) -> Segment {
    Segment { text: text.into(), color: None }
}

pub fn colored(text: impl Into<String>, color: Color) -> Segment {
    Segment { text: text.into(), color: Some(color) }
}

pub fn tag_segments(tags: &[TagInfo]) -> Vec<Segment> {
    let mut segs = Vec::new();
    for (i, tag) in tags.iter().enumerate() {
        if i > 0 {
            segs.push(plain(", "));
        }
        segs.push(colored(tag.name, tag.color));
    }
    segs
}

pub fn push_line(segments: &mut Vec<Segment>, first: &mut bool, line: Vec<Segment>) {
    if !*first {
        segments.push(plain("\n"));
    }
    segments.extend(line);
    *first = false;
}

pub const MONEY_COLOR: Color = Color::srgb(0.40, 0.92, 0.36);

/// Colors known elemental keywords (Fire/Earth/Air/Water) and `$amount` tokens
/// inside freeform item/spell description text, so descriptions get the same
/// coloring as structured stat-effect lines without needing manual markup.
pub fn highlight_description(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            segments.push(plain(" "));
        }
        let core = word.trim_end_matches([',', '.']);
        let suffix = &word[core.len()..];

        let color = match core.to_lowercase().as_str() {
            "fire" => Some(crate::towers::FIRE_COLOR),
            "earth" => Some(crate::towers::EARTH_COLOR),
            "air" => Some(crate::towers::AIR_COLOR),
            "water" => Some(crate::towers::WATER_COLOR),
            _ if core.starts_with('$') => Some(MONEY_COLOR),
            _ => None,
        };

        segments.push(match color {
            Some(c) => colored(core.to_string(), c),
            None => plain(core.to_string()),
        });
        if !suffix.is_empty() {
            segments.push(plain(suffix.to_string()));
        }
    }
    segments
}

pub fn set_tooltip_segments(
    commands: &mut Commands,
    tooltip_entity: Entity,
    text: &mut Text,
    segments: Vec<Segment>,
) {
    commands.entity(tooltip_entity).despawn_related::<Children>();
    text.0 = String::new();

    if segments.is_empty() {
        return;
    }

    commands.entity(tooltip_entity).with_children(|parent| {
        for seg in segments {
            parent.spawn((
                TextSpan::new(seg.text),
                TextFont { font_size: FONT_SIZE, ..default() },
                TextColor(seg.color.unwrap_or(DEFAULT_COLOR)),
            ));
        }
    });
}
