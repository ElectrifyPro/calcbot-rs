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

/// _Get your own custom brand name for free, although it has no legal meaning!_
///
/// Appends the trademark symbol (™) to the end of your input.
#[derive(Clone, Info)]
#[info(
    aliases = ["trademark", "tm"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
    parent = super::NotMath,
)]
pub struct Trademark;

#[async_trait]
impl Command for Trademark {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&format!("{}:tm:", ctxt.raw_input))
            .await?;
        Ok(())
    }
}
