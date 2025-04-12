use std::{fmt::{Display, Formatter}, time::Duration};

/// Given a count and a word, returns a string in the format "X word" or "X words", depending on
/// the count.
pub fn pluralize(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {}", word)
    } else {
        format!("{} {}s", count, word)
    }
}

/// Formatter for a [`Duration`].
///
/// The formatter will choose a unit of time (days, hours, minutes, seconds) based on the magnitude
/// of the duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DurationFormatter {
    /// The duration to format.
    pub value: Duration,
}

impl Display for DurationFormatter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let seconds = self.value.as_secs_f64();
        if seconds < 60.0 {
            return write!(f, "{seconds} seconds");
        }

        let minutes = seconds / 60.0;
        if minutes < 60.0 {
            return write!(f, "{minutes} minutes");
        }

        let hours = minutes / 60.0;
        if hours < 24.0 {
            return write!(f, "{hours} hours");
        }

        let days = hours / 24.0;
        write!(f, "{days} days")
    }
}

pub trait DurationExt {
    /// Wraps the [`Duration`] in a [`DurationFormatter`] with maximum precision.
    fn fmt(&self) -> DurationFormatter;
}

impl DurationExt for Duration {
    fn fmt(&self) -> DurationFormatter {
        DurationFormatter { value: *self }
    }
}
