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
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut lower = true;
        let content = ctxt.raw_input
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

        ctxt.trigger.reply(&state.http)
            .content(&content)?
            .await?;
        Ok(())
    }
}
