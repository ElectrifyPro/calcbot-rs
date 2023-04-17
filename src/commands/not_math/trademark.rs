use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::Command,
    global::State,
};
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;

/// Get your own custom brand name for free, although it has no legal meaning!
#[derive(Clone, Info)]
#[info(
    aliases = ["trademark", "tm"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
)]
pub struct Trademark;

#[async_trait]
impl Command for Trademark {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id)
            .content(&format!("{}:tm:", args.join(" ")))?
            .await?;
        Ok(())
    }
}
