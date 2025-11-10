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

/// _Bonus points on the test if you can type that second alias._
///
/// Converts all 'r's and 'l's to 'w's to make the input sound cuter.
#[derive(Clone, Info)]
#[info(
    aliases = ["aegyo", "애교"],
    syntax = ["<string>"],
    examples = ["please don't be mean to me, please?"],
    parent = super::NotMath,
)]
pub struct Aegyo;

#[async_trait]
impl Command for Aegyo {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let replaced_upper = REGEX_UPPER.replace_all(ctxt.raw_input, "W");
        let replaced_lower = REGEX_LOWER.replace_all(&replaced_upper, "w");
        ctxt.trigger.reply(&state.http)
            .content(&replaced_lower)
            .await?;
        Ok(())
    }
}
