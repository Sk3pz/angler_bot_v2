use std::env;

use crate::discord::Handler;
use crate::fishing::Attribute;
use crate::{data_management::config::Config, fishing::fish_data::fish::FishType};
use serenity::{Client, all::GatewayIntents};
use crate::data_management::version_uf_converter::convert_old_userfiles;

mod commands;
pub mod data_management;
mod discord;
pub mod error;
pub mod fishing;
pub mod helpers;
pub mod logging;
pub mod gui;

#[tokio::main]
async fn main() {
    // Initialize global log buffer
    crate::gui::logging::GLOBAL_LOG_BUFFER.set(crate::gui::logging::LogBuffer::new(1000)).ok();

    yay!("Angler Bot is starting up!");

    // Create the data directory if it doesn't exist
    let Ok(exists) = std::fs::exists("./data") else {
        nay!("Failed to check if data directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./data") {
            nay!("Failed to create data directory: {}", e);
            return;
        };
    }
    // create the guilds and data directory if it doesn't exist
    let Ok(exists) = std::fs::exists("./data/guilds") else {
        nay!("Failed to check if guilds directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./data/guilds") {
            nay!("Failed to create guilds directory: {}", e);
            return;
        };
    }
    // create the users and data directory if it doesn't exist
    let Ok(exists) = std::fs::exists("./data/users") else {
        nay!("Failed to check if users directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./data/users") {
            nay!("Failed to create users directory: {}", e);
            return;
        };
    }

    // create config.toml if it doesnt exist
    let Ok(exists) = std::fs::exists("./data/config.toml") else {
        nay!("Failed to check if the config exists!");
        return;
    };
    if !exists {
        let default_config = Config::default();
        // write default config to ./data/config.toml using toml
        default_config.save();
    }

    // create ./data/gamedata/ if it doesn't exist
    let Ok(exists) = std::fs::exists("./data/gamedata") else {
        nay!("Failed to check if gamedata directory exists");
        return;
    };
    if !exists {
        if let Err(e) = std::fs::create_dir_all("./data/gamedata") {
            nay!("Failed to create gamedata directory: {}", e);
            return;
        };
    }

    // create ./data/gamedata/fish_types.ron if it doesn't exist
    let Ok(exists) = std::fs::exists("./data/gamedata/fish_types.ron") else {
        nay!("Failed to check if fish_types.ron exists");
        return;
    };
    if !exists {
        let pond = fishing::fish_data::fish::Pond {
            fish_types: vec![FishType {
                name: "Salmon".to_string(),
                rarity: fishing::fish_data::rarity::FishRarity::Uncommon,
                category: fishing::fish_data::fish::FishCategory::Predatory,
                size_range: Attribute {
                    min: 24.0,
                    max: 58.0,
                    average: 36.0,
                },
                weight_range: Attribute {
                    min: 10.0,
                    max: 90.0,
                    average: 30.0,
                },
                depth_range: (25.0, 150.0),
                base_value: 50.0,
            }],
        };
        if let Err(e) = pond.save() {
            nay!("Failed to create fish_types.ron: {}", e);
            return;
        };
    }

    // update old userfiles
    convert_old_userfiles();

    // get the env variables
    dotenv::dotenv().expect("Failed to load .env file");

    let Ok(token) = env::var("DISCORD_TOKEN") else {
        nay!("DISCORD_TOKEN not found in environment");
        return;
    };

    // discord client
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let Ok(mut client) = Client::builder(token, intents)
        .event_handler(Handler::new())
        .await
    else {
        nay!("Error creating client");
        return;
    };

    let http = client.http.clone();

    // Spawn the client in a background task
    tokio::spawn(async move {
        if let Err(e) = client.start().await {
            nay!("Client error: {}", e);
        }
    });

    // Launch GUI
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Angler Bot Admin",
        options,
        Box::new(|cc| Ok(Box::new(crate::gui::app::AnglerApp::new(cc, Some(http))))),
    );
}
