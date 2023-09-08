use cas_eval::ctxt::Ctxt;
use mysql_async::{prelude::FromRow, FromRowError};
use serde_json::from_str;

/// Represents user-specific data across all sessions.
#[derive(Debug, Clone, Default)]
pub struct UserData {
    /// The user's evaluation context.
    pub ctxt: Ctxt,
}

impl FromRow for UserData {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError> {
        Ok(Self {
            ctxt: from_str(&row.get::<String, _>("ctxt").unwrap()).unwrap(),
        })
    }
}

/// Represents a specific field of [`UserData`] and its value.
#[derive(Debug, Clone)]
pub enum UserField {
    /// The user's evaluation context.
    Ctxt(Ctxt),
}
