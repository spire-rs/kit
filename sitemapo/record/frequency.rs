use std::fmt;

use thiserror::Error;
use time::{ext::NumericalDuration, OffsetDateTime};

/// [Frequency] parsing error.
#[derive(Debug, Error)]
#[error("not a valid change frequency variant")]
pub struct FrequencyError;

/// Used to specify how frequently the page is likely to change.
///
/// This value provides general information to search engines and
/// may not correlate exactly to how often they crawl the page.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frequency {
    /// Describes documents that change each time they are accessed.
    Always,
    /// Describes documents that change every hour.
    Hourly,
    /// Describes documents that change every day.
    Daily,
    /// Describes documents that change every week (i.e. 7 days).
    Weekly,
    /// Describes documents that change every month (i.e. same day each month).
    Monthly,
    /// Describes documents that change every year (i.e. 12 months).
    Yearly,
    /// Describes archived documents.
    Never,
}

impl Frequency {
    /// Tries to parse the string into the valid change frequency value.
    ///
    /// ``` rust
    /// use sitemapo::record::Frequency;
    ///
    /// let frequency = Frequency::parse("Daily");
    /// assert_eq!(frequency.unwrap(), Frequency::Daily);
    /// ```
    pub fn parse(frequency: &str) -> Result<Self, FrequencyError> {
        match frequency.trim().to_lowercase().as_str() {
            "always" => Ok(Self::Always),
            "hourly" => Ok(Self::Hourly),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "yearly" => Ok(Self::Yearly),
            "never" => Ok(Self::Never),
            _ => Err(FrequencyError),
        }
    }

    /// Calculates the date when the entry becomes outdated.
    /// TODO: Fix months and years.
    ///
    /// ```rust
    /// use time::macros::datetime;
    /// use sitemapo::record::Frequency;
    ///
    /// let d0 = datetime!(2022-09-12 12:00 UTC);
    /// let rs = Frequency::Monthly.next_date(d0);
    /// assert_eq!(rs.unwrap(), datetime!(2022-10-12 12:00 UTC))
    /// ```
    #[must_use]
    pub fn next_date(&self, date: OffsetDateTime) -> Option<OffsetDateTime> {
        match &self {
            Self::Always | Self::Never => None,
            Self::Hourly => Some(date + 1.hours()),
            Self::Daily => Some(date + 1.days()),
            Self::Weekly => Some(date + 7.days()),
            Self::Monthly => Some(date + 30.days()),
            Self::Yearly => Some(date + 365.days()),
        }
    }

    /// Calculates if the entry is currently outdated.
    ///
    /// ```rust
    /// use time::macros::datetime;
    /// use sitemapo::record::Frequency;
    ///
    /// let d0 = datetime!(2022-09-12 12:00 UTC);
    /// let d1 = datetime!(2022-10-12 12:00 UTC);
    /// assert!(Frequency::Monthly.is_outdated(d0, d1));
    /// ```
    #[must_use]
    pub fn is_outdated(&self, date: OffsetDateTime, now: OffsetDateTime) -> bool {
        match &self {
            Self::Always => true,
            Self::Never => false,
            _ => self
                .next_date(date)
                .map_or_else(|| unreachable!(), |x| x <= now),
        }
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = match self {
            Self::Always => "always",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
            Self::Never => "never",
        };

        fmt::Display::fmt(inner, f)
    }
}

impl TryFrom<&str> for Frequency {
    type Error = FrequencyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}
