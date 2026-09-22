pub mod set_prefix;

use calcbot_attrs::{Command, Info};
use crate::commands::Role;

/// Contains commands only accessible to server admins.
#[derive(Clone, Command, Info)]
#[info(
    role = Role::Admin,
    category = "Settings",
    aliases = ["admin", "adm"],
    children = [
        set_prefix::SetPrefix,
    ],
)]
pub struct Admin;
