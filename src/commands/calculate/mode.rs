use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_compute::numerical::ctxt::TrigMode;
use crate::{
    commands::{Command, Context},
    database::{user::Ctxt, Database},
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// View or set the angle calculation mode of the calculator. (default **radians**)
#[derive(Clone, Info)]
#[info(
    syntax = ["", "[radians | radian | rad | r]", "[degrees | degree | deg | d]"],
)]
pub struct Mode;

#[async_trait]
impl Command for Mode {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let mut database = database.lock().await;
        let eval_ctxt = database.get_user_field_mut::<Ctxt>(ctxt.trigger.author_id()).await;

        let new_mode = match ctxt.raw_input.get(0..1) {
            Some("r") => TrigMode::Radians,
            Some("d") => TrigMode::Degrees,
            _ => {
                ctxt.trigger.reply(&state.http)
                    .content(&format!("Current calculation mode: **{}**", eval_ctxt.trig_mode))?
                    .await?;
                return Ok(());
            },
        };

        eval_ctxt.trig_mode = new_mode;
        database.commit_user_field::<Ctxt>(ctxt.trigger.author_id()).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("Set calculation mode to **{}**", new_mode))?
            .await?;

        Ok(())
    }
}
