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

/// _This command is incredibly useful for sounding like the sloth from Zootopia._
///
/// Inserts spaces between every character in your input.
#[derive(Clone, Info)]
#[info(
    aliases = ["spacer", "space", "sp"],
    syntax = ["<string>"],
    examples = ["patience, mortal", "he is the captain now"],
    parent = super::NotMath,
)]
pub struct Spacer;

#[async_trait]
impl Command for Spacer {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&ctxt.raw_input.split("").collect::<Vec<&str>>().join(" "))
            .await?;
        Ok(())
    }
}
