use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::Command,
    database::Database,
    global::State,
};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::channel::message::Message;

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
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        message: &Message,
        raw_input: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(message.channel_id)
            .content(&format!("{}:tm:", raw_input.split_whitespace().collect::<Vec<_>>().join(":tm: ")))?
            .await?;
        Ok(())
    }
}
