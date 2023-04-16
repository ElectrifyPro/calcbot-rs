pub mod commands;
pub mod global;
pub mod util;

use dotenv::dotenv;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv()?;
    let token = env::var("DISCORD_TOKEN")?;

    let intents = Intents::GUILDS
        | Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT;
    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);

    let http = Arc::new(HttpClient::new(token));
    let cache = Arc::new(
        InMemoryCache::builder()
            .resource_types(ResourceType::USER_CURRENT | ResourceType::MESSAGE)
            .build(),
    );
    let state = Arc::new(global::State::new());

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
        cache.update(&event);

        tokio::spawn(handle_event(
            event,
            Arc::clone(&http),
            Arc::clone(&cache),
            Arc::clone(&state),
        ));
    }

    Ok(())
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    cache: Arc<InMemoryCache>,
    state: Arc<global::State>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content == "!ping" => {
            use commands::Command;
            commands::about::About
                .execute(http, cache, state, &msg)
                .await?;
        }
        Event::Ready(_) => {
            println!("Shard is ready");
        }
        _ => {}
    }

    Ok(())
}
