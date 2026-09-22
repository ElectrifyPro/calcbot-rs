use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Set the bot's prefix used to call commands. (default `c-`).
///
/// **This server's prefix: `{prefix}`**
#[derive(Clone, Info)]
#[info(
    aliases = ["setprefix", "setp", "sp"],
    syntax = ["<prefix>"],
    examples = ["c-"],
    parent = super::Admin,
)]
pub struct SetPrefix;

#[async_trait]
impl Command for SetPrefix {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let Some(guild_id) = ctxt.trigger.guild_id() else {
            unreachable!();
        };

        let new_prefix = parse_args_full::<Word>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                (self.info().build_embed(ctxt.prefix), ctxt.trigger.author_id()).into()
            } else {
                err
            })?;

        database.lock().await
            .set_server_prefix(guild_id, new_prefix.0).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("**Set server prefix to `{}`**", new_prefix.0))
            .await?;
        Ok(())
    }
}
