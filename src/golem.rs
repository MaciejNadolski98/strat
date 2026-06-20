use bevy::prelude::*;

use crate::components::{TowerKind, TowerKillCount};
use crate::game::game_is_running;
use crate::projectiles::move_projectiles;
use crate::resources::{EarthDamage, EnemyKilledEvent};

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
