use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context, remind::action::{self, Reason}},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Delete a specified reminder by its reminder ID. You can view your reminders and their reminder
/// IDs with `{prefix}remind view`.
#[derive(Clone, Info)]
#[info(
    aliases = ["delete", "del", "d"],
    syntax = ["<reminder id>"],
    examples = ["4bxB"],
    parent = super::Remind,
)]
pub struct Delete;

#[async_trait]
impl Command for Delete {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let timer_id = ctxt.raw_input;

        let mut database = database.lock().await;
        let timer = action::complete(
            timer_id,
            ctxt.trigger.author_id(),
            state,
            &mut database,
            Reason::Deleted,
        ).await;

        if timer.is_some() {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully deleted the reminder with ID `{timer_id}`.**"))
                .await?;
        } else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))
                .await?;
        }

        Ok(())
    }
}
