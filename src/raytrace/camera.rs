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
    pub far_distance: f32,
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
            far_distance: 10000.0,
            right,
            up,
            forward,
        }
    }
}

// -----------------------------------------------------------------------------------------
// Tracer
pub struct Tracer {
    image_width: u32,
    image_height: u32,
    near_width: f32,
    near_height: f32,
    far_width: f32,
    far_height: f32,
    near_bottom_left: Vec3,
    far_bottom_left: Vec3,
    right: Vec3,
    up: Vec3,
}

impl Tracer {
    pub fn new(camera: &Camera, image_width: u32, image_height: u32) -> Tracer {
        // Calculate aspect
        let aspect = image_width as f32 / image_height as f32;

        // Calculate frustum
        let frustum_mult = (camera.fov * 0.5).tan();
        let near_half_width = camera.near_distance * frustum_mult;
        let near_half_height = near_half_width / aspect;
        let far_half_width = camera.far_distance * frustum_mult;
        let far_half_height = far_half_width / aspect;

        // Calculate frustum bottom left corners
        let near_bottom_left: Vec3 = camera.position + (camera.forward * camera.near_distance)
            - (camera.right * near_half_width)
            - (camera.up * near_half_height);
        let far_bottom_left: Vec3 = camera.position + (camera.forward * camera.far_distance)
            - (camera.right * far_half_width)
            - (camera.up * far_half_height);

        // Return tracer object
        Tracer {
            image_width,
            image_height,
            near_width: near_half_width * 2.0,
            near_height: near_half_height * 2.0,
            far_width: far_half_width * 2.0,
            far_height: far_half_height * 2.0,
            near_bottom_left,
            far_bottom_left,
            right: camera.right,
            up: camera.up,
        }
    }

    pub fn get_ray(&self, pixel_x: u32, pixel_y: u32) -> Ray {
        let perc_x = pixel_x as f32 / self.image_width as f32;
        let perc_y = pixel_y as f32 / self.image_height as f32;

        let origin = self.near_bottom_left
            + (self.right * self.near_width * perc_x)
            + (self.up * self.near_height * perc_y);
        let ray_end = self.far_bottom_left
            + (self.right * self.far_width * perc_x)
            + (self.up * self.far_height * perc_y);

        let direction = Vec3::normalize(ray_end - origin);
        Ray { origin, direction }
    }
}

// -----------------------------------------------------------------------------------------
