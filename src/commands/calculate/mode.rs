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
            .get_user(ctxt.message.author.id).await
            .clone();

        let new_mode = match ctxt.raw_input.get(0..1) {
            Some("r") => TrigMode::Radians,
            Some("d") => TrigMode::Degrees,
            _ => {
                state.http.create_message(ctxt.message.channel_id)
                    .content(&format!("Current calculation mode: **{}**", user_data.ctxt.trig_mode))
                    .unwrap()
                    .await?;
                return Ok(());
            },
        };

        user_data.ctxt.trig_mode = new_mode;
        database.lock().await
            .set_user_field(ctxt.message.author.id, UserField::Ctxt(user_data.ctxt)).await;

        state.http.create_message(ctxt.message.channel_id)
            .content(&format!("Set calculation mode to **{}**", new_mode))
            .unwrap()
            .await?;

        Ok(())
    }
}
