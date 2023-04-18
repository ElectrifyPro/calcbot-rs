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

/// View a list of available commands.
#[derive(Clone, Info)]
#[info(aliases = ["commands", "cmds", "list", "cmd", "l", "c"])]
pub struct Commands;

#[async_trait]
impl Command for Commands {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        message: &Message,
        _: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(message.channel_id)
            .embeds(&[state.build_commands_embed()])?
            .await?;
        Ok(())
    }
}
