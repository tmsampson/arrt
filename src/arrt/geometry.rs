// -----------------------------------------------------------------------------------------

use super::misc::StringLiteral;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------
// Sphere Type
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
    pub material: StringLiteral,
}

// -----------------------------------------------------------------------------------------
// Plane Type
pub struct Plane {
    pub position: Vec3,
    pub normal: Vec3,
    pub diffuse: Vec3,
}

// -----------------------------------------------------------------------------------------
