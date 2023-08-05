use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    global::State,
};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

/// Get your own custom brand name for free, although it has no legal meaning!
#[derive(Clone, Info)]
#[info(
    aliases = ["trademark", "tm"],
    syntax = ["<string>"],
    examples = ["The Perfect Bite"],
)]
pub struct Trademark;

#[async_trait]
impl Command for Trademark {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(ctxt.message.channel_id)
            .content(&format!("{}:tm:", ctxt.raw_input))?
            .await?;
        Ok(())
    }
}
