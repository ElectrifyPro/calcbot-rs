use super::commands::{self, CommandGroup};
use std::{collections::HashMap, time::Instant};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_http::Client as HttpClient;
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// The global state of the bot.
pub struct State {
    /// The [`Instant`] the bot was started. This can be used to determine the bot's uptime.
    pub start_time: Instant,

    /// The commands at the base of the command tree.
    pub commands: CommandGroup,

    /// The HTTP client, used to make requests to the Discord API.
    pub http: HttpClient,

    /// The cache, which stores information received from Discord.
    pub cache: InMemoryCache,
}

impl State {
    /// Creates a new [`State`] with the given token.
    pub fn new(token: String) -> Self {
        Self {
            start_time: Instant::now(),
            commands: CommandGroup::new(vec![
                Box::new(commands::about::About),
                Box::new(commands::help::Help),
                Box::new(commands::not_math::NotMath),
            ]),
            http: HttpClient::new(token),
            cache: InMemoryCache::builder()
                .resource_types(ResourceType::USER_CURRENT | ResourceType::MESSAGE)
                .build(),
        }
    }

    /// Build the `c-help commands` embed.
    pub fn build_commands_embed(&self) -> Embed {
        let mut embed = EmbedBuilder::new()
            .title("Available commands")
            .color(0xda70d6)
            .description(format!(
                "
                This server's prefix is `{0}`. Type `{0}<command>` to access one of the commands below, and type `{0}help <command>` to learn more about that command. You can find documentation for all commands [here](https://chillant.gitbook.io/calcbot/reference/commands).

                CalcBot's command system can be confusing for those new to the bot. This short [guide](https://chillant.gitbook.io/calcbot/commands/command-system) will hopefully clear up that confusion.
            ",
                "c-", // TODO
            ));

        let mut categories = HashMap::new();
        let category_emoji = |category: &str| match category {
            "Calculate" => "🔰",
            "Graphing" => "📈",
            "Miscellaneous" => "🤹",
            "Resources" => "📚",
            "Settings" => "⚙️",
            "Text" => "📝",
            _ => "❓",
        };

        for cmd in &self.commands.commands {
            let info = cmd.info();
            let category = info.category.unwrap();
            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(info.default_alias());
        }

        for (category, commands) in categories {
            embed = embed.field(
                EmbedFieldBuilder::new(
                    format!("{} {}", category_emoji(category), category),
                    format!("`{}`", commands.join("`, `")),
                )
                .inline()
                .build(),
            );
        }

        embed.build()
    }
}
