use bevy::prelude::*;

use crate::components::{CustomTooltip, DamageFormula, DefaultAim, DefaultFire};
use crate::game::game_is_running;
use crate::resources::{FireDamage, PlayerStatKind, SpellShop, TowerDraft, TowerDraftPhase, TowerStatEffect};
use crate::tags;
use crate::tooltip::{colored, plain};
use crate::towers::FIRE_COLOR;
use crate::tower_definitions::TowerKind;
use super::{TowerDefinition, TooltipConfig, TowerRegistry};
use super::templates::{BASE_PENTAGON_M, BARREL_NONE};

#[derive(Component)]
pub struct CatalystTower {
    pub progress: f32,
}

#[derive(Component)]
pub struct CatalystProgressBar {
    pub owner: Entity,
    pub width: f32,
    pub fill: bool,
}

pub struct CatalystPlugin;

impl Plugin for CatalystPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<TowerRegistry>().kinds.push(KIND);
        app.add_systems(Update, attach_catalyst_marker.run_if(game_is_running));
        app.add_systems(Update, generate_spell.run_if(game_is_running));
        app.add_systems(Update, update_catalyst_progress_bar);
        app.add_systems(Update, update_catalyst_tooltip);
    }
}

pub const TOWER_CATALYST: TowerDefinition = TowerDefinition {
    name: "Catalyst",
    range: 0.0,
    cooldown: 999.0,
    damage_formula: DamageFormula {
        flat: 0,
        crit_multiplier: 1.0,
        earth_multiplier: 0.0,
        fire_multiplier: 0.0,
        air_multiplier: 0.0,
        water_multiplier: 0.0,
    },
    projectile_speed: 0.0,
    explosion_radius: 0.0,
    angular_speed: 0.0,
    spread: 0.0,
    piercing: 0,
    piercing_damage: 0.0,
    base_color: Color::srgb(0.95, 0.72, 0.12),
    barrel_color: Color::srgb(0.95, 0.72, 0.12),
    base: BASE_PENTAGON_M,
    barrel: BARREL_NONE,
    stat_effects: &[TowerStatEffect::new(PlayerStatKind::FireDamage, 2.0)],
    tooltip_config: TooltipConfig::UTILITY,
    tags: &[tags::INFERNAL],
};

pub const KIND: TowerKind = TowerKind(&TOWER_CATALYST);

pub fn catalyst_seconds_per_spell(fire: f32) -> f32 {
    20.0 / (0.2 + 1.8 * fire / 100.0)
}

fn attach_catalyst_marker(
    mut commands: Commands,
    new_towers: Query<(Entity, &TowerKind), Added<TowerKind>>,
) {
    for (entity, kind) in &new_towers {
        if *kind == KIND {
            commands
                .entity(entity)
                .insert((CatalystTower { progress: 0.0 }, CustomTooltip::default()))
                .remove::<(DefaultAim, DefaultFire)>()
                .with_children(|parent| {
                    let width = 40.0;
                    let height = 5.0;
                    let y = -28.0;
                    parent.spawn((
                        Sprite::from_color(Color::srgb(0.05, 0.06, 0.06), Vec2::new(width + 2.0, height + 2.0)),
                        Transform::from_translation(Vec3::new(0.0, y, 1.0)),
                        CatalystProgressBar { owner: entity, width, fill: false },
                    ));
                    parent.spawn((
                        Sprite::from_color(Color::srgb(0.95, 0.72, 0.12), Vec2::new(width, height)),
                        Transform::from_translation(Vec3::new(0.0, y, 2.0)),
                        CatalystProgressBar { owner: entity, width, fill: true },
                    ));
                });
        }
    }
}

fn generate_spell(
    time: Res<Time>,
    draft: Res<TowerDraft>,
    mut catalyst_towers: Query<&mut CatalystTower>,
    mut spell_shop: ResMut<SpellShop>,
    fire_damage: Res<FireDamage>,
) {
    if draft.phase != TowerDraftPhase::WaveRunning {
        return;
    }
    let seconds_per_spell = catalyst_seconds_per_spell(fire_damage.value());
    let progress_per_second = 1.0 / seconds_per_spell;
    for mut catalyst in &mut catalyst_towers {
        catalyst.progress += progress_per_second * time.delta_secs();
        while catalyst.progress >= 1.0 {
            catalyst.progress -= 1.0;
            spell_shop.store_random_spell();
        }
    }
}

fn update_catalyst_progress_bar(
    towers: Query<&CatalystTower>,
    mut bars: Query<(&CatalystProgressBar, &mut Transform, &mut Sprite)>,
) {
    for (bar, mut transform, mut sprite) in &mut bars {
        let Ok(catalyst) = towers.get(bar.owner) else { continue; };
        if bar.fill {
            let progress = catalyst.progress.clamp(0.0, 1.0);
            transform.scale.x = progress;
            transform.translation.x = -bar.width * (1.0 - progress) * 0.5;
            sprite.color = Color::srgb(0.95, 0.72, 0.12);
        }
    }
}

fn update_catalyst_tooltip(
    fire_damage: Res<FireDamage>,
    mut towers: Query<(&mut CustomTooltip, &CatalystTower)>,
) {
    let seconds_per_spell = catalyst_seconds_per_spell(fire_damage.value());
    let static_extras = vec![
        plain("Generates a spell every "),
        colored(format!("{seconds_per_spell:.1}s"), FIRE_COLOR),
        plain("\n(20 / (0.2 + "),
        colored("fire", FIRE_COLOR),
        plain(" x 1.8%) s/spell)"),
    ];
    for (mut tooltip, catalyst) in &mut towers {
        let pct = catalyst.progress * 100.0;
        let mut segments = static_extras.clone();
        segments.push(plain(format!("\nProgress: {pct:.0}%")));
        tooltip.0 = segments;
    }
}
