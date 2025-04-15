use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{unit::Time, Measurement, Quantity, Unit};
use crate::{
    arg_parse::{Word, Remainder, parse_args_full},
    commands::{Command, Context, Info},
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
)]
pub struct Edit;

#[async_trait]
impl Command for Edit {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(Word, f64, Word, Remainder)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let timer_id = parsed.0.0;
        let quantity = parsed.1;
        let unit = parsed.2.0;
        let message = parsed.3.0;

        let mut db = database.lock().await;
        let Some(timer) = db.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
            .get_mut(timer_id) else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**You do not have a reminder set with the ID `{timer_id}`.**"))?
                .await?;
            return Ok(());
        };

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

        timer.message = message.to_string();
        timer.set_new_duration(time_amount);
        timer.create_task(Arc::clone(state), Arc::clone(database));
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
