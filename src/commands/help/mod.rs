pub mod commands;

use calcbot_attrs::Info;
use commands::Commands;
use crate::{
    commands::{Command, Info},
    global::State,
};
use async_trait::async_trait;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;

/// Get information on how to use a command. For example, to learn about `{prefix}calculate stats`,
/// run `{prefix}help calculate stats`. All commands have aliases, which are alternative (always
/// shorter) names for commands. You can find them in a command's help embed.
///
/// For a list of all commands, run `{prefix}help commands`.
#[derive(Clone, Info)]
#[info(
    category = "Resources",
    aliases = ["help", "h"],
    syntax = ["[command]"],
    examples = ["calculate stats"],
    children = [Commands],
)]
pub struct Help;

#[async_trait]
impl Command for Help {
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        state: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // extract the path to the command the user wants help with
        let mut path = args.into_iter().peekable();
        let embed = match state.commands.find_command(&mut path) {
            Some(cmd) => cmd.info(),
            None => self.info(),
        }.build_embed(Some("c-"));

        http.create_message(message.channel_id)
            .embeds(&[embed])?
            .await?;
        Ok(())
    }
}
