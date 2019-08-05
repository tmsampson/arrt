// -----------------------------------------------------------------------------------------

use super::vector::Vec3;
use super::StringLiteral;

// -----------------------------------------------------------------------------------------
// Ray Type
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

// -----------------------------------------------------------------------------------------
// Ray Constructor
impl Ray {
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
    pub material: StringLiteral,
}

// -----------------------------------------------------------------------------------------
// RayHitResult Constructor
impl RayHitResult {
    pub fn new(
        hit: bool,
        distance: f32,
        position: Vec3,
        normal: Vec3,
        material: StringLiteral,
    ) -> RayHitResult {
        RayHitResult {
            hit,
            distance,
            position,
            normal,
            material,
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
        material: "debug",
    };

    pub const MAX_HIT: RayHitResult = RayHitResult {
        hit: false,
        distance: std::f32::MAX,
        position: Vec3::ZERO,
        normal: Vec3::UP,
        material: "debug",
    };
}

// -----------------------------------------------------------------------------------------
