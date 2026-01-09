use serde::{Deserialize, Serialize};
use crate::nay;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ValueCalculationType {
    Averaged,
    Multiplicative,
}

// general section of the config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct General {
    pub motd: String,
    pub log_cast_data: bool,
}

// fishing section of the config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fishing {
    pub fish_weight_time_multiplier: f32,
    pub base_catch_chance: f32,
    pub base_cast_wait: f32,
    pub min_cast_wait: f32,
    pub max_cast_time_variation: f32,
    pub base_qte_time: f32,
    pub min_qte_time: f32,
    pub fish_value_calculation: ValueCalculationType,
}

// bait section of the config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaitConfig {
    pub low_bait_weight: f32,
    pub medium_bait_weight: f32,
    pub high_bait_weight: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub fishing: Fishing,
    pub bait: BaitConfig,
}

impl Config {
    pub fn load() -> Self {
        let raw_path = "./data/config.toml".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            nay!("Config file does not exist at path: {}", raw_path);
            return Self::default();
        }

        let contents = std::fs::read_to_string(path).unwrap();

        toml::from_str(contents.as_str()).unwrap()
    }

    pub fn save(&self) {
        let raw_path = "./data/config.toml".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        let config_string = toml::to_string(self).unwrap();

        if let Err(e) = std::fs::write(path, config_string) {
            nay!("Failed to write config: {}", e);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General {
                motd: "Welcome to Angler Bot!".to_string(),
                log_cast_data: false,
            },
            fishing: Fishing {
                fish_weight_time_multiplier: 1.2,
                base_catch_chance: 0.5,
                base_cast_wait: 20.0,
                min_cast_wait: 3.0,
                max_cast_time_variation: 5.0,
                base_qte_time: 15.0,
                min_qte_time: 2.0,
                fish_value_calculation: ValueCalculationType::Multiplicative,
            },
            bait: BaitConfig {
                low_bait_weight: 1.5,
                medium_bait_weight: 3.5,
                high_bait_weight: 5.0,
            },
        }
    }
}
