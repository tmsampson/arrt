// -----------------------------------------------------------------------------------------

use super::vector::Vec3;

// -----------------------------------------------------------------------------------------
// Type
pub struct Camera {
    pub position: Vec3,
    pub lookat: Vec3,
    pub fov: f32,
    pub near_distance: f32,
    pub far_distance: f32,
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
}

// -----------------------------------------------------------------------------------------
// Constructor
impl Camera {
    pub fn new(position: Vec3, lookat: Vec3, fov: f32) -> Camera {
        // Calculate basis
        let forward = Vec3::normalize(lookat - position);
        let right = Vec3::normalize(Vec3::cross(Vec3::UP, forward));
        let up = Vec3::normalize(Vec3::cross(forward, right));

        // Construct
        Camera {
            position,
            lookat,
            fov,
            near_distance: 1.0,
            far_distance: 10000.0,
            right,
            up,
            forward,
        }
    }
}

// -----------------------------------------------------------------------------------------
