use super::{commands::{remind::action, Context, Role}, database::Database, global::State};
use std::{error::Error, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use twilight_gateway::ShardId;
use twilight_mention::Mention;
use twilight_model::{
    application::interaction::InteractionData,
    channel::message::MessageFlags,
    gateway::payload::incoming::{InteractionCreate, MessageCreate},
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::{GuildMarker, UserMarker}, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

/// Returns true if the given user is the owner or has the ADMINISTRATOR permission in the specified
/// guild.
async fn has_admin_permission(
    state: &State,
    guild: Id<GuildMarker>,
    user: Id<UserMarker>,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let is_owner = state.http.guild(guild)
        .await?
        .model()
        .await?
        .owner_id == user;
    if is_owner {
        return Ok(true);
    }

    let member = state.http.guild_member(guild, user)
        .await?
        .model()
        .await?;
    let is_admin = {
        let guild_roles = state.http.roles(guild)
            .await?
            .model()
            .await?;
        let member_has_admin_permission = member.roles
            .iter()
            .filter_map(|role_id|
                guild_roles
                .iter()
                .find(|role| role.id == *role_id)
            )
            .any(|role| role.permissions.contains(Permissions::ADMINISTRATOR));

        member_has_admin_permission
    };
    if is_admin {
        return Ok(true);
    }

    Ok(false)
}

/// Handles a message being created in some text channel.
pub async fn message_create(
    shard_id: ShardId,
    msg: MessageCreate,
    state: Arc<State>,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // never respond to bots
    if msg.author.bot {
        return Ok(());
    }

    // NOTE: using old CalcBot
    if !database.lock().await.is_using_preview(msg.author.id).await {
        return Ok(());
    }

    // if in guild, fetch guild's prefix
    // in dm channels, there is no prefix
    // NOTE: async closures are unstable
    let prefix = match msg.guild_id {
        Some(id) => {
            let mut db = database.lock().await;
            let Ok(prefix) = db.get_server(id).await else {
                // database connection lost
                state.http.create_message(msg.channel_id)
                    .content("**Oops!** CalcBot is having trouble reaching its database. Please try again in a moment.\nIf this issue persists after a few minutes, please report it to the developers!")
                    .await?;
                return Ok(());
            };
            Some(prefix.to_owned())
        },
        None => None,
    };

    if prefix.is_none() || msg.content.starts_with(prefix.as_ref().unwrap()) {
        let prefix_len = prefix.as_ref().map(|p| p.len()).unwrap_or(0);
        let mut trimmed = msg.content[prefix_len..].split_whitespace().peekable();

        let now = Instant::now();
        match state.commands.find_command(&mut trimmed) {
            Some(cmd) => {
                match cmd.info().required_role() {
                    Role::User => (), // anyone can run this command
                    Role::Admin => if let Some(guild_id) = msg.guild_id {
                        if !has_admin_permission(&state, guild_id, msg.author.id).await? {
                            state.http.create_message(msg.channel_id)
                                .content("**You don't have permission to access this command.**")
                                .await?;
                            return Ok(());
                        }
                    } else {
                        state.http.create_message(msg.channel_id)
                            .content("**This command is only accessible in servers.**")
                            .await?;
                        return Ok(());
                    },
                }

                let raw_input = trimmed.peek()
                    .map(|s| {
                        // trimmed is a view into msg.content, so we can find the start of the
                        // arguments with some pointer arithmetic
                        let byte = s.as_ptr() as usize - msg.content.as_ptr() as usize;
                        &msg.content[byte..]
                    })
                    .unwrap_or_default();
                let ctxt = Context {
                    shard_id,
                    trigger: (&msg.0).into(),
                    prefix: prefix.as_deref(),
                    raw_input,
                };
                if let Err(discord_error) = cmd.execute(&state, &database, ctxt).await {
                    discord_error.rich_fmt(state.http.create_message(msg.channel_id))?
                        .await?;
                };

                log::info!(
                    "Command executed in {}ms: {}",
                    now.elapsed().as_millis(),
                    msg.content
                );
            }
            None => log::info!(
                "Command not found ({}ms spent): {}",
                now.elapsed().as_millis(),
                msg.content
            ),
        }
    }

    Ok(())
}

/// Handles a user interaction with a component on a message sent by the bot.
pub async fn interaction_create(
    interaction: InteractionCreate,
    state: Arc<State>,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(data) = interaction.data.as_ref() else {
        return Ok(());
    };

    match data {
        InteractionData::ApplicationCommand(_) => todo!(),
        InteractionData::MessageComponent(delete) if delete.custom_id.starts_with("delete:") => {
            let Some(message) = &interaction.message else {
                return Ok(());
            };

            let (Some(interacting_user_id), Some(expected_user_id)) = (
                interaction.author_id(),
                delete.custom_id
                    .trim_start_matches("delete:")
                    .parse::<Id<UserMarker>>()
                    .ok(),
            ) else {
                return Ok(());
            };

            if interacting_user_id == expected_user_id {
                state.http.delete_message(message.channel_id, message.id)
                    .await?;
            } else {
                state.http.interaction(state.application_id)
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(InteractionResponseDataBuilder::new()
                                .content(format!(
                                    "The user who triggered this message is {}; **only they can delete it.**",
                                    expected_user_id.mention(),
                                ))
                                .flags(MessageFlags::EPHEMERAL)
                                .build()),
                        },
                    )
                .await?;
            }
        },
        InteractionData::MessageComponent(_) => {
            let Some(message) = &interaction.message else {
                return Ok(());
            };

            let mut db = database.lock().await;

            if let Some((author, timer)) = db.get_shared_reminder(message.id).await {
                action::toggle_shared(
                    &interaction,
                    &state,
                    &database,
                    &mut db,
                    author,
                    &timer,
                    message,
                ).await?;
            } else if let Some(channel) = &interaction.channel
                && let Some(sender) = db.get_paged_message(channel.id, message.id) {
                let _ = sender.send(interaction);
            }
        },
        _ => {},
    }

    Ok(())
}
