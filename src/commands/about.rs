use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    fmt::DurationExt,
    global::State,
};
use std::{env, num::NonZeroU64, sync::Arc};
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use tokio::sync::Mutex;
use twilight_mention::Mention;
use twilight_util::builder::embed::EmbedBuilder;

/// View information about CalcBot.
#[derive(Clone, Info)]
#[info(category = "Miscellaneous")]
pub struct About;

#[async_trait]
impl Command for About {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let mut system = System::new_all();
        system.refresh_all();

        let pid = Pid::from(std::process::id() as usize);
        let process = system.process(pid).unwrap();

        // we fetch the author's tag from the api because just using the "<@author_id>" syntax will
        // not work if the author is not in the same server as the user who ran the command
        let author_id = env::var("AUTHOR_ID")?.parse::<NonZeroU64>().unwrap();
        let author = {
            let user = state.http.user(author_id.into()).await?.model().await?;
            format!("{}#{}", user.name, user.discriminator())
        };

        let bot = state.cache.current_user()
            .expect("should be received upon login")
            .mention();

        let embed = EmbedBuilder::new()
            .title("About me")
            .color(0x988bc2)
            .description(format!("
            {} is constantly being developed by **@{}**.

            Uptime: {}
            Shard CPU usage: {}%
            Shard memory usage: {} MB
            Commands: {}
            ",
                bot,
                author,
                state.start_time.elapsed().fmt(),
                process.cpu_usage(),
                process.memory() / 1024 / 1024,
                state.commands.count(),
            ))
            .build();

        ctxt.trigger.reply(&state.http)
            .embeds(&[embed])?
            .await?;

        Ok(())
    }
}
