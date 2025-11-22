use async_trait::async_trait;
use calcbot_attrs::Info;
use chrono::{Days, FixedOffset, Local, NaiveTime};
use crate::{
    arg_parse::{Parse, Parser, Remainder, Word, parse_args_full},
    commands::{Command, Context, Info, remind::{Label, create_timer_and_confirm}},
    database::Database,
    error::Error,
    global::State,
};
use std::{sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

/// 12-hour or 24-hour.
pub(crate) enum ClockMode {
    Twelve(AmPm),
    TwentyFour,
}

/// AM or PM.
pub(crate) enum AmPm {
    AM,
    PM,
}

impl<'a> Parse<'a> for ClockMode {
    fn parse(parser: &mut Parser<'a>) -> Result<Self, Error> {
        let mut clone = parser.clone();
        let Ok(Word(word)) = Word::parse(&mut clone) else {
            return Ok(ClockMode::TwentyFour);
        };

        let am_pm = match &*word.to_lowercase() {
            "am" => AmPm::AM,
            "pm" => AmPm::PM,
            _ => return Ok(ClockMode::TwentyFour),
        };

        // consume the characters
        *parser = clone;
        Ok(ClockMode::Twelve(am_pm))
    }
}

/// Set a reminder with an optional message that will be triggered at the specified time. This
/// command supports both 24-hr and 12-hr times; if you include "AM" or "PM" (case-insensitive)
/// after the time, the time will be interpreted as a 12-hr time. Otherwise, it will be interpreted
/// as a 24-hr time.
///
/// **Make sure that you have correctly set your time zone settings with the `{prefix}settings`
/// command, or this command will not calculate the end time correctly!**
#[derive(Clone, Info)]
#[info(
    aliases = ["at", "@"],
    syntax = ["<hour>:<minute> [\"AM\" | \"PM\"] [message]"],
    examples = ["10:30 am party time", "22:00 stop watching youtube"],
    parent = super::Remind,
)]
pub struct At;

#[async_trait]
impl Command for At {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(Word, ClockMode, Remainder)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let time = parsed.0.0;
        let clock_mode = parsed.1;
        let message = parsed.2.0;

        let [hour, minute] = {
            let mut parts = time.split(':');
            let (
                Some(Ok(hour)),
                Some(Ok(minute)),
                None,
            ) = (
                parts.next().map(|s| s.parse::<u8>()),
                parts.next().map(|s| s.parse::<u8>()),
                parts.next(),
            ) else {
                ctxt.trigger.reply(&state.http)
                    .content("**You must specify a valid time in `hh:mm` format.**")
                    .await?;
                return Ok(());
            };
            [hour, minute]
        };

        if minute > 59 {
            ctxt.trigger.reply(&state.http)
                .content("**The minute must be between 0 and 59.**")
                .await?;
            return Ok(());
        }

        match clock_mode {
            ClockMode::Twelve(_) if hour > 12 => {
                ctxt.trigger.reply(&state.http)
                    .content("**The hour of a 12-hr time must be between 1 and 12.**")
                .await?;
                return Ok(());
            },
            ClockMode::TwentyFour if hour > 23 => {
                ctxt.trigger.reply(&state.http)
                    .content("**The hour of a 24-hr time must be between 0 and 23.**")
                .await?;
                return Ok(());
            },
            _ => {}
        }

        let hour = match clock_mode {
            ClockMode::Twelve(AmPm::AM) => hour % 12,
            ClockMode::Twelve(AmPm::PM) => hour % 12 + 12,
            _ => hour,
        };

        let time_zone = database.lock().await
            .get_user_settings(ctxt.trigger.author_id()).await
            .time_zone;

        // calculate the amount of time until the next occurrence of the specified time

        // user's current time
        let now = Local::now()
            .with_timezone(&FixedOffset::east_opt(time_zone as i32 * 3600).unwrap());

        // create a time object for the specified time
        let mut converted = now.with_time(NaiveTime::from_hms_opt(hour as u32, minute as u32, 0).unwrap()).unwrap();

        // if the specified time is in the past, add 24 hours to it
        if converted < now {
            converted = converted.checked_add_days(Days::new(1)).unwrap();
        }

        let duration = converted.signed_duration_since(now).to_std().unwrap();
        create_timer_and_confirm(
            state,
            database,
            ctxt,
            SystemTime::now() + duration,
            message.to_string(),
            Label::At(clock_mode, hour, minute, duration),
        ).await?;

        Ok(())
    }
}
