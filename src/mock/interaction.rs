use super::{Channel, Message};

/// Mock message.
pub struct Interaction {
    /// The channel of the interaction.
    pub channel: Option<Channel>,

    /// The message of the interaction.
    pub message: Option<Message>,
}
