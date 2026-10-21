use std::fmt;
use std::num::ParseFloatError;
use std::str::FromStr;

/// [`Priority`] parsing error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PriorityError {
    /// The value is not a number.
    #[error("not a number: {0}")]
    Parse(#[from] ParseFloatError),
    /// The value is outside of the `0.0..=1.0` range.
    #[error("not within the 0.0..=1.0 range")]
    Range,
}

/// The priority of the page relative to other pages of the same site, from
/// `0.0` to `1.0`.
///
/// It only tells crawlers which pages matter most to the site, and does not
/// affect how the pages compare to the pages of other sites.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Priority(f32);

impl Priority {
    /// The default priority of the pages without one.
    pub const AVG: Self = Self(0.5);
    /// The highest priority.
    pub const MAX: Self = Self(1.0);
    /// The lowest priority.
    pub const MIN: Self = Self(0.0);

    /// Creates the priority from the value within the `0.0..=1.0` range.
    ///
    /// # Errors
    ///
    /// Returns [`PriorityError::Range`] if the value is outside of the range.
    ///
    /// ```rust
    /// use sitemapo::record::Priority;
    ///
    /// assert_eq!(Priority::new(0.6).unwrap().get(), 0.6);
    /// assert!(Priority::new(1.5).is_err());
    /// ```
    pub fn new(priority: f32) -> Result<Self, PriorityError> {
        if (0.0..=1.0).contains(&priority) {
            Ok(Self(priority))
        } else {
            Err(PriorityError::Range)
        }
    }

    /// Creates the priority from any value, clamped to the `0.0..=1.0` range.
    ///
    /// ```rust
    /// use sitemapo::record::Priority;
    ///
    /// assert_eq!(Priority::clamped(2.6), Priority::MAX);
    /// ```
    #[must_use]
    pub fn clamped(priority: f32) -> Self {
        if priority.is_nan() {
            Self::AVG
        } else {
            Self(priority.clamp(0.0, 1.0))
        }
    }

    /// Returns the value.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

impl Default for Priority {
    fn default() -> Self {
        Self::AVG
    }
}

impl FromStr for Priority {
    type Err = PriorityError;

    fn from_str(priority: &str) -> Result<Self, Self::Err> {
        Self::new(priority.trim().parse()?)
    }
}

impl fmt::Display for Priority {
    /// Writes the shortest decimal that parses back into the same value.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::Priority;

    #[test]
    fn display() {
        for (priority, text) in [(0.0, "0.0"), (0.25, "0.25"), (0.6, "0.6"), (1.0, "1.0")] {
            assert_eq!(Priority::new(priority).unwrap().to_string(), text);
        }
    }

    #[test]
    fn parse() {
        assert_eq!(" 0.8 ".parse(), Ok(Priority::new(0.8).unwrap()));
        assert!("2".parse::<Priority>().is_err());
        assert!("NaN".parse::<Priority>().is_err());
        assert!("high".parse::<Priority>().is_err());
    }
}
