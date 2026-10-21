use std::fmt;
use std::str::FromStr;

use jiff::tz::TimeZone;
use jiff::{Span, Timestamp, ToSpan};

/// [`Frequency`] parsing error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a valid change frequency")]
pub struct FrequencyError;

/// How often the page is likely to change.
///
/// It is a hint for crawlers, which may crawl the page more or less often.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// The page changes every time it is accessed.
    Always,
    /// The page changes every hour.
    Hourly,
    /// The page changes every day.
    Daily,
    /// The page changes every week.
    Weekly,
    /// The page changes every month.
    Monthly,
    /// The page changes every year.
    Yearly,
    /// The page is archived and does not change.
    Never,
}

impl Frequency {
    /// Returns the value of the `changefreq` element.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
            Self::Never => "never",
        }
    }

    /// Returns the time the page is next expected to change after it changed
    /// at `modified`, in calendar months and years for the longer periods,
    /// or `None` if it never changes.
    ///
    /// ```rust
    /// use sitemapo::record::Frequency;
    ///
    /// let modified = "2024-01-31T12:00:00Z".parse().unwrap();
    /// let next = Frequency::Monthly.next_change(modified).unwrap();
    /// assert_eq!(next.to_string(), "2024-02-29T12:00:00Z");
    /// ```
    #[must_use]
    pub fn next_change(self, modified: Timestamp) -> Option<Timestamp> {
        let period: Span = match self {
            Self::Always => return Some(modified),
            Self::Never => return None,
            Self::Hourly => 1.hour(),
            Self::Daily => 1.day(),
            Self::Weekly => 1.week(),
            Self::Monthly => 1.month(),
            Self::Yearly => 1.year(),
        };

        let modified = modified.to_zoned(TimeZone::UTC);
        modified
            .checked_add(period)
            .ok()
            .map(|next| next.timestamp())
    }
}

impl FromStr for Frequency {
    type Err = FrequencyError;

    /// Parses the value of the `changefreq` element, ignoring case.
    fn from_str(frequency: &str) -> Result<Self, Self::Err> {
        let frequencies = [
            Self::Always,
            Self::Hourly,
            Self::Daily,
            Self::Weekly,
            Self::Monthly,
            Self::Yearly,
            Self::Never,
        ];

        let frequency = frequency.trim();
        frequencies
            .into_iter()
            .find(|known| known.as_str().eq_ignore_ascii_case(frequency))
            .ok_or(FrequencyError)
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::Frequency;

    #[test]
    fn parse() {
        assert_eq!(" Daily ".parse(), Ok(Frequency::Daily));
        assert!("sometimes".parse::<Frequency>().is_err());
    }

    #[test]
    fn next_change() {
        let modified = "2023-02-28T00:00:00Z".parse().unwrap();
        let next = |frequency: Frequency| frequency.next_change(modified).map(|t| t.to_string());
        assert_eq!(
            next(Frequency::Always).as_deref(),
            Some("2023-02-28T00:00:00Z")
        );
        assert_eq!(
            next(Frequency::Weekly).as_deref(),
            Some("2023-03-07T00:00:00Z")
        );
        assert_eq!(
            next(Frequency::Yearly).as_deref(),
            Some("2024-02-28T00:00:00Z")
        );
        assert_eq!(next(Frequency::Never), None);
    }
}
