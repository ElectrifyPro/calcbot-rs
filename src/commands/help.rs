use calcbot_attrs::Info;
use crate::commands::Command;

/// Get information on how to use a command. For example, to learn about `{prefix}calculate stats`,
/// run `{prefix}help calculate stats`. All commands have aliases, which are alternative (always
/// shorter) names for commands. You can find them in a command's help embed.
///
/// For a list of all commands, run `{prefix}help commands`.
#[derive(Info)]
#[info(
    aliases = &["help", "h"],
    syntax = &["help [command]"],
    examples = &["help calculate stats"],
)]
pub struct Help;

impl Command for Help {}
