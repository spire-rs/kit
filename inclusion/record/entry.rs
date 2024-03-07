use time::OffsetDateTime;
use url::Url;

use crate::record::{Frequency, Priority};

/// Represents a single record in the Text or XML sitemap.
///
/// ```rust
/// use time::macros::datetime;
/// use url::Url;
/// use sitemapo::record::*;
///
/// let _ = Entry::new(Url::parse("https://example.com/").unwrap())
///     .with_modified(datetime!(2020-01-01 0:00 UTC))
///     .with_priority(Priority::MAX)
///     .with_frequency(Frequency::Daily);
/// ```
#[derive(Debug, Clone)]
pub struct Entry {
    pub location: Url,
    pub modified: Option<OffsetDateTime>,
    pub priority: Option<Priority>,
    pub frequency: Option<Frequency>,
}

impl Entry {
    /// Creates a new instance with the given location.
    pub fn new(location: Url) -> Self {
        Self {
            location,
            modified: None,
            priority: None,
            frequency: None,
        }
    }

    /// Creates a new record with the given modify timestamp.
    pub fn with_modified(mut self, modified: OffsetDateTime) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Creates a new record with the given priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Creates a new record with the given change frequency.
    pub fn with_frequency(mut self, frequency: Frequency) -> Self {
        self.frequency = Some(frequency);
        self
    }
}

impl From<Url> for Entry {
    fn from(location: Url) -> Self {
        Entry::new(location)
    }
}
