use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_compute::numerical::ctxt::TrigMode;
use crate::{
    commands::{Command, Context},
    database::{user::UserField, Database},
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
    async fn execute(
        &self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut user_data = database.lock().await
            .get_user(ctxt.trigger.author_id()).await
            .clone();

        let new_mode = match ctxt.raw_input.get(0..1) {
            Some("r") => TrigMode::Radians,
            Some("d") => TrigMode::Degrees,
            _ => {
                ctxt.trigger.reply(&state.http)
                    .content(&format!("Current calculation mode: **{}**", user_data.ctxt.trig_mode))?
                    .await?;
                return Ok(());
            },
        };

        user_data.ctxt.trig_mode = new_mode;
        database.lock().await
            .set_user_field(ctxt.trigger.author_id(), UserField::Ctxt(user_data.ctxt)).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("Set calculation mode to **{}**", new_mode))?
            .await?;

        Ok(())
    }
}
