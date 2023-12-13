use std::{ops::{Add, AddAssign, Deref, Sub, SubAssign}, time::Duration};

/// A wrapper around [`usize`] that is clamped to a range. When adding or subtracting to this
/// wrapper, the value will wrap around to the other end of the range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clamped {
    /// The inner value.
    value: usize,

    /// The maximum value of the wrapper. This is **non-inclusive**.
    max: usize,
}

impl Clamped {
    /// Creates a new [`Clamped`] with the given value and maximum.
    pub fn new(value: usize, max: usize) -> Self {
        Self { value: value % max, max }
    }

    /// Returns the inner value.
    pub fn value(&self) -> usize {
        self.value
    }

    /// Returns the maximum value.
    pub fn max(&self) -> usize {
        self.max
    }
}

impl Deref for Clamped {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Add<usize> for Clamped {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new((self.value + rhs) % self.max, self.max)
    }
}

impl AddAssign<usize> for Clamped {
    fn add_assign(&mut self, rhs: usize) {
        self.value = (self.value + rhs) % self.max;
    }
}

impl Sub<usize> for Clamped {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self::new((self.value + self.max - rhs) % self.max, self.max)
    }
}

impl SubAssign<usize> for Clamped {
    fn sub_assign(&mut self, rhs: usize) {
        self.value = (self.value + self.max - rhs) % self.max;
    }
}

/// Given a count and a word, returns a string in the format "X word" or "X words", depending on
/// the count.
pub fn pluralize(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {}", word)
    } else {
        format!("{} {}s", count, word)
    }
}

/// Formats a time duration as a string. The output will contain one unit of time, and is formatted
/// as "X y", where X is the amount of time and y is the unit of time.
pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() as usize;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        pluralize(days, "day")
    } else if hours > 0 {
        pluralize(hours, "hour")
    } else if minutes > 0 {
        pluralize(minutes, "minute")
    } else {
        pluralize(seconds, "second")
    }
}
