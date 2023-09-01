use cas_eval::ctxt::{Ctxt, TrigMode};
use mysql_async::{prelude::FromRow, FromRowError};
use serde_json::from_str;

/// Represents user-specific data across all sessions.
#[derive(Debug, Clone, Default)]
pub struct UserData {
    /// The user's preferred trigonometric mode.
    pub calculate: TrigMode,

    /// The user's evaluation context.
    pub ctxt: Ctxt,
}

impl FromRow for UserData {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError> {
        Ok(Self {
            calculate: match row.get::<u8, _>("calculate") {
                Some(1) => TrigMode::Degrees,
                _ => TrigMode::Radians,
            },
            ctxt: from_str(&row.get::<String, _>("ctxt").unwrap()).unwrap(),
        })
    }
}
