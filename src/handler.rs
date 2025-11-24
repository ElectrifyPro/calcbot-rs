use super::{commands::{remind::toggle_shared, Context}, database::Database, global::State};
use std::{error::Error, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use twilight_gateway::ShardId;
use twilight_model::{
    application::interaction::InteractionData,
    gateway::payload::incoming::{InteractionCreate, MessageCreate},
};

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
        InteractionData::MessageComponent(_) => {
            let Some(message) = &interaction.message else {
                return Ok(());
            };

            let mut db = database.lock().await;

            if let Some((author, timer)) = db.get_shared_reminder(message.id).await {
                toggle_shared::toggle_shared(
                    &interaction,
                    &state,
                    &database,
                    &mut db,
                    author,
                    &timer,
                    message,
                ).await?;
            } else if let Some(channel) = &interaction.channel {
                db
                    .get_paged_message(channel.id, message.id)
                    .map(|sender| sender.send(interaction));
            }
        },
        _ => {},
    }

    Ok(())
}
