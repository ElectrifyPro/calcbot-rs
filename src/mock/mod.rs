//! Contains mock implementations of various `twilight` types for offline testing / compile
//! performance.

#![cfg(feature = "mock")]

pub mod channel;
pub mod client;
pub mod event;
pub mod interaction;
pub mod message;

pub use channel::Channel;
pub use client::HttpClient;
pub use event::{Event, InteractionCreate, MessageCreate};
pub use interaction::Interaction;
pub use message::Message;
