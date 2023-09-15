use super::commands::{self, CommandGroup};
use std::{collections::HashMap, time::Instant};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_http::Client as HttpClient;
use twilight_model::{channel::message::Embed, id::{marker::ApplicationMarker, Id}};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// The global state of the bot.
pub struct State {
    /// The application ID of the bot.
    pub application_id: Id<ApplicationMarker>,

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
    pub async fn new(token: String) -> Self {
        let http = HttpClient::new(token);
        Self {
            application_id: http.current_user_application().await.unwrap()
                .model().await.unwrap().id,
            start_time: Instant::now(),
            commands: commands::root(),
            http,
            cache: InMemoryCache::builder()
                .resource_types(ResourceType::USER_CURRENT | ResourceType::MESSAGE)
                .build(),
        }
    }

    /// Build the `c-help commands` embed.
    pub fn build_commands_embed(&self, prefix: Option<&str>) -> Embed {
        let mut embed = EmbedBuilder::new()
            .title("Available commands")
            .color(0xda70d6)
            .description(format!(
                "{}help <command>` to learn more about that command. You can find documentation for all commands [here](https://chillant.gitbook.io/calcbot/reference/commands).

                CalcBot's command system can be confusing for those new to the bot. This short [guide](https://chillant.gitbook.io/calcbot/commands/command-system) will hopefully clear up that confusion.
            ",
                if let Some(prefix) = prefix {
                    format!("This server's prefix is `{0}`. Type `{0}<command>` to run one of the commands below, and type `{0}", prefix)
                } else {
                    "Type `<command>` to run one of the commands below, and type `".to_string()
                }
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
