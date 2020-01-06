// -----------------------------------------------------------------------------------------

use super::ray::Ray;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------
// Camera
pub struct Camera {
    pub position: Vec3,
    pub lookat: Vec3,
    pub fov: f32,
    pub near_distance: f32,
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
}

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
            right,
            up,
            forward,
        }
    }
}

// -----------------------------------------------------------------------------------------
// Tracer
pub struct Tracer {
    eye: Vec3,           // Camera position
    near_origin: Vec3,   // Bottom-left of furstum near-plane
    pixel_size: f32,     // World space
    frustum_right: Vec3,
    frustum_up: Vec3,
}

impl Tracer {
    pub fn new(camera: &Camera, image_width: u32, image_height: u32) -> Tracer {
        // Calculate aspect
        let aspect = image_width as f32 / image_height as f32;

        // Calculate frustum
        let near_half_width = camera.near_distance * (camera.fov * 0.5).to_radians().tan();
        let near_half_height = near_half_width / aspect;
        let near_width = near_half_width * 2.0;

        // Calculate frustum origin (bottom left corner)
        let near_origin: Vec3 = camera.position + (camera.forward * camera.near_distance)
            - (camera.right * near_half_width)
            - (camera.up * near_half_height);

        // Return tracer object
        Tracer {
            eye: camera.position,
            near_origin,
            pixel_size: near_width / image_width as f32,
            frustum_up: camera.up,
            frustum_right: camera.right,
        }
    }

    pub fn get_ray(&self, pixel_x: f32, pixel_y: f32) -> Ray {
        let centroid_offset = self.pixel_size * 0.5;
        let horizontal_offset = (pixel_x * self.pixel_size) + centroid_offset;
        let vertical_offset = (pixel_y * self.pixel_size) + centroid_offset;
        let near_position = self.near_origin
            + (self.frustum_right * horizontal_offset)
            + (self.frustum_up * vertical_offset);
        let direction = Vec3::normalize(near_position - self.eye);

        Ray {
            origin: self.eye,
            direction,
        }
    }
}

// -----------------------------------------------------------------------------------------
