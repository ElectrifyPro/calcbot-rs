use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    global::State,
};
use std::{error::Error, sync::Arc};
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
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(ctxt.message.channel_id)
            .content(&ctxt.raw_input.chars().rev().collect::<String>())?
            .await?;
        Ok(())
    }
}
