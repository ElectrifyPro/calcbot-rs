use serde::{Deserialize, Serialize};
use twilight_mention::{timestamp::{Timestamp, TimestampStyle}, Mention};
use std::{error::Error, ops::{Add, AddAssign}, sync::Arc, time::{Duration, SystemTime}};
use tokio::{task::JoinHandle, time::Sleep};
use twilight_model::id::{marker::{ChannelMarker, UserMarker}, Id};

use crate::global::State;

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
#[derive(Deserialize, Serialize)]
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

    /// Global state of the bot.
    #[serde(skip)]
    bot_state: Option<Arc<State>>,

    /// The task that will send the reminder message.
    #[serde(skip)]
    task: Option<JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>>,
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("channel_id", &self.channel_id)
            .field("state", &self.state)
            .field("message", &self.message)
            .finish()
    }
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
            bot_state: self.bot_state.clone(),
            task: None,
        }
    }
}

impl Timer {
    /// Creates a new timer that ends at the given time.
    ///
    /// The created timer is actively running.
    pub fn running(
        state: &Arc<State>,
        user_id: Id<UserMarker>,
        channel_id: Id<ChannelMarker>,
        end_time: SystemTime,
        message: String,
    ) -> Self {
        let mut timer = Self {
            id: random_string::generate(4, random_string::charsets::ALPHA_LOWER),
            user_id,
            channel_id,
            state: TimerState::Running { end_time },
            message,
            bot_state: Some(Arc::clone(state)),
            task: None,
        };
        timer.create_task();
        timer
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
                    format!("Running, {} sec left", duration.as_secs_f64()),
                    Some(Timestamp::new(unix_secs, Some(TimestampStyle::LongDateTime)).mention()),
                )
            },
            TimerState::Paused { remaining } => {
                (
                    format!("Paused, {} sec left", remaining.as_secs_f64()),
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
    fn create_task(&mut self) {
        let bot_state = Arc::clone(self.bot_state.as_ref().unwrap());
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let message = self.message.clone();
        let future = self.sleep();

        // kill the old task if it exists
        if let Some(ref mut task) = self.task {
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
        self.create_task();
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
        self.create_task();
    }
}
