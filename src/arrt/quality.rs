// -----------------------------------------------------------------------------------------

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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

pub struct QualityPresetBank {
    _name: String,
    presets: QualityPresetTable,
}

// -----------------------------------------------------------------------------------------

impl QualityPresetBank
{
     // -------------------------------------------------------------------------------------

     pub fn load_from_file(file: &str) -> QualityPresetBank {
        // Load material bank file
        let data = fs::read_to_string(file)
            .expect(&format!("ERROR: Could not load quality presets file: '{}'", file));

        // Deserialise
        let mut presets: QualityPresetTable = serde_json::from_str(&data).unwrap();

        // Use JSON key names as preset names
        for (key, value) in &mut presets {
            value.name = key.clone();
        }

        // Return bank
        QualityPresetBank {
            _name: String::from(file),
            presets,
        }
    }

    // -----------------------------------------------------------------------------------------

    pub fn get(&self, name: &str) -> QualityPreset {
        match self.presets.get(name) {
            Some(preset) => preset.clone(),
            None => {
                println!("Failed to find preset: {}", name);
                QualityPresetBank::get_default()
            }
        }
    }

    // -------------------------------------------------------------------------------------

    pub fn get_default() -> QualityPreset {
        QualityPreset {
            name: String::from("default"),
            image_width: 640,
            image_height: 480,
            samples_per_pixel: 8,
            max_bounces: 8,
        }
    }

    // -------------------------------------------------------------------------------------
}

// -----------------------------------------------------------------------------------------
