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

/// No one will ever figure out your password now!
#[derive(Clone, Info)]
#[info(
    aliases = ["reverse", "rev"],
    syntax = ["<string>"],
    examples = ["!yadhtrib yppah"],
)]
pub struct Reverse;

#[async_trait]
impl Command for Reverse {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id)
            .content(&args.join(" ").chars().rev().collect::<String>())?
            .await?;
        Ok(())
    }
}