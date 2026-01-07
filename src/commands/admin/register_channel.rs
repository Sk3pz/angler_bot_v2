use serenity::all::PartialChannel;

use crate::{
    command, commands::command_response_ephemeral, data_management::guildfile::GuildSettings,
};

command! {
    struct: RegisterChannelCommand,
    name: "register",
    desc: "Manage channels that AnglerBot commands can be run in.",
    requires_guild: true,

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

    WITH [ MANAGE_GUILD ] {
        command_response_ephemeral(
            &data.ctx,
            &data.command,
            "Please select a subcommand: `add`, `remove`, or `list`."
        ).await;
        Ok(())
    }
}

/* OLD SYSTEM (without subcommands):
use serenity::all::PartialChannel;

use crate::{
    command,
    commands::{command_response_ephemeral, error_command_response},
    data_management::guildfile::{GuildFile, GuildSettings},
    helpers::generate_error_code,
    nay,
};

command! {
     struct: RegisterChannelCommand,
     name: "register",
     desc: "Manage channels that AnglerBot commands can be run in.",
     requires_guild: true,

     run: async |data,
         action("The action which you wish to enact":
          ["Add Channel": "add", "Remove Channel": "remove", "List Channels": "list"]): String,
         option("The channel you want to modify"): Option<&PartialChannel>
     | WITH [ MANAGE_GUILD ] {

         // unwrap guild id
         let guild_id = data.guild_id.unwrap();
         // guild is required so this (should be) safe

         // get the guildfile
         let mut guild_file = GuildSettings::get(&guild_id);

         match action.as_str() {
             "add" => {
                 if let Some(option) = option {
                     guild_file.add_channel(option.id.get());
                     command_response_ephemeral(&data.ctx, &data.command, "Channel added to allowed channels.").await;
                 } else {
                     command_response_ephemeral(&data.ctx, &data.command, "You must specify a channel to add.").await;
                 }
             },
             "remove" => {
                 if let Some(option) = option {
                     guild_file.remove_channel(option.id.get());
                     command_response_ephemeral(&data.ctx, &data.command, "Channel added to allowed channels.").await;
                 } else {
                     command_response_ephemeral(&data.ctx, &data.command, "You must specify a channel to remove.").await;
                 }
             },
             "list" => {
                 let channels = guild_file.get_channels();
                 if channels.is_empty() {
                     command_response_ephemeral(&data.ctx, &data.command, "No channels have been registered - Angler bot can operate in any channel of the server.").await;
                     return Ok(());
                 }
                 let mut response = String::from("Registered Channels:\n");
                 for channel_id in channels {
                     response.push_str(&format!("- <#{}>\n", channel_id));
                 }
                 command_response_ephemeral(&data.ctx, &data.command, response).await;
                 return Ok(());
             }
             _ => {
                 // unreachable, throw error
                 let error_code = format!("CMD_NOT_FOUND-{}", generate_error_code());
                 nay!(
                     "Unknown Command Run: {} CODE: {}",
                     data.command_name,
                     error_code.clone()
                 );
                 error_command_response(&data.ctx, &data.command, error_code).await;
             },
         }

         Ok(())
     }
 }
 */
