use crate::command;
use crate::data_management::userfile::UserFile;
use serenity::all::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditMessage,
};
use serenity::futures::StreamExt;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum InventoryCategory {
    Rods = 0,
    Reels = 1,
    Lines = 2,
    Sinkers = 3,
}

impl InventoryCategory {
    fn next(&self) -> Self {
        match self {
            Self::Rods => Self::Reels,
            Self::Reels => Self::Lines,
            Self::Lines => Self::Sinkers,
            Self::Sinkers => Self::Rods,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Rods => Self::Sinkers,
            Self::Reels => Self::Rods,
            Self::Lines => Self::Reels,
            Self::Sinkers => Self::Lines,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Rods => "Fishing Rods",
            Self::Reels => "Reels",
            Self::Lines => "Lines",
            Self::Sinkers => "Sinkers",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::Rods => "View and equip your fishing rods.",
            Self::Reels => "View and equip your reels.",
            Self::Lines => "View and equip your fishing lines.",
            Self::Sinkers => "View and equip your sinkers.",
        }
    }
}

command! {
    struct: InventoryCommand,
    name: "inventory",
    desc: "View and equip your fishing gear.",
    run: async |data| {
        let load_file = || UserFile::read(&data.sender.id);
        let mut user_file = load_file();

        let mut category = InventoryCategory::Rods;
        let mut cursor_index = 0; // The item currently highlighted by the user
        let mut feedback: Option<String> = None;

        // --- PREVENT EXPLOIT: Check if fishing ---
        {
            let fishing_set = data.handler.users_fishing.lock().await;
            if fishing_set.contains(&data.sender.id) {
                let embed = CreateEmbed::new()
                    .title("🎒 Inventory")
                    .description("You can't change your gear while your rod is cast!")
                    .color(0xFA5050);

                data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
                )).await.map_err(|e| e.to_string())?;
                return Ok(());
            }
        }

        // Initial Embed Construction
        let embed = build_inventory_embed(&user_file, category, cursor_index, &feedback);
        let components = build_inventory_components();

        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true);

        data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(response)).await.map_err(|e| e.to_string())?;

        // Interaction Loop
        let mut message = data.command.get_response(&data.ctx.http).await.map_err(|e| e.to_string())?;

        let mut collector = message.await_component_interactions(&data.ctx.shard).stream();

        // 120s timeout loop
        while let Ok(Some(interaction)) = tokio::time::timeout(Duration::from_secs(120), collector.next()).await {
            let custom_id = match &interaction.data.kind {
                ComponentInteractionDataKind::Button => interaction.data.custom_id.clone(),
                _ => continue,
            };

            // Refresh file data to ensure persistence integrity
            user_file = load_file();

            if custom_id != "inv_equip" {
                feedback = None;
            }

            match custom_id.as_str() {
                "inv_left" => {
                    category = category.prev();
                    cursor_index = 0;
                },
                "inv_right" => {
                    category = category.next();
                    cursor_index = 0;
                },
                "inv_up" => {
                    if cursor_index > 0 {
                        cursor_index -= 1;
                    } else {
                        let max_items = get_item_count(&user_file, category);
                        cursor_index = max_items.saturating_sub(1);
                    }
                },
                "inv_down" => {
                    let max_items = get_item_count(&user_file, category);
                    if cursor_index < max_items.saturating_sub(1) {
                        cursor_index += 1;
                    } else {
                        cursor_index = 0;
                    }
                },
                "inv_equip" => {
                    match handle_equip(&mut user_file, category, cursor_index) {
                        Ok(msg) => feedback = Some(msg),
                        Err(msg) => feedback = Some(msg),
                    }
                },
                _ => {}
            }

            let embed = build_inventory_embed(&user_file, category, cursor_index, &feedback);

            let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed).components(build_inventory_components())
            )).await;
        }

        // Timeout Handling
        let closed_embed = CreateEmbed::new()
            .title("🎒 Inventory - Closed")
            .description("Inventory closed to save resources.\nReopen with `/inventory`.")
            .color(0x2B2D31);

        let _ = message.edit(&data.ctx.http, EditMessage::new()
            .embed(closed_embed)
            .components(vec![])
        ).await;

        Ok(())
    }
}

fn get_item_count(user_file: &UserFile, category: InventoryCategory) -> usize {
    match category {
        InventoryCategory::Rods => user_file.file.inventory.rods.len(),
        InventoryCategory::Reels => user_file.file.inventory.reels.len(),
        InventoryCategory::Lines => user_file.file.inventory.lines.len(),
        InventoryCategory::Sinkers => user_file.file.inventory.sinkers.len(),
    }
}

fn handle_equip(
    user_file: &mut UserFile,
    category: InventoryCategory,
    index: usize,
) -> Result<String, String> {
    let inventory = &mut user_file.file.inventory;
    let item_name;

    match category {
        InventoryCategory::Rods => {
            if index >= inventory.rods.len() { return Err("Invalid selection.".to_string()); }
            inventory.selected_rod = index;
            item_name = inventory.rods[index].name.clone();
        },
        InventoryCategory::Reels => {
            if index >= inventory.reels.len() { return Err("Invalid selection.".to_string()); }
            inventory.selected_reel = index;
            item_name = inventory.reels[index].name.clone();
        },
        InventoryCategory::Lines => {
            if index >= inventory.lines.len() { return Err("Invalid selection.".to_string()); }
            inventory.selected_line = index;
            item_name = inventory.lines[index].name.clone();
        },
        InventoryCategory::Sinkers => {
            if index >= inventory.sinkers.len() { return Err("Invalid selection.".to_string()); }
            inventory.selected_sinker = index;
            item_name = inventory.sinkers[index].name.clone();
        },
    }

    user_file.update();
    Ok(format!("Equipped **{}**!", item_name))
}

fn build_inventory_embed(
    user_file: &UserFile,
    category: InventoryCategory,
    cursor_index: usize,
    feedback: &Option<String>,
) -> CreateEmbed {
    let mut description = String::new();

    // Feedback Banner
    if let Some(msg) = feedback {
        let icon = if msg.contains("Invalid") { "❌" } else { "✅" };
        description.push_str(&format!("### {} {}\n\n", icon, msg));
    }

    description.push_str(&format!("ℹ️ *{}*\n\n", category.description()));

    // Helper closure to generate item lines
    // cursor_index: The item the user is hovering over
    // active_index: The item actually equipped in the database
    let mut add_item_line = |index: usize, name: &str, desc: &str, active_index: usize| {
        let is_equipped = index == active_index;
        let equipped_tag = if is_equipped { " *(Equipped)*" } else { "" };

        if index == cursor_index {
            // Selected (Cursor)
            description.push_str(&format!(
                "🔷 **{}{}**\n╰ *{}*\n",
                name, equipped_tag, desc
            ));
        } else {
            // Unselected
            description.push_str(&format!(
                "▪️ {}{}\n",
                name, equipped_tag
            ));
        }
    };

    match category {
        InventoryCategory::Rods => {
            let active = user_file.file.inventory.selected_rod;
            for (i, item) in user_file.file.inventory.rods.iter().enumerate() {
                add_item_line(i, &item.name, &item.description, active);
            }
        },
        InventoryCategory::Reels => {
            let active = user_file.file.inventory.selected_reel;
            for (i, item) in user_file.file.inventory.reels.iter().enumerate() {
                add_item_line(i, &item.name, &item.description, active);
            }
        },
        InventoryCategory::Lines => {
            let active = user_file.file.inventory.selected_line;
            for (i, item) in user_file.file.inventory.lines.iter().enumerate() {
                add_item_line(i, &item.name, &item.description, active);
            }
        },
        InventoryCategory::Sinkers => {
            let active = user_file.file.inventory.selected_sinker;
            for (i, item) in user_file.file.inventory.sinkers.iter().enumerate() {
                add_item_line(i, &item.name, &item.description, active);
            }
        },
    }

    CreateEmbed::new()
        .title(format!("🎒 Inventory - {}", category.name()))
        .description(description)
        .color(0x2B2D31)
        .footer(CreateEmbedFooter::new("Use arrows to browse | 'Equip' to use selected item"))
}

fn build_inventory_components() -> Vec<CreateActionRow> {
    let left = CreateButton::new("inv_left").label("◀ Category").style(ButtonStyle::Secondary);
    let right = CreateButton::new("inv_right").label("Category ▶").style(ButtonStyle::Secondary);
    let up = CreateButton::new("inv_up").label("▲ Up").style(ButtonStyle::Primary);
    let down = CreateButton::new("inv_down").label("▼ Down").style(ButtonStyle::Primary);
    let equip = CreateButton::new("inv_equip").label("🎣 Equip").style(ButtonStyle::Success);

    vec![
        CreateActionRow::Buttons(vec![left, right, up, down]),
        CreateActionRow::Buttons(vec![equip]),
    ]
}