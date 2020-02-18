// -----------------------------------------------------------------------------------------

use super::material::MaterialBank;
use super::quality::QualityPreset;

// -----------------------------------------------------------------------------------------

type ImageBuffer = std::vec::Vec<[u8; 4]>;

// -----------------------------------------------------------------------------------------

pub struct Job {
    pub quality: QualityPreset,
    pub materials: MaterialBank,
    pub debug_normals: bool,
    pub debug_heatmap: bool,
}

// -----------------------------------------------------------------------------------------

impl Job {
    pub fn new(
        quality: QualityPreset,
        materials: MaterialBank,
        debug_normals: bool,
        debug_heatmap: bool,
    ) -> Job {
        // Setup job
        Job {
            quality: quality,
            materials: materials,
            debug_normals,
            debug_heatmap,
        }
    }

    pub fn save_image(&self, filename: &str) {
        // Create bitmap
        let mut output_bmp = bmp::Image::new(self.quality.image_width, self.quality.image_height);

        // Copy image buffer to bitmap
        // for x in 0..self.quality.image_width {
        //     for y in 0..self.quality.image_height {
        //         let pixel_index = ((self.quality.image_width * y) + x) as usize;
        //         let pixel = bmp::Pixel {
        //             r: self.image_buffer[pixel_index][0],
        //             g: self.image_buffer[pixel_index][1],
        //             b: self.image_buffer[pixel_index][2],
        //         };
        //         output_bmp.set_pixel(x, self.quality.image_height - y - 1, pixel);
        //     }
        // }

        // Save bitmap
        output_bmp.save(filename).expect("Failed");
    }
}

// -----------------------------------------------------------------------------------------
