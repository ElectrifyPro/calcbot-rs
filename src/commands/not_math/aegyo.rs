use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::Command,
    global::State,
};
use regex::Regex;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;

lazy_static::lazy_static! {
    static ref REGEX_LOWER: Regex = Regex::new(r"[lr]").unwrap();
    static ref REGEX_UPPER: Regex = Regex::new(r"[LR]").unwrap();
}

/// Bonus points on the test if you can type that second alias.
#[derive(Clone, Info)]
#[info(
    aliases = ["aegyo", "애교"],
    syntax = ["<string>"],
    examples = ["please don't be mean to me, please?"],
)]
pub struct Aegyo;

#[async_trait]
impl Command for Aegyo {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let input = args.join(" ");
        let replaced_upper = REGEX_UPPER.replace_all(&input, "W");
        let replaced_lower = REGEX_LOWER.replace_all(&replaced_upper, "w");
        http.create_message(message.channel_id)
            .content(&replaced_lower)?
            .await?;
        Ok(())
    }
}