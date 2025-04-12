use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    commands::{Command, Context},
    database::{user::Timers, Database},
    error::Error,
    global::State,
    timer::Timer,
};
use std::{sync::Arc, time::{Duration, SystemTime}};
use tokio::sync::Mutex;

/// Set a **recurring** reminder with an optional message for a specified interval. Upon
/// triggering, the reminder will automatically be renewed for the specified time interval. To
/// avoid spam, the reminder interval must be **at least 1 minute.**
#[derive(Clone, Info)]
#[info(
    aliases = ["every", "ev"],
    syntax = ["<quantity> <time unit> [message]"],
    examples = ["10 min", "1 min level up"],
    args = [f64, String, Unlimited],
)]
pub struct Every;

#[async_trait]
impl Command for Every {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (quantity, unit, message) = parse_args(ctxt.raw_input.split_whitespace().collect::<Vec<_>>())?;

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

        let end_time = SystemTime::now() + time_amount;
        let mut timer = Timer::running(
            ctxt.trigger.author_id(),
            ctxt.trigger.channel_id(),
            end_time,
            message,
        );
        timer.recur = Some(time_amount);
        timer.create_task(Arc::clone(&state), Arc::clone(&database));
        let id = timer.id.clone();

        // add to local and remote database so timer can be loaded if bot restarts mid-timer
        let mut database = database.lock().await;
        database.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .insert(id.clone(), timer);
        database.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("**You will be mentioned repeatedly in this channel every `{quantity} {unit}`.** This reminder's ID is `{id}`."))?
            .await?;

        Ok(())
    }
}
