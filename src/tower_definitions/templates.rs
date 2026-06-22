#![allow(dead_code)]

use bevy::math::primitives::{Circle, Rectangle, RegularPolygon};
use bevy::prelude::*;

// ── Base shape ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TowerShape {
    Rectangle,
    Triangle,
    Diamond,
    Pentagon,
    Hexagon,
    Circle,
}

impl TowerShape {
    pub fn into_mesh(self, size: Vec2) -> Mesh {
        let r = size.x * 0.5;
        let h = size.y * 0.5;
        match self {
            Self::Rectangle => Rectangle::new(size.x, size.y).into(),
            Self::Triangle  => Triangle2d::new(
                Vec2::new(-r, -h),
                Vec2::new(r, -h),
                Vec2::new(0.0, h),
            ).into(),
            Self::Diamond   => RegularPolygon::new(r, 4).into(),
            Self::Pentagon  => RegularPolygon::new(r, 5).into(),
            Self::Hexagon   => RegularPolygon::new(r, 6).into(),
            Self::Circle    => Circle::new(r).into(),
        }
    }

    pub fn is_rectangle(self) -> bool { self == Self::Rectangle }
}

// ── Base templates ────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct BaseTemplate {
    pub size: Vec2,
    pub shape: TowerShape,
}

// Rectangle bases
pub const BASE_LIGHT:    BaseTemplate = BaseTemplate { size: Vec2::new(32.0, 32.0), shape: TowerShape::Rectangle };
pub const BASE_STANDARD: BaseTemplate = BaseTemplate { size: Vec2::new(36.0, 36.0), shape: TowerShape::Rectangle };
pub const BASE_HEAVY:    BaseTemplate = BaseTemplate { size: Vec2::new(38.0, 38.0), shape: TowerShape::Rectangle };
pub const BASE_SIEGE:    BaseTemplate = BaseTemplate { size: Vec2::new(40.0, 40.0), shape: TowerShape::Rectangle };

// Triangle bases
pub const BASE_TRIANGLE_S: BaseTemplate = BaseTemplate { size: Vec2::new(34.0, 40.0), shape: TowerShape::Triangle };
pub const BASE_TRIANGLE_M: BaseTemplate = BaseTemplate { size: Vec2::new(38.0, 45.0), shape: TowerShape::Triangle };

// Diamond bases
pub const BASE_DIAMOND_S: BaseTemplate = BaseTemplate { size: Vec2::new(34.0, 34.0), shape: TowerShape::Diamond };
pub const BASE_DIAMOND_M: BaseTemplate = BaseTemplate { size: Vec2::new(38.0, 38.0), shape: TowerShape::Diamond };

// Pentagon bases
pub const BASE_PENTAGON_S: BaseTemplate = BaseTemplate { size: Vec2::new(36.0, 36.0), shape: TowerShape::Pentagon };
pub const BASE_PENTAGON_M: BaseTemplate = BaseTemplate { size: Vec2::new(40.0, 40.0), shape: TowerShape::Pentagon };

// Hexagon bases
pub const BASE_HEX_S: BaseTemplate = BaseTemplate { size: Vec2::new(34.0, 34.0), shape: TowerShape::Hexagon };
pub const BASE_HEX_M: BaseTemplate = BaseTemplate { size: Vec2::new(38.0, 38.0), shape: TowerShape::Hexagon };

// Circle bases
pub const BASE_CIRCLE_S: BaseTemplate = BaseTemplate { size: Vec2::new(32.0, 32.0), shape: TowerShape::Circle };
pub const BASE_CIRCLE_M: BaseTemplate = BaseTemplate { size: Vec2::new(36.0, 36.0), shape: TowerShape::Circle };

// ── Barrel templates ──────────────────────────────────────────────────────────
//
// `size` is (width, length). `offset` is how far the barrel centre sits above
// the tower centre. `spacing` on `Double` is centre-to-centre barrel separation.

#[derive(Clone, Copy)]
pub enum BarrelTemplate {
    None,
    Single { size: Vec2, offset: f32 },
    Double { size: Vec2, offset: f32, spacing: f32 },
}

impl BarrelTemplate {
    pub fn size(self) -> Vec2 {
        match self {
            Self::None => Vec2::ZERO,
            Self::Single { size, .. } | Self::Double { size, .. } => size,
        }
    }

    pub fn offset(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Single { offset, .. } | Self::Double { offset, .. } => offset,
        }
    }

    pub fn is_none(self) -> bool { matches!(self, Self::None) }
}

// Single barrels
pub const BARREL_NONE:     BarrelTemplate = BarrelTemplate::None;
pub const BARREL_SNIPER:   BarrelTemplate = BarrelTemplate::Single { size: Vec2::new(8.0,  48.0), offset: 20.0 };
pub const BARREL_LIGHT:    BarrelTemplate = BarrelTemplate::Single { size: Vec2::new(10.0, 28.0), offset: 12.0 };
pub const BARREL_STANDARD: BarrelTemplate = BarrelTemplate::Single { size: Vec2::new(12.0, 38.0), offset: 16.0 };
pub const BARREL_HEAVY:    BarrelTemplate = BarrelTemplate::Single { size: Vec2::new(14.0, 34.0), offset: 15.0 };
pub const BARREL_CANNON:   BarrelTemplate = BarrelTemplate::Single { size: Vec2::new(18.0, 30.0), offset: 13.0 };

// Double barrels (each barrel is slightly slimmer than its single counterpart)
pub const BARREL_DOUBLE_LIGHT:    BarrelTemplate = BarrelTemplate::Double { size: Vec2::new(7.0,  28.0), offset: 12.0, spacing: 12.0 };
pub const BARREL_DOUBLE_STANDARD: BarrelTemplate = BarrelTemplate::Double { size: Vec2::new(9.0,  38.0), offset: 16.0, spacing: 14.0 };
pub const BARREL_DOUBLE_HEAVY:    BarrelTemplate = BarrelTemplate::Double { size: Vec2::new(11.0, 34.0), offset: 15.0, spacing: 16.0 };
pub const BARREL_DOUBLE_CANNON:   BarrelTemplate = BarrelTemplate::Double { size: Vec2::new(13.0, 28.0), offset: 12.0, spacing: 18.0 };

// ── Color palettes ────────────────────────────────────────────────────────────

pub struct ColorPalette {
    pub base: Color,
    pub barrel: Color,
}

pub const PALETTE_BLUE:    ColorPalette = ColorPalette { base: Color::srgb(0.22, 0.42, 0.74), barrel: Color::srgb(0.67, 0.83, 0.96) };
pub const PALETTE_BRONZE:  ColorPalette = ColorPalette { base: Color::srgb(0.42, 0.36, 0.30), barrel: Color::srgb(0.74, 0.66, 0.54) };
pub const PALETTE_TEAL:    ColorPalette = ColorPalette { base: Color::srgb(0.20, 0.52, 0.46), barrel: Color::srgb(0.62, 0.92, 0.78) };
pub const PALETTE_VIOLET:  ColorPalette = ColorPalette { base: Color::srgb(0.34, 0.28, 0.56), barrel: Color::srgb(0.82, 0.76, 0.98) };
pub const PALETTE_EARTH:   ColorPalette = ColorPalette { base: Color::srgb(0.40, 0.34, 0.22), barrel: Color::srgb(0.66, 0.56, 0.36) };
pub const PALETTE_FOREST:  ColorPalette = ColorPalette { base: Color::srgb(0.20, 0.55, 0.18), barrel: Color::srgb(0.20, 0.55, 0.18) };
pub const PALETTE_CRIMSON: ColorPalette = ColorPalette { base: Color::srgb(0.60, 0.16, 0.14), barrel: Color::srgb(0.98, 0.56, 0.32) };
pub const PALETTE_GOLD:    ColorPalette = ColorPalette { base: Color::srgb(0.50, 0.40, 0.10), barrel: Color::srgb(0.96, 0.84, 0.34) };
pub const PALETTE_SLATE:   ColorPalette = ColorPalette { base: Color::srgb(0.28, 0.32, 0.38), barrel: Color::srgb(0.70, 0.76, 0.84) };
pub const PALETTE_SHADOW:  ColorPalette = ColorPalette { base: Color::srgb(0.14, 0.12, 0.18), barrel: Color::srgb(0.44, 0.38, 0.54) };
