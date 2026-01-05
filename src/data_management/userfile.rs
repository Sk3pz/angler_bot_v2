use serde::{Deserialize, Serialize};
#[cfg(feature = "guild_relative_userdata")]
use serenity::all::GuildId;
use serenity::all::UserId;

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

#[derive(Clone)]
pub struct UserFile {
    pub user_id: UserId,
    pub file: UserValues,
}

impl UserFile {
    pub fn new(id: &UserId) -> Self {
        Self {
            user_id: id.clone(),
            file: UserValues {},
        }
    }

    #[cfg(feature = "guild_relative_userdata")]
    pub fn get_path(&self, guild_id: &GuildId) -> String {
        format!(
            "{}/guilds/{}/users/{}.json",
            DATA_DIR, guild_id, self.user_id
        )
    }

    #[cfg(not(feature = "guild_relative_userdata"))]
    pub fn get_path(&self) -> String {
        format!("{}/users/{}.json", DATA_DIR, self.user_id)
    }
}
