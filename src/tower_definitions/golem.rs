use bevy::prelude::*;

use crate::components::{DamageFormula, TowerKillCount};
use crate::game::game_is_running;
use crate::projectiles::move_projectiles;
use crate::resources::{EarthDamage, EnemyKilledEvent, PlayerStatKind, TowerStatEffect};
use crate::tower_definitions::TowerKind;
use crate::tower_definitions::templates::BASE_TRIANGLE_M;
use super::{TowerDefinition, TooltipConfig};
use super::templates::{BARREL_HEAVY, PALETTE_EARTH};

pub struct GolemPlugin;

impl Plugin for GolemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_golem_kill_count,
                golem_kill_tracking.after(move_projectiles),
            )
                .run_if(game_is_running),
        );
    }
}

pub const TOWER_GOLEM: TowerDefinition = TowerDefinition {
    name: "Golem",
    range: 160.0,
    cooldown: 1.1,
    damage_formula: DamageFormula {
        flat: 20,
        crit_multiplier: 1.8,
        earth_multiplier: 0.5,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 350.0,
    explosion_radius: 0.0,
    angular_speed: 1.2,
    base_color: PALETTE_EARTH.base,
    barrel_color: PALETTE_EARTH.barrel,
    base: BASE_TRIANGLE_M,
    barrel: BARREL_HEAVY,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::EarthDamage, 3.0)],
    tooltip_config: TooltipConfig::STANDARD,
};

fn attach_golem_kill_count(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == TowerKind::Golem {
            commands.entity(entity).insert(TowerKillCount { kills: 0 });
        }
    }
}

fn golem_kill_tracking(
    mut events: EventReader<EnemyKilledEvent>,
    mut golems: Query<&mut TowerKillCount>,
    mut earth_damage: ResMut<EarthDamage>,
) {
    for event in events.read() {
        if let Ok(mut kc) = golems.get_mut(event.source_tower) {
            let prev = kc.kills / 10;
            kc.kills += 1;
            let gained = kc.kills / 10 - prev;
            earth_damage.value += gained as f32;
        }
    }
}
