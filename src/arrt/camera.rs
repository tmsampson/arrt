// -----------------------------------------------------------------------------------------

use super::ray::Ray;
use super::vector::Vec3;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

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
    eye: Vec3,                  // Camera position
    near_origin: Vec3,          // Bottom-left of furstum near-plane
    pixel_size: f32,            // World space
    frustum_right: Vec3,        // Frustum right axis
    frustum_up: Vec3,           // Frustup up axis
    pub initial_rays: Vec<Ray>, // Initial rays (multiple per-pixel)
}

impl Tracer {
    pub fn new(
        camera: &Camera,
        rng: &mut StdRng,
        image_width: u32,
        image_height: u32,
        samples_per_pixel: usize,
    ) -> Tracer {
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

        // Calculate initial ray count
        let initial_ray_count = image_width as usize * image_height as usize * samples_per_pixel;

        // Setup tracer
        let mut tracer = Tracer {
            eye: camera.position,
            near_origin,
            pixel_size: near_width / image_width as f32,
            frustum_up: camera.up,
            frustum_right: camera.right,
            initial_rays: vec![Ray::FORWARD; initial_ray_count],
        };

        // Cache off initial rays
        for pixel_y in 0..image_height {
            let pixel_y_f = pixel_y as f32;
            for pixel_x in 0..image_width {
                let pixel_x_f = pixel_x as f32;
                let pixel_index =
                    Tracer::get_pixel_index(pixel_x, pixel_y, image_width, samples_per_pixel);

                // Store centroid ray
                let ray = tracer.get_ray(pixel_x_f, pixel_y_f);
                tracer.initial_rays[pixel_index] = ray;

                // Generate random sampling offsets
                let additional_samples: usize = samples_per_pixel - 1;
                let mut sample_offsets_x = vec![0.0; additional_samples];
                let mut sample_offsets_y = vec![0.0; additional_samples];
                for sample_index in 0..additional_samples {
                    sample_offsets_x[sample_index] = rng.gen();
                    sample_offsets_y[sample_index] = rng.gen();
                }

                // Store additional sample rays
                for sample_index in 0..additional_samples {
                    let offset_x = (sample_offsets_x[sample_index] - 0.5) * 0.99;
                    let offset_y = (sample_offsets_y[sample_index] - 0.5) * 0.99;
                    let ray = tracer.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y);
                    tracer.initial_rays[pixel_index + sample_index + 1] = ray;
                }
            }
        }

        // Return tracer object
        tracer
    }

    pub fn get_pixel_index(x: u32, y: u32, image_width: u32, samples_per_pixel: usize) -> usize {
        (((y * image_width) + x) as usize) * samples_per_pixel
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
