pub mod commands;

use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context, Info},
    database::Database,
    global::State,
};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

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
    children = [commands::Commands],
)]
pub struct Help;

#[async_trait]
impl Command for Help {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // extract the path to the command the user wants help with
        let mut path = ctxt.raw_input.split_whitespace().peekable();
        let embed = match state.commands.find_command(&mut path) {
            Some(cmd) => cmd.info(),
            None => self.info(),
        }.build_embed(ctxt.prefix);

        state.http.create_message(ctxt.message.channel_id)
            .embeds(&[embed])?
            .await?;
        Ok(())
    }
}
