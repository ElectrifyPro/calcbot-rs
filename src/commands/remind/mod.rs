// pub mod at;
pub mod delete;
pub mod edit;
pub mod every;
pub mod increment;
pub mod pause;
pub mod recur;
pub mod resume;
pub mod view;

use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    arg_parse::{Word, Remainder, parse_args_full},
    commands::{Command, Context, Info},
    database::{user::Timers, Database},
    error::Error,
    global::State,
    timer::Timer,
};
use std::{sync::Arc, time::{Duration, SystemTime}};
use tokio::sync::Mutex;

/// Set a reminder with an optional message for a specified interval. You can find the available
/// time units with `{prefix}unitconvert units`. You can view your active reminders and their IDs
/// with `{prefix}remind view`. See the **children commands** field to see the various ways you can
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
    children = [
        delete::Delete,
        edit::Edit,
        every::Every,
        increment::Increment,
        pause::Pause,
        recur::Recur,
        resume::Resume,
        view::View,
    ],
)]
pub struct Remind;
    // children = [
    //     at::At,
    // ],

#[async_trait]
impl Command for Remind {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(f64, Word, Remainder)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                Error::Embed(self.info().build_embed(ctxt.prefix))
            } else {
                err
            })?;
        let quantity = parsed.0;
        let unit = parsed.1.0;
        let message = parsed.2.0;

        let Ok(unit) = unit.try_into() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{unit}` is not a valid time unit.**"))?
                .await?;
            return Ok(());
        };
        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, Unit::new(Quantity::Time(unit)))
            .convert(Time::Second)
            .unwrap()
            .value());

        let end_time = SystemTime::now() + time_amount;
        let mut timer = Timer::running(
            ctxt.trigger.author_id(),
            ctxt.trigger.channel_id(),
            end_time,
            message.to_string(),
        );
        timer.create_task(Arc::clone(&state), Arc::clone(&database));
        let id = timer.id.clone();

        // add to local and remote database so timer can be loaded if bot restarts mid-timer
        let mut database = database.lock().await;
        database.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .insert(id.clone(), timer);
        database.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

        ctxt.trigger.reply(&state.http)
            .content(&format!("**You will be mentioned in this channel in `{quantity} {unit}`.** This reminder's ID is `{id}`."))?
            .await?;

        Ok(())
    }
}
