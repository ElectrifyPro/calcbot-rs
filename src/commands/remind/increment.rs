use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_unit_convert::{unit::Time, Base, Measurement, Unit};
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::{user::Timers, Database},
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
    parent = super::Remind,
)]
pub struct Increment;

#[async_trait]
impl Command for Increment {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(Word, f64, Option<Word>)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let timer_id = parsed.0.0;
        let quantity = parsed.1;
        let unit = parsed.2.map(|unit| unit.0);

        let unit = match unit {
            Some(unit) => {
                let Ok(unit) = unit.try_into() else {
                    ctxt.trigger.reply(&state.http)
                        .content(&format!("**`{unit}` is not a valid time unit.**"))?
                    .await?;
                    return Ok(());
                };
                unit
            },
            None => Unit::new(Base::Time(Time::Second)),
        };

        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, unit)
            .convert(Time::Second)
            .unwrap()
            .value());

        let mut db = database.lock().await;
        let Some(timer) = db.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(timer_id) else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
            return Ok(());
        };

        *timer += time_amount;
        timer.create_task(Arc::clone(state), Arc::clone(database));

        ctxt.trigger.reply(&state.http)
            .content(&format!("**Successfully added `{quantity} {unit}` to the reminder with ID `{timer_id}`.**"))?
        .await?;

        Ok(())
    }
}
