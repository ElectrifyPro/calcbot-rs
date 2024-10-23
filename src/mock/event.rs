use std::ops::Deref;
use super::{Interaction, Message};
use twilight_model::id::{marker::{ChannelMarker, MessageMarker, UserMarker}, Id};

/// Mirror of `Event` from `twilight_gateway`, used to implement the mocking client.
pub enum Event {
    /// A message was created in a channel.
    MessageCreate(MessageCreate),

    /// A message was deleted in a channel.
    MessageDelete(MessageDelete),

    /// An interaction was created in a channel.
    InteractionCreate(InteractionCreate),
}

impl Event {
    /// Creates a [`Event::MessageCreate`] with the given content.
    pub fn message_create(content: String) -> Self {
        Self::MessageCreate(MessageCreate::new(content))
    }
}

/// Mirror of `MessageCreate` from `twilight_model`, used to implement the mocking client.
pub struct MessageCreate(pub Message);

impl Deref for MessageCreate {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageCreate {
    /// Creates a new `MessageCreate` with the given content.
    pub fn new(content: String) -> Self {
        Self(Message::new(content))
    }
}

/// Mirror of `MessageDelete` from `twilight_model`, used to implement the mocking client.
pub struct MessageDelete {
    /// The ID of the message.
    pub id: Id<MessageMarker>,

    /// The channel ID of the message.
    pub channel_id: Id<ChannelMarker>,
}

/// Mirror of `InteractionCreate` from `twilight_model`, used to implement the mocking client.
pub struct InteractionCreate(pub Interaction);

impl Deref for InteractionCreate {
    type Target = Interaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
