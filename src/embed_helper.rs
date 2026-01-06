#[macro_export]
macro_rules! response {
    (
        // Optional Metadata
        // We use $(...)? for optional single lines
        $(ephemeral: $ephemeral:expr,)?
        $(title: $title:expr,)?
        $(desc: $desc:expr,)?
        $(color: $color:expr,)?
        $(image: $image:expr,)?
        $(thumbnail: $thumbnail:expr,)?
        $(footer: $footer:expr,)?

        // ---------------------------------------------------------
        // FIX 1: Fields
        // Changed from `),*` to `) $(,)? )*`
        // This allows trailing commas without demanding another field.
        // ---------------------------------------------------------
        $(
            field: $f_name:expr => $f_val:expr, ($f_inline:expr) $(,)?
        )*

        // ---------------------------------------------------------
        // FIX 2: Rows
        // Same fix here. Allows trailing commas after row blocks.
        // ---------------------------------------------------------
        $(
            row: {
                $(
                    $comp_type:ident : $b_label:expr => $b_id:expr, $( ( $b_style:ident ) )? $(,)?
                )*
            } $(,)?
        )*
    ) => {
        {
            #[allow(unused_mut)]
            let mut embed = serenity::builder::CreateEmbed::new();

            // Metadata Application
            $( embed = embed.title($title); )?
            $( embed = embed.description($desc); )?
            $( embed = embed.colour($color); )?
            $( embed = embed.image($image); )?
            $( embed = embed.thumbnail($thumbnail); )?
            $( embed = embed.footer(serenity::builder::CreateEmbedFooter::new($footer)); )?

            // Field Application
            $(
                embed = embed.field($f_name, $f_val, $f_inline);
            )*

            #[allow(unused_mut)]
            let mut msg = serenity::builder::CreateInteractionResponseMessage::new();

            // Ephemeral Logic
            let is_ephemeral = false $( || $ephemeral )?;
            msg = msg.ephemeral(is_ephemeral);

            #[allow(unused_mut)]
            let mut rows = Vec::new();

            // Component Logic
            $(
                #[allow(unused_mut)]
                let mut components = Vec::new();
                $(
                    let style = {
                        #[allow(unused_variables)]
                        let s = serenity::all::ButtonStyle::Primary;
                        $(
                            let s = match stringify!($b_style) {
                                "Primary" => serenity::all::ButtonStyle::Primary,
                                "Secondary" => serenity::all::ButtonStyle::Secondary,
                                "Success" => serenity::all::ButtonStyle::Success,
                                "Danger" => serenity::all::ButtonStyle::Danger,
                                _ => serenity::all::ButtonStyle::Primary,
                            };
                        )?
                        s
                    };

                    let button = match stringify!($comp_type) {
                        "link" => serenity::builder::CreateButton::new_link($b_id).label($b_label),
                        _ => serenity::builder::CreateButton::new($b_id).label($b_label).style(style),
                    };

                    components.push(button);
                )*
                rows.push(serenity::builder::CreateActionRow::Buttons(components));
            )*

            msg = msg.embed(embed);
            msg = msg.components(rows);

            msg
        }
    };
}