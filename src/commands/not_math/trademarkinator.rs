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

/// I don't know why you would want to make the word "The" your custom brand name, but you do you.
#[derive(Clone, Info)]
#[info(
    aliases = ["trademarkinator", "tmor"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
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
            .content(&format!("{}:tm:", ctxt.raw_input.split_whitespace().collect::<Vec<_>>().join(":tm: ")))?
            .await?;
        Ok(())
    }
}
