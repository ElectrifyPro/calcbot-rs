use std::future::IntoFuture;
use twilight_http::{
    request::channel::message::CreateMessage,
    response::{DeserializeBodyError, ResponseFuture},
};
use twilight_model::channel::message::Message;
use twilight_validate::message::MessageValidationError;

/// Describes an error that can format itself into a rich Discord message.
pub trait Error {
    /// Creates a rich Discord message with the given base, describing the error.
    ///
    /// Because [`CreateMessage`] borrows its content, this makes it impossible to return a
    /// [`CreateMessage`] directly, as many error types need to generate their own data. Instead,
    /// this method takes an extra step and returns a [`ResponseFuture`] (which can be done by
    /// using the [`std::future::IntoFuture`] trait). When awaited, the message will be sent.
    fn fmt<'a>(&self, init: CreateMessage<'a>) -> Result<ResponseFuture<Message>, MessageValidationError>;
}

impl<T> From<T> for Box<dyn Error + Send + Sync>
where
    T: Error + Send + Sync + 'static,
{
    fn from(err: T) -> Self {
        Box::new(err)
    }
}

// these are generic impl for all possible errors
// where possible, we should create specific error types for each error with much better messages
// than this
macro_rules! generic_error_impl {
    ($($name:ty)+) => {
        $(
            impl Error for $name {
                fn fmt<'a>(&self, init: CreateMessage<'a>) -> Result<ResponseFuture<Message>, MessageValidationError> {
                    Ok(init.content(&format!("**Oops!** CalcBot processed your command correctly, but Discord rejected the response message. This could be a bug!\nPlease report this to the developers, and include this error code:\n```\n{}\n```", stringify!($name)))?
                        .into_future())
                }
            }
        )+
    };
}

generic_error_impl! {
    DeserializeBodyError
    MessageValidationError
    std::env::VarError
    twilight_http::Error
}

impl Error for &str {
    fn fmt<'a>(&self, init: CreateMessage<'a>) -> Result<ResponseFuture<Message>, MessageValidationError> {
        Ok(init.content(self)?
            .into_future())
    }
}

/// An argument was missing from a command invocation.
#[derive(Debug)]
pub struct MissingArgument {
    /// The index of the argument that was missing.
    pub index: usize,
}

impl Error for MissingArgument {
    fn fmt<'a>(&self, init: CreateMessage<'a>) -> Result<ResponseFuture<Message>, MessageValidationError> {
        Ok(init.content(&format!("Missing argument at index {}.", self.index))?
            .into_future())
    }
}
