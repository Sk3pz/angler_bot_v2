use serenity::all::PartialChannel;

use crate::{
    command, commands::command_response_ephemeral, data_management::guildfile::GuildSettings,
};

command! {
    struct: RegisterChannelCommand,
    name: "register",
    desc: "Manage channels that AnglerBot commands can be run in.",
    requires_guild: true,
    is_admin_command: true,

    run: async |data|

    // SUBCOMMANDS:
    sub: add("Add a channel to the allowed channels") => async |data, channel("The channel you wish Angler Bot commands to be allowed in"): &PartialChannel| {
        let guild_id = data.guild_id.unwrap(); // Safe because requires_guild: true
        let mut guild_file = GuildSettings::get(&guild_id);

        guild_file.add_channel(channel.id.get());

        command_response_ephemeral(
            &data.ctx,
            &data.command,
            format!("✅ <#{}> added to allowed channels.", channel.id)
        ).await;

        Ok(())
    }

    sub: remove("Remove a channel from the allowed channels") => async |data, channel("The channel to remove from Angler Bot's allowed operating channels"): &PartialChannel| {
        let guild_id = data.guild_id.unwrap();
        let mut guild_file = GuildSettings::get(&guild_id);

        guild_file.remove_channel(channel.id.get());

        command_response_ephemeral(
            &data.ctx,
            &data.command,
            format!("🗑️ <#{}> removed from allowed channels.", channel.id)
        ).await;

        Ok(())
    }

    sub: list("List all allowed channels") => async |data| {
        let guild_id = data.guild_id.unwrap();
        let mut guild_file = GuildSettings::get(&guild_id);
        let channels = guild_file.get_channels();

        if channels.is_empty() {
            command_response_ephemeral(
                &data.ctx,
                &data.command,
                "No channels have been registered - Angler bot can operate in any channel."
            ).await;
            return Ok(());
        }

        let mut response = String::from("📋 **Registered Channels:**\n");
        for channel_id in channels {
            response.push_str(&format!("- <#{}>\n", channel_id));
        }

        command_response_ephemeral(&data.ctx, &data.command, response).await;
        Ok(())
    }

    WITH [ ADMINISTRATOR, MANAGE_GUILD, MANAGE_CHANNELS ] {
        command_response_ephemeral(
            &data.ctx,
            &data.command,
            "Please select a subcommand: `add`, `remove`, or `list`."
        ).await;
        Ok(())
    }
}
