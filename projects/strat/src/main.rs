use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 700.0;
const TOWER_COST: i32 = 40;
const STARTING_MONEY: i32 = 120;
const STARTING_LIVES: i32 = 20;
const TOWER_RANGE: f32 = 185.0;
const GRID_SIZE: f32 = 48.0;
const PATH_HALF_WIDTH: f32 = GRID_SIZE * 0.5;
const HUD_BUILD_LIMIT: f32 = WINDOW_HEIGHT * 0.5 - 90.0;

const PATH: [Vec2; 8] = [
    Vec2::new(-504.0, 216.0),
    Vec2::new(-264.0, 216.0),
    Vec2::new(-264.0, -120.0),
    Vec2::new(-24.0, -120.0),
    Vec2::new(-24.0, 120.0),
    Vec2::new(264.0, 120.0),
    Vec2::new(264.0, -168.0),
    Vec2::new(504.0, -168.0),
];

#[derive(Resource)]
struct Game {
    money: i32,
    lives: i32,
    kills: u32,
    game_over: bool,
}

#[derive(Resource)]
struct Wave {
    number: u32,
    remaining: u32,
    spawn_timer: Timer,
    next_wave_timer: Timer,
}

#[derive(Component)]
struct Tower {
    fire_cooldown: Timer,
    rotational_speed: f32,
}

#[derive(Component)]
struct Enemy {
    waypoint: usize,
    progress: f32,
    health: f32,
    max_health: f32,
    speed: f32,
    reward: i32,
}

#[derive(Component)]
struct Projectile {
    target: Entity,
    speed: f32,
    damage: f32,
}

#[derive(Component)]
struct HudText;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.07, 0.09, 0.11)))
        .insert_resource(Game {
            money: STARTING_MONEY,
            lives: STARTING_LIVES,
            kills: 0,
            game_over: false,
        })
        .insert_resource(Wave {
            number: 1,
            remaining: enemies_in_wave(1),
            spawn_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            next_wave_timer: Timer::from_seconds(2.5, TimerMode::Once),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Simple Tower Defense".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                progress_cooldown,
                place_tower,
                spawn_enemies,
                move_enemies,
                aim_towers,
                move_projectiles,
                update_enemy_colors,
                update_hud,
                restart_game,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.10, 0.15, 0.13),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
    ));

    spawn_grid(&mut commands);

    for segment in PATH.windows(2) {
        spawn_path_segment(&mut commands, segment[0], segment[1]);
    }

    commands.spawn((
        Sprite::from_color(Color::srgb(0.35, 0.13, 0.12), Vec2::new(52.0, 52.0)),
        Transform::from_translation(PATH[0].extend(0.0)),
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.12, 0.35, 0.36), Vec2::new(58.0, 58.0)),
        Transform::from_translation(PATH[PATH.len() - 1].extend(0.0)),
    ));

    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.94, 0.88)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
        HudText,
    ));
}

fn spawn_grid(commands: &mut Commands) {
    let grid_color = Color::srgba(0.68, 0.76, 0.70, 0.12);
    let min_x = -WINDOW_WIDTH * 0.5;
    let max_x = WINDOW_WIDTH * 0.5;
    let min_y = -WINDOW_HEIGHT * 0.5;
    let max_y = WINDOW_HEIGHT * 0.5;

    let mut x = snap_axis(min_x);
    while x <= max_x {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(1.0, WINDOW_HEIGHT)),
            Transform::from_translation(Vec3::new(x, 0.0, -8.0)),
        ));
        x += GRID_SIZE;
    }

    let mut y = snap_axis(min_y);
    while y <= max_y {
        commands.spawn((
            Sprite::from_color(grid_color, Vec2::new(WINDOW_WIDTH, 1.0)),
            Transform::from_translation(Vec3::new(0.0, y, -8.0)),
        ));
        y += GRID_SIZE;
    }
}

fn spawn_path_segment(commands: &mut Commands, start: Vec2, end: Vec2) {
    let delta = end - start;
    let midpoint = (start + end) * 0.5;
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.43, 0.39, 0.31),
            Vec2::new(
                delta.length() + PATH_HALF_WIDTH * 2.0,
                PATH_HALF_WIDTH * 2.0,
            ),
        ),
        Transform {
            translation: midpoint.extend(-2.0),
            rotation: Quat::from_rotation_z(delta.y.atan2(delta.x)),
            ..default()
        },
    ));
}

fn place_tower(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    towers: Query<&Transform, With<Tower>>,
    mut game: ResMut<Game>,
) {
    if game.game_over || !mouse.just_pressed(MouseButton::Left) || game.money < TOWER_COST {
        return;
    }

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
    let grid_position = snap_to_grid(world_position);

    if !is_buildable_cell(grid_position)
        || towers
            .iter()
            .any(|tower| tower.translation.truncate().distance(grid_position) < GRID_SIZE * 0.5)
    {
        return;
    }

    game.money -= TOWER_COST;
    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.22, 0.42, 0.74), Vec2::new(36.0, 36.0)),
            Transform::from_translation(grid_position.extend(2.0)),
            Tower {
                fire_cooldown: Timer::from_seconds(0.42, TimerMode::Once),
                rotational_speed: 1.5,
            },
        ))
        .with_child((
            Sprite::from_color(Color::srgb(0.67, 0.83, 0.96), Vec2::new(12.0, 38.0)),
            Transform::from_translation(Vec3::new(0.0, 16.0, 1.0)),
        ));
}

fn progress_cooldown(
    towers: Query<&mut Tower>,
    time: Res<Time>,
) {
    let delta = time.delta();
    for mut tower in towers {
        tower.fire_cooldown.tick(delta);
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut wave: ResMut<Wave>,
    game: Res<Game>,
    enemies: Query<(), With<Enemy>>,
) {
    if game.game_over {
        return;
    }

    if wave.remaining == 0 {
        if enemies.is_empty() {
            wave.next_wave_timer.tick(time.delta());
            if wave.next_wave_timer.just_finished() {
                wave.number += 1;
                wave.remaining = enemies_in_wave(wave.number);
                wave.next_wave_timer.reset();
            }
        }
        return;
    }

    wave.spawn_timer.tick(time.delta());
    if !wave.spawn_timer.just_finished() {
        return;
    }

    wave.remaining -= 1;
    let max_health = 55.0 + wave.number as f32 * 16.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.72, 0.22, 0.18), Vec2::new(26.0, 26.0)),
        Transform::from_translation(PATH[0].extend(3.0)),
        Enemy {
            waypoint: 1,
            progress: 0.0,
            health: max_health,
            max_health,
            speed: 58.0 + wave.number as f32 * 3.5,
            reward: 12 + wave.number as i32,
        },
    ));
}

fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
) {
    if game.game_over {
        return;
    }

    for (entity, mut transform, mut enemy) in &mut enemies {
        if enemy.health <= 0.0 {
            continue;
        }

        let target = PATH[enemy.waypoint];
        let position = transform.translation.truncate();
        let to_target = target - position;
        let step = enemy.speed * time.delta_secs();
        enemy.progress += step;

        if to_target.length() <= step {
            transform.translation = target.extend(3.0);
            enemy.waypoint += 1;
            if enemy.waypoint >= PATH.len() {
                commands.entity(entity).despawn();
                game.lives -= 1;
                if game.lives <= 0 {
                    game.game_over = true;
                }
            }
        } else {
            transform.translation += (to_target.normalize() * step).extend(0.0);
        }
    }
}

fn aim_towers(
    mut commands: Commands,
    mut towers: Query<(&mut Transform, &mut Tower)>,
    enemies: Query<(Entity, &Transform, &Enemy), Without<Tower>>,
    game: Res<Game>,
    time: Res<Time>,
) {
    if game.game_over {
        return;
    }

    for (mut tower_transform, mut tower) in &mut towers {
        let tower_position = tower_transform.translation.truncate();
        let Some((target, target_position)) = enemies
            .iter()
            .filter(|(_, _, enemy)| enemy.health > 0.0)
            .filter_map(|(entity, transform, enemy)| {
                let enemy_position = transform.translation.truncate();
                let distance = enemy_position.distance(tower_position);
                let progress = enemy.progress;
                (distance <= TOWER_RANGE).then_some((entity, enemy_position, progress))
            })
            .max_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(entity, position, _)| (entity, position)) else { continue };

        let direction = target_position - tower_position;
        let target_rotation = Quat::from_rotation_z(direction.y.atan2(direction.x) - PI / 2.0);
        let current_rotation = tower_transform.rotation;
        let step = time.delta_secs() * tower.rotational_speed;

        let ready_to_shoot = current_rotation.angle_between(target_rotation) <= step;
        tower_transform.rotation = tower_transform.rotation.rotate_towards(target_rotation, step);

        if ready_to_shoot && tower.fire_cooldown.finished() {
            tower.fire_cooldown.reset();
            commands.spawn((
                Sprite::from_color(Color::srgb(0.96, 0.84, 0.28), Vec2::new(10.0, 10.0)),
                Transform::from_translation(tower_position.extend(4.0)),
                Projectile {
                    target,
                    speed: 430.0,
                    damage: 24.0,
                },
            ));
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile), Without<Enemy>>,
    mut enemies: Query<(&Transform, &mut Enemy)>,
) {
    for (projectile_entity, mut projectile_transform, projectile) in &mut projectiles {
        let Ok((enemy_transform, mut enemy)) = enemies.get_mut(projectile.target) else {
            commands.entity(projectile_entity).despawn();
            continue;
        };

        if enemy.health <= 0.0 {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        let projectile_position = projectile_transform.translation.truncate();
        let enemy_position = enemy_transform.translation.truncate();
        let to_enemy = enemy_position - projectile_position;
        let step = projectile.speed * time.delta_secs();

        if to_enemy.length() <= step + 10.0 {
            enemy.health -= projectile.damage;
            commands.entity(projectile_entity).despawn();

            if enemy.health <= 0.0 {
                game.money += enemy.reward;
                game.kills += 1;
                commands.entity(projectile.target).despawn();
            }
        } else {
            projectile_transform.translation += (to_enemy.normalize() * step).extend(0.0);
        }
    }
}

fn update_enemy_colors(mut enemies: Query<(&Enemy, &mut Sprite)>) {
    for (enemy, mut sprite) in &mut enemies {
        let health_ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
        sprite.color = Color::srgb(0.95 - health_ratio * 0.25, 0.18 + health_ratio * 0.58, 0.16);
    }
}

fn update_hud(game: Res<Game>, wave: Res<Wave>, mut hud: Query<&mut Text, With<HudText>>) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };

    let status = if game.game_over {
        "Game over - press R to restart"
    } else {
        "Left click: place tower ($40)"
    };

    text.0 = format!(
        "Money: ${}   Lives: {}   Wave: {}   Kills: {}\n{}",
        game.money, game.lives, wave.number, game.kills, status
    );
}

fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut wave: ResMut<Wave>,
    towers: Query<Entity, With<Tower>>,
    enemies: Query<Entity, With<Enemy>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in towers
        .iter()
        .chain(enemies.iter())
        .chain(projectiles.iter())
    {
        commands.entity(entity).despawn();
    }

    game.money = STARTING_MONEY;
    game.lives = STARTING_LIVES;
    game.kills = 0;
    game.game_over = false;

    wave.number = 1;
    wave.remaining = enemies_in_wave(1);
    wave.spawn_timer.reset();
    wave.next_wave_timer.reset();
}

fn enemies_in_wave(wave: u32) -> u32 {
    8 + wave * 3
}

fn is_on_path(position: Vec2) -> bool {
    PATH.windows(2).any(|segment| {
        distance_to_segment(position, segment[0], segment[1]) < PATH_HALF_WIDTH + 18.0
    })
}

fn is_buildable_cell(position: Vec2) -> bool {
    let half_cell = GRID_SIZE * 0.5;
    position.x >= -WINDOW_WIDTH * 0.5 + half_cell
        && position.x <= WINDOW_WIDTH * 0.5 - half_cell
        && position.y >= -WINDOW_HEIGHT * 0.5 + half_cell
        && position.y <= HUD_BUILD_LIMIT
        && !is_on_path(position)
}

fn snap_to_grid(position: Vec2) -> Vec2 {
    Vec2::new(
        snap_to_cell_center(position.x),
        snap_to_cell_center(position.y),
    )
}

fn snap_axis(value: f32) -> f32 {
    (value / GRID_SIZE).round() * GRID_SIZE
}

fn snap_to_cell_center(value: f32) -> f32 {
    (value / GRID_SIZE).floor() * GRID_SIZE + GRID_SIZE * 0.5
}

fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared == 0.0 {
        return point.distance(start);
    }

    let t = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}
