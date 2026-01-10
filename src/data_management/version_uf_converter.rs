use std::fs;
use serde::{Deserialize, Serialize};
use serenity::all::UserId;
use crate::data_management::monetary::MonetaryAmount;
use crate::data_management::userfile::{UserFile, UserValues};
use crate::fishing::bait_bucket::BaitBucket;
use crate::fishing::rod_data::RodLoadout;
use crate::nay;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OLD_UserValues {
    // stored user values here
    pub balance: MonetaryAmount,
    pub loadout: RodLoadout,
    pub caught_fish: Vec<String>,
    pub total_catches: u64,
    pub bait_bucket: BaitBucket,
}

impl Default for OLD_UserValues {
    fn default() -> Self {
        Self {
            balance: MonetaryAmount::new(100.0),
            loadout: RodLoadout::default(),
            caught_fish: Vec::new(),
            total_catches: 0,
            bait_bucket: BaitBucket::default(),
        }
    }
}

// requires old files to be moved to ./data/users/old/
pub fn convert_old_userfiles() {

    // loop through the files in ./data/users and convert them to the new format
    let path = std::path::Path::new("./data/users/old");

    let new_path = std::path::Path::new("./data/users");

    // loop through all files in the directory
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let user_id = path.file_stem().unwrap().to_str().unwrap().parse::<u64>().unwrap();

            let Ok(data) = fs::read_to_string(path.clone()) else {
                // invalid data, return default values
                return;
            };

            let Ok(old_userfile) = ron::from_str::<OLD_UserValues>(&data) else {
                // invalid data
                nay!("Failed to parse old user file: {}", path.display());
                return;
            };

            // convert data to new format
            let uid = UserId::new(user_id);
            let new_userfile = UserFile::new(&uid);

            // todo: update userfile values with old userfile values

            new_userfile.update();

            // delete the old file
            if let Err(e) = fs::remove_file(&path) {
                nay!("Failed to delete old user file: {}", e);
            }
        }
    }

}