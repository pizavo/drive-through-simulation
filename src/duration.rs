use serde::de::{self, Visitor};
use serde::Deserializer;
use std::fmt;
use std::time::Duration;

/// Parses a duration string using `humantime`, falling back to raw number (seconds) if necessary.
fn parse_duration(s: &str) -> Result<f64, String> {
    if let Ok(duration) = humantime::parse_duration(s) {
        return Ok(duration.as_secs_f64());
    }

    s.trim().parse::<f64>().map_err(|_| {
        format!("Invalid duration: '{}'. Expected human-readable format (e.g., '1m 30s') or a number of seconds.", s)
    })
}

/// Formats a duration in seconds as a human-readable string
///
/// # Examples
/// ```
/// assert_eq!(format_duration(0.0), "0s");
/// assert_eq!(format_duration(90.0), "1m 30s");
/// ```
pub fn format_duration(seconds: f64) -> String {
    if seconds == 0.0 {
        return "0s".to_string();
    }
    let seconds = seconds.max(0.0);
    let millis = (seconds * 1000.0).round() as u64;
    let duration = Duration::from_millis(millis);
    humantime::format_duration(duration).to_string()
}

/// Formats a duration in seconds as a fixed-width human-readable string for column alignment
///
/// Format: "Xy Xm Xd Xh Xmin Xs Xms" with zero-padding and consistent spacing
/// Examples: "09s 123ms", "1y 02m 03d 04h 05min 06s 007ms"
pub fn format_duration_fixed_width(seconds: f64) -> String {
    if seconds < 0.0 {
        return format!("{:>30}", "INVALID");
    }

    let total_millis = (seconds * 1000.0).round() as u64;

    let millis = total_millis % 1000;
    let total_secs = total_millis / 1000;
    let secs = total_secs % 60;
    let total_mins = total_secs / 60;
    let mins = total_mins % 60;
    let total_hours = total_mins / 60;
    let hours = total_hours % 24;
    let total_days = total_hours / 24;
    let days = total_days % 30;  // Approximate month as 30 days
    let total_months = total_days / 30;
    let months = total_months % 12;
    let years = total_months / 12;

    let mut parts = Vec::new();
    let mut has_started = false;  // Track if we've added any non-zero component

    // Add years (4 digits with leading zeros if we have larger components)
    if years > 0 {
        parts.push(format!("{:04}y", years));
        has_started = true;
    }

    // Add months (2 digits with leading zeros if years present)
    if months > 0 || (has_started && (days > 0 || hours > 0 || mins > 0 || secs > 0 || millis > 0)) {
        if has_started {
            parts.push(format!("{:02}m", months));
        } else if months > 0 {
            parts.push(format!("{}m", months));
            has_started = true;
        }
    }

    // Add days (2 digits with leading zeros if months/years present)
    if days > 0 || (has_started && (hours > 0 || mins > 0 || secs > 0 || millis > 0)) {
        if has_started {
            parts.push(format!("{:02}d", days));
        } else if days > 0 {
            parts.push(format!("{}d", days));
            has_started = true;
        }
    }

    // Add hours (2 digits with leading zeros if larger components present)
    if hours > 0 || (has_started && (mins > 0 || secs > 0 || millis > 0)) {
        if has_started {
            parts.push(format!("{:02}h", hours));
        } else if hours > 0 {
            parts.push(format!("{}h", hours));
            has_started = true;
        }
    }

    // Add minutes (2 digits with leading zeros if larger components present)
    if mins > 0 || (has_started && (secs > 0 || millis > 0)) {
        if has_started {
            parts.push(format!("{:02}min", mins));
        } else if mins > 0 {
            parts.push(format!("{}min", mins));
            has_started = true;
        }
    }

    // Add seconds (2 digits with leading zeros if larger components present)
    if secs > 0 || (has_started && millis > 0) {
        if has_started {
            parts.push(format!("{:02}s", secs));
        } else if secs > 0 {
            parts.push(format!("{}s", secs));
            has_started = true;
        }
    }

    // Add milliseconds (3 digits with leading zeros if seconds present, always shown if nothing else)
    if millis > 0 || parts.is_empty() {
        if has_started {
            parts.push(format!("{:03}ms", millis));
        } else {
            parts.push(format!("{}ms", millis));
        }
    }

    let result = parts.join(" ");
    format!("{:>30}", result)  // Right-align in 30-character field
}

/// Custom deserializer for durations that accepts both human-readable strings and numeric values
///
/// Accepts formats like "1m 30s", "90s", or plain numbers representing seconds
pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a duration string or a number representing seconds")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as f64)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v as f64)
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_duration(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_any(DurationVisitor)
}
