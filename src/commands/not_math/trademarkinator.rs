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

/// _Got your brand name? Now make it stand out!_
///
/// Appends the trademark symbol (™) after every word in your input.
#[derive(Clone, Info)]
#[info(
    aliases = ["trademarkinator", "tmor"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
    parent = super::NotMath,
)]
pub struct Trademarkinator;

#[async_trait]
impl Command for Trademarkinator {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&format!("{}:tm:", ctxt.raw_input.split_whitespace().collect::<Vec<_>>().join(":tm: ")))
            .await?;
        Ok(())
    }
}
