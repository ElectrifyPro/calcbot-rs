use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use getrandom::getrandom;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Generates the random integer.
fn random(min: u32, max: u32) -> u32 {
    let mut buf = [0u8; 4];
    getrandom(&mut buf).unwrap();

    let mut num = u32::from_le_bytes(buf);
    num %= max - min; // limit number to range
    num += min; // offset number to range
    num
}

/// Generate a random integer; boundaries are inclusive.
#[derive(Clone, Info)]
#[info(
    aliases = ["random", "rand", "r"],
    syntax = ["<max>", "<min> <max>"],
    examples = ["11", "4 11"],
    args = [u32, Option<u32>],
)]
pub struct Random;

#[async_trait]
impl Command for Random {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (min, max) = match parse_args(ctxt.raw_input.split_whitespace().collect())? {
            (a, Some(b)) => (a, b),
            (a, None) => (0, a),
        };
        let num = random(min, max + 1);
        ctxt.trigger.reply(&state.http)
            .content(&format!(
                "**Random number** from {} to {}\n{}",
                min, max, num
            ))?
            .await?;
        Ok(())
    }
}
