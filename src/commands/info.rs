use serenity::all::{Color, CreateAttachment, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use crate::{command, nay};
use crate::data_management::userfile::UserFile;
use crate::fishing::fish_data::fish::Pond;

command! {
    struct: InfoCommand,
    name: "info",
    desc: "List your stats and loadout information.",
    requires_guild: false,

    run: async |data| {
        let userfile = UserFile::read(&data.sender.id);

        let loadout = &userfile.file.loadout;

        // --- Gear Formatting ---
        let bait_display = match &loadout.bait {
            Some(bait) => format!("**{}**\n╰ *{}*", bait.name, bait.description),
            None => "None".to_string(),
        };

        // --- Tech/Unique Items Check ---
        let mut tech_list = Vec::new();
        if loadout.has_depth_finder { tech_list.push("✅ Depth Finder"); }
        if loadout.has_underwater_camera { tech_list.push("✅ Underwater Camera"); }

        let tech_display = if tech_list.is_empty() {
            "No special equipment equipped.".to_string()
        } else {
            tech_list.join("\n")
        };

        let fish_count = if let Ok(pond) = Pond::load() {
            format!("{}", pond.fish_types.len())
        } else {
            nay!("Failed to load pond data, cannot determine fish count.");
            "???".to_string()
        };

        // --- Build Embed ---
        let embed = CreateEmbed::new()
            .title(format!("🎣 Angler Profile: {}", data.sender.display_name()))
            .color(0x00A2FF) // Nice Ocean Blue
            //.thumbnail("attachment://rod_with_fish.png")

            // Profile Stats
            .description(format!(
                "**💳 Balance:** {}\n**🐟 Total Catches:** {}\n🐠 **Fish Discovered:** {}/{}\n",
                userfile.file.balance, userfile.file.total_catches, userfile.file.caught_fish.len(), fish_count
            ))

            // Main Gear
            .field("🎒 Fishing Gear", format!(
                "🎣 **Rod:** {}\n⚙️ **Reel:** {}\n🧵 **Line:** {}\n⚓ **Sinker:** {}\n🪱 **Bait:** {}",
                loadout.rod.name,
                loadout.reel.name,
                loadout.line.name,
                loadout.sinker.name,
                bait_display
            ), false)

            // Performance Stats (Inline)
            .field("📊 Stats", format!(
                "**Strength:** {}\n**Speed:** {}\n**Luck:** {}\n**Depth:** {}",
                loadout.get_total_strength_display(),
                loadout.get_speed_multiplier_display(),
                loadout.get_catch_chance_display(),
                loadout.get_depth_range_display()
            ), true)

            // Tech & Accessories
            .field("📟 Tech & Accessories", tech_display, true);

        // --- Send Response ---
        let message = CreateInteractionResponseMessage::new().embed(embed);
        //let mut message = CreateInteractionResponseMessage::new().embed(embed);

        // let attachment = CreateAttachment::path("./assets/rod_with_fish.png").await;
        // if let Ok(file) = attachment {
        //     message = message.add_file(file);
        // }

        let builder = CreateInteractionResponse::Message(message);
        if let Err(e) = data.command.create_response(&data.ctx, builder).await {
            nay!("Failed to send info message: {}", e);
        }

        Ok(())
    }
}