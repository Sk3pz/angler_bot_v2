use rand::seq::IndexedRandom;
use serenity::{
    all::{
        Attachment, ChannelId, Command, CommandInteraction, CommandOptionType, Context,
        CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId,
        PartialChannel, ResolvedOption, ResolvedValue, Role, User,
    },
    async_trait,
};

use crate::nay;

mod ping;

pub fn get_all_cmds() -> Vec<Box<dyn BotCommand>> {
    vec![Box::new(ping::PingCommand)]
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

    let chosen_response = format!("{}{}", chosen_response, code);

    command_response(ctx, command, chosen_response).await;
}

#[async_trait]
pub trait BotCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn requires_guild(&self) -> bool;

    // returns the builder to send to Discord
    fn register(&self) -> CreateCommand;

    // execution logic
    async fn run(&self, data: &CommandData<'_>) -> Result<String, String>;
}

pub struct CommandData<'a> {
    pub command_name: String,
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
/// command! {
///     struct: SellCommand,
///     name: "sell",
///     desc: "Sell an item from your inventory",
///
///     run: async |data, item_index: i64| {
///         // 'item_index' is available here as an i64
///         let adjusted_index = item_index - 1;
///
///         // Simulate Logic
///         if adjusted_index < 0 {
///              return Err(ReelError::CommandError("Index too small".into()));
///         }
///
///         Ok(format!("Sold item at index {}", adjusted_index))
///     }
/// }
///
#[macro_export]
macro_rules! command {
    (
        // Metadata Block
        struct: $struct_name:ident,
        name: $name:expr,
        desc: $desc:expr,
        requires_guild: $req_guild:expr,

        // Logic Block
        // Parse the pattern |data, arg: type| { body }
        run: async |$data:ident $(, $arg_name:ident : $arg_type:ty ),*| $body:block
    ) => {
        pub struct $struct_name;

        #[serenity::async_trait]
        impl $crate::commands::BotCommand for $struct_name {
            fn name(&self) -> &'static str { $name }
            fn description(&self) -> &'static str { $desc }
            fn requires_guild(&self) -> bool { $req_guild }

            fn register(&self) -> serenity::builder::CreateCommand {
                let mut cmd = serenity::builder::CreateCommand::new($name)
                .description($desc)
                .dm_permission(!$req_guild);

                // Auto Registration
                $(
                    let opt = serenity::builder::CreateCommandOption::new(
                        <$arg_type as $crate::commands::CommandArgument>::option_type(),
                        stringify!($arg_name),
                        "Argument"
                    )
                    .required(<$arg_type as $crate::commands::CommandArgument>::is_required());

                    cmd = cmd.add_option(opt);
                )*
                cmd
            }

            async fn run(&self, $data: &$crate::commands::CommandData<'_>) -> Result<String, String> {
                // Extraction
                $(
                    // Find the option by name
                    let option_val = $data.command_options.iter()
                        .find(|opt| opt.name == stringify!($arg_name))
                        .map(|opt| &opt.value);

                    // Extract using the type's trait implementation
                    let $arg_name = <$arg_type as $crate::commands::CommandArgument>::extract(option_val)?;
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

        run: async |$data:ident $(, $arg_name:ident : $arg_type:ty ),*| $body:block
    ) => {
        // Recursively call the main macro, injecting 'false'
        command!(
            struct: $struct_name,
            name: $name,
            desc: $desc,
            requires_guild: false, // <--- DEFAULT APPLIED HERE
            run: async |$data, $( $arg_name : $arg_type ),*| $body
        );
    };
}
