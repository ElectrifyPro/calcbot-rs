use serde::{Deserialize, Serialize};
use std::{error::Error, sync::Arc, time::{Duration, SystemTime}};
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
    /// The created timer is actively running.
    pub fn running(
        state: &Arc<State>,
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
        }.with_task(state)
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

    /// Create the timer's task that will send a reminder message to the given channel when the
    /// timer ends.
    fn with_task(mut self, state: &Arc<State>) -> Self {
        let state = Arc::clone(state);
        let user_id = self.user_id;
        let channel_id = self.channel_id;
        let message = self.message.clone();
        let future = self.sleep();

        self.task = Some(tokio::spawn(async move {
            future.await;

            let msg = match message.len() {
                0 => format!("<@{}>'s reminder: _no message provided_", user_id),
                _ => format!("<@{}>'s reminder: **{}**", user_id, message),
            };
            state.http.create_message(channel_id)
                .content(&msg)?
                .await?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        }));
        self
    }
}
