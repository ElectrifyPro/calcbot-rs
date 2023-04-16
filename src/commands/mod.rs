pub mod about;
pub mod help;

use super::global::State;
use async_trait::async_trait;
use std::{error::Error, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::{Embed, Message};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// Formats a list of commands into a code block. Each string is displayed on a separate line,
/// prepended with the given prefix.
pub fn format_code_block(prefix: &str, strings: &[&str]) -> String {
    format!(
        "```\n{}\n```",
        strings
            .iter()
            .map(|string| format!("{}{}", prefix, string))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

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
    pub syntax: Option<&'static [&'static str]>,

    /// Example usage of the command. This is generally not needed for simple commands.
    pub examples: Option<&'static [&'static str]>,

    /// The children of this command. This will be displayed in the help embed.
    pub children: Vec<Box<dyn Command>>,
}

impl CommandInfo {
    /// Retrieves the default alias for this command.
    pub fn default_alias(&self) -> &'static str {
        self.aliases
            .map(|aliases| aliases.first())
            .flatten()
            .unwrap_or_else(|| &self.name)
    }

    /// Build the help embed for this command.
    ///
    /// Fields in the embed can contain special tags that will be replaced with the appropriate
    /// values. The following tags are supported, and will be replaced with the following values:
    ///
    /// - `{prefix}`: the bot's prefix in the current server / DM channel.
    /// - `{setting}`: if this command is a setting, the value of the setting
    pub fn build_embed(&self, prefix: Option<&str>) -> Embed {
        let prefix = prefix.unwrap_or("");
        let mut embed = EmbedBuilder::new()
            .title(self.name)
            .color(0x66d2e8)
            .field(EmbedFieldBuilder::new("Description", self.description.replace("{prefix}", prefix)));

        if let Some(syntax) = self.syntax.map(|syntax| format_code_block(prefix, syntax)) {
            embed = embed.field(EmbedFieldBuilder::new("Syntax", syntax));
        }

        if let Some(examples) = self.examples.map(|examples| format_code_block(prefix, examples)) {
            embed = embed.field(EmbedFieldBuilder::new("Examples", examples));
        }

        if let Some(aliases) = self.aliases {
            let shortest = aliases.iter().min_by_key(|s| s.len()).unwrap();
            embed = embed
                .field(EmbedFieldBuilder::new("Shorthand", format!("`{}{}`", prefix, shortest)))
                .field(EmbedFieldBuilder::new("Aliases", format!("`{}`", aliases.join("`, `"))));
        }

        if !self.children.is_empty() {
            let children = self
                .children
                .iter()
                .map(|child| format!("`{}`", child.info().default_alias()))
                .collect::<Vec<_>>()
                .join("\n");
            embed = embed.field(EmbedFieldBuilder::new("Children commands", children));
        }

        embed.build()
    }
}

/// Represents any command.
#[async_trait]
pub trait Command: Info + Send + Sync {
    /// Executes the command.
    async fn execute(
        &self,
        http: Arc<Client>,
        _: Arc<InMemoryCache>,
        _: Arc<State>,
        message: &Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // send the help embed by default
        let embed = self.info().build_embed(Some("c-"));
        http.create_message(message.channel_id)
            .embeds(&[embed])?
            .await?;
        Ok(())
    }
}

/// Represents a command with information on how to use it.
///
/// This trait can be derived using the `#[derive(Info)]` attribute, provided in `calcbot-attrs`.
pub trait Info {
    /// Returns the command's metadata.
    fn info(&self) -> CommandInfo;
}
