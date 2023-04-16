use crate::commands::{Command, CommandInfo};

/// Get information on how to use a command. For example, to learn about `{prefix}calculate stats`,
/// run `{prefix}help calculate stats`. All commands have aliases, which are alternative (always
/// shorter) names for commands. You can find them in a command's help embed.
///
/// For a list of all commands, run `{prefix}help commands`.
pub struct Help;

impl Command for Help {
    fn info(&self) -> CommandInfo {
        CommandInfo {
            name: "Help",
            description: "Get information on how to use a command.",
            aliases: Some(&["help", "h"]),
            syntax: Some(&["help [command]"]),
            examples: Some(&["help calculate stats"]),
        }
    }
}
