use crate::command;
use crate::data_management::monetary::MonetaryAmount;
use crate::data_management::userfile::UserFile;
use crate::fishing::shop::Shop;
use serenity::all::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::futures::StreamExt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ShopCategory {
    Rods = 0,
    Reels = 1,
    Lines = 2,
    Sinkers = 3,
    Bait = 4,
}

impl ShopCategory {
    fn next(&self) -> Self {
        match self {
            Self::Rods => Self::Reels,
            Self::Reels => Self::Lines,
            Self::Lines => Self::Sinkers,
            Self::Sinkers => Self::Bait,
            Self::Bait => Self::Rods,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Rods => Self::Bait,
            Self::Reels => Self::Rods,
            Self::Lines => Self::Reels,
            Self::Sinkers => Self::Lines,
            Self::Bait => Self::Sinkers,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Rods => "Fishing Rods",
            Self::Reels => "Reels",
            Self::Lines => "Lines",
            Self::Sinkers => "Sinkers",
            Self::Bait => "Bait (Daily Stock)",
        }
    }
}

command! {
    struct: ShopCommand,
    name: "shop",
    desc: "Open the Angler Shop to buy gear and bait.",
    run: async |data| {
        let shop = Shop::load();

        let mut category = ShopCategory::Rods;
        let mut item_index = 0;

        // Initial Embed Construction
        #[cfg(feature = "guild_relative_userdata")]
        let embed = build_shop_embed(&shop, category, item_index, data.sender.id, data.guild_id);
        #[cfg(not(feature = "guild_relative_userdata"))]
        let embed = build_shop_embed(&shop, category, item_index, data.sender.id);
        let components = build_shop_components();

        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true); // Handled locally as requested

        data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(response)).await.map_err(|e| e.to_string())?;

        // Component Interaction Loop
        let mut message = data.command.get_response(&data.ctx.http).await.map_err(|e| e.to_string())?;

        let mut collector = message.await_component_interactions(&data.ctx.shard)
            .timeout(Duration::from_secs(120)) // Shop closes after 2 minutes of inactivity
            .stream();

        while let Some(interaction) = collector.next().await {
            let custom_id = match &interaction.data.kind {
                ComponentInteractionDataKind::Button => interaction.data.custom_id.clone(),
                _ => continue,
            };

            let mut feedback = None;

            // Handle Navigation & Buying
            match custom_id.as_str() {
                "shop_left" => {
                    category = category.prev();
                    item_index = 0; // Reset selection on page change
                },
                "shop_right" => {
                    category = category.next();
                    item_index = 0;
                },
                "shop_up" => {
                    if item_index > 0 {
                        item_index -= 1;
                    }
                },
                "shop_down" => {
                    let max_items = get_item_count(&shop, category);
                    if item_index < max_items.saturating_sub(1) {
                        item_index += 1;
                    }
                },
                "shop_buy" => {
                    #[cfg(not(feature = "guild_relative_userdata"))]
                    match handle_purchase(&shop, category, item_index, data) {
                        Ok(msg) => feedback = Some(msg),
                        Err(msg) => feedback = Some(msg),
                    }
                    #[cfg(feature = "guild_relative_userdata")]
                    match handle_purchase(&shop, category, item_index, data, data.guild_id) {
                        Ok(msg) => feedback = Some(msg),
                        Err(msg) => feedback = Some(msg),
                    }
                },
                _ => {}
            }

            // Rebuild UI
            #[cfg(feature = "guild_relative_userdata")]
            let mut embed = build_shop_embed(&shop, category, item_index, data.sender.id, data.guild_id);
            #[cfg(not(feature = "guild_relative_userdata"))]
            let mut embed = build_shop_embed(&shop, category, item_index, data.sender.id);

            // Add feedback field if purchase attempted
            if let Some(msg) = feedback {
                embed = embed.field("Transaction", msg, false);
            }

            // Acknowledge and Update
            let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed).components(build_shop_components())
            )).await;
        }

        Ok(())
    }
}

fn get_item_count(shop: &Shop, category: ShopCategory) -> usize {
    match category {
        ShopCategory::Rods => shop.rods.len(),
        ShopCategory::Reels => shop.reels.len(),
        ShopCategory::Lines => shop.lines.len(),
        ShopCategory::Sinkers => shop.sinkers.len(),
        ShopCategory::Bait => shop.state.daily_baits.len(),
    }
}

fn handle_purchase(
    shop: &Shop,
    category: ShopCategory,
    index: usize,
    data: &crate::commands::CommandData,
    #[cfg(feature = "guild_relative_userdata")]
    guild_id: Option<&serenity::all::GuildId>
) -> Result<String, String> {
    // Load User Data
    #[cfg(feature = "guild_relative_userdata")]
    let mut user_file = {
        let guild_id = guild_id.expect("Bruh?? Guild ID should be present here but wasn't; shop.rs - handle_purchase");

        UserFile::read(&data.sender.id, guild_id)
    };
    #[cfg(not(feature = "guild_relative_userdata"))]
    let mut user_file = UserFile::read(&data.sender.id);

    let balance = user_file.file.balance.get();
    let price;
    let item_name;

    // Check Price and Item
    match category {
        ShopCategory::Rods => {
            let item = shop.rods.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.clone();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            user_file.file.loadout.rod = item.clone();
        },
        ShopCategory::Reels => {
            let item = shop.reels.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.clone();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            user_file.file.loadout.reel = item.clone();
        },
        ShopCategory::Lines => {
            let item = shop.lines.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.clone();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            user_file.file.loadout.line = item.clone();
        },
        ShopCategory::Sinkers => {
            let item = shop.sinkers.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.clone();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            user_file.file.loadout.sinker = item.clone();
        },
        ShopCategory::Bait => {
            let item = shop.state.daily_baits.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.clone();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            user_file.file.loadout.bait = Some(item.clone());
        },
    }

    // Deduct Balance
    let new_balance = balance - price;
    user_file.file.balance = MonetaryAmount::new(new_balance);

    // Save
    user_file.update();

    Ok(format!("Successfully bought **{}** for ${:.2}!", item_name, price))
}

fn build_shop_embed(
    shop: &Shop,
    category: ShopCategory,
    selected_index: usize,
    user_id: serenity::all::UserId,
    #[cfg(feature = "guild_relative_userdata")]
    guild_id: Option<&serenity::all::GuildId>
) -> CreateEmbed {
    // Load balance for display
    #[cfg(feature = "guild_relative_userdata")]
    let user_file = {
        let guild_id = guild_id.expect("Bruh?? Guild ID should be present here but wasn't; shop.rs - build_shop_embed");

        UserFile::read(&user_id, guild_id)
    };
    #[cfg(not(feature = "guild_relative_userdata"))]
    let user_file = UserFile::read(&user_id);

    let mut description = String::new();

    // Helper to format item lines
    let mut add_item_line = |index: usize, name: &str, price: f32, desc: &str| {
        let cursor = if index == selected_index { "👉" } else { "⬛" };
        let style = if index == selected_index { "**" } else { "" };

        description.push_str(&format!(
            "{} {}{}{} - ${:.2}\n└ *{}*\n\n",
            cursor, style, name, style, price, desc
        ));
    };

    match category {
        ShopCategory::Rods => {
            for (i, item) in shop.rods.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description);
            }
        },
        ShopCategory::Reels => {
            for (i, item) in shop.reels.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description);
            }
        },
        ShopCategory::Lines => {
            for (i, item) in shop.lines.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description);
            }
        },
        ShopCategory::Sinkers => {
            for (i, item) in shop.sinkers.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description);
            }
        },
        ShopCategory::Bait => {
            for (i, item) in shop.state.daily_baits.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description);
            }
            if shop.state.daily_baits.is_empty() {
                description = "Sold out for today!".to_string();
            }
        },
    }

    CreateEmbed::new()
        .title(format!("🛒 Angler Shop - {}", category.name()))
        .description(description)
        .color(0x00FF00)
        .footer(CreateEmbedFooter::new(format!("Balance: {}", user_file.file.balance)))
}

fn build_shop_components() -> Vec<CreateActionRow> {
    let left = CreateButton::new("shop_left").label("◀ Category").style(ButtonStyle::Secondary);
    let right = CreateButton::new("shop_right").label("Category ▶").style(ButtonStyle::Secondary);
    let up = CreateButton::new("shop_up").label("▲ Up").style(ButtonStyle::Primary);
    let down = CreateButton::new("shop_down").label("▼ Down").style(ButtonStyle::Primary);
    let buy = CreateButton::new("shop_buy").label("🛒 Buy Selected").style(ButtonStyle::Success);

    // Layout:
    // [Left] [Up] [Right]
    //       [Down]
    //       [Buy]
    // Ideally:
    // Row 1: Left, Up, Right
    // Row 2: Down, Buy

    vec![
        CreateActionRow::Buttons(vec![left, up, right]),
        CreateActionRow::Buttons(vec![down, buy]),
    ]
}