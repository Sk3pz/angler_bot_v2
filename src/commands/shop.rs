use crate::command;
use crate::data_management::monetary::MonetaryAmount;
use crate::data_management::userfile::UserFile;
use crate::fishing::shop::Shop;
use chrono::Local;
use serenity::all::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse, // We use this instead of EditMessage
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
    Unique = 5,
}

impl ShopCategory {
    fn next(&self) -> Self {
        match self {
            Self::Rods => Self::Reels,
            Self::Reels => Self::Lines,
            Self::Lines => Self::Sinkers,
            Self::Sinkers => Self::Bait,
            Self::Bait => Self::Unique,
            Self::Unique => Self::Rods,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Rods => Self::Unique,
            Self::Reels => Self::Rods,
            Self::Lines => Self::Reels,
            Self::Sinkers => Self::Lines,
            Self::Bait => Self::Sinkers,
            Self::Unique => Self::Bait,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Rods => "Fishing Rods",
            Self::Reels => "Reels",
            Self::Lines => "Lines",
            Self::Sinkers => "Sinkers",
            Self::Bait => "Bait (Daily Stock)",
            Self::Unique => "Unique Equipment",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Rods => "The main tool for fishing. Better rods increase catch chance and handle heavier loads.",
            Self::Reels => "Determines how fast you can reel in a catch. Faster reels reduce catch time.",
            Self::Lines => "Determines the maximum weight you can pull. Stronger lines prevent snapping.",
            Self::Sinkers => "Determines the depth range you can reach. Different fish live at different depths.",
            Self::Bait => "Consumables that attract specific fish, sizes, or rarities. Refreshes daily.",
            Self::Unique => "Special utility items that provide permanent bonuses or information.",
        }
    }
}

struct UniqueItem {
    name: &'static str,
    price: f32,
    description: &'static str,
}

const UNIQUE_ITEMS: &[UniqueItem] = &[
    UniqueItem {
        name: "Depth Finder",
        price: 5000.0,
        description: "Reveals the exact depth your line reaches when casting.",
    },
    UniqueItem {
        name: "Underwater Camera",
        price: 2500.0,
        description: "Allows you to see which fish got away if your line snaps.",
    },
];

command! {
    struct: ShopCommand,
    name: "shop",
    desc: "Open the Angler Shop to buy gear and bait.",
    run: async |data| {
        let shop = Shop::load();

        let mut category = ShopCategory::Rods;
        let mut item_index = 0;

        // Store feedback (Success/Failure, Message)
        let mut feedback: Option<(bool, String)> = None;

        // Initial Embed Construction
        #[cfg(feature = "guild_relative_userdata")]
        let embed = build_shop_embed(&shop, category, item_index, data.sender.id, data.guild_id, &feedback);
        #[cfg(not(feature = "guild_relative_userdata"))]
        let embed = build_shop_embed(&shop, category, item_index, data.sender.id, &feedback);

        let components = build_shop_components();

        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true);

        data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(response)).await.map_err(|e| e.to_string())?;

        // Component Interaction Loop
        let message = data.command.get_response(&data.ctx.http).await.map_err(|e| e.to_string())?;

        let mut collector = message.await_component_interactions(&data.ctx.shard)
            .timeout(Duration::from_secs(120)) // 120 Seconds Timeout
            .stream();

        while let Some(interaction) = collector.next().await {
            let custom_id = match &interaction.data.kind {
                ComponentInteractionDataKind::Button => interaction.data.custom_id.clone(),
                _ => continue,
            };

            if custom_id != "shop_buy" {
                feedback = None;
            }

            match custom_id.as_str() {
                "shop_left" => {
                    category = category.prev();
                    item_index = 0;
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
                    let res = handle_purchase(&shop, category, item_index, data);
                    #[cfg(feature = "guild_relative_userdata")]
                    let res = handle_purchase(&shop, category, item_index, data, data.guild_id);

                    match res {
                        Ok(msg) => feedback = Some((true, msg)),
                        Err(msg) => feedback = Some((false, msg)),
                    }
                },
                _ => {}
            }

            #[cfg(feature = "guild_relative_userdata")]
            let embed = build_shop_embed(&shop, category, item_index, data.sender.id, data.guild_id, &feedback);
            #[cfg(not(feature = "guild_relative_userdata"))]
            let embed = build_shop_embed(&shop, category, item_index, data.sender.id, &feedback);

            let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed).components(build_shop_components())
            )).await;
        }

        // --- Timeout Handling ---
        // We use data.command.edit_response because the message is ephemeral.
        let closed_embed = CreateEmbed::new()
            .title("🛒 Angler Shop - Closed")
            .description("Shop closed to save resources.\nReopen with `/shop` to continue browsing.")
            .color(0x2B2D31);

        let _ = data.command.edit_response(&data.ctx.http, EditInteractionResponse::new()
            .embed(closed_embed)
            .components(vec![]) // Removes all buttons
        ).await;

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
        ShopCategory::Unique => UNIQUE_ITEMS.len(),
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
    #[cfg(feature = "guild_relative_userdata")]
    let mut user_file = {
        let guild_id = guild_id.expect("Guild ID should be present here");
        UserFile::read(&data.sender.id, guild_id)
    };
    #[cfg(not(feature = "guild_relative_userdata"))]
    let mut user_file = UserFile::read(&data.sender.id);

    let balance = user_file.file.balance.get();
    let price;
    let item_name;

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
        ShopCategory::Unique => {
            let item = UNIQUE_ITEMS.get(index).ok_or("Item not found")?;
            price = item.price;
            item_name = item.name.to_string();

            if balance < price { return Err(format!("Insufficient funds! Need ${:.2}", price)); }

            match index {
                0 => { // Depth Finder
                    if user_file.file.loadout.has_depth_finder {
                        return Err("You already own a Depth Finder!".to_string());
                    }
                    user_file.file.loadout.has_depth_finder = true;
                },
                1 => { // Underwater Camera
                    if user_file.file.loadout.has_underwater_camera {
                        return Err("You already own an Underwater Camera!".to_string());
                    }
                    user_file.file.loadout.has_underwater_camera = true;
                },
                _ => return Err("Unknown Item".to_string()),
            }
        },
    }

    let new_balance = balance - price;
    user_file.file.balance = MonetaryAmount::new(new_balance);
    user_file.update();

    Ok(format!("Bought **{}** for ${:.2}!", item_name, price))
}

fn build_shop_embed(
    shop: &Shop,
    category: ShopCategory,
    selected_index: usize,
    user_id: serenity::all::UserId,
    #[cfg(feature = "guild_relative_userdata")]
    guild_id: Option<&serenity::all::GuildId>,
    feedback: &Option<(bool, String)>,
) -> CreateEmbed {
    #[cfg(feature = "guild_relative_userdata")]
    let user_file = {
        let guild_id = guild_id.expect("Guild ID should be present here");
        UserFile::read(&user_id, guild_id)
    };
    #[cfg(not(feature = "guild_relative_userdata"))]
    let user_file = UserFile::read(&user_id);

    let mut description = String::new();

    // Transaction Feedback Banner
    if let Some((success, msg)) = feedback {
        let icon = if *success { "✅" } else { "❌" };
        description.push_str(&format!("### {} {}\n\n", icon, msg));
    }

    // Category Info
    description.push_str(&format!("ℹ️ *{}*\n", category.description()));
    description.push_str(&format!("💳 **Balance:** {}\n\n", user_file.file.balance));

    // Item List Construction
    let mut add_item_line = |index: usize, name: &str, price: f32, desc: &str, is_owned: bool| {
        let owned_mark = if is_owned { " (Owned)" } else { "" };

        if index == selected_index {
            description.push_str(&format!(
                "🔷 **{}{}** — ${:.2}\n╰ *{}*\n",
                name, owned_mark, price, desc
            ));
        } else {
            description.push_str(&format!(
                "▪️ {}{} — ${:.2}\n",
                name, owned_mark, price
            ));
        }
    };

    match category {
        ShopCategory::Rods => {
            for (i, item) in shop.rods.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description, false);
            }
        },
        ShopCategory::Reels => {
            for (i, item) in shop.reels.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description, false);
            }
        },
        ShopCategory::Lines => {
            for (i, item) in shop.lines.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description, false);
            }
        },
        ShopCategory::Sinkers => {
            for (i, item) in shop.sinkers.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description, false);
            }
        },
        ShopCategory::Bait => {
            for (i, item) in shop.state.daily_baits.iter().enumerate() {
                add_item_line(i, &item.name, item.price, &item.description, false);
            }
            if shop.state.daily_baits.is_empty() {
                description.push_str("*Sold out for today! Check back tomorrow.*");
            }
        },
        ShopCategory::Unique => {
            for (i, item) in UNIQUE_ITEMS.iter().enumerate() {
                let is_owned = match i {
                    0 => user_file.file.loadout.has_depth_finder,
                    1 => user_file.file.loadout.has_underwater_camera,
                    _ => false,
                };
                add_item_line(i, item.name, item.price, item.description, is_owned);
            }
        },
    }

    // --- Footer Logic with Time Calculation ---
    let now = Local::now();
    let tomorrow_midnight = now.date_naive().succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap();
    let tomorrow_midnight_local = tomorrow_midnight.and_local_timezone(Local).unwrap();

    let duration = tomorrow_midnight_local - now;
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;

    CreateEmbed::new()
        .title(format!("🛒 Angler Shop - {}", category.name()))
        .description(description)
        .color(0x2B2D31)
        .footer(CreateEmbedFooter::new(format!(
            "Bait refresh in: {}h {}m",
            hours, minutes
        )))
}

fn build_shop_components() -> Vec<CreateActionRow> {
    let left = CreateButton::new("shop_left").label("◀ Category").style(ButtonStyle::Secondary);
    let right = CreateButton::new("shop_right").label("Category ▶").style(ButtonStyle::Secondary);
    let up = CreateButton::new("shop_up").label("▲ Up").style(ButtonStyle::Primary);
    let down = CreateButton::new("shop_down").label("▼ Down").style(ButtonStyle::Primary);
    let buy = CreateButton::new("shop_buy").label("🛒 Buy Selected").style(ButtonStyle::Success);

    vec![
        CreateActionRow::Buttons(vec![left, right, up, down]),
        CreateActionRow::Buttons(vec![buy]),
    ]
}