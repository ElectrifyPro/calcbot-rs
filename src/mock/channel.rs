use twilight_model::id::{marker::ChannelMarker, Id};

/// Mock channel.
pub struct Channel {
    /// The channel ID.
    pub id: Id<ChannelMarker>,
}
