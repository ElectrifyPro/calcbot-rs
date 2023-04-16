use calcbot_attrs::{Command, Info};

/// View a list of available commands.
#[derive(Command, Info)]
#[info(aliases = ["commands", "cmds", "list", "cmd", "l", "c"])]
pub struct Commands;
