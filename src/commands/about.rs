use crate::{
    commands::{Command, CommandInfo},
    global::State,
    util::format_duration,
};
use async_trait::async_trait;
use std::{env, error::Error, num::NonZeroU64, sync::Arc};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::EmbedBuilder;

/// `c-about` command.
pub struct About;

#[async_trait]
impl Command for About {
    fn info(&self) -> CommandInfo {
        CommandInfo {
            name: "about",
            description: "View information about CalcBot.",
            aliases: None,
            syntax: None,
            examples: None,
        }
    }

    async fn execute(
        &self,
        http: Arc<Client>,
        cache: Arc<InMemoryCache>,
        state: Arc<State>,
        message: &Message,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = Pid::from(std::process::id() as usize);
        let process = system.process(pid).unwrap();

        // we fetch the author's tag from the api because just using the "<@author_id>" syntax will
        // not work if the author is not in the same server as the user who ran the command
        let author_id = env::var("AUTHOR_ID")?.parse::<NonZeroU64>().unwrap();
        let author = {
            let user = http.user(author_id.into()).await?.model().await?;
            format!("{}#{}", user.name, user.discriminator())
        };

        let bot_id = cache
            .current_user()
            .expect("should be received upon login")
            .id
            .get();

        let embed = EmbedBuilder::new()
            .title("About me")
            .color(0x988bc2)
            .description(format!(
                "
            <@{}> is being constantly developed by {}.

            Uptime: {}
            Shard CPU usage: {}%
            Shard memory usage: {} MB
            ",
                bot_id,
                author,
                format_duration(state.start_time.elapsed()),
                process.cpu_usage(),
                process.memory() / 1024 / 1024
            ))
            .build();

        http.create_message(message.channel_id)
            .embeds(&[embed])
            .unwrap()
            .await?;

        Ok(())
    }
}