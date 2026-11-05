//! UTC wall-clock times in the form the result file uses: `2026-11-04T21:37:06.123Z`.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The current UTC time with milliseconds.
pub fn now() -> String {
    format_system_time(SystemTime::now())
}

/// Formats a system time as UTC with milliseconds.
pub fn format_system_time(t: SystemTime) -> String {
    let since = t.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    format_unix_millis(since.as_millis() as i64)
}

/// Formats milliseconds since the Unix epoch as UTC with milliseconds.
pub fn format_unix_millis(ms: i64) -> String {
    let (y, mo, d, h, mi, s, milli) = civil(ms);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{milli:03}Z")
}

/// A compact UTC stamp `YYYYMMDD-HHMMSS`, used in session and run ids.
pub fn stamp(t: SystemTime) -> String {
    let ms = t.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis() as i64;
    let (y, mo, d, h, mi, s, _) = civil(ms);
    format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}")
}

/// Splits Unix milliseconds into the UTC calendar date and time (Howard Hinnant's
/// days-from-civil algorithm, inverted).
fn civil(ms: i64) -> (i64, u32, u32, u32, u32, u32, u32) {
    let secs = ms.div_euclid(1000);
    let milli = ms.rem_euclid(1000) as u32;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (h, mi, s) = ((sod / 3600) as u32, ((sod % 3600) / 60) as u32, (sod % 60) as u32);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(mo <= 2);
    (y, mo, d, h, mi, s, milli)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_epoch_and_a_known_time() {
        assert_eq!(format_unix_millis(0), "1970-01-01T00:00:00.000Z");
        // 2026-11-04T21:37:06.123Z
        assert_eq!(format_unix_millis(1_793_828_226_123), "2026-11-04T21:37:06.123Z");
    }

    #[test]
    fn handles_a_leap_day() {
        // 2024-02-29T12:00:00.000Z
        assert_eq!(format_unix_millis(1_709_208_000_000), "2024-02-29T12:00:00.000Z");
    }

    #[test]
    fn stamp_has_the_id_form() {
        let t = UNIX_EPOCH + Duration::from_millis(1_793_828_226_123);
        assert_eq!(stamp(t), "20261104-213706");
    }
}
