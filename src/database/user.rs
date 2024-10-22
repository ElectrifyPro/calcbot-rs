use cas_compute::numerical::ctxt::Ctxt;
use crate::timer::Timer;
use mysql_async::{prelude::FromRow, FromRowError};
use serde_json::from_str;
use std::collections::HashMap;

/// Represents user-specific data across all sessions.
#[derive(Debug, Clone, Default)]
pub struct UserData {
    /// The user's evaluation context.
    pub ctxt: Ctxt,

    /// The timers the user has set.
    pub timers: HashMap<String, Timer>,
}

impl FromRow for UserData {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError> {
        Ok(Self {
            ctxt: from_str(&row.get::<String, _>("ctxt").unwrap()).unwrap(),
            timers: from_str(&row.get::<String, _>("timers").unwrap()).unwrap(),
        })
    }
}

/// Represents a specific field of [`UserData`] and its value.
#[derive(Debug, Clone)]
pub enum UserField {
    /// The user's evaluation context.
    Ctxt(Ctxt),

    /// The timers the user has set.
    Timers(HashMap<String, Timer>),
}
