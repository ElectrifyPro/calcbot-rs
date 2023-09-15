use super::{commands::Context, database::Database, global::State};
use std::{error::Error, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use twilight_model::gateway::payload::incoming::MessageCreate;

/// Handles a message being created in some text channel.
pub async fn message_create(
    msg: MessageCreate,
    state: Arc<State>,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // never respond to bots
    if msg.author.bot {
        return Ok(());
    }

    // if in guild, fetch guild's prefix
    // in dm channels, there is no prefix
    // NOTE: async closures are unstable
    let prefix = match msg.guild_id {
        Some(id) => {
            let mut db = database.lock().await;
            Some(db.get_server(id).await.to_owned())
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
                let ctxt = Context { message: &msg, prefix: prefix.as_deref(), raw_input };
                if let Err(discord_error) = cmd.execute(&state, &database, &ctxt).await {
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
