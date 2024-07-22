use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use regex::Regex;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref REGEX_LOWER: Regex = Regex::new(r"[lr]").unwrap();
    static ref REGEX_UPPER: Regex = Regex::new(r"[LR]").unwrap();
}

/// Bonus points on the test if you can type that second alias.
#[derive(Clone, Info)]
#[info(
    aliases = ["aegyo", "애교"],
    syntax = ["<string>"],
    examples = ["please don't be mean to me, please?"],
)]
pub struct Aegyo;

#[async_trait]
impl Command for Aegyo {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let replaced_upper = REGEX_UPPER.replace_all(&ctxt.raw_input, "W");
        let replaced_lower = REGEX_LOWER.replace_all(&replaced_upper, "w");
        ctxt.trigger.reply(&state.http)
            .content(&replaced_lower)?
            .await?;
        Ok(())
    }
}
