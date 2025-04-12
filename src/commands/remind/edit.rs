use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    commands::{Command, Context},
    database::{user::Timers, Database},
    error::Error,
    global::State,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Change the interval of a specified reminder by its reminder ID. If the `[new message]` argument
/// is left empty, it will not be changed. You can find the available time units with
/// `{prefix}unitconvert units`. You can view your reminders and their reminder IDs with
/// `{prefix}remind view`.
#[derive(Clone, Info)]
#[info(
    aliases = ["edit", "e"],
    syntax = ["<reminder id> <new quantity> <new time unit> [new message]"],
    examples = ["10 minutes", "10 minutes stop watching tv"],
    args = [String, f64, String, Unlimited],
)]
pub struct Edit;

#[async_trait]
impl Command for Edit {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (timer_id, quantity, unit, message) = parse_args(ctxt.raw_input.split_whitespace().collect::<Vec<_>>())?;

        let mut db = database.lock().await;
        let Some(timer) = db.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(&timer_id) else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
            return Ok(());
        };

        let Ok(unit) = (&*unit).try_into() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{unit}` is not a valid time unit.**"))?
                .await?;
            return Ok(());
        };
        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, Unit::new(Quantity::Time(unit)))
            .convert(Time::Second)
            .unwrap()
            .value());

        timer.message = message;
        timer.set_new_duration(time_amount);
        timer.create_task(Arc::clone(&state), Arc::clone(&database));
        let is_running = timer.is_running();

        db.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

        if is_running {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully edited the reminder with ID `{timer_id}`.** It will trigger in `{quantity} {unit}`."))?
                .await?;
        } else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully edited the reminder with ID `{timer_id}`.** Once resumed, it will trigger in `{quantity} {unit}`."))?
                .await?;
        }

        Ok(())
    }
}
