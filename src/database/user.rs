use cas_compute::numerical::ctxt::Ctxt as EvalCtxt;
use twilight_model::id::{marker::UserMarker, Id};
use crate::timer::Timer;
use mysql_async::{FromRowError, Value, prelude::FromRow};
use serde_json::from_str;
use std::collections::HashMap;

/// Represents user preferences and settings.
#[derive(Debug, Default, Clone)]
pub struct UserSettings {
    /// The user's timezone offset.
    pub time_zone: i8,
}

impl FromRow for UserSettings {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError> {
        Ok(Self {
            time_zone: row.get::<i8, _>("time_zone").unwrap(),
        })
    }
}

/// Represents user-specific data across all sessions.
#[derive(Debug, Clone)]
pub struct UserData {
    /// The ID of the user.
    pub id: Id<UserMarker>,

    /// The user's evaluation context.
    pub ctxt: EvalCtxt,

    /// The timers the user has set.
    pub timers: HashMap<String, Timer>,

    /// Whether the user is in preview mode.
    ///
    /// This is a temporary mode that exists while the new CalcBot is being developed. Both
    /// instances of CalcBot run simultaneously, but will selectively ignore users based on this
    /// flag. If this is `true`, the user is using the new CalcBot (i.e. this instance). If
    /// `false`, the user is using the old Node.js-based CalcBot.
    pub using_preview: bool,
}

impl FromRow for UserData {
    fn from_row_opt(row: mysql_async::Row) -> Result<Self, FromRowError> {
        Ok(Self {
            id: row.get::<String, _>("id").unwrap().parse().unwrap(),
            ctxt: from_str(&row.get::<String, _>("ctxt").unwrap()).unwrap(),
            timers: from_str(&row.get::<String, _>("timers").unwrap()).unwrap(),
            using_preview: row.get::<bool, _>("using_preview").unwrap(),
        })
    }
}

impl UserData {
    /// Creates a new [`UserData`] instance with the given user ID.
    pub fn new(id: Id<UserMarker>) -> Self {
        Self {
            id,
            ctxt: EvalCtxt::default(),
            timers: HashMap::new(),
            using_preview: false,
        }
    }
}

/// A helper trait used to generically access and modify specific fields of [`UserData`].
///
/// This makes it easy to fetch specific fields from the database and modify them without having to
/// manually specify the field as a string. When fetching a specific field from the database,
/// simply use one of the following types as the generic parameter to get the field:
///
/// - [`Ctxt`]: The user's evaluation context.
/// - [`Timers`]: The timers the user has set.
pub trait UserField {
    /// The name of the column in the database that corresponds to this field.
    const COLUMN_NAME: &'static str;

    /// The type of the field to be serialized to and deserialized from the database.
    type Type;

    /// Gets mutable access to the field in the [`UserData`] instance.
    fn get_mut(user_data: &mut UserData) -> &mut Self::Type;

    /// Gets the [`Value`] from the field in the [`UserData`] instance.
    fn value(user_data: &UserData) -> Value;
}

/// [`UserData::ctxt`]
pub struct Ctxt;

/// [`UserData::timers`]
pub struct Timers;

/// [`UserData::using_preview`]
pub struct UsingPreview;

impl UserField for Ctxt {
    const COLUMN_NAME: &'static str = "ctxt";

    type Type = EvalCtxt;

    fn get_mut(user_data: &mut UserData) -> &mut Self::Type {
        &mut user_data.ctxt
    }

    fn value(user_data: &UserData) -> Value {
        serde_json::to_string(&user_data.ctxt)
            .unwrap()
            .into()
    }
}

impl UserField for Timers {
    const COLUMN_NAME: &'static str = "timers";

    type Type = HashMap<String, Timer>;

    fn get_mut(user_data: &mut UserData) -> &mut Self::Type {
        &mut user_data.timers
    }

    fn value(user_data: &UserData) -> Value {
        serde_json::to_string(&user_data.timers)
            .unwrap()
            .into()
    }
}

impl UserField for UsingPreview {
    const COLUMN_NAME: &'static str = "using_preview";

    type Type = bool;

    fn get_mut(user_data: &mut UserData) -> &mut Self::Type {
        &mut user_data.using_preview
    }

    fn value(user_data: &UserData) -> Value {
        user_data.using_preview.into()
    }
}
