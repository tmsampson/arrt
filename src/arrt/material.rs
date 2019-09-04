// -----------------------------------------------------------------------------------------

use super::vector::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// -----------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Material {
    #[serde(default)]
    pub name: String,
    pub diffuse: Vec3,
    pub absorbed: f32,
}

// -----------------------------------------------------------------------------------------

type MaterialTable = HashMap<String, Material>;

// -----------------------------------------------------------------------------------------

pub struct MaterialBank {
    _name: String,
    materials: MaterialTable,
}

// -----------------------------------------------------------------------------------------

impl MaterialBank {
    // -------------------------------------------------------------------------------------

    pub fn load_from_file(file: &str) -> MaterialBank {
        // Load material bank file
        let data = fs::read_to_string(file)
            .expect(&format!("ERROR: Could not load materials file: '{}'", file));

        // Deserialise
        let mut materials: MaterialTable = serde_json::from_str(&data).unwrap();

        // Use JSON key names as material names
        for (key, value) in &mut materials {
            value.name = key.clone();
        }

        // Return bank
        MaterialBank {
            _name: String::from(file),
            materials,
        }
    }

    // -------------------------------------------------------------------------------------

    pub fn get(&self, name: &str) -> Material {
        match self.materials.get(name) {
            Some(material) => material.clone(),
            None => {
                println!("Failed to find material: {}", name);
                MaterialBank::get_default()
            }
        }
    }

    // -------------------------------------------------------------------------------------

    pub fn get_default() -> Material {
        Material {
            name: String::from("default"),
            absorbed: 0.3,
            diffuse: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }

    // -------------------------------------------------------------------------------------
}

// -----------------------------------------------------------------------------------------
