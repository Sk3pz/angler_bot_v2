use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serenity::all::UserId;

use crate::{data_management::monetary::MonetaryAmount, hey};
use crate::fishing::inventory::Inventory;

const DATA_DIR: &str = "./data";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserValues {
    // stored user values here
    pub balance: MonetaryAmount,
    pub inventory: Inventory,
    pub caught_fish: Vec<String>,
    pub total_catches: u64,
    pub autobait: bool,
}

impl Default for UserValues {
    fn default() -> Self {
        Self {
            balance: MonetaryAmount::new(100.0),
            inventory: Inventory::default(),
            caught_fish: Vec::new(),
            total_catches: 0,
            autobait: false,
        }
    }
}

#[derive(Clone)]
pub struct UserFile {
    pub user_id: UserId,
    pub file: UserValues,
}

impl UserFile {

    pub fn new(id: &UserId) -> Self {
        Self {
            user_id: id.clone(),
            file: UserValues::default(),
        }
    }

    pub fn get_path(&self) -> String {
        format!("{}/users/{}.ron", DATA_DIR, self.user_id)
    }

    pub fn read(id: &UserId) -> Self {
        let default_values = UserValues::default();

        // create a new user file with default values
        let mut file = Self {
            user_id: id.clone(),
            file: default_values,
        };

        let raw_path = file.get_path();
        let path = Path::new(raw_path.as_str());

        // check if the file exists
        if !path.exists() {
            // file doesn't exist, return default values and generate new file
            Self::generate(id);
            return file;
        };

        // read the file
        let Ok(data) = fs::read_to_string(path) else {
            // invalid data, return default values
            Self::generate(id);
            return file;
        };

        file.file = ron::from_str(data.as_str())
            .expect(format!("failed to deserialize user data with ID {}", id).as_str());

        file
    }

    fn generate(id: &UserId) {
        let default_values = UserValues::default();

        // create a new user file with default values
        let default_file = Self {
            user_id: id.clone(),
            file: default_values,
        };

        let raw_path = default_file.get_path();
        let path = Path::new(raw_path.as_str());

        if path.exists() {
            hey!("User data already exists: {}", id);
            return;
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .open(path)
        else {
            hey!("Failed to get file for user data: {}", id);
            return;
        };

        let Ok(data) = ron::to_string(&default_file.file) else {
            hey!("Failed to serialize user data: {}", id.clone());
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for user {}: {}", id, e);
        }
    }

    pub fn reload(&mut self) {
        *self = Self::read(&self.user_id);
    }

    pub fn update(&self) {
        let raw_path = self.get_path();
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            #[cfg(not(feature = "guild_relative_userdata"))]
            Self::generate(&self.user_id);
        };

        let Ok(mut file) = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .truncate(true)
            .open(path)
        else {
            hey!("Failed to get file for user data: {}", &self.user_id);
            return;
        };

        let Ok(data) = ron::to_string(&self.file) else {
            hey!("Failed to serialize user data: {}", &self.user_id);
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for user {}: {}", &self.user_id, e);
        }
    }
}
