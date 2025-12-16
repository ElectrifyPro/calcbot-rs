use std::future::IntoFuture;
use twilight_http::{request::channel::message::CreateMessage, response::ResponseFuture};
use twilight_model::channel::{message::Embed, Message};
use twilight_validate::message::MessageValidationError;

/// An error that can format itself into a user-friendly Discord message.
#[derive(Debug)]
pub enum Error {
    /// A string that is formatted as-is.
    String(String),

    /// A command is missing an argument.
    MissingArgument(MissingArgument),

    /// A required argument was not provided.
    NoArgument,

    /// A command has too many arguments.
    TooManyArguments,

    /// Show an embed.
    ///
    /// The embed is boxed to reduce the size of the enum (600 bytes!).
    Embed(Box<Embed>),

    /// Custom error. It can only be formatted with `&self` and not `self`.
    Custom(Box<dyn CustomErrorFmt + Send + Sync>),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::String(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::String(err.to_string())
    }
}

impl From<MissingArgument> for Error {
    fn from(missing: MissingArgument) -> Self {
        Self::MissingArgument(missing)
    }
}

impl From<Embed> for Error {
    fn from(embed: Embed) -> Self {
        Self::Embed(Box::new(embed))
    }
}

impl<T> From<T> for Error
where
    T: CustomErrorFmt + Send + Sync + 'static,
{
    fn from(custom: T) -> Self {
        Self::Custom(Box::new(custom))
    }
}

impl Error {
    /// Creates a rich Discord message describing the error.
    ///
    /// Because [`CreateMessage`] borrows its content, this makes it impossible to return a
    /// [`CreateMessage`] directly, as many error types need to generate their own data. Instead,
    /// this method takes an extra step and returns a [`ResponseFuture`] (which can be done by
    /// using the [`std::future::IntoFuture`] trait). When awaited, the message will be sent.
    pub fn rich_fmt(self, init: CreateMessage<'_>) -> Result<ResponseFuture<Message>, MessageValidationError> {
        match self {
            Self::String(err) => Ok(init.content(&err).into_future()),
            Self::MissingArgument(missing) => Ok(init.content(&format!("Missing argument at index {}.", missing.index)).into_future()),
            Self::NoArgument => Ok(init.content("No argument provided.").into_future()),
            Self::TooManyArguments => Ok(init.content("Too many arguments.").into_future()),
            Self::Embed(embed) => Ok(init.embeds(&[*embed]).into_future()),
            Self::Custom(custom) => custom.rich_fmt(init),
        }
    }
}

pub trait CustomErrorFmt: std::fmt::Debug {
    /// Creates a rich Discord message describing the error.
    ///
    /// Because [`CreateMessage`] borrows its content, this makes it impossible to return a
    /// [`CreateMessage`] directly, as many error types need to generate their own data. Instead,
    /// this method takes an extra step and returns a [`ResponseFuture`] (which can be done by
    /// using the [`std::future::IntoFuture`] trait). When awaited, the message will be sent.
    fn rich_fmt(&self, init: CreateMessage<'_>) -> Result<ResponseFuture<Message>, MessageValidationError>;
}

// these are generic impl for all possible errors
// where possible, we should create specific error types for each error with much better messages
// than this
macro_rules! generic_error_impl {
    ($($name:ty)+) => {
        $(
            impl CustomErrorFmt for $name {
                fn rich_fmt(&self, init: CreateMessage<'_>) -> Result<ResponseFuture<Message>, MessageValidationError> {
                    Ok(init.content(&format!("**Oops!** CalcBot processed your command correctly, but Discord rejected the response message. This could be a bug!\nPlease report this to the developers, and include this error code:\n```\n{}\n```", stringify!($name)))
                        .into_future())
                }
            }
        )+
    };
}

generic_error_impl! {
    twilight_http::response::DeserializeBodyError
    twilight_validate::message::MessageValidationError
    std::env::VarError
    twilight_http::Error
}

/// An argument was missing from a command invocation.
#[derive(Debug)]
pub struct MissingArgument {
    /// The index of the argument that was missing.
    pub index: usize,
}
