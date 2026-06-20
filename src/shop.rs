use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    ShopSlot, ShopSlotBarrel, ShopSlotIcon, ShopSlotLabel, ShopText, ShopTooltip,
};
use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::effects::spawn_floating_text;
use crate::resources::{
    AirDamage, AttackSpeed, CriticalChance, CurrentHp, EarthDamage, ExplosionSize, FireDamage,
    GameOver, MaxHp, Money, PassiveIncome, PlayerStatKind, Regeneration, Shop, SpellKind,
    SpellShop, StatUpgradeKind, TowerDraft, TowerDraftPhase, TowerStatEffect, WaterDamage,
    WaveNumber,
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

impl<'w> PlayerStatsMut<'w> {
    pub fn attack_speed_value(&self) -> f32 {
        self.attack_speed.value
    }

    pub fn apply_tower_effect(&mut self, effect: TowerStatEffect) {
        let delta = effect.amount;
        match effect.kind {
            PlayerStatKind::MaxHp => {
                let change = delta.round() as i32;
                self.max_hp.amount = (self.max_hp.amount + change).max(1);
                self.current_hp.amount =
                    (self.current_hp.amount + change).clamp(0, self.max_hp.amount);
            }
            PlayerStatKind::Regeneration => {
                self.regeneration.amount = (self.regeneration.amount + delta.round() as i32).max(0);
            }
            PlayerStatKind::AttackSpeed => {
                self.attack_speed.value = (self.attack_speed.value + delta).max(0.1);
            }
            PlayerStatKind::PassiveIncome => {
                self.passive_income.amount =
                    (self.passive_income.amount + delta.round() as i32).max(0);
            }
            PlayerStatKind::CriticalChance => {
                self.critical_chance.value = (self.critical_chance.value + delta).clamp(0.0, 1.0);
            }
            PlayerStatKind::ExplosionSize => {
                self.explosion_size.value = (self.explosion_size.value + delta).max(0.0);
            }
            PlayerStatKind::EarthDamage => {
                self.earth_damage.value = (self.earth_damage.value + delta).max(0.0);
            }
            PlayerStatKind::FireDamage => {
                self.fire_damage.value = (self.fire_damage.value + delta).max(0.0);
            }
            PlayerStatKind::AirDamage => {
                self.air_damage.value = (self.air_damage.value + delta).max(0.0);
            }
            PlayerStatKind::WaterDamage => {
                self.water_damage.value = (self.water_damage.value + delta).max(0.0);
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
    shop_slots: Query<(&ShopSlot, &Transform)>,
    mut shop: ResMut<Shop>,
    mut money: ResMut<Money>,
    game_over: Res<GameOver>,
    wave_number: Res<WaveNumber>,
    draft: Res<TowerDraft>,
    mut spell_shop: ResMut<SpellShop>,
    mut stats: PlayerStatsMut,
) {
    if game_over.value || draft.phase == TowerDraftPhase::Picking {
        return;
    }

    shop.update_prices_for_wave(wave_number.value);

    let mut selected = None;

    if mouse.just_pressed(MouseButton::Left) {
        if let (Ok(window), Ok((cam, cam_transform))) = (windows.single(), camera.single()) {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Ok(world_pos) = cam.viewport_to_world_2d(cam_transform, cursor_pos) {
                    for (slot, transform) in &shop_slots {
                        let pos = transform.translation.truncate();
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

        if let Some(upgrade) = offer.item.stat_upgrade_kind() {
            money.amount -= offer.cost;
            shop.take_offer(selected);
            apply_stat_upgrade(upgrade, &mut stats);
            spawn_floating_text(
                &mut commands,
                format!("-${}", offer.cost),
                Vec2::new(-WINDOW_WIDTH * 0.5 + 420.0, -WINDOW_HEIGHT * 0.5 + 72.0),
                Color::srgb(1.0, 0.86, 0.20),
                20.0,
            );
            return;
        }

        if let Some(spell) = offer.item.spell_kind() {
            if !spell_shop.store_spell(spell) {
                return;
            }

            money.amount -= offer.cost;
            shop.take_offer(selected);
            spawn_floating_text(
                &mut commands,
                format!("-${}", offer.cost),
                Vec2::new(-WINDOW_WIDTH * 0.5 + 420.0, -WINDOW_HEIGHT * 0.5 + 72.0),
                Color::srgb(1.0, 0.86, 0.20),
                20.0,
            );
        }
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
        "Shop     click: buy item     E: reroll ${}",
        shop.reroll_cost
    );

    for (slot, mut sprite) in &mut slots.p0() {
        sprite.color = if shop.offers[slot.index].is_none() {
            Color::srgb(0.09, 0.10, 0.10)
        } else {
            Color::srgb(0.15, 0.17, 0.16)
        };
    }

    for (icon, mut sprite, mut visibility) in &mut slots.p1() {
        let Some(offer) = shop.offers[icon.index] else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let Some(kind) = offer.item.stat_upgrade_kind() {
            sprite.color = kind.icon_color();
            *visibility = Visibility::Visible;
        } else if let Some(kind) = offer.item.spell_kind() {
            sprite.color = kind.icon_color();
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
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

        tooltip_text.0 = match offer.item.stat_upgrade_kind() {
            Some(kind) => upgrade_tooltip(kind, offer.cost),
            None => match offer.item.spell_kind() {
                Some(kind) => spell_tooltip(kind, offer.cost),
                None => offer.item.name().to_string(),
            },
        };
        *tooltip_visibility = Visibility::Visible;
        return;
    }
}


fn upgrade_tooltip(kind: StatUpgradeKind, cost: i32) -> String {
    format!(
        "{}  ${}\nPermanent upgrade\n{}",
        kind.name(),
        cost,
        kind.effect_text()
    )
}

fn spell_tooltip(kind: SpellKind, cost: i32) -> String {
    format!(
        "{}  ${}\nOne use spell\n{}\nBuys into the first free spell slot",
        kind.name(),
        cost,
        kind.description()
    )
}

fn apply_stat_upgrade(kind: StatUpgradeKind, stats: &mut PlayerStatsMut) {
    for effect in kind.effects() {
        stats.apply_tower_effect(*effect);
    }
}
