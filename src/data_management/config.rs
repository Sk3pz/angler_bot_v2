use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // configuration values here
}

impl Config {
    pub fn load() -> Self {
        let raw_path = "./data/config.data".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load config: file does not exist");
        }

        let contents = std::fs::read_to_string(path).unwrap();

        ron::from_str(contents.as_str()).unwrap()
    }
}
