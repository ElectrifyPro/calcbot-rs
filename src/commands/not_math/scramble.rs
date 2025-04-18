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

/// Randomize the order of characters in a string.
fn scramble(string: &str) -> String {
    let mut chars: Vec<char> = string.chars().collect();
    let mut rng = [0u8; 1];
    for i in 0..chars.len() {
        getrandom(&mut rng).unwrap();
        let j = rng[0] as usize % chars.len();
        chars.swap(i, j);
    }
    chars.into_iter().collect()
}

/// daw tbtlesr l armsreo/sec.
///
/// (If you can figure out what the unscrambled sentence is, hit me up. I put this in three years
/// ago and I didn't write anything to help me remember what it was.)
#[derive(Clone, Info)]
#[info(
    aliases = ["scramble", "sc"],
    syntax = ["<string>"],
    examples = ["invention", "life is quite a mystery."],
    parent = super::NotMath,
)]
pub struct Scramble;

#[async_trait]
impl Command for Scramble {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        ctxt.trigger.reply(&state.http)
            .content(&scramble(ctxt.raw_input))?
            .await?;
        Ok(())
    }
}
