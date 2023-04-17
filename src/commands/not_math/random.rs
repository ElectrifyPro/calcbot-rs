use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::Command,
    global::State,
};
use getrandom::getrandom;
use std::{error::Error, sync::Arc};
use twilight_model::channel::message::Message;

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
)]
pub struct Random;

#[async_trait]
impl Command for Random {
    async fn execute(
        &self,
        state: Arc<State>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (min, max) = match args.len() {
            1 => (0, args[0].parse::<u32>()?),
            2 => (args[0].parse::<u32>()?, args[1].parse::<u32>()?),
            _ => return Err("Invalid number of arguments".into()),
        };
        let num = random(min, max + 1);
        state.http.create_message(message.channel_id)
            .content(&format!(
                "**Random number** from {} to {}\n{}",
                min, max, num
            ))?
            .await?;
        Ok(())
    }
}
