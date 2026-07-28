use bevy::prelude::*;

use crate::components::{TemporaryAttackSpeed, TemporaryProjectileSpeed};
use crate::resources::{GamePhase, Shop};
use crate::tags;
use crate::tower_definitions::golem::{self, GolemTower};
use super::{golem_heart, unlock, ItemDefinition, ItemKind, UnlockCondition};

const ATTACK_SPEED_PER_STACK: f32 = 0.3;
const PROJECTILE_SPEED_PER_STACK: f32 = 100.0;

pub static ITEM: ItemDefinition = ItemDefinition::new(
    "Golem's Leg",
    &[],
    6,
    Color::srgb(0.46, 0.36, 0.26),
)
    .with_description("+0.3 Golem attack speed, +100 projectile speed")
    .with_tags(&[tags::BIOTIC, tags::MECHANICAL])
    .with_max_purchases(2);

pub static KIND: ItemKind = ItemKind(&ITEM);

pub struct GolemLegPlugin;

impl Plugin for GolemLegPlugin {
    fn build(&self, app: &mut App) {
        unlock(app, UnlockCondition::Tower(golem::KIND), KIND);
        app.add_systems(Update, apply_speed_bonus.in_set(GamePhase::TemporaryTowerEffects));
    }
}

fn apply_speed_bonus(
    shop: Res<Shop>,
    mut towers: Query<(&mut TemporaryAttackSpeed, &mut TemporaryProjectileSpeed), With<GolemTower>>,
) {
    let stacks = shop.purchase_count(KIND) as f32;
    let multiplier = if shop.purchase_count(golem_heart::KIND) > 0 { 2.0 } else { 1.0 };
    let attack_speed_bonus = stacks * ATTACK_SPEED_PER_STACK * multiplier;
    let projectile_speed_bonus = stacks * PROJECTILE_SPEED_PER_STACK * multiplier;

    for (mut attack_speed, mut projectile_speed) in &mut towers {
        attack_speed.flat += attack_speed_bonus;
        projectile_speed.flat += projectile_speed_bonus;
    }
}
