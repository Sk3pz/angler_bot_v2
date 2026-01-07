use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ValueCalculationType {
    Averaged,
    Multiplicitive,
}

// general section of the config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct General {
    pub motd: String,
}

// fishing section of the config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fishing {
    pub fish_value_calculation: ValueCalculationType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub fishing: Fishing,
}

impl Config {
    pub fn load() -> Self {
        let raw_path = "./data/config.toml".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load config: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        toml::from_str(contents.as_str()).unwrap()
    }

    pub fn save(&self) {
        let raw_path = "./data/config.toml".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        let config_string = toml::to_string(self).unwrap();

        if let Err(e) = std::fs::write(path, config_string) {
            panic!("Failed to write config: {}", e);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: General {
                motd: "Welcome to Angler Bot!".to_string(),
            },
            fishing: Fishing {
                fish_value_calculation: ValueCalculationType::Multiplicitive,
            },
        }
    }
}
