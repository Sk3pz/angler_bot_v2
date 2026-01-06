pub mod shop;

use serenity::all::{
    ComponentInteraction, Context, CreateActionRow, CreateEmbed,
};
use serenity::async_trait;

pub fn get_all_embeds() -> Vec<Box<dyn InteractiveEmbed>> {
    vec![
        Box::new(shop::FishShopEmbed), // fix error
        // Add other embeds here
    ]
}

#[async_trait]
pub trait InteractiveEmbed: Send + Sync {
    fn name(&self) -> &'static str;
    // We use fully qualified names to prevent type mismatch errors
    fn create(&self) -> (CreateEmbed, Vec<CreateActionRow>);

    async fn handle(
        &self,
        ctx: &Context,
        interaction: &ComponentInteraction,
    ) -> Result<bool, String>;
}

#[macro_export]
macro_rules! embed {
    (
        struct: $struct_name:ident,
        title: $title:expr,
        desc: $desc:expr,
        color: $color:expr,
        // Make footer optional with $(...)?
        $(footer: $footer:expr,)?
        // Make thumbnail optional with $(...)?
        $(thumbnail: $thumbnail:expr,)?

        $(fields: [ $( ($f_title:expr, $f_desc:expr, $f_inline:expr) ),* ],)?
        rows: [
            $(
                row: [
                    $(
                        $comp_type:ident (
                            id: $id:literal
                            $(, $key:ident : $value:expr )*
                        ) => async |$ctx:ident, $msg:ident| $body:block
                    ),*
                ]
            ),*
        ]
    ) => {
        pub struct $struct_name;

        #[serenity::async_trait]
        impl crate::embeds::InteractiveEmbed for $struct_name {
            fn name(&self) -> &'static str { stringify!($struct_name) }

            fn create(&self) -> (serenity::all::CreateEmbed, Vec<serenity::all::CreateActionRow>) {
                #[allow(unused_mut)]
                let mut embed = serenity::builder::CreateEmbed::new()
                    .title($title)
                    .description($desc)
                    .color($color);

                // Handle Optional Footer
                $(
                    embed = embed.footer(serenity::builder::CreateEmbedFooter::new($footer));
                )?

                // Handle Optional Thumbnail
                $(
                     embed = embed.thumbnail($thumbnail);
                )?

                $(
                    $(
                        embed = embed.field($f_title, $f_desc, $f_inline);
                    )*
                )?

                let mut rows = Vec::new();

                $(
                    let mut buttons = Vec::new();
                    let mut select_menu = None;

                    $(
                        let custom_id = concat!(stringify!($struct_name), "_", $id);

                        #[allow(unused_assignments, unused_mut)]
                        if stringify!($comp_type) == "button" {
                             let mut btn = serenity::builder::CreateButton::new(custom_id);
                             $(
                                 btn = $crate::apply_button_prop!(btn, $key, $value);
                             )*
                             buttons.push(btn);
                        }

                        #[allow(unused_assignments, unused_mut)]
                        if stringify!($comp_type) == "select" {
                             let mut options = Vec::new();
                             $(
                                 $crate::extract_select_option!($key, $value, options);
                             )*
                             
                             let kind = serenity::builder::CreateSelectMenuKind::String { options };
                             let mut sel = serenity::builder::CreateSelectMenu::new(
                                 custom_id,
                                 kind
                             );
                             $(
                                 sel = $crate::apply_select_prop!(sel, $key, $value);
                             )*
                             select_menu = Some(sel);
                        }
                    )*

                    if let Some(sel) = select_menu {
                        rows.push(serenity::builder::CreateActionRow::SelectMenu(sel));
                    } else if !buttons.is_empty() {
                        rows.push(serenity::builder::CreateActionRow::Buttons(buttons));
                    }
                )*

                (embed, rows)
            }

            async fn handle(
                &self,
                ctx: &serenity::all::Context,
                interaction: &serenity::all::ComponentInteraction
            ) -> Result<bool, String> {
                if !interaction.data.custom_id.starts_with(concat!(stringify!($struct_name), "_")) {
                    return Ok(false);
                }

                match interaction.data.custom_id.as_str() {
                    $(
                        $(
                            concat!(stringify!($struct_name), "_", $id) => {
                                let $ctx = ctx;
                                let $msg = interaction;

                                async {
                                    $body
                                }.await;

                                return Ok(true);
                            }
                        ),*
                    ),*
                    _ => Ok(false)
                }
            }
        }
    };
}

// Helpers
#[macro_export]
macro_rules! apply_button_prop {
    ($b:ident, label, $v:expr) => { $b.label($v) };
    ($b:ident, style, $v:expr) => { $b.style($v) };
    ($b:ident, emoji, $v:expr) => { $b.emoji($v) };
    ($b:ident, disabled, $v:expr) => { $b.disabled($v) };
    ($b:ident, $k:ident, $v:expr) => { $b };
}

#[macro_export]
macro_rules! extract_select_option {
    (options, $v:expr, $vec:ident) => {
        $vec = $v.into_iter().map(|(label, val)|
            serenity::builder::CreateSelectMenuOption::new(label, val)
        ).collect();
    };
    ($k:ident, $v:expr, $vec:ident) => {};
}

#[macro_export]
macro_rules! apply_select_prop {
    ($s:ident, placeholder, $v:expr) => { $s.placeholder($v) };
    ($s:ident, options, $v:expr) => { $s }; // Already handled
    ($s:ident, $k:ident, $v:expr) => { $s };
}