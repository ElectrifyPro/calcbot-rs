use serde::{Deserialize, Serialize};
use twilight_mention::{timestamp::{Timestamp, TimestampStyle}, Mention};
use std::{error::Error, ops::{Add, AddAssign}, sync::Arc, time::{Duration, SystemTime}};
use tokio::{sync::Mutex, task::JoinHandle, time::Sleep};
use twilight_model::id::{marker::{ChannelMarker, UserMarker}, Id};

use crate::{database::{user::Timers, Database}, fmt::DurationExt, global::State};

/// State of a timer.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TimerState {
    /// The timer is running.
    Running {
        /// The system time the timer will end.
        end_time: SystemTime,
    },

    /// The timer is paused.
    Paused {
        /// The amount of time remaining on the timer.
        remaining: Duration,
    },
}

/// A timer set by a user using the `c-remind` and its related commands.
///
/// When cloning a timer, the task that sends the reminder message is not cloned.
#[derive(Debug, Deserialize, Serialize)]
pub struct Timer {
    /// The ID of the timer.
    pub id: String,

    /// The ID of the user who set the timer.
    pub user_id: Id<UserMarker>,

    /// The ID of the channel where the message setting the timer was sent.
    pub channel_id: Id<ChannelMarker>,

    /// State of the timer.
    pub state: TimerState,

    /// The amount of time to recur with once the timer triggers. If [`None`], the timer does not
    /// recur (e.g. one-shot).
    pub recur: Option<Duration>,

    /// The message to send when the timer ends.
    pub message: String,

    /// The task that will send the reminder message.
    #[serde(skip)]
    task: Option<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>>,
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some(ref mut task) = self.task {
            task.abort();
        }
    }
}

impl Clone for Timer {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            user_id: self.user_id,
            channel_id: self.channel_id,
            state: self.state.clone(),
            recur: self.recur,
            message: self.message.clone(),
            task: None,
        }
    }
}

impl Timer {
    /// Creates a new timer that ends at the given time.
    ///
    /// The [`Timer::create_task`] function must be called after this to create the task that
    /// sends the reminder message.
    pub fn running(
        user_id: Id<UserMarker>,
        channel_id: Id<ChannelMarker>,
        end_time: SystemTime,
        message: String,
    ) -> Self {
        Self {
            id: random_string::generate(4, random_string::charsets::ALPHA_LOWER),
            user_id,
            channel_id,
            state: TimerState::Running { end_time },
            recur: None,
            message,
            task: None,
        }
    }

    /// Returns true if the timer is running.
    ///
    /// This does not check the task, only [`Timer::state`].
    pub fn is_running(&self) -> bool {
        matches!(self.state, TimerState::Running { .. })
    }

    /// Pauses a running timer. The underlying task will be aborted until [`Timer::create_task`] is
    /// called again. This is a no-op if the timer is already paused.
    pub fn pause(&mut self) {
        if let TimerState::Running { end_time } = &self.state {
            let remaining = end_time.duration_since(SystemTime::now()).unwrap_or_default();
            self.state = TimerState::Paused { remaining };
        }

        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    /// Resumes a paused timer. This is a no-op if the timer is already running.
    ///
    /// After resuming, call [`Timer::create_task`] to create the task that sends the reminder
    /// message.
    pub fn resume(&mut self) {
        if let TimerState::Paused { remaining } = &self.state {
            let end_time = SystemTime::now() + *remaining;
            self.state = TimerState::Running { end_time };
        }

        if let Some(task) = self.task.take() {
            task.abort();
        }
    }

    /// Sets a new remaining duration of the timer.
    ///
    /// After calling this function, call [`Timer::create_task`] to update the task that sends the
    /// reminder message.
    pub fn set_new_duration(&mut self, duration: Duration) {
        match &self.state {
            TimerState::Running { .. } => {
                let new_end_time = SystemTime::now() + duration;
                self.state = TimerState::Running { end_time: new_end_time };
            },
            TimerState::Paused { .. } => {
                self.state = TimerState::Paused { remaining: duration };
            },
        }
    }

    /// Sets a new end time for the timer. If the timer is paused, the remaining time is
    /// calculated from the new end time.
    ///
    /// After calling this function, call [`Timer::create_task`] to update the task that sends the
    /// reminder message.
    pub fn set_new_end_time(&mut self, new_end_time: SystemTime) {
        match &mut self.state {
            TimerState::Running { end_time } => {
                *end_time = new_end_time;
            },
            TimerState::Paused { remaining } => {
                *remaining = new_end_time.duration_since(SystemTime::now()).unwrap_or(*remaining)
            },
        }
    }

    /// Creates a [`Sleep`] future that will complete when the timer ends.
    pub fn sleep(&self) -> Sleep {
        match &self.state {
            TimerState::Running { end_time } => {
                let duration = end_time.duration_since(SystemTime::now()).unwrap_or_default();
                tokio::time::sleep(duration)
            },
            TimerState::Paused { remaining } => {
                tokio::time::sleep(*remaining)
            },
        }
    }

    /// Builds the timer's description for the `c-remind view` command.
    pub fn build_description(&self) -> String {
        let (state, timestamp) = match &self.state {
            TimerState::Running { end_time } => {
                let unix_secs = end_time
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let duration = end_time.duration_since(SystemTime::now()).unwrap_or_default();
                (
                    format!("Running, {} left", duration.fmt()),
                    Some(Timestamp::new(unix_secs, Some(TimestampStyle::LongDateTime)).mention()),
                )
            },
            TimerState::Paused { remaining } => {
                (
                    format!("Paused, {} left", remaining.fmt()),
                    None,
                )
            },
        };
        let trigger_location = self.channel_id.mention();
        let recur = if let Some(recur) = self.recur {
            format!("\nRecurs for: {}", recur.fmt())
        } else {
            String::new()
        };
        let message = if self.message.is_empty() {
            String::new()
        } else {
            format!("\n\"{}\"", self.message)
        };
        let timestamp = if let Some(timestamp) = timestamp {
            format!("\n{timestamp}")
        } else {
            String::new()
        };
        format!("{state}\nTriggers in: {trigger_location}{recur}{message}{timestamp}")
    }

    /// Create the timer's task that will send a reminder message to the given channel when the
    /// timer ends.
    ///
    /// This function **must** be called when the timer is created or when the timer is modified.
    pub(crate) fn create_task(&mut self, bot_state: Arc<State>, db: Arc<Mutex<Database>>) {
        let TimerState::Running { end_time } = self.state else {
            return;
        };

        let timer_id = self.id.clone();
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let recur = self.recur;
        let message = self.message.clone();
        let future = self.sleep();

        // kill the old task if it exists
        if let Some(task) = self.task.take() {
            task.abort();
        }

        self.task = Some(tokio::spawn(async move {
            let mut end_time = end_time;
            let mut future = future;

            loop {
                future.await;

                let msg = match message.len() {
                    0 => format!("{}'s reminder: _no message provided_", user_id.mention()),
                    _ => format!("{}'s reminder: **{}**", user_id.mention(), message),
                };
                bot_state.http.create_message(channel_id).content(&msg).await?;

                if let Some(recur) = recur {
                    // to handle cases where the bot restarts after a recurring timer has
                    // triggered, we calculate the new end time based on the number of times the
                    // timer would have triggered while the bot was offline
                    let secs_since_end = SystemTime::now()
                        .duration_since(end_time)
                        .unwrap_or_default()
                        .as_secs_f64();
                    let recur_secs = recur.as_secs_f64();
                    let num_triggers = 1.0 + (secs_since_end / recur_secs).floor();
                    let adjusted_offset = Duration::from_secs_f64(recur_secs * num_triggers);
                    let new_end_time = end_time + adjusted_offset;

                    let mut db = db.lock().await;
                    let timer = db.get_user_field_mut::<Timers>(user_id).await
                        .get_mut(&timer_id)
                        .unwrap();
                    timer.set_new_end_time(new_end_time);

                    end_time = new_end_time;
                    future = timer.sleep();

                    // TODO: could committing clog the database with too many writes? if the bot
                    // does restart, we can figure out where we were anyway, that's the whole point
                    // of doing that calculation at the start of this `if let`
                    db.commit_user_field::<Timers>(user_id).await;
                } else {
                    break;
                }
            }

            let mut db = db.lock().await;

            // NOTE: **must** bind returned timer here to a variable to avoid dropping it before
            // `commit_user_field` is called. if we don't bind, timer's Drop impl will abort the
            // task (i.e. this function), which will abort execution of the `commit_user_field`
            // function that immediately follows
            // this results in timers only being removed from local cache, but not database;
            // whenever the restarts, the timer will still be in the database and will always get
            // restored again and again
            // i think we only have to make this distinction inside the task; elsewhere, we can
            // call `remove` without binding
            let _timer = db.get_user_field_mut::<Timers>(user_id).await.remove(&timer_id);
            db.commit_user_field::<Timers>(user_id).await;

            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }));
    }
}

impl Add<Duration> for Timer {
    type Output = Self;

    fn add(mut self, rhs: Duration) -> Self::Output {
        self.state = match self.state {
            TimerState::Running { end_time } => {
                TimerState::Running { end_time: end_time + rhs }
            },
            TimerState::Paused { remaining } => {
                TimerState::Paused { remaining: remaining + rhs }
            },
        };
        self
    }
}

impl AddAssign<Duration> for Timer {
    fn add_assign(&mut self, rhs: Duration) {
        self.state = match self.state {
            TimerState::Running { end_time } => {
                TimerState::Running { end_time: end_time + rhs }
            },
            TimerState::Paused { remaining } => {
                TimerState::Paused { remaining: remaining + rhs }
            },
        };
    }
}
