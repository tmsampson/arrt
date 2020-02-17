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
    pub cached_rays: Vec<Ray>,
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
            cached_rays: Vec::new(),
        }
    }

    pub fn get_pixel_index(x: u32, y: u32, image_width: u32, samples_per_pixel: usize) -> usize {
        (((y * image_width) + x) as usize) * samples_per_pixel
    }

    pub fn update_cached_rays(&mut self, image_width: u32, image_height: u32, samples_per_pixel: usize, rng: &mut StdRng) {
        // Calculate aspect
        let aspect = image_width as f32 / image_height as f32;

        // Calculate frustum
        let near_half_width = self.near_distance * (self.fov * 0.5).to_radians().tan();
        let near_half_height = near_half_width / aspect;
        let near_width = near_half_width * 2.0;

        // Calculate frustum origin (bottom left corner)
        let near_origin: Vec3 = self.position + (self.forward * self.near_distance)
            - (self.right * near_half_width)
            - (self.up * near_half_height);

        // Calculate pixel size
        let pixel_size = near_width / image_width as f32;

        // Calculate cached ray count
        let cached_ray_count = image_width as usize * image_height as usize * samples_per_pixel;
        self.cached_rays.resize(cached_ray_count, Ray::FORWARD);

         // Cache off rays
         for pixel_y in 0..image_height {
            let pixel_y_f = pixel_y as f32;
            for pixel_x in 0..image_width {
                let pixel_x_f = pixel_x as f32;
                let pixel_index =
                    Camera::get_pixel_index(pixel_x, pixel_y, image_width, samples_per_pixel);

                // Store centroid ray
                let ray = self.get_ray(pixel_x_f, pixel_y_f, near_origin, pixel_size);
                self.cached_rays[pixel_index] = ray;

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
                    let ray = self.get_ray(pixel_x_f + offset_x, pixel_y_f + offset_y, near_origin, pixel_size);
                    self.cached_rays[pixel_index + sample_index + 1] = ray;
                }
            }
        }
    }

    pub fn get_ray(&self, pixel_x: f32, pixel_y: f32, near_origin: Vec3, pixel_size: f32) -> Ray {
        let centroid_offset = pixel_size * 0.5;
        let horizontal_offset = (pixel_x * pixel_size) + centroid_offset;
        let vertical_offset = (pixel_y * pixel_size) + centroid_offset;
        let near_position = near_origin
            + (self.right * horizontal_offset)
            + (self.up * vertical_offset);
        let direction = Vec3::normalize(near_position - self.position);

        Ray {
            origin: self.position,
            direction,
        }
    }
}

// -----------------------------------------------------------------------------------------
