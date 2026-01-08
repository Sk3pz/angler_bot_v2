use serenity::all::{Color, CreateAttachment, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use crate::{command, nay};
use crate::data_management::userfile::UserFile;

command! {
    struct: InfoCommand,
    name: "info",
    desc: "List your stats and loadout information.",
    requires_guild: false,

    run: async |data| {
        #[cfg(feature = "guild_relative_userdata")]
        let userfile = {
            let Some(guild_id) = data.guild_id else {
                command_response_ephemeral(&data.ctx, &data.command,
                    "You must be in a guild to execute that command!").await;
                return Ok(());
            };

            UserFile::read(&data.sender.id, guild_id)
        };
        #[cfg(not(feature = "guild_relative_userdata"))]
        let userfile = UserFile::read(&data.sender.id);

        let loadout = &userfile.file.loadout;

        let strength_stat = loadout.get_total_strength_display();
        let speed_stat = loadout.get_speed_multiplier_display();
        let chance_stat = loadout.get_catch_chance_display();
        let depth_stat = loadout.get_depth_range_display();

        // Format special components
        let bait_display = match &loadout.bait {
            Some(bait) => format!("**{}** (Use Chance: {:.0}%)", bait.name, bait.use_chance * 100.0),
            None => "None".to_string(),
        };

        let depth_finder_display = if loadout.has_depth_finder {
            "✅ Equipped"
        } else {
            "❌ Not Equipped"
        };

        // Build the Embed
        let embed = CreateEmbed::new()
            .title(format!("🎣 Angler Info: {}", data.sender.display_name()))
            .description("Here are your current statistics and equipment loadout.")
            .color(Color::BLUE)
            .thumbnail("attachment://rod_with_fish.png")
            // -- General Stats --
            .field("💰 Balance", format!("{}", userfile.file.balance), true)
            .field("🐟 Total Catches", format!("{}", userfile.file.total_catches), true)
            .field("", "", false) // Spacer
            // -- Equipment Breakdown --
            .field("🎒 Equipment", format!(
                "**Rod:** {}\n**Reel:** {}\n**Line:** {}\n**Sinker:** {}\n**Bait:** {}\n**Depth Finder:** {}",
                loadout.rod.name,
                loadout.reel.name,
                loadout.line.name,
                loadout.sinker.name,
                bait_display,
                depth_finder_display
            ), false)
            // -- Loadout Performance --
            .field("📊 Loadout Performance", format!(
                "**Max Strength:** {}\n**Cast Speed:** {}\n**Catch Chance:** {}\n**Depth Range:** {}",
                strength_stat, speed_stat, chance_stat, depth_stat
            ), false);

        // Create Response Message
        let mut message = CreateInteractionResponseMessage::new()
            .embed(embed);

        // Attach the image
        let attachment = CreateAttachment::path("./assets/rod_with_fish.png").await;
        if let Ok(file) = attachment {
            message = message.add_file(file);
        }

        // Send
        let builder = CreateInteractionResponse::Message(message);

        if let Err(e) = data.command.create_response(&data.ctx, builder).await {
            nay!("Failed to send info message: {}", e);
        }

        Ok(())
    }
}