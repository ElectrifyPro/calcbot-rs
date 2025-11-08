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

/// Resume a specified reminder by its reminder ID. You can view your active reminders and their
/// IDs with `{prefix}remind view`.
#[derive(Clone, Info)]
#[info(
    aliases = ["resume", "res", "r"],
    syntax = ["<reminder id>"],
    examples = ["4bxB"],
    parent = super::Remind,
)]
pub struct Resume;

#[async_trait]
impl Command for Resume {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let timer_id = ctxt.raw_input;

        let mut db = database.lock().await;
        let timer = db.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(timer_id);

        if let Some(timer) = timer {
            timer.resume();
            timer.create_task(Arc::clone(state), Arc::clone(database));
            db.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully resumed the reminder with ID `{timer_id}`.**"))
                .await?;
        } else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))
                .await?;
        }

        Ok(())
    }
}
