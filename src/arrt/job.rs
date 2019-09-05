// -----------------------------------------------------------------------------------------

use super::material::MaterialBank;
use super::quality::QualityPreset;

// -----------------------------------------------------------------------------------------

use rand::prelude::*;

// -----------------------------------------------------------------------------------------

type ImageBuffer = std::vec::Vec<[u8; 4]>;

// -----------------------------------------------------------------------------------------

pub struct Job<'a> {
    pub quality: &'a QualityPreset,
    pub materials: &'a MaterialBank,
    pub image_buffer: ImageBuffer,
    pub rng: StdRng,
    pub debug_normals: bool,
    pub debug_heatmap: bool,
}

// -----------------------------------------------------------------------------------------

impl<'a> Job<'a> {
    pub fn new(
        quality: &'a QualityPreset,
        materials: &'a MaterialBank,
        rng_seed: u64,
        debug_normals: bool,
        debug_heatmap: bool,
    ) -> Job<'a> {
        // Setup image buffer
        let pixel_count = quality.image_width * quality.image_height;
        let clear_colour = [0u8, 0u8, 0u8, 255u8];
        let image_buffer = vec![clear_colour; pixel_count as usize];

        // Setup rng
        let rng = SeedableRng::seed_from_u64(rng_seed);

        // Setup job
        Job {
            quality: &quality,
            materials: &materials,
            image_buffer: image_buffer,
            rng,
            debug_normals,
            debug_heatmap
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