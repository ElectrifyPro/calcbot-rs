use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_unit_convert::{unit::Time, Base, Measurement, Unit};
use crate::{
    arg_parse::{Remainder, Word, parse_args_full},
    commands::{Command, Context, Info, remind::{Label, Metadata, create_timer_and_confirm}},
    database::Database,
    error::Error,
    global::State,
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
    parent = super::Remind,
)]
pub struct Every;

#[async_trait]
impl Command for Every {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(f64, Word, Remainder)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let quantity = parsed.0;
        let unit = parsed.1.0;
        let message = parsed.2.0;

        let Ok(unit) = unit.try_into() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{unit}` is not a valid time unit.**"))
                .await?;
            return Ok(());
        };
        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, Unit::new(Base::Time(unit)))
            .convert(Time::Second)
            .unwrap()
            .value());

        if time_amount < Duration::from_secs(60) {
            ctxt.trigger.reply(&state.http)
                .content("**The recurring reminder interval must be at least 1 minute long.**")
                .await?;
            return Ok(());
        }

        let end_time = SystemTime::now() + time_amount;
        create_timer_and_confirm(
            state,
            database,
            ctxt,
            end_time,
            message.to_string(),
            Metadata {
                duration: time_amount,
                label: Label::Every,
            },
        ).await?;

        Ok(())
    }
}
