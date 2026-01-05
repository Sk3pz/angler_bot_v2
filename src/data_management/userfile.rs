use serde::{Deserialize, Serialize};

// if true, user data is stored in a directory relative to each guild, and will not carry across other guilds.
// set to false for global user data.
const GUILD_RELATIVE_USERDATA: bool = true;
const DATA_DIR: &str = "./data";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserValues {
    // stored user values here
}
