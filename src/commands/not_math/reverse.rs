use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&ctxt.raw_input.chars().rev().collect::<String>())?
            .await?;
        Ok(())
    }
}
