use bevy::prelude::*;
use bevy::math::primitives::Circle;

use crate::components::{ExplosionEffect, FloatingText, PulseEffect};

pub fn spawn_floating_text(
    commands: &mut Commands,
    text: impl Into<String>,
    position: Vec2,
    color: Color,
    font_size: f32,
) {
    let spawn_offset = Vec2::new(
        rand::random::<f32>() * 28.0 - 14.0,
        rand::random::<f32>() * 18.0 - 9.0,
    );

    commands.spawn((
        Text2d::new(text.into()),
        TextFont {
            font_size,
            ..default()
        },
        TextColor(color),
        TextShadow::default(),
        Transform::from_translation((position + spawn_offset).extend(8.0)),
        FloatingText {
            lifetime: Timer::from_seconds(0.85, TimerMode::Once),
            velocity: Vec3::new(0.0, 46.0, 0.0),
        },
    ));
}

pub fn spawn_explosion_effect(
    commands: &mut Commands,
    position: Vec2,
    radius: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(radius))),
        MeshMaterial2d(materials.add(Color::srgba(0.45, 0.45, 0.45, 0.78))),
        Transform::from_translation(position.extend(7.5)),
        ExplosionEffect {
            lifetime: Timer::from_seconds(0.38, TimerMode::Once),
        },
    ));
}

pub fn update_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut texts: Query<(Entity, &mut FloatingText, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut floating, mut transform, mut text_color) in &mut texts {
        floating.lifetime.tick(time.delta());
        transform.translation += floating.velocity * time.delta_secs();

        let alpha = 1.0 - floating.lifetime.fraction();
        text_color.0 = text_color.0.with_alpha(alpha);

        if floating.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_pulse(
    commands: &mut Commands,
    position: Vec2,
    max_radius: f32,
    color: Color,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(position.extend(7.5)).with_scale(Vec3::splat(4.0)),
        PulseEffect {
            lifetime: Timer::from_seconds(0.55, TimerMode::Once),
            max_radius,
        },
    ));
}

pub fn update_pulses(
    mut commands: Commands,
    time: Res<Time>,
    mut pulses: Query<(
        Entity,
        &mut PulseEffect,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut pulse, mut transform, material) in &mut pulses {
        pulse.lifetime.tick(time.delta());
        let progress = pulse.lifetime.fraction();

        let scale = 4.0 + (pulse.max_radius - 4.0) * progress;
        transform.scale = Vec3::splat(scale);

        if let Some(mat) = materials.get_mut(material) {
            let alpha = mat.color.alpha() * (1.0 - progress);
            mat.color = mat.color.with_alpha(alpha);
        }

        if pulse.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_explosion_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(
        Entity,
        &mut ExplosionEffect,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut explosion, material) in &mut explosions {
        explosion.lifetime.tick(time.delta());
        let progress = explosion.lifetime.fraction();

        if let Some(material) = materials.get_mut(material) {
            material.color = material.color.with_alpha(0.78 * (1.0 - progress));
        }

        if explosion.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
