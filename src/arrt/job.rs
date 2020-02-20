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

    
}

// -----------------------------------------------------------------------------------------
