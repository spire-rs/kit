use std::fmt::{Display, Formatter};

use thiserror::Error;
use time::{ext::NumericalDuration, OffsetDateTime};
use timext::ext::NumericCalendarDuration;

/// [Frequency] parsing error.
#[derive(Debug, Error)]
#[error("not a valid change frequency variant")]
pub struct FrequencyError;

/// Used to specify how frequently the page is likely to change.
///
/// This value provides general information to search engines and
/// may not correlate exactly to how often they crawl the page.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let frequency = frequency.trim().to_lowercase();

        use Frequency::*;
        match frequency.as_str() {
            "always" => Ok(Always),
            "hourly" => Ok(Hourly),
            "daily" => Ok(Daily),
            "weekly" => Ok(Weekly),
            "monthly" => Ok(Monthly),
            "yearly" => Ok(Yearly),
            "never" => Ok(Never),
            _ => Err(FrequencyError),
        }
    }

    /// Calculates the date when the entry becomes outdated.
    ///
    /// ```rust
    /// use time::macros::datetime;
    /// use sitemapo::record::Frequency;
    ///
    /// let d0 = datetime!(2022-09-12 12:00 UTC);
    /// let rs = Frequency::Monthly.next_date(d0);
    /// assert_eq!(rs.unwrap(), datetime!(2022-10-12 12:00 UTC))
    /// ```
    pub fn next_date(&self, date: OffsetDateTime) -> Option<OffsetDateTime> {
        use Frequency::*;
        match &self {
            Always | Never => None,
            Hourly => Some(date + 1.hours()),
            Daily => Some(date + 1.days()),
            Weekly => Some(date + 7.days()),
            Monthly => Some(date + 1.months()),
            Yearly => Some(date + 1.years()),
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
    pub fn is_outdated(&self, date: OffsetDateTime, now: OffsetDateTime) -> bool {
        match &self {
            Self::Always => true,
            Self::Never => false,
            _ => match self.next_date(date) {
                Some(next) => next <= now,
                None => unreachable!(),
            },
        }
    }
}

impl Display for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let underlying = match self {
            Self::Always => "always",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
            Self::Never => "never",
        };

        Display::fmt(underlying, f)
    }
}

impl TryFrom<&str> for Frequency {
    type Error = FrequencyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}
