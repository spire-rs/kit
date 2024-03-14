use time::OffsetDateTime;
use url::Url;

/// Represents a single record in the XML sitemap index.
///
/// ```rust
/// use time::macros::datetime;
/// use url::Url;
/// use sitemapo::record::*;
///
/// let _ = Index::new(Url::parse("https://example.com/").unwrap())
///     .with_modified(datetime!(2020-01-01 0:00 UTC));
/// ```
#[derive(Debug, Clone)]
pub struct Index {
    pub location: Url,
    pub modified: Option<OffsetDateTime>,
}

impl Index {
    /// Creates a new record with the given location.
    pub fn new(location: Url) -> Self {
        Self {
            location,
            modified: None,
        }
    }

    /// Creates a new record with the given modify timestamp.
    pub fn with_modified(self, modified: OffsetDateTime) -> Self {
        Self {
            modified: Some(modified),
            ..self
        }
    }
}

impl From<Url> for Index {
    fn from(location: Url) -> Self {
        Index::new(location)
    }
}
