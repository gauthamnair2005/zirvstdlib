//! Civil time conversion and a monotonic millisecond clock.
//!
//! The kernel updates [`set_monotonic_ms`] from the PIT.  Calendar time is
//! derived from the CMOS RTC at boot and advanced by the monotonic clock.

use core::sync::atomic::{AtomicU64, Ordering};

static MONOTONIC_MS: AtomicU64 = AtomicU64::new(0);

/// Set the monotonic clock.  Called from the timer interrupt.
pub fn set_monotonic_ms(value: u64) {
    MONOTONIC_MS.store(value, Ordering::Relaxed);
}

/// Current monotonic time in milliseconds since boot.
pub fn monotonic_ms() -> u64 {
    MONOTONIC_MS.load(Ordering::Relaxed)
}

/// A civil date and time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    pub const fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Convert a Unix timestamp (seconds since 1970-01-01 UTC) to civil time.
    pub fn from_unix(seconds: i64) -> Self {
        let days = seconds.div_euclid(86_400);
        let seconds_of_day = seconds.rem_euclid(86_400) as u32;
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            hour: (seconds_of_day / 3_600) as u8,
            minute: ((seconds_of_day / 60) % 60) as u8,
            second: (seconds_of_day % 60) as u8,
        }
    }

    /// Convert this civil time to a Unix timestamp.
    pub fn to_unix(self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 86_400
            + self.hour as i64 * 3_600
            + self.minute as i64 * 60
            + self.second as i64
    }

    /// ISO weekday, where Monday is 1 and Sunday is 7.
    pub fn weekday(self) -> u8 {
        let days = days_from_civil(self.year, self.month, self.day);
        // 1970-01-01 was a Thursday (4).
        (((days + 3).rem_euclid(7)) + 1) as u8
    }

    pub fn weekday_name(self) -> &'static str {
        const NAMES: [&str; 7] = [
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];
        NAMES[(self.weekday() - 1) as usize]
    }

    pub fn month_name(self) -> &'static str {
        const NAMES: [&str; 12] = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        NAMES[(self.month.clamp(1, 12) - 1) as usize]
    }
}

/// Number of days in a month, accounting for leap years.
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days since 1970-01-01 from a civil date (Howard Hinnant's algorithm).
pub const fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}

/// Civil date from days since 1970-01-01.
pub const fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    (
        (year + if month <= 2 { 1 } else { 0 }) as i32,
        month as u8,
        day as u8,
    )
}

/// Format as `HH:MM:SS` into a fixed buffer.
pub fn format_time(time: DateTime) -> (usize, [u8; 8]) {
    let mut buffer = [0u8; 8];
    buffer[0] = b'0' + time.hour / 10;
    buffer[1] = b'0' + time.hour % 10;
    buffer[2] = b':';
    buffer[3] = b'0' + time.minute / 10;
    buffer[4] = b'0' + time.minute % 10;
    buffer[5] = b':';
    buffer[6] = b'0' + time.second / 10;
    buffer[7] = b'0' + time.second % 10;
    (8, buffer)
}

/// Format as `YYYY-MM-DD` into a fixed buffer.
pub fn format_date(time: DateTime) -> (usize, [u8; 10]) {
    let mut buffer = [0u8; 10];
    let mut year = time.year;
    if year < 0 {
        year = 0;
    }
    buffer[0] = b'0' + ((year / 1000) % 10) as u8;
    buffer[1] = b'0' + ((year / 100) % 10) as u8;
    buffer[2] = b'0' + ((year / 10) % 10) as u8;
    buffer[3] = b'0' + (year % 10) as u8;
    buffer[4] = b'-';
    buffer[5] = b'0' + time.month / 10;
    buffer[6] = b'0' + time.month % 10;
    buffer[7] = b'-';
    buffer[8] = b'0' + time.day / 10;
    buffer[9] = b'0' + time.day % 10;
    (10, buffer)
}

/// Format as `HH:MM` into a fixed buffer.
pub fn format_hm(time: DateTime) -> (usize, [u8; 5]) {
    let mut buffer = [0u8; 5];
    buffer[0] = b'0' + time.hour / 10;
    buffer[1] = b'0' + time.hour % 10;
    buffer[2] = b':';
    buffer[3] = b'0' + time.minute / 10;
    buffer[4] = b'0' + time.minute % 10;
    (5, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch() {
        let time = DateTime::from_unix(0);
        assert_eq!((time.year, time.month, time.day), (1970, 1, 1));
        assert_eq!(time.weekday(), 4);
        assert_eq!(time.to_unix(), 0);
    }

    #[test]
    fn round_trip() {
        let time = DateTime::new(2026, 10, 23, 18, 30, 15);
        assert_eq!(DateTime::from_unix(time.to_unix()), time);
    }

    #[test]
    fn leap_years() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2024));
        assert_eq!(days_in_month(2024, 2), 29);
    }
}
