use bevy::prelude::*;

use crate::resources::NewRoundEvent;
use super::{SpellCastEvent, SpellDefinition, SpellKind, SpellRegistry};

const SLOW_MULTIPLIER: f32 = 0.5;

pub static SPELL: SpellDefinition = SpellDefinition {
    name: "Slow",
    description: "Slows all enemies until wave end",
    icon_color: Color::srgb(0.42, 0.82, 0.92),
    tags: &[],
};

pub static KIND: SpellKind = SpellKind(&SPELL);

#[derive(Resource)]
pub struct SlowActive {
    pub multiplier: f32,
}

impl Default for SlowActive {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

pub struct SlowPlugin;

impl Plugin for SlowPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<SpellRegistry>().kinds.push(KIND);
        app.init_resource::<SlowActive>();
        app.add_systems(Update, (on_cast, reset_on_new_round));
    }
}

fn on_cast(
    mut events: EventReader<SpellCastEvent>,
    mut active: ResMut<SlowActive>,
) {
    for event in events.read() {
        if event.kind == KIND {
            active.multiplier = SLOW_MULTIPLIER;
        }
    }
}

fn reset_on_new_round(
    mut events: EventReader<NewRoundEvent>,
    mut active: ResMut<SlowActive>,
) {
    if events.read().next().is_some() {
        active.multiplier = 1.0;
    }
}
