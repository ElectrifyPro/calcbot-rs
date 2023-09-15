pub mod commands;
pub mod database;
pub mod error;
pub mod global;
pub mod handler;
pub mod util;

use database::Database;
use dotenv::dotenv;
use global::State;
use simple_logger::SimpleLogger;
use std::{env, error::Error, sync::Arc};
use tokio::sync::Mutex;
use twilight_gateway::{Event, Intents, Shard, ShardId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    SimpleLogger::new()
        .with_module_level("rustls", log::LevelFilter::Warn)
        .with_module_level("mio", log::LevelFilter::Warn)
        .with_module_level("tokio_tungstenite", log::LevelFilter::Warn)
        .with_module_level("tungstenite", log::LevelFilter::Warn)
        .with_module_level("want", log::LevelFilter::Warn)
        .init()
        .unwrap();

    dotenv()?;
    let token = env::var("DISCORD_TOKEN")?;

    let intents = Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    let state = Arc::new(State::new(token).await);
    let database = Arc::new(Mutex::new(Database::new()));

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };
        state.cache.update(&event);

        tokio::spawn(handle_event(
            event,
            Arc::clone(&state),
            Arc::clone(&database),
        ));
    }

    Ok(())
}

/// Handles events relevant to the bot, delegating each event to the appropriate handler.
async fn handle_event(
    event: Event,
    state: Arc<State>,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => handler::message_create(*msg, state, database).await?,
        Event::Ready(ready) => log::info!(
            "Shard {} connected",
            ready.shard.unwrap_or(ShardId::new(0, 1))
        ),
        Event::InteractionCreate(interaction) => {
            if let (Some(channel), Some(message)) = (
                &interaction.channel,
                &interaction.message,
            ) {
                database.lock()
                    .await
                    .get_paged_message(channel.id, message.id)
                    .map(|sender| sender.send(*interaction));
            }
        }
        _ => {}
    }

    Ok(())
}
