use crate::command;

command! {
    struct: InfoCommand,
    name: "info",
    desc: "List your stats and loadout information.",
    requires_guild: false,

    run: async |data| {

        Ok(())
    }
}