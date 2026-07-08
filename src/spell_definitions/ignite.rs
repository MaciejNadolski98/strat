use bevy::prelude::*;

use crate::components::{Burning, Enemy, Health};
use crate::resources::FireDamage;
use crate::tags;
use super::{SpellCastEvent, SpellDefinition, SpellKind, SpellRegistry};

const BURN_DURATION: f32 = 6.0;
const BURN_TICK: f32 = 0.5;
const BURN_DAMAGE_PER_TICK: f32 = 8.0;

pub const SPELL: SpellDefinition = SpellDefinition {
    name: "Ignite",
    description: "Sets all enemies on fire, scaling with fire damage",
    icon_color: Color::srgb(0.92, 0.26, 0.12),
    tags: &[tags::INFERNAL],
};

pub const KIND: SpellKind = SpellKind(&SPELL);

pub struct IgnitePlugin;

impl Plugin for IgnitePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<SpellRegistry>().kinds.push(KIND);
        app.add_systems(Update, on_cast);
    }
}

fn on_cast(
    mut commands: Commands,
    mut events: EventReader<SpellCastEvent>,
    fire_damage: Res<FireDamage>,
    enemies: Query<(Entity, &Health), With<Enemy>>,
) {
    for event in events.read() {
        if event.kind != KIND {
            continue;
        }
        let damage_per_tick = BURN_DAMAGE_PER_TICK + fire_damage.value();
        for (enemy, health) in &enemies {
            if health.current <= 0.0 {
                continue;
            }
            commands.entity(enemy).try_insert(Burning {
                timer: Timer::from_seconds(BURN_DURATION, TimerMode::Once),
                tick_timer: Timer::from_seconds(BURN_TICK, TimerMode::Repeating),
                damage_per_tick,
            });
        }
    }
}
