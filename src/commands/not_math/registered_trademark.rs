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

/// Get your own custom brand name for free!
#[derive(Clone, Info)]
#[info(
    aliases = ["registeredtrademark", "reg", "rt"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite", "[Brand name here]"],
)]
pub struct RegisteredTrademark;

#[async_trait]
impl Command for RegisteredTrademark {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id)
            .content(&format!("{}:registered:", args.join(" ")))?
            .await?;
        Ok(())
    }
}
