use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::{Database, user::UsingPreview},
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Disable **preview mode**.
///
/// **You are currently using the preview version of CalcBot.** You can disable preview mode and
/// return to the stable version by running `{prefix}preview off`.
#[derive(Clone, Info)]
#[info(
    category = "Miscellaneous",
    syntax = ["off"],
)]
pub struct Preview;

#[async_trait]
impl Command for Preview {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        if ctxt.raw_input != "off" {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You are currently using the preview version of CalcBot.** You can disable preview mode and return to the stable version by running `{}preview off`.", ctxt.prefix.unwrap_or_default()))?
                .await?;
            return Ok(());
        }

        let mut database = database.lock().await;
        let using_preview = database.get_user_field_mut::<UsingPreview>(ctxt.trigger.author_id()).await;
        *using_preview = false;
        database.commit_user_field::<UsingPreview>(ctxt.trigger.author_id()).await;

        ctxt.trigger.reply(&state.http)
            .content("**Preview mode disabled.** You are now using the stable version of CalcBot.")?
            .await?;

        Ok(())
    }
}
