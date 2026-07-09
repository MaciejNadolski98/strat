use bevy::prelude::*;

use crate::tags::TagInfo;

const FONT_SIZE: f32 = 16.0;
const DEFAULT_COLOR: Color = Color::srgb(0.94, 0.94, 0.86);

/// One piece of a tooltip's text, rendered as its own child `TextSpan` so it
/// can carry a color different from the rest of the tooltip.
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

/// Comma-separated, each tag name colored with its own `TagInfo::color`.
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

/// Sets a `ShopTooltip`-style entity's text from a list of segments, each
/// rendered as its own child `TextSpan` so different pieces can have
/// different colors. Re-spawns the tooltip's child spans every call, which is
/// fine at tooltip scale (a handful of short-lived spans, updated only while
/// hovering).
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
