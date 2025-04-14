pub mod commands;

use async_trait::async_trait;
use crate::{
    commands::{Command, CommandInfo, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Get information on how to use a command in a neat embed.
#[derive(Clone)]
pub struct Help;

// manual impl to allow the bullet list in the description to happen without newlines getting
// omitted by #[doc = "..."]
impl Info for Help {
    fn info(&self) -> CommandInfo {
        CommandInfo {
            name: "Help",
            description: "Get information on how to use a command in a neat embed.

A command's help embed contains the following information:

- **Description**: A brief explanation of what the command does.
- **Syntax**: Shows you how to use the command, including all pieces of data you must provide.
- **Examples**: A list of example uses of the command that you can run right away to see it in action.
- **Shorthand**: Shows the shortest possible way to run the command, by taking the shortest aliases from the command and its parent commands.
- **Aliases**: A list of alternative (usually shorter) names for the command which you can use to trigger the command if you'd prefer.
- **Children commands**: If the command has subcommands, they are listed here.

For a list of all commands, run `{prefix}help commands`.",
            category: Some("Resources"),
            aliases: Some(&["help", "h"]),
            syntax: Some(&["[command]"]),
            examples: Some(&["calculate stats"]),
            children: vec![Box::new(commands::Commands) as Box<dyn Command>].into(),
        }
    }
}

#[async_trait]
impl Command for Help {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        // extract the path to the command the user wants help with
        let mut path = ctxt.raw_input.split_whitespace().peekable();
        let embed = match state.commands.find_command(&mut path) {
            Some(cmd) => cmd.info(),
            None => self.info(),
        }.build_embed(ctxt.prefix);

        ctxt.trigger.reply(&state.http)
            .embeds(&[embed])?
            .await?;
        Ok(())
    }
}
