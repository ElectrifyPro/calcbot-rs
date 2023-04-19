pub mod about;
pub mod help;
pub mod link;
pub mod not_math;

use super::{database::Database, global::State};
use async_trait::async_trait;
use std::{error::Error, iter::Peekable, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::channel::message::{Embed, Message};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// Formats a list of commands into a code block. Each string is displayed on a separate line,
/// prepended with the given prefix.
pub fn format_code_block(prefix: &str, default_alias: &str, strings: &[&str]) -> String {
    format!(
        "```\n{}\n```",
        strings
            .iter()
            .map(|string| format!("{}{} {}", prefix, default_alias, string))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// A group of commands. It wraps a [`Vec`] of existing commands and provides extra functionality
/// on the collection.
pub struct CommandGroup {
    /// The commands in this group.
    pub commands: Vec<Box<dyn Command>>,
}

impl CommandGroup {
    /// Create a new command group.
    pub fn new(commands: Vec<Box<dyn Command>>) -> Self {
        Self { commands }
    }

    /// Search for the command that matches the given input aliases.
    ///
    /// Commands in CalcBot are organized in a tree-like structure. In order to access commands and
    /// their subcommands, the user will generally type a string that describes the path to the
    /// command they want to use. For example, the command `c-help` is at the root of the command
    /// tree, and the command `c-help commands` is a subcommand of `c-help`.
    ///
    /// This method assumes the prefix has already been stripped from the string and will not be
    /// yielded by the iterator.
    pub fn find_command<'a, T>(&self, input: &mut Peekable<T>) -> Option<Box<dyn Command>>
    where
        T: Iterator<Item = &'a str>,
    {
        let alias = input.peek()?;
        let command = self
            .commands
            .iter()
            .find(|command| command.info().is_alias(alias))?;
        input.next();

        if let Some(command) = command.info().children.find_command(input) {
            Some(command)
        } else {
            Some(command.clone_box())
        }
    }

    /// Count the number of commands in this group.
    pub fn count(&self) -> usize {
        self.commands.len()
            + self
                .commands
                .iter()
                .map(|c| c.info().children.count())
                .sum::<usize>()
    }
}

/// Represents a command's metadata. This data is shown when the user runs the help command for
/// this command.
pub struct CommandInfo {
    /// The name of the command.
    pub name: &'static str,

    /// The description of the command.
    pub description: &'static str,

    /// The category of the command. This field only applies to root commands (commands that have
    /// no parent).
    pub category: Option<&'static str>,

    /// Allowed aliases for the command. If not provided, the only allowed alias is the name.
    pub aliases: Option<&'static [&'static str]>,

    /// The syntax of the command. This is generally not needed for simple commands.
    pub syntax: Option<&'static [&'static str]>,

    /// Example usage of the command. This is generally not needed for simple commands.
    pub examples: Option<&'static [&'static str]>,

    /// The children of this command. This will be displayed in the help embed.
    pub children: CommandGroup,
}

impl CommandInfo {
    /// Retrieves the default alias for this command.
    pub fn default_alias(&self) -> &'static str {
        self.aliases
            .and_then(|aliases| aliases.first())
            .unwrap_or(&self.name)
    }

    /// Returns true if the given string is an alias for this command.
    pub fn is_alias(&self, alias: &str) -> bool {
        self.aliases
            .map(|aliases| aliases.contains(&alias))
            .unwrap_or(self.name == alias)
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
        let mut embed =
            EmbedBuilder::new()
                .title(self.name)
                .color(0x66d2e8)
                .field(EmbedFieldBuilder::new(
                    "Description",
                    self.description.replace("{prefix}", prefix),
                ));

        if let Some(syntax) = self
            .syntax
            .map(|syntax| format_code_block(prefix, self.default_alias(), syntax))
        {
            embed = embed.field(EmbedFieldBuilder::new("Syntax", syntax));
        }

        if let Some(examples) = self
            .examples
            .map(|examples| format_code_block(prefix, self.default_alias(), examples))
        {
            embed = embed.field(EmbedFieldBuilder::new("Examples", examples));
        }

        if let Some(aliases) = self.aliases {
            let shortest = aliases.iter().min_by_key(|s| s.len()).unwrap();
            embed = embed
                .field(EmbedFieldBuilder::new(
                    "Shorthand",
                    format!("`{}{}`", prefix, shortest),
                ))
                .field(EmbedFieldBuilder::new(
                    "Aliases",
                    format!("`{}`", aliases.join("`, `")),
                ));
        }

        if !self.children.commands.is_empty() {
            let children = self
                .children
                .commands
                .iter()
                .map(|child| format!("`{}`", child.info().default_alias()))
                .collect::<Vec<_>>()
                .join(", ");
            embed = embed.field(EmbedFieldBuilder::new("Children commands", children));
        }

        embed.build()
    }
}

/// Represents any command that can be executed by a user (accounting for permissions and other
/// factors).
#[async_trait]
pub trait Command: CommandClone + Info + Send + Sync {
    /// Executes the command.
    async fn execute(
        &self,
        state: Arc<State>,
        database: Arc<Mutex<Database>>,
        message: &Message,
        args: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

/// A trait that allows cloning of any command.
pub trait CommandClone {
    /// Clones the command.
    fn clone_box(&self) -> Box<dyn Command>;
}

impl<T> CommandClone for T
where
    T: 'static + Command + Clone,
{
    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Represents a command with information on how to use it.
///
/// This trait can be derived using the `#[derive(Info)]` attribute, provided in `calcbot-attrs`.
pub trait Info {
    /// Returns the command's metadata.
    fn info(&self) -> CommandInfo;
}

/// Returns the root command group.
pub fn root() -> CommandGroup {
    CommandGroup {
        commands: vec![
            Box::new(about::About),
            Box::new(help::Help),
            Box::new(link::Link),
            Box::new(not_math::NotMath),
        ],
    }
}
