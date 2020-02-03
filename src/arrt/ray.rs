// -----------------------------------------------------------------------------------------

use super::misc::StringLiteral;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------
// Ray Type
#[derive(Debug, Default, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

// -----------------------------------------------------------------------------------------
// Ray Constructor
impl Ray {
    pub const FORWARD: Ray = Ray {
        origin: Vec3::ZERO,
        direction: Vec3::FORWARD,
    };

    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
}

// -----------------------------------------------------------------------------------------
// Ray Members
impl Ray {
    pub fn get_point(&self, t: f32) -> Vec3 {
        self.origin + (self.direction * t)
    }
}

// -----------------------------------------------------------------------------------------
// RayHitResult Type
#[derive(Copy, Clone)]
pub struct RayHitResult {
    pub hit: bool,
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub material_name: StringLiteral,
}

// -----------------------------------------------------------------------------------------
// RayHitResult Constructor
impl RayHitResult {
    pub fn new(
        hit: bool,
        distance: f32,
        position: Vec3,
        normal: Vec3,
        material_name: StringLiteral,
    ) -> RayHitResult {
        RayHitResult {
            hit,
            distance,
            position,
            normal,
            material_name,
        }
    }
}

// -----------------------------------------------------------------------------------------
// RayHitResult Constants
impl RayHitResult {
    pub const NO_HIT: RayHitResult = RayHitResult {
        hit: false,
        distance: 0.0,
        position: Vec3::ZERO,
        normal: Vec3::UP,
        material_name: "debug",
    };

    pub const MAX_HIT: RayHitResult = RayHitResult {
        hit: false,
        distance: std::f32::MAX,
        position: Vec3::ZERO,
        normal: Vec3::UP,
        material_name: "debug",
    };
}

// -----------------------------------------------------------------------------------------
