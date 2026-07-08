pub mod ignite;
pub mod elemental_surge;
pub mod slow;

use bevy::prelude::*;

use crate::tags::TagInfo;

pub use slow::SlowActive;

#[derive(Clone, Copy)]
pub struct SpellDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub icon_color: Color,
    pub tags: &'static [TagInfo],
}

#[derive(Clone, Copy)]
pub struct SpellKind(pub &'static SpellDefinition);

impl SpellKind {
    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn description(self) -> &'static str {
        self.0.description
    }

    pub fn icon_color(self) -> Color {
        self.0.icon_color
    }

    pub fn tags_text(self) -> String {
        crate::tags::tags_text(self.0.tags)
    }
}

impl PartialEq for SpellKind {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for SpellKind {}

#[derive(Resource, Default)]
pub struct SpellRegistry {
    pub kinds: Vec<SpellKind>,
}

#[derive(Event)]
pub struct SpellCastEvent {
    pub kind: SpellKind,
}

pub struct SpellPlugins;

impl Plugin for SpellPlugins {
    fn build(&self, app: &mut App) {
        app.add_event::<SpellCastEvent>();
        app.add_plugins((
            ignite::IgnitePlugin,
            elemental_surge::ElementalSurgePlugin,
            slow::SlowPlugin,
        ));
    }
}
