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
    pub(crate) location: Url,
    pub(crate) modified: Option<OffsetDateTime>,
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

    /// Returns the given location.
    pub fn location(&self) -> &Url {
        &self.location
    }

    /// Returns the given modify timestamp.
    pub fn modified(&self) -> Option<&OffsetDateTime> {
        self.modified.as_ref()
    }
}

impl From<Url> for Index {
    fn from(location: Url) -> Self {
        Index::new(location)
    }
}
