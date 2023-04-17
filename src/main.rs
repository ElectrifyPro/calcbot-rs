pub mod commands;
pub mod global;
pub mod util;

use dotenv::dotenv;
use global::State;
use simple_logger::SimpleLogger;
use std::{env, error::Error, sync::Arc, time::Instant};
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

    let state = Arc::new(State::new(token));

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
        ));
    }

    Ok(())
}

async fn handle_event(
    event: Event,
    state: Arc<global::State>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            if msg.content.starts_with("c-") {
                let mut trimmed = msg.content[2..].split_whitespace().peekable();
                let now = Instant::now();
                match state.commands.find_command(&mut trimmed) {
                    Some(cmd) => {
                        cmd.execute(state, &msg, trimmed.collect())
                            .await?;
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
        }
        Event::Ready(ready) => log::info!(
            "Shard {} connected",
            ready.shard.unwrap_or(ShardId::new(0, 1))
        ),
        _ => {}
    }

    Ok(())
}
