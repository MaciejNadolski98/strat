use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    ShopSlot, ShopSlotBarrel, ShopSlotIcon, ShopSlotLabel, ShopText, ShopTooltip, TowerKind,
};
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, MaxHp, Money, PassiveIncome, Regeneration, Shop, StatUpgradeKind, WaterDamage,
};

#[derive(SystemParam)]
pub struct PlayerStatsMut<'w> {
    current_hp: ResMut<'w, CurrentHp>,
    max_hp: ResMut<'w, MaxHp>,
    regeneration: ResMut<'w, Regeneration>,
    attack_speed: ResMut<'w, AttackSpeed>,
    passive_income: ResMut<'w, PassiveIncome>,
    critical_chance: ResMut<'w, CriticalChance>,
    explosion_size: ResMut<'w, ExplosionSize>,
    earth_damage: ResMut<'w, EarthDamage>,
    fire_damage: ResMut<'w, FireDamage>,
    air_damage: ResMut<'w, AirDamage>,
    water_damage: ResMut<'w, WaterDamage>,
}

pub fn update_shop_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shop: ResMut<Shop>,
    mut money: ResMut<Money>,
    game_over: Res<GameOver>,
    mut stats: PlayerStatsMut,
) {
    if game_over.value {
        return;
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        shop.selected = 0;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        shop.selected = 1;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        shop.selected = 2;
    }

    if keyboard.just_pressed(KeyCode::KeyE) && money.amount >= shop.reroll_cost {
        let cost = shop.reroll_cost;
        money.amount -= cost;
        shop.reroll();
        spawn_floating_text(
            &mut commands,
            format!("-${cost}"),
            Vec2::new(-WINDOW_WIDTH * 0.5 + 280.0, -WINDOW_HEIGHT * 0.5 + 72.0),
            Color::srgb(1.0, 0.86, 0.20),
            20.0,
        );
    }

    if keyboard.just_pressed(KeyCode::KeyB) {
        let Some(offer) = shop.selected_offer() else {
            return;
        };
        let Some(upgrade) = offer.item.stat_upgrade_kind() else {
            return;
        };
        if money.amount < offer.cost {
            return;
        }

        money.amount -= offer.cost;
        shop.take_selected_offer();
        apply_stat_upgrade(upgrade, &mut stats);
        spawn_floating_text(
            &mut commands,
            format!("-${}", offer.cost),
            Vec2::new(-WINDOW_WIDTH * 0.5 + 420.0, -WINDOW_HEIGHT * 0.5 + 72.0),
            Color::srgb(1.0, 0.86, 0.20),
            20.0,
        );
    }
}

pub fn update_shop_text(
    shop: Res<Shop>,
    mut text: Query<&mut Text, With<ShopText>>,
    mut slots: ParamSet<(
        Query<(&ShopSlot, &mut Sprite)>,
        Query<(&ShopSlotIcon, &mut Sprite, &mut Visibility)>,
        Query<(&ShopSlotBarrel, &mut Sprite, &mut Visibility)>,
        Query<(&ShopSlotLabel, &mut Text2d)>,
    )>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = format!(
        "Shop     1-3: select     B: buy upgrade     E: reroll ${}",
        shop.reroll_cost
    );

    for (slot, mut sprite) in &mut slots.p0() {
        let is_selected = slot.index == shop.selected;
        let is_empty = shop.offers[slot.index].is_none();
        sprite.color = match (is_selected, is_empty) {
            (true, false) => Color::srgb(0.32, 0.34, 0.24),
            (true, true) => Color::srgb(0.26, 0.25, 0.20),
            (false, false) => Color::srgb(0.15, 0.17, 0.16),
            (false, true) => Color::srgb(0.09, 0.10, 0.10),
        };
    }

    for (icon, mut sprite, mut visibility) in &mut slots.p1() {
        let Some(offer) = shop.offers[icon.index] else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let Some(kind) = offer.item.tower_kind() {
            sprite.color = kind.base_color();
            *visibility = Visibility::Visible;
        } else if let Some(kind) = offer.item.stat_upgrade_kind() {
            sprite.color = kind.icon_color();
            *visibility = Visibility::Visible;
        }
    }

    for (barrel, mut sprite, mut visibility) in &mut slots.p2() {
        let Some(offer) = shop.offers[barrel.index] else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let Some(kind) = offer.item.tower_kind() {
            sprite.color = kind.barrel_color();
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    for (label, mut text) in &mut slots.p3() {
        text.0 = if let Some(offer) = shop.offers[label.index] {
            format!("{} ${}", offer.item.name(), offer.cost)
        } else {
            "Empty".to_string()
        };
    }
}

pub fn update_shop_tooltip(
    shop: Res<Shop>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    slots: Query<(&ShopSlot, &Transform)>,
    mut tooltip: Query<(&mut Text, &mut Visibility), With<ShopTooltip>>,
) {
    let Ok((mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };

    *tooltip_visibility = Visibility::Hidden;

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    for (slot, transform) in &slots {
        let slot_position = transform.translation.truncate();
        let inside_slot = (world_position.x - slot_position.x).abs() <= 48.0
            && (world_position.y - slot_position.y).abs() <= 36.0;
        if !inside_slot {
            continue;
        }

        let Some(offer) = shop.offers[slot.index] else {
            return;
        };

        tooltip_text.0 = match offer.item.tower_kind() {
            Some(kind) => tower_tooltip(kind, offer.cost),
            None => match offer.item.stat_upgrade_kind() {
                Some(kind) => upgrade_tooltip(kind, offer.cost),
                None => offer.item.name().to_string(),
            },
        };
        *tooltip_visibility = Visibility::Visible;
        return;
    }
}

fn tower_tooltip(kind: TowerKind, cost: i32) -> String {
    let damage = kind.damage_formula();
    format!(
        "{}  ${}\nDamage: {}\nRange: {:.0}\nCooldown: {:.2}s\nSplash: {:.0}",
        kind.name(),
        cost,
        damage,
        kind.range(),
        kind.cooldown(),
        kind.explosion_radius(),
    )
}

fn upgrade_tooltip(kind: StatUpgradeKind, cost: i32) -> String {
    format!(
        "{}  ${}\nPermanent upgrade\n{}",
        kind.name(),
        cost,
        kind.effect_text()
    )
}

fn apply_stat_upgrade(kind: StatUpgradeKind, stats: &mut PlayerStatsMut) {
    match kind {
        StatUpgradeKind::MaxHp => {
            stats.max_hp.amount += 5;
            stats.current_hp.amount += 5;
        }
        StatUpgradeKind::Regeneration => {
            stats.regeneration.amount += 1;
        }
        StatUpgradeKind::AttackSpeed => {
            stats.attack_speed.value += 0.12;
        }
        StatUpgradeKind::PassiveIncome => {
            stats.passive_income.amount += 1;
        }
        StatUpgradeKind::CriticalChance => {
            stats.critical_chance.value = (stats.critical_chance.value + 0.04).min(1.0);
        }
        StatUpgradeKind::ExplosionSize => {
            stats.explosion_size.value += 12.0;
        }
        StatUpgradeKind::EarthDamage => {
            stats.earth_damage.value += 4.0;
        }
        StatUpgradeKind::FireDamage => {
            stats.fire_damage.value += 4.0;
        }
        StatUpgradeKind::AirDamage => {
            stats.air_damage.value += 4.0;
        }
        StatUpgradeKind::WaterDamage => {
            stats.water_damage.value += 4.0;
        }
    }
}
