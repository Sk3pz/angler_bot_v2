use crate::command;

command! {
    struct: PingCommand,
    name: "ping",
    desc: "pong!",
    requires_guild: false,

    run: async |data| {
        Ok(format!("Pong!"))
    }
}
