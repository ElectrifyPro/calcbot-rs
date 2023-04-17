use calcbot_attrs::Info;
use crate::{
    commands::Command,
    global::State,
};
use async_trait::async_trait;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;

/// View a list of available commands.
#[derive(Clone, Info)]
#[info(aliases = ["commands", "cmds", "list", "cmd", "l", "c"])]
pub struct Commands;

#[async_trait]
impl Command for Commands {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        state: Arc<State>,
        message: &Message,
        _: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        http.create_message(message.channel_id)
            .embeds(&[state.build_commands_embed()])?
            .await?;
        Ok(())
    }
}
