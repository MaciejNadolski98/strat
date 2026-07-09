use bevy::prelude::*;

/// One entry in a definition's `tags` list. Pairs a real ECS component (so
/// systems can query for it, e.g. `Query<&Biotic, With<TowerKind>>`) with a
/// display name and a color, so item/spell/tower tooltips can list their tags
/// without needing to spawn an entity just to read them back.
#[derive(Clone, Copy)]
pub struct TagInfo {
    pub name: &'static str,
    pub color: Color,
    insert: fn(&mut EntityCommands),
}

impl TagInfo {
    pub fn insert(&self, commands: &mut EntityCommands) {
        (self.insert)(commands);
    }
}

macro_rules! define_tags {
    ($($name:ident => $const_name:ident, $color:expr),* $(,)?) => {
        $(
            #[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
            pub struct $name;

            pub const $const_name: TagInfo = TagInfo {
                name: stringify!($name),
                color: $color,
                insert: |commands| { commands.insert($name); },
            };
        )*
    };
}

// Shared tag taxonomy used by items, spells, and towers. Tags carry no
// gameplay effect by themselves - they exist so future systems can query for
// synergies (e.g. "how many Infernal things does the player have") or bias
// draft rates based on what's already in play.
define_tags!(
    Biotic => BIOTIC, Color::srgb(0.38, 0.78, 0.36),
    Mechanical => MECHANICAL, Color::srgb(0.62, 0.68, 0.76),
    Infernal => INFERNAL, Color::srgb(0.92, 0.30, 0.20),
);
