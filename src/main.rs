use std::env;

use serenity::{Client, all::GatewayIntents};

use crate::discord::Handler;

mod commands;
pub mod data_management;
mod discord;
pub mod error;
pub mod fishing;
pub mod helpers;
pub mod logging;


#[tokio::main]
async fn main() {
    yay!("Angler Bot is starting up!");

    // Create the data directory if it doesn't exist
    let Ok(exists) = std::fs::exists("./data") else {
        nay!("Failed to check if guilds directory exists");
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
        .event_handler(Handler {})
        .await
    else {
        nay!("Error creating client");
        return;
    };

    // start the client
    if let Err(e) = client.start().await {
        nay!("Client error: {}", e);
    }
}
