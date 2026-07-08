use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    ShopSlot, ShopSlotBarrel, ShopSlotIcon, ShopSlotLabel, ShopText, ShopTooltip,
};
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::item_definitions::ItemKind;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, GameWon, ItemPurchasedEvent, MaxHp, Money, Loot, PlayerStatKind, Regeneration, Shop,
    TowerDraft, TowerDraftPhase, TowerStatEffect, WaterDamage, WaveNumber,
};

#[derive(SystemParam)]
pub struct PlayerStatsMut<'w> {
    current_hp: ResMut<'w, CurrentHp>,
    max_hp: ResMut<'w, MaxHp>,
    regeneration: ResMut<'w, Regeneration>,
    attack_speed: ResMut<'w, AttackSpeed>,
    loot: ResMut<'w, Loot>,
    critical_chance: ResMut<'w, CriticalChance>,
    explosion_size: ResMut<'w, ExplosionSize>,
    earth_damage: ResMut<'w, EarthDamage>,
    fire_damage: ResMut<'w, FireDamage>,
    air_damage: ResMut<'w, AirDamage>,
    water_damage: ResMut<'w, WaterDamage>,
}

impl<'w> PlayerStatsMut<'w> {
    pub fn attack_speed_value(&self) -> f32 {
        self.attack_speed.value()
    }

    pub fn apply_tower_effect(&mut self, effect: TowerStatEffect) {
        let delta = effect.amount;
        match effect.kind {
            PlayerStatKind::MaxHp => {
                let change = delta.round() as i32;
                self.max_hp.raw_value = (self.max_hp.raw_value + delta).max(1.0);
                self.current_hp.amount =
                    (self.current_hp.amount + change).clamp(0, self.max_hp.value().round() as i32);
            }
            PlayerStatKind::Regeneration => {
                self.regeneration.raw_value += delta;
            }
            PlayerStatKind::AttackSpeed => {
                self.attack_speed.raw_value += delta;
            }
            PlayerStatKind::Loot => {
                self.loot.raw_value += delta;
            }
            PlayerStatKind::CriticalChance => {
                self.critical_chance.raw_value += delta;
            }
            PlayerStatKind::ExplosionSize => {
                self.explosion_size.raw_value += delta;
            }
            PlayerStatKind::EarthDamage => {
                self.earth_damage.raw_value += delta;
            }
            PlayerStatKind::FireDamage => {
                self.fire_damage.raw_value += delta;
            }
            PlayerStatKind::AirDamage => {
                self.air_damage.raw_value += delta;
            }
            PlayerStatKind::WaterDamage => {
                self.water_damage.raw_value += delta;
            }
        }
    }
}

pub fn update_shop_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    shop_slots: Query<(&ShopSlot, &GlobalTransform)>,
    mut shop: ResMut<Shop>,
    mut money: ResMut<Money>,
    game_over: Res<GameOver>,
    game_won: Res<GameWon>,
    wave_number: Res<WaveNumber>,
    draft: Res<TowerDraft>,
    mut stats: PlayerStatsMut,
    mut item_purchased: EventWriter<ItemPurchasedEvent>,
) {
    if game_over.value || game_won.value || draft.phase == TowerDraftPhase::Picking {
        return;
    }

    shop.update_prices_for_wave(wave_number.value);

    let mut selected = None;

    if mouse.just_pressed(MouseButton::Left) {
        if let (Ok(window), Ok((cam, cam_transform))) = (windows.single(), camera.single()) {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                    for (slot, global) in &shop_slots {
                        let pos = global.translation().truncate();
                        if (world_pos.x - pos.x).abs() <= 48.0
                            && (world_pos.y - pos.y).abs() <= 36.0
                        {
                            selected = Some(slot.index);
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Some(selected) = selected {
        let Some(offer) = shop.offers[selected] else { return };
        if money.amount < offer.cost {
            return;
        };

        money.amount -= offer.cost;
        shop.take_offer(selected);
        apply_stat_upgrade(offer.item, &mut stats);
        item_purchased.write(ItemPurchasedEvent { kind: offer.item });
        spawn_floating_text(
            &mut commands,
            format!("-${}", offer.cost),
            Vec2::new(-WINDOW_WIDTH * 0.5 + 420.0, -WINDOW_HEIGHT * 0.5 + 72.0),
            Color::srgb(1.0, 0.86, 0.20),
            20.0,
        );
    }

    if keyboard.just_pressed(KeyCode::KeyE) && money.amount >= shop.reroll_cost {
        let cost = shop.reroll_cost;
        money.amount -= cost;
        shop.reroll(wave_number.value);
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
    money: Res<Money>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut text: Query<&mut Text, With<ShopText>>,
    mut slots: ParamSet<(
        Query<(&ShopSlot, &GlobalTransform, &mut Sprite)>,
        Query<(&ShopSlotIcon, &mut Sprite, &mut Visibility)>,
        Query<(&ShopSlotBarrel, &mut Sprite, &mut Visibility)>,
        Query<(&ShopSlotLabel, &mut Text2d)>,
    )>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };

    text.0 = format!(
        "Shop     click: buy item     E: reroll ${}",
        shop.reroll_cost
    );

    let cursor_world = (|| -> Option<Vec2> {
        let window = windows.single().ok()?;
        let (cam, cam_t) = camera.single().ok()?;
        cam.viewport_to_world_2d(cam_t, window.cursor_position()?).ok()
    })();

    for (slot, global, mut sprite) in &mut slots.p0() {
        let offer = shop.offers[slot.index];
        let is_empty = offer.is_none();
        let pos = global.translation().truncate();
        let is_hovered = cursor_world
            .map(|wp| (wp.x - pos.x).abs() <= 48.0 && (wp.y - pos.y).abs() <= 36.0)
            .unwrap_or(false);
        let can_afford = offer.map(|o| money.amount >= o.cost).unwrap_or(false);

        sprite.color = match (is_empty, is_hovered, can_afford) {
            (true, _, _) => Color::srgb(0.09, 0.10, 0.10),
            (false, true, true) => Color::srgb(0.30, 0.36, 0.24),
            (false, true, false) => Color::srgb(0.22, 0.13, 0.12),
            (false, false, _) => Color::srgb(0.15, 0.17, 0.16),
        };
    }

    for (icon, mut sprite, mut visibility) in &mut slots.p1() {
        let Some(offer) = shop.offers[icon.index] else {
            *visibility = Visibility::Hidden;
            continue;
        };

        sprite.color = offer.item.icon_color();
        *visibility = Visibility::Visible;
    }

    for (_, _, mut visibility) in &mut slots.p2() {
        *visibility = Visibility::Hidden;
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
    slots: Query<(&ShopSlot, &GlobalTransform)>,
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

    for (slot, global) in &slots {
        let slot_position = global.translation().truncate();
        let inside_slot = (world_position.x - slot_position.x).abs() <= 48.0
            && (world_position.y - slot_position.y).abs() <= 36.0;
        if !inside_slot {
            continue;
        }

        let Some(offer) = shop.offers[slot.index] else {
            return;
        };

        tooltip_text.0 = upgrade_tooltip(offer.item, offer.cost);
        *tooltip_visibility = Visibility::Visible;
        return;
    }
}


fn upgrade_tooltip(kind: ItemKind, cost: i32) -> String {
    let mut parts: Vec<String> = Vec::new();
    let effects = kind.effect_text();
    if !effects.is_empty() { parts.push(effects); }
    let desc = kind.description();
    if !desc.is_empty() { parts.push(desc.to_string()); }
    let tags = kind.tags_text();
    if !tags.is_empty() { parts.push(format!("Tags: {tags}")); }
    format!("{}  ${}\nPermanent upgrade\n{}", kind.name(), cost, parts.join("\n"))
}

fn apply_stat_upgrade(kind: ItemKind, stats: &mut PlayerStatsMut) {
    for effect in kind.effects() {
        stats.apply_tower_effect(*effect);
    }
}
