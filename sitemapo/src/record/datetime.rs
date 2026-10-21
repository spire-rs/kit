//! The W3C Datetime format of sitemap dates.

use jiff::civil::{Date, DateTime};
use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};

/// Parses a [W3C Datetime] value: a year, a month, a date, or a date with a
/// time in minutes, seconds or fractions of a second.
///
/// Values without a time zone, which the format does not allow but sitemaps
/// often have, are read as UTC, as are the dates without a time.
///
/// [W3C Datetime]: https://www.w3.org/TR/NOTE-datetime
pub fn parse(value: &str) -> Option<Timestamp> {
    let value = value.trim();
    if let Ok(timestamp) = value.parse::<Timestamp>() {
        return Some(timestamp);
    }

    let datetime = match value.len() {
        4 => format!("{value}-01-01").parse::<Date>().ok()?.into(),
        7 => format!("{value}-01").parse::<Date>().ok()?.into(),
        _ => value.parse::<DateTime>().ok()?,
    };

    datetime
        .to_zoned(TimeZone::UTC)
        .ok()
        .as_ref()
        .map(Zoned::timestamp)
}

#[cfg(test)]
mod test {
    use super::parse;

    fn parsed(value: &str) -> Option<String> {
        parse(value).map(|timestamp| timestamp.to_string())
    }

    #[test]
    fn precision() {
        assert_eq!(parsed("2024").as_deref(), Some("2024-01-01T00:00:00Z"));
        assert_eq!(parsed("2024-03").as_deref(), Some("2024-03-01T00:00:00Z"));
        assert_eq!(
            parsed("2024-03-04").as_deref(),
            Some("2024-03-04T00:00:00Z")
        );
        assert_eq!(
            parsed("2024-03-04T05:06+01:00").as_deref(),
            Some("2024-03-04T04:06:00Z")
        );
        assert_eq!(
            parsed("2024-03-04T05:06:07Z").as_deref(),
            Some("2024-03-04T05:06:07Z")
        );
        assert_eq!(
            parsed(" 2024-03-04T05:06:07.5-02:00 ").as_deref(),
            Some("2024-03-04T07:06:07.5Z")
        );
    }

    #[test]
    fn without_time_zone() {
        assert_eq!(
            parsed("2024-03-04T05:06:07").as_deref(),
            Some("2024-03-04T05:06:07Z")
        );
    }

    #[test]
    fn invalid() {
        for value in ["", "today", "2024-13", "2024-02-30", "24-03-04"] {
            assert_eq!(parsed(value), None, "{value}");
        }
    }
}
