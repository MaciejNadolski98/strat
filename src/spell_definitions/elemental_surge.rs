use bevy::prelude::*;

use crate::game::game_is_running;
use crate::resources::{AirDamage, EarthDamage, FireDamage, NewRoundEvent, WaterDamage,
    before_temporary_effects, after_temporary_effects};
use super::{SpellCastEvent, SpellDefinition, SpellKind, SpellRegistry};

const SURGE_BONUS: f32 = 1.0;

const SPELL: SpellDefinition = SpellDefinition {
    name: "Surge",
    description: "Doubles elemental damage until wave end",
    icon_color: Color::srgb(0.30, 0.62, 0.92),
};

pub const KIND: SpellKind = SpellKind(&SPELL);

#[derive(Resource, Default)]
struct SurgeActive(u32);

pub struct ElementalSurgePlugin;

impl Plugin for ElementalSurgePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<SpellRegistry>().kinds.push(KIND);
        app.init_resource::<SurgeActive>();
        app.add_systems(Update, (on_cast, deactivate_on_wave_end));
        app.add_systems(
            Update,
            apply_elemental_surge
                .after(before_temporary_effects)
                .before(after_temporary_effects)
                .run_if(game_is_running),
        );
    }
}

fn on_cast(mut events: EventReader<SpellCastEvent>, mut active: ResMut<SurgeActive>) {
    for event in events.read() {
        if event.kind == KIND {
            active.0 += 1;
        }
    }
}

fn deactivate_on_wave_end(mut events: EventReader<NewRoundEvent>, mut active: ResMut<SurgeActive>) {
    if events.read().next().is_some() {
        active.0 = 0;
    }
}

fn apply_elemental_surge(
    active: Res<SurgeActive>,
    mut earth: ResMut<EarthDamage>,
    mut fire: ResMut<FireDamage>,
    mut air: ResMut<AirDamage>,
    mut water: ResMut<WaterDamage>,
) {
    if active.0 > 0 {
        let bonus = SURGE_BONUS * active.0 as f32;
        earth.temporary_multiplier += bonus;
        fire.temporary_multiplier += bonus;
        air.temporary_multiplier += bonus;
        water.temporary_multiplier += bonus;
    }
}
