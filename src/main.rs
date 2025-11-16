pub mod arg_parse;
pub mod commands;
pub mod database;
pub mod error;
pub mod fmt;
pub mod global;
pub mod handler;
pub mod timer;
pub mod util;

use database::Database;
use dotenv::dotenv;
use global::State;
use simple_logger::SimpleLogger;
use std::{env, error::Error, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};
use twilight_gateway::{Config, Event, EventTypeFlags, Intents, ShardId, StreamExt};

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

    let state = Arc::new(State::new(token.clone()).await);
    let database = Arc::new(Mutex::new(Database::new()));
    {
        let state_clone = Arc::clone(&state);
        let database_clone = Arc::clone(&database);
        database.lock().await
            .resume_users_with_timers(state_clone, database_clone).await;
        log::info!("Resumed users with active timers");
    }

    let shards = twilight_gateway::create_recommended(
        &state.http,
        Config::new(token, intents),
        |_, builder| builder.build(),
    ).await?;

    let mut set = JoinSet::new();
    for mut shard in shards {
        let state = Arc::clone(&state);
        let database = Arc::clone(&database);
        set.spawn(async move {
            log::info!("Starting shard ID {}", shard.id());
            loop {
                let event = match shard.next_event(EventTypeFlags::all()).await {
                    Some(Ok(event)) => event,
                    Some(Err(source)) => {
                        log::warn!("Shard ID {} experiences gateway error: {}", shard.id().number(), source);
                        continue;
                    },
                    None => {
                        log::warn!("Shard ID {} disconnected because the stream ended", shard.id().number());
                        break;
                    },
                };
                state.cache.update(&event);

                tokio::spawn(handle_event(
                    shard.id(),
                    event,
                    Arc::clone(&state),
                    Arc::clone(&database),
                ));
            }
        });
    }

    while let Some(res) = set.join_next().await {
        if let Err(e) = res {
            log::error!("A shard task has failed: {}", e);
        }
    }

    Ok(())
}

/// Handles events relevant to the bot, delegating each event to the appropriate handler.
async fn handle_event(
    shard_id: ShardId,
    event: Event,
    state: Arc<State>,
    database: Arc<Mutex<Database>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => handler::message_create(
            shard_id,
            *msg,
            state,
            database,
        ).await?,
        Event::MessageDelete(msg) => {
            if database.lock().await.remove_paged_message(msg.channel_id, msg.id) {
                log::info!("paged message task ended: message deleted");
                // NOTE: the other log message will also appear as the task is dropped
            }
        },
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
