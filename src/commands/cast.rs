use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use std::time::Duration;
use rand::distr::Alphanumeric;
use rand::Rng;
use crate::{command, nay, say, wow};
use serenity::all::{ButtonStyle, ChannelId, Color, CommandInteraction, Context, CreateActionRow, CreateAttachment, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, EditInteractionResponse, Mentionable, UserId};
use serenity::builder::CreateEmbedFooter;
use crate::commands::{command_response_ephemeral, error_command_response};
use crate::commands::game_tips::random_tip;
use crate::data_management::config::Config;
use crate::fishing::fish_data::fish::{Fish, Pond};
use crate::fishing::fish_data::rarity::FishRarity;
use crate::fishing::rod_data::bait::Bait;
use crate::helpers::generate_error_code;

const MYSTERIOUS_MESSAGES: [&str;70] = [
    // Mysterious Comments
    "...",
    "I wonder what could be down there...",
    "The water is so still, it's almost like it's waiting for something.",
    "Maybe there's something lurking in the depths?",
    "The water is darker than it looks.",
    "Something just brushed against your line!",
    "Do you feel like somethings staring back too?",
    "Your reflection in the water doesn't look like you...",
    "A cold chill runs down your spine.",
    "Was that the wind or did someone just whisper something?",
    "Is it just me or is the fog getting thicker?",
    "My fish finder just screamed and died. Weird.",
    "The birds stopped singing.. When did that happen?",
    "Don't look down into the water, you might not like what you see.",
    "I swear that pier wasn't there yesterday...",
    "My bait is shivering.. I think it's scared of something down there.",
    "Keep your hands inside the boat.. uh.. just in case, you know?",
    "Why is the water warm? I thought it was supposed to be cold this time of year.",
    "Did the shore just get farther away? I thought I was closer to it.",
    "I don't think the water rippled from your cast..",
    "Something moved beneath the surface",
    "The GPS just died. You're lucky I know this place like the back of my hand! ...We came from that direction, right?",
    "Does your phone say it has negative signal too..? What does that even mean?",
    "This pond feels deeper today.",
    "I think my watch just moved.. backwards?",
    "The water looks.. thicker?",
    "I'm not the only one who heard that, right?",
    "I think we are being watched.. I can feel it.",

    // "Expert" Guide
    "Dont worry, the scratching on the boat is normal.",
    "My uncle said this pond is cursed. He was such a funny guy! Wonder what happened to him..",
    "You know, this pond isn't on modern maps. I wonder why that is?",
    "If you catch a boot, just throw it back. Trust me.",
    "My guidebook says 'Turn Back', but I think that's just a typo for 'Trout Back'.",
    "I bought this boat from a guy who said he was the 'soul survivor'. Dramatic much?",
    "I never see any other boats out here. I wonder if I'm the only one who knows about this place?",
    "Yeah, so the last guy who sat in that seat.. well, never mind.",
    "Check this out! The warranty for the boat expired in 1902!",
    "If something in the water asks for a 'toll', just throw it your sandwich. It seems to work.",
    "Keep your voice down, The catfish speak English here. They can be very rude!",
    "The IRS won't go near this pond.. I haven't paid taxes in years! I wonder why that is?",
    "I recommend catch and release. They hold grudges!",
    "Do not, I repeat, do NOT make eye contact with the frogs.",
    "If the pond starts glowing, reel slower. Trust me.",
    "I read about this place once. I stopped halfway through. I hate horror stories!",
    "If your line comes back warm, just cut it.",
    "If you hear your name, it's better to just ignore it.",
    "Sometimes the pond borrows things. It usually gives them back.",
    "Don't anchor the boat, it hates that.",
    "You're doing fine. Better than the last guy.",
    "I feel like the pond changes depths sometimes.. I wonder if that means the fish change too?",
    "This spot used to be shallower. I think.",
    "Don't worry, I've seen worse casts end just fine!",
    "That glow? Probably algae.",
    "I wouldn't reel too fast. Some things chase.",
    "I swear I just saw my shadow move..",
    "Never go swimming here.. I don't know why, but I just have a bad feeling about it.",

    // Incompetent / Mundane Comments
    "Do fish sleep? Maybe we should scream to wake them up.",
    "I hope I turned the stove off before I left...",
    "I think my foot is falling asleep...",
    "Wait, is this even the right spot?",
    "Does this life vest make me look fat?",
    "I forgot to tell my wife we were going.. Oops.",
    "Do you think fish like peanut butter? I have some in my bag.",
    "I hope I don't get seasick..",
    "I think I swallowed a bug.",
    "Does this hat make me look professional?",
    "My horoscope said I should 'Avoid water at all costs' today.. I wonder why that is?",
    "I think the developer forgot to program the bottom of this pond.",
    "I just remembered I don't like deep water.",
    "I should stretch more.",
];

pub struct CastHandler {
    ctx: Context,
    user: UserId,
    channel: ChannelId,
    fish: Option<Fish>,
    users_fishing: Arc<Mutex<HashSet<UserId>>>,
    canceled: Arc<AtomicBool>,
    interaction: CommandInteraction,
}

command! {
    struct: CastCommand,
    name: "cast",
    desc: "Cast your line into the pond",
    requires_guild: false,

    run: async |data| {

        // ensure the user is not already casting
        let user_id = data.sender.id;
        {
            let fishing_set = data.handler.users_fishing.lock().await;
            if fishing_set.contains(&user_id) {
                command_response_ephemeral(&data.ctx, &data.command,
                    "You are already fishing!").await;
                return Ok(());
            }
        }

        // add the user to the set of users currently fishing
        let users_fishing = data.handler.users_fishing.clone();
        {
            let mut fishing_set = users_fishing.lock().await;
            fishing_set.insert(user_id);
        }


        // get the user file
        #[cfg(feature = "guild_relative_userdata")]
        let userfile = {
            let Some(guild_id) = data.guild_id else {
                command_response_ephemeral(&data.ctx, &data.command,
                    "You must be in a guild to execute that command!");
                return Ok(());
            };

            crate::data_management::userfile::UserFile::read(&data.sender.id, guild_id)
        };
        #[cfg(not(feature = "guild_relative_userdata"))]
        let userfile = crate::data_management::userfile::UserFile::read(&data.sender.id);

        // load the pond
        let Ok(pond) = Pond::load() else {
            command_response_ephemeral(&data.ctx, &data.command,
                "Pond is closed! We are having some technical issues, please stand by!").await;
            return Ok(());
        };

        let loadout = userfile.file.loadout.clone();

        let Ok(generated_depth) = loadout.sinker.generate_depth() else {
            // Sinker Failure To Generate Error
            let error_code = format!("SINKER_FTG-{}", generate_error_code());
                    nay!(
                        "Sinker Fail To Generate Error: {} CODE: {}",
                        data.command_name,
                        error_code.clone()
                    );
                    error_command_response(&data.ctx, &data.command, error_code).await;
            return Ok(());
        };

        let bait: Option<&Bait> = if let Some(bait) = &loadout.bait {
            Some(bait)
        } else {
            None
        };

        // generate the fish from the pond
        let Ok(fish) = pond.generate_fish(generated_depth, bait) else {
            // Fish Failure To Generate Error
            let error_code = format!("FISH_FTG-{}", generate_error_code());
                    nay!(
                        "Fish Fail To Generate Error: {} CODE: {}",
                        data.command_name,
                        error_code.clone()
                    );
                    error_command_response(&data.ctx, &data.command, error_code).await;
            return Ok(());
        };

        let config = Config::load();

        // calculate the catch time
        let mut catch_time = loadout.generate_catch_time();
        if let Some(f) = &fish {
            let weight_catch_time = (f.weight - f.fish_type.weight_range.average) * config.fishing.fish_weight_time_multiplier;
            catch_time += weight_catch_time;
        }

        // log cast information
        if config.general.log_cast_data {
            match &fish {
                Some(f) => {
                    match f.fish_type.rarity {
                        FishRarity::Legendary
                        | FishRarity::Mythical => wow!("{} is attempting to catch a {} in {}seconds!", data.sender.display_name(), f, catch_time),
                        _ => say!("{} is attempting to catch a {} in {}seconds!", data.sender.display_name(), f, catch_time),
                    }
                }
                None => {
                    say!("{} is attempting to catch nothing in {}seconds!", data.sender.display_name(), catch_time);
                }
            }
        }

        let canceled = Arc::new(AtomicBool::new(false));

        let cast = CastHandler {
            ctx: data.ctx.clone(),
            user: data.sender.id.clone(),
            channel: data.channel.clone(),
            fish,
            users_fishing: users_fishing.clone(),
            canceled: canceled.clone(),
            interaction: data.command.clone(),
        };

        // schedule the catch
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(catch_time as u64)).await;
            catch(cast).await;
        });

        // let the user know they have cast their line
        let random_mysterious_message = MYSTERIOUS_MESSAGES[rand::rng().random_range(0..MYSTERIOUS_MESSAGES.len())];

        // create the embed

        let depth_display = if userfile.file.loadout.has_depth_finder {
            format!("{:.2} ft", generated_depth)
        } else {
            "??? ft".to_string()
        };

        let embed = CreateEmbed::new()
        .title(format!("You cast your {} into the pond!", loadout.rod.name))
        //.description(format!("\n**Strange Angler:** *{}*\n\nCast to {}. Waiting for a bite...", random_mysterious_message, depth_display))
        .description("Waiting for a bite...".to_string())
        .fields(vec![
            ("Cast Depth", format!("{}", depth_display), false),
            ("Strange Angler", format!("*{}*", random_mysterious_message), false),
        ])
        .thumbnail("attachment://FishingRod.png")
        .color(0x3498db)
        .footer(CreateEmbedFooter::new(format!("{}", random_tip())));

        let button_id = format!("cancel_cast_{}", user_id);
        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(&button_id)
                .label("Reel In (Cancel)")
                .style(ButtonStyle::Danger),
        ]);

        let mut message = CreateInteractionResponseMessage::new()
                .ephemeral(true)
                .embed(embed)
                .components(vec![buttons]);

        let attachment = CreateAttachment::path("./assets/FishingRod.png").await;

        // use the attachment if found
        if let Ok(file) = attachment {
            message = message.add_file(file);
        }

        let builder = CreateInteractionResponse::Message(message);

        if let Err(e) = data.command.create_response(&data.ctx, builder).await {
            nay!("Failed to send cast message: {}", e);
            let mut fishing_set = users_fishing.lock().await;
            fishing_set.remove(&user_id);
            return Ok(());
        }

        // wait for the user to click the cancel button
        let Ok(interaction) = data.command.get_response(&data.ctx).await else {
            // Failed to get the response message, likely due to a timeout or other issue. Just return and let the catch happen.
            return Ok(());
        };
        let interaction = interaction.await_component_interaction(&data.ctx)
            .author_id(user_id)
            .timeout(Duration::from_secs(catch_time as u64))
            .await;

        if let Some(interaction) = interaction {
            if interaction.data.custom_id == button_id {
                // User clicked cancel
                say!("{} clicked the cancel button!", data.sender.display_name());
                canceled.store(true, Ordering::Relaxed);

                let update_embed = CreateEmbed::new()
                    .title("Cast Canceled")
                    .description("You reeled in your line early.")
                    .color(0x95a5a6); // Grey color

                let _ = interaction.create_response(&data.ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .embed(update_embed)
                            .components(vec![]) // Remove buttons
                    )
                ).await;
            }
        }

        // command_response_ephemeral(&data.ctx, &data.command, format!("You have cast your `{}` into the pond!\n\n{}", loadout.rod.name, random_mysterious_message)).await;
        Ok(())
    }
}

// async fn send_catch_message<S: Into<String>>(catch: &CastHandler, content: S) {
//     catch.channel.send_message(&catch.ctx.http,
//                          CreateMessage::new()
//                              .content(content.into())).await.unwrap();
// }

pub async fn catch(catch: CastHandler) {
    // check if the cast was canceled during that time
    if catch.canceled.load(Ordering::Relaxed) {
        // cast was canceled, do not send a message or remove the user from the fishing set
        return;
    }

    let _ = catch.interaction.edit_response(&catch.ctx,
                                            EditInteractionResponse::new()
                                                .components(vec![]), // Empty components vector removes buttons
    ).await;

    let mut userfile = crate::data_management::userfile::UserFile::read(&catch.user);

    let config = Config::load();

    // Use up the user's bait if they had any
    if let Some(bait) = &userfile.file.loadout.bait {
        // I don't think this check is needed, but just in case I'll leave it here
        if bait.use_chance != 0.0 {
            // generate the chance of the bait being used up
            // if the bait use_chance is 0.0, it will never be used up. If it is 1.0, it will always be used up.
            let mut rng = rand::rng();
            let use_chance: f32 = rng.random_range(0.0..=1.0);
            if use_chance < bait.use_chance {
                // bait is used up
                userfile.file.loadout.bait = None;
                userfile.update();
            }
        }
    }

    // remove the user from the casting set
    {
        let mut fishing_set = catch.users_fishing.lock().await;
        fishing_set.remove(&catch.user);
    }

    // No fish on the line
    let Some(fish) = &catch.fish else {
        let embed = CreateEmbed::new()
            .title("Nothing came up!")
            .description("You felt your line go taught but nothing came up. Better luck next time!".to_string())
            .thumbnail("attachment://FishingRod.png")
            .color(0x3498db)
            .footer(CreateEmbedFooter::new(format!("{}", random_tip())));

        let mut message = CreateMessage::new()
            .content(format!("{}", catch.user.mention()))
            .embed(embed);

        let attachment = CreateAttachment::path("./assets/FishingRod.png").await;

        // use the attachment if found
        if let Ok(file) = attachment {
            message = message.add_file(file);
        }

        if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
            nay!("Failed to send cast response message: {}", e);
            return;
        }
        // send_catch_message(&catch,
        //                    format!("{} You felt your line go taught but nothing came up. Better luck next time!",
        //                             catch.user.mention())).await;
        return;
    };

    // Catch chance didn't succeed
    let caught = fish.try_hook(&userfile.file.loadout);
    if config.general.log_cast_data {
        let base = config.fishing.base_catch_chance;
        let sensitivity = userfile.file.loadout.catch_chance_multiplier();
        let fight_chance = fish.category.fight_multiplier();
        let chance = (base * sensitivity) / fight_chance;
        say!("{}'s catch chance was {}%", catch.interaction.user.display_name(), (chance * 100.0) as u32);
    }
    if !caught {
        let embed = CreateEmbed::new()
            .title("Nothing Came Up!")
            .description(format!("The fish got away. Better luck next time!\n\nYou lost a {:.2} in {} weighing {:.2} lbs.",
                                 fish.size, fish.fish_type.name, fish.weight))
            .thumbnail("attachment://FishingRod.png")
            .color(0x3498db)
            .footer(CreateEmbedFooter::new(format!("{}", random_tip())));

        let mut message = CreateMessage::new()
            .content(format!("{}", catch.user.mention()))
            .embed(embed);

        let attachment = CreateAttachment::path("./assets/FishingRod.png").await;

        // use the attachment if found
        if let Ok(file) = attachment {
            message = message.add_file(file);
        }

        if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
            nay!("Failed to send cast response message: {}", e);
            return;
        }
        // send_catch_message(&catch,
        //                    format!("{} You felt a tug on your line but the {} got away! Better luck next time!",
        //                             catch.user.mention(), fish.fish_type.name)).await;
        return;
    }

    // Weight Check
    // calculate the total weight load on the line (fish weight + sinker weight)
    let weight_load = fish.weight + userfile.file.loadout.sinker.weight;
    let max_weight = userfile.file.loadout.total_strength();

    if weight_load > max_weight {
        // Quick Time Event (QTE)

        // Generate the random code
        // split the code by spaces to prevent copy + paste
        let code: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();

        // add spaces between characters to prevent copy + paste
        let code_display = code.chars().map(|c| format!("{}", c)).collect::<Vec<String>>().join(" ");

        // Calculate time limit
        let base_time = config.fishing.base_qte_time;
        let min_time = config.fishing.min_qte_time;

        let ratio = weight_load / max_weight;
        let time_limit_secs = (base_time / ratio).max(min_time);

        let embed = CreateEmbed::new()
            .title("⚠️ LINE TENSION CRITICAL!")
            .description(format!("The fish is too heavy! Type the code below in **{:.1}s** to save the line!", time_limit_secs))
            .thumbnail("attachment://FishingRod.png")
            .field("Type The code!", format!("`{}`", code_display), false)
            .color(Color::RED)
            .footer(CreateEmbedFooter::new(format!("{}", random_tip())));

        let mut message = CreateMessage::new()
            .content(format!("{}", catch.user.mention()))
            .embed(embed);

        let attachment = CreateAttachment::path("./assets/FishingRod.png").await;

        // use the attachment if found
        if let Ok(file) = attachment {
            message = message.add_file(file);
        }

        if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
            nay!("Failed to send cast response message: {}", e);
            return;
        }

        // wait for the message
        let collector = catch.channel.await_reply(&catch.ctx)
            .author_id(catch.user)
            .timeout(Duration::from_secs_f32(time_limit_secs));

        if let Some(msg) = collector.await {
            // User Replied
            let user_input = msg.content.replace(" ", "");

            // be nice and ingore case
            if user_input.eq_ignore_ascii_case(&code) {
                // SUCCESS
                let embed = CreateEmbed::new()
                    .title("Line Stabilized!")
                    .description("You managed to reel it in safely.")
                    .color(Color::DARK_GREEN);

                let message = CreateMessage::new()
                    .content(format!("{}", catch.user.mention()))
                    .embed(embed);

                if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
                    nay!("Failed to send cast response message: {}", e);
                }

                // dont return, proceed to successful catch handling
            } else {
                // FAILURE
                let embed = CreateEmbed::new()
                    .title("SNAP!")
                    .description(format!("You typed the wrong code (`{}`). The line broke!\n\nYou lost a {:.2} in {} weighing {:.2} lbs.",
                                         user_input, fish.size, fish.fish_type.name, fish.weight))
                    .color(Color::RED);

                let message = CreateMessage::new()
                    .content(format!("{}", catch.user.mention()))
                    .embed(embed);

                if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
                    nay!("Failed to send cast response message: {}", e);
                }
                return;
            }
        } else {
            // TIMEOUT
            let embed = CreateEmbed::new()
                .title("SNAP!")
                .description(format!("You weren't fast enough and your line snapped!\n\nYou lost a {:.2} in {} weighing {:.2} lbs.",
                                     fish.size, fish.fish_type.name, fish.weight))
                .color(Color::RED);

            let message = CreateMessage::new()
                .content(format!("{}", catch.user.mention()))
                .embed(embed);

            if let Err(e) = catch.channel.send_message(&catch.ctx.http, message).await {
                nay!("Failed to send cast response message: {}", e);
            }
            return;
        }
    }

    // Treasure, Trash, Item, Turtle and other non-fish catches get handled here
    // TODO - Later Update

    // Successful catch
    // add funds to the user's account
    let earnings = fish.value.clone();
    userfile.file.balance += earnings.clone();
    // todo: add the fish type to the user's caught types if it's a new fish for them
    userfile.update();

    let embed = CreateEmbed::new()
        .title("Fish Caught!")
        .description(format!("You caught a **{}**!", fish.fish_type.name))
        .fields(
            vec![
                ("Size", format!("{:.2} in", fish.size), true),
                ("Weight", format!("{:.2} lbs", fish.weight), true),
                ("Value", format!("{}", earnings), false),
                ("New balance", format!("{}", userfile.file.balance), true),
            ]
        )
        .color(Color::GOLD)
        .thumbnail("attachment://rod_with_fish.png");

    let mut message = CreateMessage::new()
        .content(format!("{}", catch.user.mention()))
        .embed(embed);

    if let Ok(file) = CreateAttachment::path("./assets/rod_with_fish.png").await {
        message = message.add_file(file);
    }

    if let Err(e) = catch.channel.send_message(&catch.ctx, message).await {
        nay!("Failed to send cast response message: {}", e);
        return;
    }
}