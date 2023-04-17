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

/// It'll sound like you're going through puberty.
#[derive(Clone, Info)]
#[info(
    aliases = ["uglify", "ug"],
    syntax = ["<string>"],
    examples = ["mock your friends, but only if they let you"],
)]
pub struct Uglify;

#[async_trait]
impl Command for Uglify {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut lower = true;
        let content = args
            .join(" ")
            .chars()
            .map(|c| {
                if c.is_alphabetic() {
                    if lower {
                        lower = false;
                        c.to_lowercase().to_string()
                    } else {
                        lower = true;
                        c.to_uppercase().to_string()
                    }
                } else {
                    c.to_string()
                }
            })
            .collect::<String>();

        http.create_message(message.channel_id)
            .content(&content)?
            .await?;
        Ok(())
    }
}
