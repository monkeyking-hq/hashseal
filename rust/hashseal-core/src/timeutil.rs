//! Tiny time helpers (no chrono).

use std::time::{SystemTime, UNIX_EPOCH};

/// RFC3339-ish UTC timestamp without external crates: `YYYY-MM-DDTHH:MM:SSZ`
pub fn utc_now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = civil_from_days(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

pub fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Simple id for reports (not a UUID crate).
pub fn bundle_id() -> String {
    let ms = unix_ms();
    let pid = std::process::id();
    format!("{ms:x}-{pid:x}")
}

// Howard Hinnant civil_from_days (public domain algorithm), UTC.
fn civil_from_days(unix_secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = (unix_secs / 86_400) as i64;
    let rem = (unix_secs % 86_400) as u32;
    let h = rem / 3600;
    let mi = (rem % 3600) / 60;
    let s = rem % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32, h, mi, s)
}
