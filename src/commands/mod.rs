use rand::seq::IndexedRandom;
use serenity::{
    all::{
        Attachment, ChannelId, Command, CommandInteraction, CommandOptionType, Context,
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage, GuildId, PartialChannel, ResolvedOption, ResolvedValue,
        Role, User,
    },
    async_trait,
};

use crate::nay;

mod admin;
mod cast;
mod game_tips;
mod info;

pub fn get_all_cmds() -> Vec<Box<dyn BotCommand>> {
    vec![
        // regular commands:
        Box::new(cast::CastCommand),
        Box::new(info::InfoCommand),
        // admin commands
        Box::new(admin::register_channel::RegisterChannelCommand),
    ]
}

pub async fn register_command(ctx: &Context, cmd: CreateCommand) {
    if let Err(e) = Command::create_global_command(&ctx.http, cmd).await {
        nay!("Failed to register a command: {}", e);
    }
}

pub async fn command_response<S: Into<String>>(
    ctx: &Context,
    command: &CommandInteraction,
    msg: S,
) {
    let data = CreateInteractionResponseMessage::new().content(msg.into());
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub async fn command_response_ephemeral<S: Into<String>>(
    ctx: &Context,
    command: &CommandInteraction,
    msg: S,
) {
    let data = CreateInteractionResponseMessage::new()
        .content(msg.into())
        .ephemeral(true);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        nay!("Failed to respond to command: {}", err)
    }
}

pub async fn error_command_response<S: Into<String>>(
    ctx: &Context,
    command: &CommandInteraction,
    error_code: S,
) {
    let code = error_code.into();

    let responses = vec![
        "🐡🐡🐡 You discovered Bob Blowfish's Blunder! 🐡🐡🐡 Please report this! 🐡🐡🐡 Error Code: ",
        "🐟🐟🐟 You encountered Minny Minnow's Mistake! 🐟🐟🐟 Please report this! 🐟🐟🐟 Error Code: ",
        "🐋🐋🐋 You encountered Wally Whale's Whoopsie! 🐋🐋🐋 Please report this! 🐋🐋🐋 Error Code: ",
        "🦈🦈🦈 You encountered Sally Shark's Short-Circuit! 🦈🦈🦈 Please report this! 🦈🦈🦈 Error Code: ",
    ];

    let chosen_response = responses
        .choose(&mut rand::rng())
        .expect("Error message list should never be empty!");

    let chosen_response = format!("{}`{}`", chosen_response, code);

    command_response_ephemeral(ctx, command, chosen_response).await;
}

#[async_trait]
pub trait BotCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn requires_guild(&self) -> bool;
    fn is_admin(&self) -> bool {
        false
    }

    // returns the builder to send to Discord
    fn register(&self) -> CreateCommand;

    // execution logic
    async fn run(&self, data: &CommandData<'_>) -> Result<(), String>;
}

pub struct CommandData<'a> {
    pub command_name: String,
    pub handler: &'a crate::discord::Handler,
    pub ctx: &'a Context,
    pub command: &'a CommandInteraction,
    pub sender: &'a User,
    // None if in a DM
    pub guild_id: Option<&'a GuildId>,
    pub command_options: Vec<ResolvedOption<'a>>, // TODO: This may need to be changed later
    pub channel: ChannelId,
}

// Wrapper for arguments to move from an imperative style to a declarative style
pub trait CommandArgument<'a>: Sized {
    // what type does the argument map to?
    fn option_type() -> CommandOptionType;
    fn is_required() -> bool {
        true
    }
    // how do we pull this out of ResolvedValue?
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String>;
    // handle autocomplete types
    fn add_choice(
        builder: CreateCommandOption,
        name: impl Into<String>,
        value: Self,
    ) -> CreateCommandOption {
        // default impl just returns builder (useful for non-autocomplete types)
        builder
    }
}

impl<'a> CommandArgument<'a> for i64 {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Integer
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Integer(v)) => Ok(*v),
            Some(_) => Err("Expected Integer".into()),
            None => Err("Missing required Integer".into()),
        }
    }
    // Note: Serenity's add_int_choice takes i32, so we cast.
    fn add_choice(
        builder: CreateCommandOption,
        name: impl Into<String>,
        value: Self,
    ) -> CreateCommandOption {
        builder.add_int_choice(name, value as i32)
    }
}

impl<'a> CommandArgument<'a> for f64 {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Number
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Number(v)) => Ok(*v),
            Some(_) => Err("Expected Integer".into()),
            None => Err("Missing required Integer".into()),
        }
    }
    fn add_choice(
        builder: CreateCommandOption,
        name: impl Into<String>,
        value: Self,
    ) -> CreateCommandOption {
        builder.add_number_choice(name, value)
    }
}

impl<'a> CommandArgument<'a> for bool {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Boolean
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Boolean(v)) => Ok(*v),
            Some(_) => Err("Expected Boolean".into()),
            None => Err("Missing required Boolean".into()),
        }
    }
}

impl<'a> CommandArgument<'a> for String {
    fn option_type() -> CommandOptionType {
        CommandOptionType::String
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::String(v)) => Ok(v.to_string()),
            Some(_) => Err("Expected String".into()),
            None => Err("Missing required String".into()),
        }
    }
    fn add_choice(
        builder: CreateCommandOption,
        name: impl Into<String>,
        value: Self,
    ) -> CreateCommandOption {
        let name = name.into();
        builder.add_string_choice(name, value)
    }
}

impl<'a> CommandArgument<'a> for &'a User {
    fn option_type() -> CommandOptionType {
        CommandOptionType::User
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::User(user, _)) => Ok(user),
            Some(_) => Err("Expected User".into()),
            None => Err("Missing required User".into()),
        }
    }
}

impl<'a> CommandArgument<'a> for &'a Attachment {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Attachment
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Attachment(v)) => Ok(v),
            Some(_) => Err("Expected Attachment".into()),
            None => Err("Missing required Attachment".into()),
        }
    }
}

impl<'a> CommandArgument<'a> for &'a Role {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Role
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Role(v)) => Ok(v),
            Some(_) => Err("Expected User".into()),
            None => Err("Missing required User".into()),
        }
    }
}

impl<'a> CommandArgument<'a> for &'a PartialChannel {
    fn option_type() -> CommandOptionType {
        CommandOptionType::Channel
    }
    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            Some(ResolvedValue::Channel(v)) => Ok(v),
            Some(_) => Err("Expected User".into()),
            None => Err("Missing required User".into()),
        }
    }
}

// This allows you to write `Option<i64>` in your command to make it optional
impl<'a, T> CommandArgument<'a> for Option<T>
where
    T: CommandArgument<'a>,
{
    fn option_type() -> CommandOptionType {
        T::option_type()
    }
    fn is_required() -> bool {
        false
    }

    fn extract(value: Option<&'a ResolvedValue<'a>>) -> Result<Self, String> {
        match value {
            None => Ok(None),
            Some(v) => T::extract(Some(v)).map(Some),
        }
    }

    // Forward the choice adding to the inner type T
    fn add_choice(
        builder: CreateCommandOption,
        name: impl Into<String>,
        value: Self,
    ) -> CreateCommandOption {
        if let Some(v) = value {
            T::add_choice(builder, name, v)
        } else {
            builder
        }
    }
}

/// Macro to define a command with metadata and logic in a declarative style.
/// Examples:
// command! {
//     struct: PingCommand,
//     name: "ping",
//     desc: "pong!",
//     requires_guild: false,

//     run: async |data| {
//         Ok(format!("Pong!"))
//     }
// }
///
#[macro_export]
macro_rules! command {
    (
        // Metadata Block
        struct: $struct_name:ident,
        name: $name:expr,
        desc: $desc:expr,
        requires_guild: $req_guild:expr,
        $( is_admin_command: $is_admin:expr, )?

        // Logic Block
        // Parse the pattern |data, arg(description | (autofill_name : choice), ...): type, ...| WITH [PERMISSIONS...] { body }
        run: async |$data:ident $(, $arg_name:ident($arg_desc:literal $(: [ $( $choice_name:literal : $choice:literal ),* ])?) : $arg_type:ty )*|
        // Subcommand Handling
        $(
            sub: $sub_name:ident ($sub_desc:literal) => async
            |$sub_data:ident $(, $s_arg_name:ident ( $s_arg_desc:literal $(| [ $( $s_choice_name:literal : $s_choice_val:literal ),* ] )? ) : $s_arg_type:ty )*|
            $sub_body:block
        )*
        // Syntax: WITH [ADMINISTRATOR, MANAGE_GUILD, ...]
        $(WITH [ $( $perm:ident ),* ] )?
        $body:block
    ) => {
        pub struct $struct_name;

        #[serenity::async_trait]
        impl crate::commands::BotCommand for $struct_name {
            fn name(&self) -> &'static str { $name }
            fn requires_guild(&self) -> bool { $req_guild }
            $( fn is_admin(&self) -> bool { $is_admin } )?

            fn register(&self) -> serenity::builder::CreateCommand {
                let mut cmd = serenity::builder::CreateCommand::new($name)
                .description($desc)
                .dm_permission(!$req_guild);

                // default to None (allow everyone)
                #[allow(unused_mut)]
                #[allow(unused_assignments)]
                let mut required_perms: Option<serenity::all::Permissions> = None;

                // This block runs only if 'WITH [...]' is present
                $(
                    let mut p = serenity::all::Permissions::empty();
                    $(
                        // Combine flags: p = p | Permissions::FLAG
                        p |= serenity::all::Permissions::$perm;
                    )*
                    required_perms = Some(p);
                )?

                // Apply to builder if set
                if let Some(perms) = required_perms {
                    cmd = cmd.default_member_permissions(perms);
                }

                // Auto Registration
                $(
                    #[allow(unused_mut)]
                    let mut opt = serenity::builder::CreateCommandOption::new(
                        <$arg_type as crate::commands::CommandArgument>::option_type(),
                        stringify!($arg_name),
                        $arg_desc
                    )
                    .required(<$arg_type as crate::commands::CommandArgument>::is_required());

                    $(
                        $(
                            opt = <$arg_type as crate::commands::CommandArgument>::add_choice(
                                opt,
                                $choice_name.to_string(),
                                $choice.into()
                            );
                        )*
                    )?

                    cmd = cmd.add_option(opt);
                )*

                // --- Subcommands Registration ---
                $(
                    // Create the Subcommand Option
                    #[allow(unused_assignments, unused_mut)]
                    let mut sub_cmd = serenity::builder::CreateCommandOption::new(
                        serenity::all::CommandOptionType::SubCommand,
                        stringify!($sub_name),
                        $sub_desc
                    );

                    // Register Arguments for the Subcommand
                    $(
                        let mut sub_opt = serenity::builder::CreateCommandOption::new(
                            <$s_arg_type as crate::commands::CommandArgument>::option_type(),
                            stringify!($s_arg_name),
                            $s_arg_desc
                        )
                        .required(<$s_arg_type as crate::commands::CommandArgument>::is_required());

                        #[allow(unused_assignments, unused_mut)]
                        let mut sub_has_choices = false;
                        $(
                            sub_has_choices = true;
                            $( sub_opt = <$s_arg_type as crate::commands::CommandArgument>::add_choice(sub_opt, $s_choice_name.to_string(), $s_choice_val.into()); )*
                        )?
                        if sub_has_choices { sub_opt = sub_opt.set_autocomplete(false); }

                        // Add the argument TO the subcommand
                        sub_cmd = sub_cmd.add_sub_option(sub_opt);
                    )*

                    // Add the subcommand TO the main command
                    cmd = cmd.add_option(sub_cmd);
                )*

                cmd
            }

            async fn run(&self, $data: &crate::commands::CommandData<'_>) -> Result<(), String> {
                //  Subcommand Extraction
                $(
                    if let Some(sub_option) = $data.command_options.iter().find(|o| o.name == stringify!($sub_name)) {
                        // extract the INNER options (the arguments passed to the subcommand)
                        // ResolvedValue::SubCommand contains a Vec<ResolvedOption>
                        if let serenity::all::ResolvedValue::SubCommand(_inner_options) = &sub_option.value {
                            // Extract Arguments using _inner_options
                            let $sub_data = $data; // Alias data for the sub block
                            $(
                                let option_val = _inner_options.iter()
                                    .find(|opt| opt.name == stringify!($s_arg_name))
                                    .map(|opt| &opt.value);
                                let $s_arg_name = <$s_arg_type as crate::commands::CommandArgument>::extract(option_val)?;
                            )*

                            // Run Subcommand Body
                            return async move {
                                $sub_body
                            }.await;
                        }
                    }
                )*

                // Extraction
                $(
                    // Find the option by name
                    let option_val = $data.command_options.iter()
                        .find(|opt| opt.name == stringify!($arg_name))
                        .map(|opt| &opt.value);

                    // Extract using the type's trait implementation
                    let $arg_name = <$arg_type as crate::commands::CommandArgument>::extract(option_val)?;
                )*

                // Execute User Logic
                // $arg_name is available in this block
                async move {
                    $body
                }.await
            }
        }
    };

    // default requires_guild to false
    (
        struct: $struct_name:ident,
        name: $name:expr,
        desc: $desc:expr,
        $( is_admin_command: $is_admin:expr, )?

        // 1. Capture the EXACT same patterns as Arm 1
        run: async |$data:ident $(, $arg_name:ident ( $arg_desc:literal $(| [ $( $choice_name:literal : $choice_val:literal ),* ] )? ) : $arg_type:ty )*|

        // Match Subcommands
        $(
            sub: $sub_name:ident ($sub_desc:literal) => async
            |$sub_data:ident $(, $s_arg_name:ident ( $s_arg_desc:literal $(: [ $( $s_choice_name:literal : $s_choice_val:literal ),* ] )? ) : $s_arg_type:ty )*|
            $sub_body:block
        )*

        // 2. Capture Permissions
        $( WITH [ $( $perm:ident ),* ] )?

        $body:block
    ) => {
        // Recursive Call:
        // We reconstruct the tokens exactly so they match Arm 1's pattern.
        $crate::command!(
            struct: $struct_name,
            name: $name,
            desc: $desc,
            requires_guild: false, // <--- Default is applied here
            $( is_admin_command: $is_admin:expr, )?

            // Forward arguments, choices, and descriptions
            run: async |$data $(, $arg_name ( $arg_desc $(| [ $( $choice_name : $choice_val ),* ] )? ) : $arg_type )*|

            // Forward Subcommands
            $(
                sub: $sub_name ($sub_desc) => async
                |$sub_data $(, $s_arg_name ( $s_arg_desc $(| [ $( $s_choice_name : $s_choice_val ),* ] )? ) : $s_arg_type )*|
                $sub_body
           )*

            // Forward permissions
            $( WITH [ $( $perm ),* ] )?

            $body
        );
    };
}
