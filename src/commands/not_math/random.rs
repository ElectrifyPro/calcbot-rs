use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    arg_parse::parse_args_full,
    commands::{Command, Context, Info},
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
    syntax = ["<maximum>", "<minimum> <maximum>"],
    examples = ["11", "4 11"],
)]
pub struct Random;

#[async_trait]
impl Command for Random {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                Error::Embed(self.info().build_embed(ctxt.prefix))
            } else {
                err
            })?;
        let (min, max) = match parsed {
            (a, Some(b)) if a > b => (b, a),
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
