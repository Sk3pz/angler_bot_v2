use crate::command;
use crate::data_management::userfile::UserFile;
use serenity::all::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditMessage,
};
use serenity::futures::StreamExt;
use std::time::Duration;

command! {
    struct: BaitCommand,
    name: "bait",
    desc: "Open your bait bucket to view and equip bait.",
    run: async |data| {
        // Start at 0 (The "No Bait" option)
        let mut index = 0;
        let mut feedback: Option<String> = None;

        // --- PREVENT EXPLOIT: Check if fishing ---
        {
            let fishing_set = data.handler.users_fishing.lock().await;
            if fishing_set.contains(&data.sender.id) {
                let embed = CreateEmbed::new()
                    .title("🪣 Bait Bucket")
                    .description("You can't change your bait while your rod is cast!")
                    .color(0xFA5050); // Red for error

                data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
                )).await.map_err(|e| e.to_string())?;
                return Ok(());
            }
        }

        let load_file = || UserFile::read(&data.sender.id);

        let mut user_file = load_file();

        // Empty bucket and no equipped bait check
        if user_file.file.inventory.bait_bucket.is_empty() && user_file.file.inventory.selected_bait.is_none() {
             let embed = CreateEmbed::new()
                .title("🪣 Bait Bucket (Empty)")
                .description("You have no bait and nothing equipped!\nVisit the `/shop` to buy some.")
                .color(0x2B2D31);

            data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await.map_err(|e| e.to_string())?;
            return Ok(());
        }

        let embed = build_bait_embed(&user_file, index, &feedback);
        let components = build_bait_components(&user_file);

        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true);

        data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(response)).await.map_err(|e| e.to_string())?;

        let mut message = data.command.get_response(&data.ctx.http).await.map_err(|e| e.to_string())?;

        let mut collector = message.await_component_interactions(&data.ctx.shard)
            .timeout(Duration::from_secs(120)) // handled manually in the loop
            .stream();

        while let Some(interaction) = collector.next().await {
            // Reload file to ensure fresh state
            user_file = load_file();

            // Max index is the count of items.
            // If we have 5 items, indices are 0 (No Bait), 1, 2, 3, 4, 5.
            let max_index = user_file.file.inventory.bait_bucket.len();

            let custom_id = match &interaction.data.kind {
                ComponentInteractionDataKind::Button => interaction.data.custom_id.clone(),
                _ => continue,
            };

            // Reset feedback unless it's an action that produces feedback (equip or toggle)
            if custom_id != "bait_equip" && custom_id != "bait_toggle" {
                feedback = None;
            }

            match custom_id.as_str() {
                "bait_up" => {
                    if index > 0 { index -= 1; }
                    else { index = max_index; }
                },
                "bait_down" => {
                    if index < max_index { index += 1; }
                    else { index = 0; }
                },
                "bait_equip" => {
                    if index == 0 {
                        // === UNEQUIP LOGIC (No Bait) ===
                        if user_file.file.inventory.selected_bait.is_some() {
                            user_file.file.inventory.selected_bait = None;
                            user_file.update();
                            feedback = Some("Unequipped bait.".to_string());
                        } else {
                            feedback = Some("You aren't using any bait.".to_string());
                        }
                    } else {
                        // === EQUIP LOGIC (Specific Bait) ===
                        // Adjust index because visual index 1 is actually array index 0
                        let real_index = index - 1;

                        // Verify the item exists
                        if let Some(bait) = user_file.file.inventory.bait_bucket.get(real_index) {
                            let name = bait.name.clone();

                            // Set the selected index
                            user_file.file.inventory.selected_bait = Some(real_index);
                            user_file.update();

                            feedback = Some(format!("Equipped **{}**!", name));
                        } else {
                            feedback = Some("Failed to find that bait.".to_string());
                            // Reset index if out of bounds
                            if index > user_file.file.inventory.bait_bucket.len() {
                                index = 0;
                            }
                        }
                    }
                },
                "bait_toggle" => {
                    user_file.file.autobait = !user_file.file.autobait;
                    user_file.update();
                    let status = if user_file.file.autobait { "ON" } else { "OFF" };
                    feedback = Some(format!("AutoBait is now **{}**.", status));
                },
                _ => {}
            }

            let embed = build_bait_embed(&user_file, index, &feedback);
            let components = build_bait_components(&user_file);

            let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed).components(components)
            )).await;
        }

        // --- Timeout Handling ---
        // Uses message.edit directly to ensure it actually closes
        let closed_embed = CreateEmbed::new()
            .title("🪣 Bait Bucket - Closed")
            .description("Session timed out.")
            .color(0x2B2D31);

        let _ = message.edit(&data.ctx.http, EditMessage::new()
            .embed(closed_embed)
            .components(vec![])
        ).await;

        Ok(())
    }
}

fn build_bait_embed(user_file: &UserFile, selected_index: usize, feedback: &Option<String>) -> CreateEmbed {
    let mut description = String::new();

    if let Some(msg) = feedback {
        let icon = if msg.contains("Failed") { "❌" } else { "✅" };
        description.push_str(&format!("### {} {}\n\n", icon, msg));
    }

    // AutoBait Status Display
    // let autobait_status = if user_file.file.autobait { "Enabled (auto-equips duplicate baits on use)" } else { "Disabled" };
    // description.push_str(&format!("🤖 **AutoBait:** {}\n", autobait_status));

    // Get currently equipped bait name using the Inventory helper
    let current_equipped = match user_file.file.inventory.get_loadout().bait {
        Some(b) => b.name,
        None => "None".to_string(),
    };
    description.push_str(&format!("🎣 **Currently Equipped:** {}\n\n", current_equipped));

    // === Option 0: No Bait ===
    if selected_index == 0 {
        description.push_str("🔷 **No Bait**\n╰ *Unequip your current bait.*\n");
    } else {
        description.push_str("▪️ No Bait\n");
    }

    // === Option 1+: Actual Items ===
    // We check which index is actually equipped in the inventory data
    let actual_equipped_index = user_file.file.inventory.selected_bait;

    for (i, bait) in user_file.file.inventory.bait_bucket.baits.iter().enumerate() {
        let display_index = i + 1;
        let is_equipped = actual_equipped_index == Some(i);
        let equipped_mark = if is_equipped { " *(Equipped)*" } else { "" };

        if display_index == selected_index {
            description.push_str(&format!("🔷 **{}{}**\n╰ *{}*\n", bait.name, equipped_mark, bait.description));
        } else {
            description.push_str(&format!("▪️ {}{}\n", bait.name, equipped_mark));
        }
    }

    if user_file.file.inventory.bait_bucket.is_empty() {
        description.push_str("\n*Your bucket is empty. Visit /shop to buy more.*");
    }

    CreateEmbed::new()
        .title("🪣 Bait Bucket")
        .description(description)
        .color(0x2B2D31)
        .footer(CreateEmbedFooter::new(format!("AutoBait will equip the same bait on use if you have multiple.\nItems: {} | This closes automatically after 2min of inactivity", user_file.file.inventory.bait_bucket.len())))
}

fn build_bait_components(user_file: &UserFile) -> Vec<CreateActionRow> {
    let up = CreateButton::new("bait_up").label("▲ Up").style(ButtonStyle::Primary);
    let down = CreateButton::new("bait_down").label("▼ Down").style(ButtonStyle::Primary);
    let equip = CreateButton::new("bait_equip").label("🎣 Select").style(ButtonStyle::Success);

    // Dynamic Toggle Button
    let (label, style) = if user_file.file.autobait {
        ("🤖 AutoBait: ON", ButtonStyle::Success)
    } else {
        ("🤖 AutoBait: OFF", ButtonStyle::Secondary)
    };
    let toggle = CreateButton::new("bait_toggle").label(label).style(style);

    vec![
        CreateActionRow::Buttons(vec![up, down]),
        CreateActionRow::Buttons(vec![equip, toggle]),
    ]
}