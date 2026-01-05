use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};
#[cfg(feature = "guild_relative_userdata")]
use serenity::all::GuildId;
use serenity::all::UserId;

use crate::hey;

const DATA_DIR: &str = "./data";

/**
 *  NOTE: If the feature "guild_relative_userdata" is enabled,
 *  user data files will be stored in a guild-relative path:
 *  ./data/guilds/{guild_id}/users/{user_id}.json
 *  Otherwise, they will be stored in a global path:
 *  ./data/users/{user_id}.json
 *
 * To run in guild-relative mode, enable the flag on run: `cargo run --features guild_relative_userdata`
 *   or add `default = ["guild_relative_userdata"]` to the [features] section of Cargo.toml
**/

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserValues {
    // stored user values here
}

impl Default for UserValues {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct UserFile {
    pub user_id: UserId,
    pub file: UserValues,
    #[cfg(feature = "guild_relative_userdata")]
    pub guild_id: GuildId,
}

impl UserFile {
    #[cfg(feature = "guild_relative_userdata")]
    pub fn new(id: &UserId, guild_id: &GuildId) -> Self {
        Self {
            user_id: id.clone(),
            file: UserValues {},
            guild_id: guild_id.clone(),
        }
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
    pub fn new(id: &UserId) -> Self {
        Self {
            user_id: id.clone(),
            file: UserValues {},
        }
    }

    #[cfg(feature = "guild_relative_userdata")]
    pub fn get_path(&self) -> String {
        format!(
            "{}/guilds/{}/users/{}.json",
            DATA_DIR, self.guild_id, self.user_id
        )
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
    pub fn get_path(&self) -> String {
        format!("{}/users/{}.json", DATA_DIR, self.user_id)
    }

    #[cfg(feature = "guild_relative_userdata")]
    fn read(id: &UserId, guild: &GuildId) -> Self {
        let default_values = UserValues::default();

        // create a new user file with default values
        let mut file = Self {
            user_id: id.clone(),
            guild_id: guild.clone(),
            file: default_values,
        };

        let raw_path = file.get_path();
        let path = Path::new(raw_path.as_str());

        // check if the file exists
        if !path.exists() {
            // file doesn't exist, return default values and generate new file
            Self::generate(id, guild);
            return file;
        };

        // read the file
        let Ok(data) = fs::read_to_string(path) else {
            // invalid data, return default values
            Self::generate(id, guild);
            return file;
        };

        file.file = serde_json::from_str(data.as_str())
            .expect(format!("failed to deserialize user data with ID {}", id).as_str());

        file
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
    fn read(id: &UserId) -> Self {
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

        file.file = serde_json::from_str(data.as_str())
            .expect(format!("failed to deserialize user data with ID {}", id).as_str());

        file
    }

    #[cfg(feature = "guild_relative_userdata")]
    fn generate(id: &UserId, guild: &GuildId) {
        let default_values = UserValues::default();

        // create a new user file with default values
        let default_file = Self {
            user_id: id.clone(),
            guild_id: guild.clone(),
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

        let Ok(data) = serde_json::to_string(&default_file.file) else {
            hey!("Failed to serialize user data: {}", id.clone());
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for user {}: {}", id, e);
        }
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
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

        let Ok(data) = serde_json::to_string(&default_file.file) else {
            hey!("Failed to serialize user data: {}", id.clone());
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for user {}: {}", id, e);
        }
    }

    #[cfg(feature = "guild_relative_userdata")]
    fn reload(&mut self) {
        *self = Self::read(&self.user_id, &self.guild_id);
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
    fn reload(&mut self) {
        *self = Self::read(&self.user_id);
    }

    fn update(&self) {
        let raw_path = self.get_path();
        let path = Path::new(raw_path.as_str());

        if !path.exists() {
            #[cfg(feature = "guild_relative_userdata")]
            Self::generate(&self.user_id, &self.guild_id);
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

        let Ok(data) = serde_json::to_string(&self.file) else {
            hey!("Failed to serialize user data: {}", &self.user_id);
            return;
        };

        if let Err(e) = write!(file, "{}", data) {
            hey!("Failed to write to file for user {}: {}", &self.user_id, e);
        }
    }

    // TODO: implement variable specific methods here
}
