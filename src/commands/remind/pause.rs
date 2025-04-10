use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::{user::Timers, Database},
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Pause a specified reminder by its reminder ID. You can view your active reminders and their IDs
/// with `{prefix}remind view`.
#[derive(Clone, Info)]
#[info(
    aliases = ["pause", "p"],
    syntax = ["<reminder id>"],
    examples = ["4bxB"],
)]
pub struct Pause;

#[async_trait]
impl Command for Pause {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let timer_id = ctxt.raw_input;

        let mut database = database.lock().await;
        let timer = database.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(timer_id);

        if let Some(timer) = timer {
            timer.pause();
            database.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully paused the reminder with ID `{timer_id}`.**"))?
                .await?;
        } else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
        }

        Ok(())
    }
}
