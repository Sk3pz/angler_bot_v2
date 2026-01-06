use crate::command;

// Ping command is for testing new systems. Here we are implementing an example shop system to test the response macro.
// This command will create a navigatable shop with buttons to buy items.
// You will also be able to select the item you want to buy.
// The shop should update dynamically and show the item you have selected. The item selected should be shown per user
// The shop should have a back and forward button to navigate through the shop.
command! {
    struct: PingCommand,
    name: "ping",
    desc: "pong!",
    requires_guild: false,

    run: async |data| {
        use crate::embeds::InteractiveEmbed;
        use crate::embeds::shop::FishShopEmbed;

        let shop = FishShopEmbed;
        let (embed, components) = shop.create();

        let builder = serenity::builder::CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(components)
            .ephemeral(true);

        let response = serenity::builder::CreateInteractionResponse::Message(builder);
        if let Err(e) = data.command.create_response(&data.ctx.http, response).await {
            crate::nay!("Failed to send shop embed: {}", e);
        }

        Ok(())
    }
}
