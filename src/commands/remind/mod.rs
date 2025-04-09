// pub mod at;
pub mod delete;
// pub mod edit;
// pub mod every;
pub mod increment;
// pub mod pause;
// pub mod recur;
// pub mod resume;
pub mod view;

use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
    timer::Timer,
};
use std::{sync::Arc, time::{Duration, SystemTime}};
use tokio::sync::Mutex;

/// Set a reminder with an optional message for a specified interval. You can find the available
/// time units with `{prefix}unitconvert units`. You can view your reminders and their IDs with
/// `{prefix}remind view`. See the **children commands** field to see the various ways you can
/// interact with reminders.
///
/// For reminders (set in servers) that are 2 minutes or longer, members can click the `Remind me`
/// button on the reminder message in order to receive the reminder with you.
#[derive(Clone, Info)]
#[info(
    category = "Miscellaneous",
    aliases = ["remind", "rem"],
    syntax = ["<quantity> <time unit> [message]"],
    examples = ["10 minutes", "10 minutes stop watching tv"],
    args = [f64, String, Unlimited],
    children = [
        delete::Delete,
        increment::Increment,
        view::View,
    ],
)]
pub struct Remind;
    // children = [
    //     at::At,
    //     edit::Edit,
    //     every::Every,
    //     pause::Pause,
    //     recur::Recur,
    //     resume::Resume,
    // ],

#[async_trait]
impl Command for Remind {
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
            .convert(Unit::new(Quantity::Time(Time::Second)))
            .unwrap()
            .value());

        let end_time = SystemTime::now() + time_amount;
        let mut timer = Timer::running(
            ctxt.trigger.author_id(),
            ctxt.trigger.channel_id(),
            end_time,
            message,
        );
        timer.create_task(Arc::clone(&state), Arc::clone(&database));
        let id = timer.id.clone();

        // add to local and remote database so timer can be loaded if bot restarts mid-timer
        database.lock().await.add_timer(timer).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("**You will be mentioned in this channel in `{quantity} {unit}`.** This reminder's ID is `{id}`."))?
            .await?;

        Ok(())
    }
}
