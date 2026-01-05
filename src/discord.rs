use serenity::{
    all::{
        ActivityData, Command, Context, EventHandler, Interaction, Message, OnlineStatus, Ready,
        ResumedEvent,
    },
    async_trait,
};

use crate::{
    commands::{
        CommandData, command_response, command_response_ephemeral, error_command_response,
        get_all_cmds, register_command,
    },
    helpers::generate_error_code,
    nay, yay,
};

pub struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _: Context, msg: Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        // handle messages
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let debug = cfg!(debug_assertions);

        // unregister previous commands if in debug mode (debug mode means we are activtly developing and commands may change frequently)
        // note: for this to take effect on a client they must restart or hit Ctrl+R in discord.
        if debug {
            let commands = Command::get_global_commands(&ctx.http).await.unwrap();
            for command in commands {
                if let Err(e) = Command::delete_global_command(&ctx.http, command.id).await {
                    nay!("Failed to delete command: {}", e);
                }
            }
        }

        // register commands
        for cmd in get_all_cmds() {
            register_command(&ctx, cmd.register()).await;
        }

        // Log that the bot is ready
        yay!("{} is connected!", ready.user.name);

        // set bot activity
        if debug {
            ctx.set_presence(
                Some(ActivityData::custom("Stocking the pond...")),
                OnlineStatus::Online,
            );
        } else {
            ctx.set_presence(Some(ActivityData::playing("/fish")), OnlineStatus::Online);
        }
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        // no resume logic needed right now
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // get the name of the command
            let command_name = &command.data.name;

            // build the data
            let cmd_data = CommandData {
                //command_name: command_name.clone(),
                ctx: &ctx,
                command: &command,
                sender: &command.user,
                guild_id: command.guild_id.as_ref(),
                command_options: command.data.options(),
                channel: command.channel_id,
            };

            // Find the command to run
            let commands = get_all_cmds();
            let cmd_name_str = command_name.as_str();
            if let Some(cmd) = commands.iter().find(|c| c.name() == cmd_name_str) {
                // guild-only command check
                if cmd.requires_guild() && cmd_data.guild_id.is_none() {
                    command_response(
                        &ctx,
                        &command,
                        "You must be in a server to use that command!",
                    )
                    .await;
                    return;
                }

                // run
                if let Err(e) = cmd.run(&cmd_data).await {
                    command_response_ephemeral(&ctx, &command, e).await;
                }
            } else {
                // command not found (shouldn't happen)
                let error_code = format!("CMD_NOT_FOUND-{}", generate_error_code());
                nay!(
                    "Unknown Command Run: {} CODE: {}",
                    command_name,
                    error_code.clone()
                );
                error_command_response(&ctx, &command, error_code).await;
                return;
            }
        }
    }
}
