use std::fmt;
use std::num::ParseFloatError;

use thiserror::Error;

/// [Priority] parsing error.
#[derive(Debug, Error)]
pub enum PriorityError {
    #[error("parse float error: {0}")]
    Parse(#[from] ParseFloatError),
    #[error("range error: [0.0, 1.0]")]
    Range,
}

/// Used to specify the priority of this URL relative to other URLs on your site.
///
/// Valid values range from 0.0 to 1.0. This value does not affect how your
/// pages are compared to pages on other sites. It only lets the search engines
/// know which pages you deem most important for the crawlers.
#[derive(Debug, Clone, PartialEq)]
pub struct Priority(f32);

impl Priority {
    /// Creates the priority from the valid underlying value.
    ///
    /// ```rust
    /// use sitemapo::record::Priority;
    ///
    /// let frequency = Priority::new(0.6f32).unwrap();
    /// assert_eq!(frequency.as_inner(), 0.6);
    /// ```
    pub fn new(priority: f32) -> Result<Self, PriorityError> {
        match priority {
            x if (0.0..=1.0).contains(&priority) => Ok(Self(x)),
            _ => Err(PriorityError::Range),
        }
    }

    /// Creates the priority from any underlying value by
    /// clamping the input into the acceptable range.
    ///
    /// ```rust
    /// use sitemapo::record::Priority;
    ///
    /// let frequency = Priority::new_fallback(2.6f32);
    /// assert_eq!(frequency.as_inner(), 1.0);
    /// ```
    pub fn new_fallback(priority: f32) -> Self {
        Self(priority.max(0.0).min(1.0))
    }

    /// Tries to parse the string into the valid priority value.
    ///
    /// ```rust
    /// use sitemapo::record::Priority;
    ///
    /// let frequency = Priority::parse("0.6").unwrap();
    /// assert_eq!(frequency.as_inner(), 0.6);
    /// ```
    pub fn parse(priority: &str) -> Result<Self, PriorityError> {
        let priority = priority.parse::<f32>()?;
        Self::new(priority)
    }

    /// Returns the internal value.
    pub fn as_inner(&self) -> f32 {
        self.0
    }

    /// Default (or average) priority value.
    pub const AVG: Self = Self(0.5);

    /// Minimal priority value.
    pub const MIN: Self = Self(0.0);

    /// Maximal priority value.
    pub const MAX: Self = Self(1.0);
}

impl Default for Priority {
    fn default() -> Self {
        Self::AVG.clone()
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(format!("{:.1}", self.0).as_str(), f)
    }
}

impl TryFrom<&str> for Priority {
    type Error = PriorityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}
