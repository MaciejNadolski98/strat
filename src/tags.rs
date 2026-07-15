use bevy::prelude::*;

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

define_tags!(
    Biotic => BIOTIC, Color::srgb(0.38, 0.78, 0.36),
    Mechanical => MECHANICAL, Color::srgb(0.62, 0.68, 0.76),
    Infernal => INFERNAL, Color::srgb(0.92, 0.30, 0.20),
    Conduit => CONDUIT, Color::srgb(0.55, 0.90, 0.98),
);
