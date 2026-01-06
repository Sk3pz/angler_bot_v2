use crate::embed;

fn get_bait_options() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Worms ($5)", "worms"),
        ("Crickets ($10)", "crickets"),
        ("Minnow ($15)", "minnow"),
    ]
}

embed! {
    struct: FishShopEmbed,
    title: "🎣 The Angler's Shop",
    desc: "Welcome! Select an item below to purchase.",
    color: 0x00a8ff,
    footer: "Happy Fishing! 🎣",

    fields: [
        ("Current Balance", "$500", true),
        ("Daily Deal", "Golden Rod - 50% off!", true)
    ],

    rows: [
        row: [
            select(
                id: "bait_select",
                placeholder: "Select your bait...",
                options: get_bait_options()
            ) => async |ctx, interaction| {
                // Safely extract values using fully qualified path
                let values = match &interaction.data.kind {
                    serenity::all::ComponentInteractionDataKind::StringSelect { values } => values,
                    _ => return,
                };

                if values.is_empty() { return; }
                let selected_value = &values[0];

                let r = serenity::builder::CreateInteractionResponseMessage::new()
                    .content(format!("You selected: **{}**", selected_value))
                    .ephemeral(true);

                let b = serenity::builder::CreateInteractionResponse::Message(r);
                let _ = interaction.create_response(&ctx.http, b).await;
            }
        ],

        row: [
            button(
                id: "buy_btn",
                label: "Buy Item",
                style: serenity::all::ButtonStyle::Success
            ) => async |ctx, interaction| {
                let r = serenity::builder::CreateInteractionResponseMessage::new()
                    .content("Transaction complete! 🐟")
                    .ephemeral(true);
                let b = serenity::builder::CreateInteractionResponse::Message(r);
                let _ = interaction.create_response(&ctx.http, b).await;
            },

            button(
                id: "cancel_btn",
                label: "Leave Shop",
                style: serenity::all::ButtonStyle::Danger
            ) => async |ctx, interaction| {
                let _ = interaction.message.delete(&ctx.http).await;
            }
        ]
    ]
}