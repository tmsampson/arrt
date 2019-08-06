// -----------------------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// -----------------------------------------------------------------------------------------

const QUALITY_PRESETS_FILE: super::misc::StringLiteral = "quality_presets.json";

// -----------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QualityPreset {
    #[serde(default)]
    pub name: String,
    pub image_width: u32,
    pub image_height: u32,
    pub samples_per_pixel: usize,
    pub max_bounces: u32,
}

// -----------------------------------------------------------------------------------------

type QualityPresetTable = HashMap<String, QualityPreset>;

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

pub fn get_preset(name: &str) -> QualityPreset {
    // Load presets file
    let data = fs::read_to_string(QUALITY_PRESETS_FILE).expect(&format!(
        "ERROR: Could not load quality presets file: '{}'",
        QUALITY_PRESETS_FILE
    ));

    // Parse presets file
    let presets: QualityPresetTable = serde_json::from_str(&data).unwrap();

    // Find preset by name (or return default)
    match presets.get(name) {
        Some(preset) => {
            let mut result = preset.clone();
            result.name = String::from(name); // Use map key as name
            result
        }
        None => {
            println!(
                "WARNING: Failed to find quality preset: '{}', using 'default' instead",
                name
            );
            get_default_preset()
        }
    }
}

// -----------------------------------------------------------------------------------------
