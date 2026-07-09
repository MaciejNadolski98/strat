use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{
    Burning, DropsSpell, Enemy, Health, Reward, ShopTooltip, SpellSlot, SpellSlotIcon,
    SpellSlotLabel,
};
use crate::effects::spawn_floating_text;
use crate::resources::{GameOver, GameWon, KillCount, Money, Loot, SpellShop};
use crate::spell_definitions::SpellCastEvent;
use crate::tooltip::{plain, tag_segments};

pub fn update_spell_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut spell_shop: ResMut<SpellShop>,
    game_over: Res<GameOver>,
    game_won: Res<GameWon>,
    mut spell_cast: EventWriter<SpellCastEvent>,
) {
    if game_over.value || game_won.value {
        return;
    }

    let slot = if keyboard.just_pressed(KeyCode::KeyZ) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::KeyX) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        Some(2)
    } else {
        None
    };

    let Some(slot) = slot else {
        return;
    };
    let Some(spell) = spell_shop.take_spell(slot) else {
        return;
    };
    spell_cast.write(SpellCastEvent { kind: spell });
}

pub fn update_spell_slots(
    spell_shop: Res<SpellShop>,
    mut slots: ParamSet<(
        Query<(&SpellSlot, &mut Sprite)>,
        Query<(&SpellSlotIcon, &mut Sprite, &mut Visibility)>,
        Query<(&SpellSlotLabel, &mut Text2d)>,
    )>,
) {
    for (slot, mut sprite) in &mut slots.p0() {
        let is_empty = spell_shop.slots[slot.index].is_none();
        sprite.color = if is_empty {
            Color::srgb(0.08, 0.09, 0.10)
        } else {
            Color::srgb(0.12, 0.13, 0.14)
        };
    }

    for (icon, mut sprite, mut visibility) in &mut slots.p1() {
        let Some(spell) = spell_shop.slots[icon.index] else {
            *visibility = Visibility::Hidden;
            continue;
        };

        sprite.color = spell.icon_color();
        *visibility = Visibility::Visible;
    }

    for (label, mut text) in &mut slots.p2() {
        text.0 = if let Some(spell) = spell_shop.slots[label.index] {
            format!("{}\n{}", spell.name(), spell_key(label.index))
        } else {
            "Empty".to_string()
        };
    }
}

pub fn update_spell_tooltip(
    spell_shop: Res<SpellShop>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    slots: Query<(&SpellSlot, &GlobalTransform)>,
    mut tooltip: Query<(Entity, &mut Text, &mut Visibility), With<ShopTooltip>>,
    mut commands: Commands,
) {
    let Ok((tooltip_entity, mut tooltip_text, mut tooltip_visibility)) = tooltip.single_mut() else {
        return;
    };

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
        let inside_slot = (world_position.x - slot_position.x).abs() <= 46.0
            && (world_position.y - slot_position.y).abs() <= 38.0;
        if !inside_slot {
            continue;
        }

        let Some(spell) = spell_shop.slots[slot.index] else {
            return;
        };

        let tags = spell.tags();
        let mut segments = vec![plain(format!("{}\nOne use spell\n{}\n", spell.name(), spell.description()))];
        if tags.is_empty() {
            segments.push(plain(format!("Press {} to cast", spell_key(slot.index))));
        } else {
            segments.push(plain("Tags: "));
            segments.extend(tag_segments(tags));
            segments.push(plain(format!("\nPress {} to cast", spell_key(slot.index))));
        }
        crate::tooltip::set_tooltip_segments(&mut commands, tooltip_entity, &mut tooltip_text, segments);
        *tooltip_visibility = Visibility::Visible;
        return;
    }
}

pub fn update_burning_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut money: ResMut<Money>,
    mut kills: ResMut<KillCount>,
    loot: Res<Loot>,
    mut spell_shop: ResMut<SpellShop>,
    mut enemies: Query<(Entity, &Transform, &mut Health, &Reward, &mut Burning, Option<&DropsSpell>), With<Enemy>>,
) {
    for (entity, transform, mut health, reward, mut burning, drops_spell) in &mut enemies {
        if health.current <= 0.0 {
            continue;
        }

        burning.timer.tick(time.delta());
        burning.tick_timer.tick(time.delta());

        if burning.tick_timer.just_finished() {
            let hp_lost = burning.damage_per_tick.min(health.current).max(0.0);
            health.current -= burning.damage_per_tick;
            if hp_lost > 0.0 {
                spawn_floating_text(
                    &mut commands,
                    format!("-{:.0}", hp_lost),
                    transform.translation.truncate() + Vec2::new(0.0, 20.0),
                    Color::srgb(1.0, 0.34, 0.10),
                    20.0,
                );
            }

            if health.current <= 0.0 {
                let kill_loot = (reward.amount + loot.value().round() as i32).max(0);
                money.amount += kill_loot;
                kills.amount += 1;
                spawn_floating_text(
                    &mut commands,
                    format!("+${kill_loot}"),
                    transform.translation.truncate() + Vec2::new(34.0, 30.0),
                    Color::srgb(1.0, 0.86, 0.20),
                    19.0,
                );
                if drops_spell.is_some() {
                    spell_shop.store_random_spell();
                    spawn_floating_text(
                        &mut commands,
                        "Spell!".to_string(),
                        transform.translation.truncate() + Vec2::new(-20.0, 52.0),
                        Color::srgb(0.72, 0.30, 0.92),
                        22.0,
                    );
                }
                commands.entity(entity).despawn();
                continue;
            }
        }

        if burning.timer.finished() {
            commands.entity(entity).remove::<Burning>();
        }
    }
}

fn spell_key(index: usize) -> &'static str {
    match index {
        0 => "Z",
        1 => "X",
        _ => "C",
    }
}
