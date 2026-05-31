use bevy::prelude::*;

use crate::components::FloatingText;

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
