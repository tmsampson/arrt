// -----------------------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::fs;

// -----------------------------------------------------------------------------------------

const QUALITY_PRESETS_FILE: super::misc::StringLiteral = "quality_presets.json";

// -----------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QualityPreset {
    pub name: String,
    pub image_width: u32,
    pub image_height: u32,
    pub samples_per_pixel: usize,
    pub max_bounces: u32,
}

// -----------------------------------------------------------------------------------------

fn get_default_preset() -> QualityPreset {
    QualityPreset {
        name: String::from("default"),
        image_width: 640,
        image_height: 480,
        samples_per_pixel: 16,
        max_bounces: 16,
    }
}

// -----------------------------------------------------------------------------------------

pub fn get_preset(name: String) -> QualityPreset {
    // Load presets file
    let data = fs::read_to_string(QUALITY_PRESETS_FILE).expect(&format!(
        "ERROR: Could not load quality presets file: '{}'",
        QUALITY_PRESETS_FILE
    ));

    // Parse presets file
    let presets: Vec<QualityPreset> = serde_json::from_str(&data).unwrap();

    // Find preset by name (or return default)
    let result = presets.iter().find(|&p| p.name == name);
    match result {
        Some(preset) => return preset.clone(),
        None => {
            println!(
                "WARNING: Failed to find quality preset: '{}', using 'default' instead",
                name
            );
            return get_default_preset();
        }
    };
}

// -----------------------------------------------------------------------------------------
