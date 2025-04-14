use async_trait::async_trait;
use calcbot_attrs::Info;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    fmt::pluralize,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// View all of your active reminders.
#[derive(Clone, Info)]
#[info(aliases = ["view", "list", "v", "l"])]
pub struct View;

#[async_trait]
impl Command for View {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let prefix = ctxt.prefix.unwrap_or_default();
        let mut database = database.lock().await;
        let user_data = database
            .get_user(ctxt.trigger.author_id()).await;

        let mut embed = EmbedBuilder::new()
            .title("Reminders")
            .color(0xda70d6)
            .footer(EmbedFooterBuilder::new(pluralize(user_data.timers.len(), "reminder")));

        if user_data.timers.is_empty() {
            embed = embed.description(&format!("You have no active reminders. Use the `{prefix}remind` command to set one."));
        } else {
            for (id, timer) in &user_data.timers {
                embed = embed.field(EmbedFieldBuilder::new(format!("`{id}`"), timer.build_description()).inline());
            }
        }

        ctxt.trigger.reply(&state.http)
            .embeds(&[embed.build()])?
            .await?;
        Ok(())
    }
}
