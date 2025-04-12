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

/// Toggles the recurring status of a specified reminder. If the reminder is already set to recur,
/// this command will disable its recurring status. If the reminder is not set to recur, you must
/// also specify the time interval the reminder will recur with once it triggers.
#[derive(Clone, Info)]
#[info(
    aliases = ["recur", "rec"],
    syntax = ["<reminder id> [<quantity> <time unit>]"],
    examples = ["4bxB", "4bxB 1 min"],
    args = [String, Option<f64>, Option<String>],
)]
pub struct Recur;

#[async_trait]
impl Command for Recur {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (timer_id, quantity, unit) = parse_args(ctxt.raw_input.split_whitespace().collect::<Vec<_>>())?;

        let mut db = database.lock().await;
        let Some(timer) = db.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(&timer_id) else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
            return Ok(());
        };

        if timer.recur.is_some() {
            // disable recurring status
            timer.recur = None;
            timer.create_task(Arc::clone(&state), Arc::clone(&database));

            db.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully disabled the recurring status of the reminder with ID `{timer_id}`. It will not recur when it triggers."))?
                .await?;
            return Ok(());
        }

        // enable recurring status
        let (Some(quantity), Some(unit)) = (quantity, unit) else {
            ctxt.trigger.reply(&state.http)
                .content("**You must specify a time interval for the reminder to recur with.**")?
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

        if time_amount < Duration::from_secs(60) {
            ctxt.trigger.reply(&state.http)
                .content("**The recurring reminder interval must be at least 1 minute.**")?
                .await?;
            return Ok(());
        }

        timer.recur = Some(time_amount);
        timer.create_task(Arc::clone(&state), Arc::clone(&database));
        let id = timer.id.clone();

        db.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("**Successfully set the recurring status of the reminder with ID `{id}`. Once it triggers, it will repeatedly recur for `{quantity} {unit}`.**"))?
            .await?;

        Ok(())
    }
}
