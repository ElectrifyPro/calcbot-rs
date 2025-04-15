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

/// Get your own custom brand name for free!
#[derive(Clone, Info)]
#[info(
    aliases = ["registeredtrademark", "reg", "rt"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite", "[Brand name here]"],
    parent = super::NotMath,
)]
pub struct RegisteredTrademark;

#[async_trait]
impl Command for RegisteredTrademark {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&format!("{}:registered:", ctxt.raw_input))?
            .await?;
        Ok(())
    }
}
