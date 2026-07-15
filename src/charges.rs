use bevy::math::primitives::Circle;
use bevy::prelude::*;
use bevy::sprite::ColorMaterial;

use crate::components::{Charge, ChargeConsumer, Tower};
use crate::resources::ChargeConsumedEvent;
use crate::tags::Conduit;
use crate::tower_definitions::TowerKind;

const MAX_JUMPS: u32 = 3;
const CHARGE_SPEED: f32 = 320.0;
const CHARGE_COLOR: Color = Color::srgb(0.55, 0.90, 0.98);

pub fn try_emit_charge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    conduits: &Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
    from: Entity,
    from_pos: Vec2,
    range: f32,
) -> bool {
    let Some((target, target_pos)) = pick_conduit_in_range(conduits, from_pos, range, from, None) else {
        return false;
    };
    spawn_charge(commands, meshes, materials, from, from_pos, target, target_pos, MAX_JUMPS);
    true
}

pub fn advance_charges(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut charges: Query<(Entity, &mut Charge, &mut Transform), Without<Tower>>,
    towers: Query<(&Transform, &TowerKind, Option<&ChargeConsumer>), With<Tower>>,
    conduits: Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
    mut consumed_events: EventWriter<ChargeConsumedEvent>,
) {
    for (entity, mut charge, mut transform) in &mut charges {
        charge.travel.tick(time.delta());

        let Ok((from_transform, _, _)) = towers.get(charge.from) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Ok((to_transform, to_kind, to_consumer)) = towers.get(charge.to) else {
            commands.entity(entity).despawn();
            continue;
        };

        let from_pos = from_transform.translation.truncate();
        let to_pos = to_transform.translation.truncate();
        transform.translation = from_pos.lerp(to_pos, charge.travel.fraction()).extend(6.0);

        if !charge.travel.finished() {
            continue;
        }

        commands.entity(entity).despawn();

        if to_consumer.is_some() {
            consumed_events.write(ChargeConsumedEvent { tower: charge.to });
            continue;
        }

        if charge.jumps_left == 0 {
            continue;
        }

        let Some((next_target, next_pos)) =
            pick_conduit_in_range(&conduits, to_pos, to_kind.range(), charge.to, Some(charge.from))
        else {
            continue;
        };

        spawn_charge(
            &mut commands, &mut meshes, &mut materials,
            charge.to, to_pos, next_target, next_pos, charge.jumps_left - 1,
        );
    }
}

fn pick_conduit_in_range(
    conduits: &Query<(Entity, &Transform), (With<Tower>, With<Conduit>)>,
    origin: Vec2,
    range: f32,
    exclude: Entity,
    exclude_prev: Option<Entity>,
) -> Option<(Entity, Vec2)> {
    let candidates: Vec<(Entity, Vec2)> = conduits
        .iter()
        .filter(|(candidate, _)| *candidate != exclude && Some(*candidate) != exclude_prev)
        .map(|(candidate, t)| (candidate, t.translation.truncate()))
        .filter(|(_, pos)| pos.distance(origin) <= range)
        .collect();

    if candidates.is_empty() {
        return None;
    }
    Some(candidates[rand::random::<usize>() % candidates.len()])
}

fn spawn_charge(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    from: Entity,
    from_pos: Vec2,
    to: Entity,
    to_pos: Vec2,
    jumps_left: u32,
) {
    let travel_secs = (from_pos.distance(to_pos) / CHARGE_SPEED).max(0.05);
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(5.0))),
        MeshMaterial2d(materials.add(CHARGE_COLOR)),
        Transform::from_translation(from_pos.extend(6.0)),
        Charge {
            from,
            to,
            travel: Timer::from_seconds(travel_secs, TimerMode::Once),
            jumps_left,
        },
    ));
}
