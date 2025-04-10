use serde::{Deserialize, Serialize};
use twilight_mention::{timestamp::{Timestamp, TimestampStyle}, Mention};
use std::{error::Error, ops::{Add, AddAssign}, sync::Arc, time::{Duration, SystemTime}};
use tokio::{sync::Mutex, task::JoinHandle, time::Sleep};
use twilight_model::id::{marker::{ChannelMarker, UserMarker}, Id};

use crate::{database::{user::Timers, Database}, global::State, util::format_duration};

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

    /// The instant in time the timer was set. TODO: unneeded?
    // pub start_time: Instant,

    /// State of the timer.
    pub state: TimerState,

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
                    format!("Running, {} left", format_duration(duration)),
                    Some(Timestamp::new(unix_secs, Some(TimestampStyle::LongDateTime)).mention()),
                )
            },
            TimerState::Paused { remaining } => {
                (
                    format!("Paused, {} left", format_duration(*remaining)),
                    None,
                )
            },
        };
        let trigger_location = self.channel_id.mention();
        let message = if self.message.is_empty() {
            String::new()
        } else {
            format!("\n\"{}\"", self.message)
        };

        format!(
            "{state}\nTriggers in: {trigger_location}{message}{}",
            if let Some(timestamp) = timestamp {
                format!("\n{timestamp}")
            } else {
                "".to_string()
            },
        )
    }

    /// Create the timer's task that will send a reminder message to the given channel when the
    /// timer ends.
    ///
    /// This function **must** be called when the timer is created or when the timer is modified.
    pub(crate) fn create_task(&mut self, bot_state: Arc<State>, db: Arc<Mutex<Database>>) {
        let timer_id = self.id.clone();
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let message = self.message.clone();
        let future = self.sleep();

        // kill the old task if it exists
        if let Some(task) = self.task.take() {
            task.abort();
        }

        self.task = Some(tokio::spawn(async move {
            future.await;

            let msg = match message.len() {
                0 => format!("<@{}>'s reminder: _no message provided_", user_id),
                _ => format!("<@{}>'s reminder: **{}**", user_id, message),
            };
            bot_state.http.create_message(channel_id)
                .content(&msg)?
                .await?;

            let mut db = db.lock().await;
            db.get_user_field_mut::<Timers>(user_id).await.remove(&timer_id);

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
