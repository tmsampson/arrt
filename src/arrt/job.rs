// -----------------------------------------------------------------------------------------

use super::camera::Camera;
use super::material::MaterialBank;
use super::quality::QualityPreset;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

// -----------------------------------------------------------------------------------------

type ImageBuffer = std::vec::Vec<[u8; 4]>;

// -----------------------------------------------------------------------------------------

pub struct Job {
    pub quality: QualityPreset,
    pub materials: MaterialBank,
    pub image_buffer: ImageBuffer,
    pub rng_seed: u64,
    pub rng: StdRng,
    pub camera: Camera,
    pub debug_normals: bool,
    pub debug_heatmap: bool,
}

// -----------------------------------------------------------------------------------------

impl Job {
    pub fn new(
        quality: QualityPreset,
        materials: MaterialBank,
        rng_seed: u64,
        mut camera: Camera,
        debug_normals: bool,
        debug_heatmap: bool,
    ) -> Job {
        // Setup image buffer
        let pixel_count = quality.image_width * quality.image_height;
        let clear_colour = [0u8, 0u8, 0u8, 255u8];
        let image_buffer = vec![clear_colour; pixel_count as usize];

        // Setup rng
        let mut rng = SeedableRng::seed_from_u64(rng_seed);

        // Cache camera rays
        camera.update_cached_rays(quality.image_width, quality.image_height, quality.samples_per_pixel, &mut rng);

        // Setup job
        Job {
            quality: quality,
            materials: materials,
            image_buffer: image_buffer,
            rng_seed,
            rng,
            camera,
            debug_normals,
            debug_heatmap,
        }
    }

    pub fn save_image(&self, filename: &str) {
        // Create bitmap
        let mut output_bmp = bmp::Image::new(self.quality.image_width, self.quality.image_height);

        // Copy image buffer to bitmap
        for x in 0..self.quality.image_width {
            for y in 0..self.quality.image_height {
                let pixel_index = ((self.quality.image_width * y) + x) as usize;
                let pixel = bmp::Pixel {
                    r: self.image_buffer[pixel_index][0],
                    g: self.image_buffer[pixel_index][1],
                    b: self.image_buffer[pixel_index][2],
                };
                output_bmp.set_pixel(x, self.quality.image_height - y - 1, pixel);
            }
        }

        // Save bitmap
        output_bmp.save(filename).expect("Failed");
    }
}

// -----------------------------------------------------------------------------------------
