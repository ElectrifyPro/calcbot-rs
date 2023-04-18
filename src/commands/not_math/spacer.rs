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

/// This command is incredibly useful for sounding like the sloth from Zootopia.
#[derive(Clone, Info)]
#[info(
    aliases = ["spacer", "space", "sp"],
    syntax = ["<string>"],
    examples = ["patience, mortal", "he is the captain now"],
)]
pub struct Spacer;

#[async_trait]
impl Command for Spacer {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(message.channel_id)
            .content(&args.join(" ").split("").collect::<Vec<&str>>().join(" "))?
            .await?;
        Ok(())
    }
}
