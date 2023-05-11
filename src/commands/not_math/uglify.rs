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

/// It'll sound like you're going through puberty.
#[derive(Clone, Info)]
#[info(
    aliases = ["uglify", "ug"],
    syntax = ["<string>"],
    examples = ["mock your friends, but only if they let you"],
)]
pub struct Uglify;

#[async_trait]
impl Command for Uglify {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        message: &Message,
        raw_input: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut lower = true;
        let content = raw_input
            .chars()
            .map(|c| {
                if c.is_alphabetic() {
                    if lower {
                        lower = false;
                        c.to_lowercase().to_string()
                    } else {
                        lower = true;
                        c.to_uppercase().to_string()
                    }
                } else {
                    c.to_string()
                }
            })
            .collect::<String>();

        state.http.create_message(message.channel_id)
            .content(&content)?
            .await?;
        Ok(())
    }
}
