use std::time::Instant;

/// The global state of the bot.
pub struct State {
    /// The [`Instant`] the bot was started. This can be used to determine the bot's uptime.
    pub start_time: Instant,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}
