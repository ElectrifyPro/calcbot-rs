pub mod about;
pub mod calculate;
pub mod dictionary;
pub mod help;
pub mod link;
pub mod not_math;
pub mod remind;
pub mod unit_convert;

use super::{database::Database, error::Error, global::State};
use async_trait::async_trait;
use std::{iter::Peekable, sync::Arc};
use tokio::sync::Mutex;
use twilight_http::{request::channel::message::CreateMessage, Client};
use twilight_model::{channel::message::{Embed, Message}, id::{marker::{ChannelMarker, UserMarker}, Id}};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// Formats a list of commands into a code block. Each string is displayed on a separate line,
/// prepended with the given prefix.
///
/// The output will look like this:
/// ```text
/// <prefix> <string1>
/// <prefix> <string2>
/// <prefix> <string3>
/// ...
/// ```
pub fn format_code_block(prefix: &str, strings: &[&str]) -> String {
    format!(
        "```\n{}\n```",
        strings
            .iter()
            .map(|string| format!("{} {}", prefix, string))
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

impl From<Vec<Box<dyn Command>>> for CommandGroup {
    fn from(commands: Vec<Box<dyn Command>>) -> Self {
        Self { commands }
    }
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

    /// The parent of this command. This is used to build the command tree.
    pub parent: Option<Box<dyn Command>>,
}

impl CommandInfo {
    /// Retrieves the default alias for this command.
    pub fn default_alias(&self) -> &'static str {
        self.aliases
            .and_then(|aliases| aliases.first())
            .unwrap_or(&self.name)
    }

    /// Retrieves the shortest alias for this command.
    fn shortest_alias(&self) -> &'static str {
        self.aliases
            .and_then(|aliases| aliases.iter().min_by_key(|s| s.len()))
            .unwrap_or(&self.name)
    }

    /// Returns true if the given string is an alias for this command.
    pub fn is_alias(&self, alias: &str) -> bool {
        self.aliases
            .map(|aliases| aliases.contains(&alias))
            .unwrap_or(self.name == alias)
    }

    /// Builds the path of commands to type to execute `self`. The first path is the normalized
    /// path, while the second path is a shortened version of the path that uses the shortest
    /// aliases of each command in the tree.
    fn build_path(&self) -> (String, String) {
        let mut out = (String::from(self.default_alias()), String::from(self.shortest_alias()));
        let mut current = self.parent.as_ref().map(|p| p.clone_box());

        while let Some(parent) = current {
            let parent_info = parent.info();
            out.0.insert(0, ' ');
            out.0.insert_str(0, parent_info.default_alias());
            out.1.insert(0, ' ');
            out.1.insert_str(0, parent_info.shortest_alias());
            current = parent_info.parent;
        }

        out
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
        let (path, short_path) = self.build_path();
        let full_path = format!("{}{}", prefix, path);

        let mut embed =
            EmbedBuilder::new()
                .title(&full_path)
                .color(0x66d2e8)
                .field(EmbedFieldBuilder::new(
                    "Description",
                    self.description.replace("{prefix}", prefix),
                ));

        if let Some(syntax) = self
            .syntax
            .map(|syntax| format_code_block(&full_path, syntax))
        {
            embed = embed.field(EmbedFieldBuilder::new("Syntax", syntax));
        }

        if let Some(examples) = self
            .examples
            .map(|examples| format_code_block(&full_path, examples))
        {
            embed = embed.field(EmbedFieldBuilder::new("Examples", examples));
        }

        embed = embed.field(EmbedFieldBuilder::new(
            "Shorthand",
            format!("`{}{}`", prefix, short_path),
        ));

        if let Some(aliases) = self.aliases {
            embed = embed
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

/// Some event within Discord that triggered a command.
///
/// TODO: this will later be extended with slash command support
#[derive(Clone, Copy, Debug)]
pub enum Trigger<'a> {
    /// A message was sent in a channel.
    Message(&'a Message),
}

impl<'a> From<&'a Message> for Trigger<'a> {
    fn from(msg: &'a Message) -> Self {
        Trigger::Message(msg)
    }
}

impl Trigger<'_> {
    /// Returns the ID of the author who triggered this event.
    pub fn author_id(&self) -> Id<UserMarker> {
        match self {
            Trigger::Message(msg) => msg.author.id,
        }
    }

    /// Returns the ID of the channel where this event was triggered.
    ///
    /// TODO: this is only used for sending paged messages
    pub fn channel_id(&self) -> Id<ChannelMarker> {
        match self {
            Trigger::Message(msg) => msg.channel_id,
        }
    }

    /// Create a reply to this event trigger.
    pub fn reply<'c>(&self, http: &'c Client) -> CreateMessage<'c> {
        match self {
            Trigger::Message(msg) => http.create_message(msg.channel_id),
        }
    }
}

/// The context passed to a command's [`Command::execute`] method. This wraps various fields needed
/// by most commands in one convenient struct.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    /// The event that triggered the command.
    pub trigger: Trigger<'a>,

    /// The prefix used to invoke the command. If [`None`], the command was invoked from a DM
    /// channel.
    pub prefix: Option<&'a str>,

    /// The user's raw input to the command. This includes only the arguments passed to the command
    /// and does not include the prefix, command name, or any whitespace at the start or end of the
    /// string.
    pub raw_input: &'a str,
}

/// Represents any command that can be executed by a user (accounting for permissions and other
/// factors).
#[async_trait]
pub trait Command: CommandClone + Info + Send + Sync {
    /// Executes the command.
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error>;
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
            Box::new(calculate::Calculate),
            Box::new(dictionary::Dictionary),
            Box::new(help::Help),
            Box::new(link::Link),
            Box::new(not_math::NotMath),
            Box::new(remind::Remind),
            Box::new(unit_convert::UnitConvert),
        ],
    }
}
