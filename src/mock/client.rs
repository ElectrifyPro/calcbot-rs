use twilight_model::id::{marker::ChannelMarker, Id};

/// Mock HTTP client that prints out requests and responses.
pub struct HttpClient {
}

impl HttpClient {
    /// Creates a new [`HttpClient`] with the given token.
    pub fn new() -> Self {
        Self {  }
    }

    /// Create a message in a channel.
    pub async fn create_message(&self, channel_id: Id<ChannelMarker>) {

    }
}
