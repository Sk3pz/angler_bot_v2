use crate::command;
use crate::data_management::userfile::UserFile;
use serenity::all::{
    ButtonStyle, ComponentInteractionDataKind, CreateActionRow, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    EditInteractionResponse,
};
use serenity::futures::StreamExt;
use std::time::Duration;

command! {
    struct: BaitCommand,
    name: "bait",
    desc: "Open your bait bucket to view and equip bait.",
    run: async |data| {
        let mut index = 0;
        let mut feedback: Option<String> = None;

        // Helper to load user file
        #[cfg(feature = "guild_relative_userdata")]
        let load_file = || UserFile::read(&data.sender.id, data.guild_id.unwrap());
        #[cfg(not(feature = "guild_relative_userdata"))]
        let load_file = || UserFile::read(&data.sender.id);

        let mut user_file = load_file();

        // Initial check for empty bucket
        if user_file.file.bait_bucket.is_empty() {
             let embed = CreateEmbed::new()
                .title("🪣 Bait Bucket (Empty)")
                .description("You have no bait!\nVisit the `/shop` to buy some.")
                .color(0x2B2D31);

            data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed).ephemeral(true)
            )).await.map_err(|e| e.to_string())?;
            return Ok(());
        }

        let embed = build_bait_embed(&user_file, index, &feedback);
        let components = build_bait_components();

        let response = CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true);

        data.command.create_response(&data.ctx.http, CreateInteractionResponse::Message(response)).await.map_err(|e| e.to_string())?;

        let message = data.command.get_response(&data.ctx.http).await.map_err(|e| e.to_string())?;
        let mut collector = message.await_component_interactions(&data.ctx.shard)
            .timeout(Duration::from_secs(120))
            .stream();

        while let Some(interaction) = collector.next().await {
            // Reload file to ensure fresh state
            user_file = load_file();

            if user_file.file.bait_bucket.is_empty() {
                 let embed = CreateEmbed::new()
                    .title("🪣 Bait Bucket (Empty)")
                    .description("You have ran out of bait!\nVisit the `/shop` to buy more.")
                    .color(0x2B2D31);

                let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed).components(vec![])
                )).await;
                break;
            }

            let custom_id = match &interaction.data.kind {
                ComponentInteractionDataKind::Button => interaction.data.custom_id.clone(),
                _ => continue,
            };

            if custom_id != "bait_equip" { feedback = None; }

            match custom_id.as_str() {
                "bait_up" => {
                    if index > 0 { index -= 1; }
                },
                "bait_down" => {
                    if index < user_file.file.bait_bucket.len().saturating_sub(1) { index += 1; }
                },
                "bait_equip" => {
                    if let Some(bait) = user_file.file.bait_bucket.remove_index(index) {
                        // If they already have bait equipped, put it back in the bucket?
                        // For simplicity, let's say equipping DESTROYS the currently equipped bait (swapping)
                        // OR we push the currently equipped bait back to the bucket.

                        // Option A: Swap (Preserve old bait)
                        if let Some(old_bait) = user_file.file.loadout.bait.take() {
                            user_file.file.bait_bucket.add(old_bait);
                        }

                        // Equip new bait
                        let name = bait.name.clone();
                        user_file.file.loadout.bait = Some(bait);
                        user_file.update(); // Save changes

                        feedback = Some(format!("Equipped **{}**!", name));

                        // Adjust index if out of bounds after removal
                        if index >= user_file.file.bait_bucket.len() && index > 0 {
                            index -= 1;
                        }
                    } else {
                        feedback = Some("Failed to equip bait.".to_string());
                    }
                },
                _ => {}
            }

            // If bucket became empty after equip (and it was the last one)
            if user_file.file.bait_bucket.is_empty() && user_file.file.loadout.bait.is_some() {
                 let embed = CreateEmbed::new()
                    .title("🪣 Bait Bucket")
                    .description(format!("### ✅ {}\n\n*Bucket is now empty.*", feedback.unwrap_or_default()))
                    .color(0x2B2D31);

                 let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().embed(embed).components(vec![])
                )).await;
                break;
            }

            let embed = build_bait_embed(&user_file, index, &feedback);
            let _ = interaction.create_response(&data.ctx.http, CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().embed(embed).components(build_bait_components())
            )).await;
        }

        // Close on timeout
        let closed_embed = CreateEmbed::new()
            .title("🪣 Bait Bucket - Closed")
            .color(0x2B2D31);
        let _ = data.command.edit_response(&data.ctx.http, EditInteractionResponse::new().embed(closed_embed).components(vec![])).await;

        Ok(())
    }
}

fn build_bait_embed(user_file: &UserFile, selected_index: usize, feedback: &Option<String>) -> CreateEmbed {
    let mut description = String::new();

    if let Some(msg) = feedback {
        description.push_str(&format!("### ✅ {}\n\n", msg));
    }

    let current_equipped = match &user_file.file.loadout.bait {
        Some(b) => &b.name,
        None => "None",
    };
    description.push_str(&format!("🎣 **Currently Equipped:** {}\n\n", current_equipped));

    for (i, bait) in user_file.file.bait_bucket.baits.iter().enumerate() {
        if i == selected_index {
            description.push_str(&format!("🔷 **{}**\n╰ *{}*\n", bait.name, bait.description));
        } else {
            description.push_str(&format!("▪️ {}\n", bait.name));
        }
    }

    CreateEmbed::new()
        .title("🪣 Bait Bucket")
        .description(description)
        .color(0x2B2D31)
        .footer(CreateEmbedFooter::new(format!("Items: {}", user_file.file.bait_bucket.len())))
}

fn build_bait_components() -> Vec<CreateActionRow> {
    let up = CreateButton::new("bait_up").label("▲ Up").style(ButtonStyle::Primary);
    let down = CreateButton::new("bait_down").label("▼ Down").style(ButtonStyle::Primary);
    let equip = CreateButton::new("bait_equip").label("🎣 Equip Selected").style(ButtonStyle::Success);

    vec![
        CreateActionRow::Buttons(vec![up, down]),
        CreateActionRow::Buttons(vec![equip]),
    ]
}