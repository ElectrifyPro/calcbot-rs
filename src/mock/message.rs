use twilight_model::id::{marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker}, Id};

/// An author of a message.
#[derive(Debug)]
pub struct Author {
    /// The ID of the author.
    pub id: Id<UserMarker>,

    /// Whether the author is a bot. During mocking, this is always `false`.
    pub bot: bool,
}

/// Mock message.
#[derive(Debug)]
pub struct Message {
    /// The message ID.
    pub id: Id<MessageMarker>,

    /// The author of the message.
    pub author: Author,

    /// The guild ID of the message.
    pub guild_id: Option<Id<GuildMarker>>,

    /// The content of the message.
    pub content: String,

    /// The channel ID of the message.
    pub channel_id: Id<ChannelMarker>,
}

impl Message {
    /// Creates a new [`Message`] with the given content.
    pub fn new(content: String) -> Self {
        Self {
            id: Id::new(1),
            author: Author {
                id: Id::new(1),
                bot: false,
            },
            guild_id: None,
            content,
            channel_id: Id::new(1),
        }
    }
}
