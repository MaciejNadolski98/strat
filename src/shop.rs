use bevy::prelude::*;

use crate::components::{ShopSlot, ShopSlotBarrel, ShopSlotIcon, ShopSlotLabel, ShopText};
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{GameOver, Money, Shop};

pub fn update_shop_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shop: ResMut<Shop>,
    mut money: ResMut<Money>,
    game_over: Res<GameOver>,
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

    text.0 = format!("Shop     1-3: select     E: reroll ${}", shop.reroll_cost);

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
