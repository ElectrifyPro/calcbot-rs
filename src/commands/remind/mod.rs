pub mod at;
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
use cas_unit_convert::{unit::Time, Measurement};
use twilight_model::channel::message::{Component, EmojiReactionType, component::{ActionRow, Button, ButtonStyle}};
use crate::{
    arg_parse::{Remainder, Word, parse_args_full},
    commands::{Command, Context, Info, remind::at::{AmPm, ClockMode}},
    database::{Database, user::Timers},
    error::Error,
    fmt::DurationExt,
    global::State,
    timer::Timer,
};
use std::{sync::Arc, time::{Duration, SystemTime}};
use tokio::sync::Mutex;

/// Timer metadata.
struct Metadata {
    duration: Duration,
    label: Label,
}

/// Determines which type of timer is being created.
enum Label {
    /// One-time.
    In,

    /// One-time, at a specific 12-hour or 24-hour time.
    At(ClockMode, u8, u8),

    /// Recurring.
    Every,
}

/// Create a timer, add it to the database, and send the confirmation message in one function.
async fn create_timer_and_confirm(
    state: &Arc<State>,
    database: &Arc<Mutex<Database>>,
    ctxt: Context<'_>,
    end_time: SystemTime,
    message: String,
    metadata: Metadata,
) -> Result<(), Error> {
    let mut timer = Timer::running(
        ctxt.trigger.author_id(),
        ctxt.trigger.channel_id(),
        end_time,
        message,
    );
    if let Label::Every = metadata.label {
        timer.recur = Some(metadata.duration);
    }
    timer.create_task(Arc::clone(state), Arc::clone(database));

    let supports_multiple_receivers =
        ctxt.prefix.is_some() && metadata.duration >= Duration::from_secs(120);

    let label = match metadata.label {
        Label::In => format!("**You will be mentioned in this channel in `{}`.**", metadata.duration.fmt()),
        Label::At(clock_mode, hour, minute) => {
            let time_input = match clock_mode {
                ClockMode::Twelve(AmPm::AM) => format!("{hour}:{minute:02} AM"),
                ClockMode::Twelve(AmPm::PM) => format!("{hour}:{minute:02} PM"),
                ClockMode::TwentyFour => format!("{hour}:{minute:02}"),
            };
            format!("**You will be mentioned in this channel at `{time_input}`** (in `{}`).", metadata.duration.fmt())
        },
        Label::Every => format!("**You will be mentioned _repeatedly_ in this channel every `{}`.**", metadata.duration.fmt()),
    };
    let multiply_receivers_msg = if supports_multiple_receivers {
        " Other users can click the `Remind me` button to receive the reminder with you."
    } else {
        ""
    };
    let content = format!("{label} This reminder's ID is `{}`.{multiply_receivers_msg}", timer.id);

    // add to local and remote database so timer can be loaded if bot restarts mid-timer
    let mut database = database.lock().await;
    database.get_user_field_mut::<Timers>(ctxt.trigger.author_id()).await
        .insert(timer.id.clone(), timer);
    database.commit_user_field::<Timers>(ctxt.trigger.author_id()).await;

    let msg = ctxt.trigger.reply(&state.http)
        .content(&content);
    if supports_multiple_receivers {
        msg.components(&[
            Component::ActionRow(ActionRow {
                components: vec![
                    Component::Button(Button {
                        custom_id: Some("remind-me-too".to_owned()),
                        disabled: false,
                        emoji: Some(EmojiReactionType::Unicode {
                            name: String::from("⏰"),
                        }),
                        label: Some(String::from("Remind me")),
                        style: ButtonStyle::Primary,
                        url: None,
                        sku_id: None,
                    }),
                ],
            }),
        ]).await?;
    } else {
        msg.await?;
    }

    Ok(())
}

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
        at::At,
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
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let quantity = parsed.0;
        let unit = parsed.1.0;
        let message = parsed.2.0;

        let Ok(unit): Result<Time, _> = unit.try_into() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{unit}` is not a valid time unit.**"))
                .await?;
            return Ok(());
        };
        let time_amount = Duration::from_secs_f64(*Measurement::new(quantity, unit)
            .convert(Time::Second)
            .unwrap()
            .value());

        let end_time = SystemTime::now() + time_amount;
        create_timer_and_confirm(
            state,
            database,
            ctxt,
            end_time,
            message.to_string(),
            Metadata {
                duration: time_amount,
                label: Label::In,
            },
        ).await?;

        Ok(())
    }
}
