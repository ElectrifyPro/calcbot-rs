pub mod about;

use super::global::State;
use async_trait::async_trait;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;

/// Represents a command's metadata. This data is shown when the user runs the help command for
/// this command.
pub struct CommandInfo {
    /// The name of the command.
    pub name: &'static str,

    /// The description of the command.
    pub description: &'static str,

    /// Allowed aliases for the command. If not provided, the only allowed alias is the name.
    pub aliases: Option<&'static [&'static str]>,

    /// The syntax of the command. This is generally not needed for simple commands.
    pub syntax: Option<&'static str>,

    /// Example usage of the command. This is generally not needed for simple commands.
    pub examples: Option<&'static [&'static str]>,
}

/// Represents any command.
#[async_trait]
pub trait Command {
    /// Returns the command's metadata.
    fn info(&self) -> CommandInfo;

    /// Executes the command.
    async fn execute(
        &self,
        http: Arc<Client>,
        cache: Arc<InMemoryCache>,
        state: Arc<State>,
        message: &Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}
