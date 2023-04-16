use std::time::Duration;

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
