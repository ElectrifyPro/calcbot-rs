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

/// View a list of available commands.
#[derive(Clone, Info)]
#[info(aliases = ["commands", "cmds", "list", "cmd", "l"])]
pub struct Commands;

#[async_trait]
impl Command for Commands {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state.http.create_message(ctxt.message.channel_id)
            .embeds(&[state.build_commands_embed(ctxt.prefix)])?
            .await?;
        Ok(())
    }
}
