pub mod commands;

use calcbot_attrs::{Command, Info};
use commands::Commands;

/// Get information on how to use a command. For example, to learn about `{prefix}calculate stats`,
/// run `{prefix}help calculate stats`. All commands have aliases, which are alternative (always
/// shorter) names for commands. You can find them in a command's help embed.
///
/// For a list of all commands, run `{prefix}help commands`.
#[derive(Command, Info)]
#[info(
    aliases = ["help", "h"],
    syntax = ["help [command]"],
    examples = ["help calculate stats"],
    children = [Commands],
)]
pub struct Help;
