use crate::{command, commands::command_response_ephemeral};

command! {
    struct: PingCommand,
    name: "ping",
    desc: "pong!",
    requires_guild: false,

    run: async |data, times("How many times to print pong"): Option<i64>| {
        let times = times.unwrap_or(1).clamp(1, 5);
        let response = "pong!\n".repeat(times as usize);
        command_response_ephemeral(&data.ctx, &data.command, response).await;
        Ok(())
    }
}
