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

/// I don't know why you would want to make the word "The" your custom brand name, but you do you.
#[derive(Clone, Info)]
#[info(
    aliases = ["trademarkinator", "tmor"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
)]
pub struct Trademarkinator;

#[async_trait]
impl Command for Trademarkinator {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id)
            .content(&format!("{}:tm:", args.join(":tm: ")))?
            .await?;
        Ok(())
    }
}
