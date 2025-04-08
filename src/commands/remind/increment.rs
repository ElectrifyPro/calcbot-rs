use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Adds or removes time from a reminder, given its reminder ID.
///
/// You can find the list of available time units with `{prefix}unitconvert units`. You can view
/// your active reminders and their IDs with `{prefix}remind view`.
#[derive(Clone, Info)]
#[info(
    aliases = ["increment", "incr", "inc", "i"],
    syntax = ["<reminder id> <quantity> <time unit>"],
    examples = ["4bxB 1", "4bxB 2.5 hr", "4bxB -10 min"],
    args = [String, f64, Option<String>],
)]
pub struct Increment;

#[async_trait]
impl Command for Increment {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (timer_id, quantity, unit) = parse_args(ctxt.raw_input.split_whitespace().collect::<Vec<_>>())?;
        let unit = match unit {
            Some(unit) => {
                let Ok(unit) = (&*unit).try_into() else {
                    ctxt.trigger.reply(&state.http)
                        .content(&format!("**`{unit}` is not a valid time unit.**"))?
                    .await?;
                    return Ok(());
                };
                unit
            },
            None => Unit::new(Quantity::Time(Time::Second)),
        };

        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, unit)
            .convert(Unit::new(Quantity::Time(Time::Second)))
            .unwrap()
            .value());

        let mut database = database.lock().await;

        if database.increment_timer(&ctxt.trigger.author_id(), &timer_id, time_amount).await.is_some() {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**Successfully added `{quantity} {unit}` to the reminder with ID `{timer_id}`.**"))?
                .await?;
        } else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
        }

        Ok(())
    }
}
