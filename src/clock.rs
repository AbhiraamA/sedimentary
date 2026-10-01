//! Wall-clock helpers. Persisted timestamps are Unix seconds. Anything a
//! misbehaving clock could break is clamped rather than trusted.

use std::time::{SystemTime, UNIX_EPOCH};

pub const MINUTE: i64 = 60;
pub const HOUR: i64 = 60 * MINUTE;
pub const DAY: i64 = 24 * HOUR;

/// Current Unix time in seconds. A clock set before 1970 reads as 0.
pub fn now_unix() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

fn plural(n: i64, unit: &str) -> String {
    if n == 1 {
        format!("1 {unit}")
    } else {
        format!("{n} {unit}s")
    }
}

/// Human phrase for a span of time: "a few seconds", "1 hour", "31 days".
pub fn duration_words(secs: i64) -> String {
    let secs = secs.max(0);
    if secs < MINUTE {
        "a few seconds".to_string()
    } else if secs < HOUR {
        plural(secs / MINUTE, "minute")
    } else if secs < DAY {
        plural(secs / HOUR, "hour")
    } else if secs < 60 * DAY {
        plural(secs / DAY, "day")
    } else if secs < 365 * DAY {
        plural(secs / (30 * DAY), "month")
    } else {
        plural(secs / (365 * DAY), "year")
    }
}

/// Compact relative label for journal entries: "now", "4m", "3h", "2d".
pub fn short_ago(secs: i64) -> String {
    let secs = secs.max(0);
    if secs < MINUTE {
        "now".to_string()
    } else if secs < HOUR {
        format!("{}m", secs / MINUTE)
    } else if secs < DAY {
        format!("{}h", secs / HOUR)
    } else {
        format!("{}d", secs / DAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_words_picks_sensible_units() {
        assert_eq!(duration_words(-5), "a few seconds");
        assert_eq!(duration_words(90), "1 minute");
        assert_eq!(duration_words(3 * HOUR + 5), "3 hours");
        assert_eq!(duration_words(31 * DAY), "31 days");
        assert_eq!(duration_words(90 * DAY), "3 months");
        assert_eq!(duration_words(800 * DAY), "2 years");
    }

    #[test]
    fn short_ago_is_compact() {
        assert_eq!(short_ago(10), "now");
        assert_eq!(short_ago(5 * MINUTE), "5m");
        assert_eq!(short_ago(2 * DAY), "2d");
    }
}
