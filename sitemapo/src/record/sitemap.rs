use jiff::Timestamp;
use url::Url;

/// A sitemap listed in an XML `sitemapindex`.
///
/// ```rust
/// use sitemapo::record::Sitemap;
///
/// let location = "https://example.com/sitemap.xml".parse().unwrap();
/// let sitemap = Sitemap::new(location).with_modified("2024-01-01T00:00:00Z".parse().unwrap());
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sitemap {
    /// The URL of the sitemap.
    pub location: Url,
    /// The time the sitemap was last modified.
    pub modified: Option<Timestamp>,
}

impl Sitemap {
    /// Creates the record of the sitemap.
    pub const fn new(location: Url) -> Self {
        Self {
            location,
            modified: None,
        }
    }

    /// Sets the time the sitemap was last modified.
    pub const fn with_modified(mut self, modified: Timestamp) -> Self {
        self.modified = Some(modified);
        self
    }
}

impl From<Url> for Sitemap {
    fn from(location: Url) -> Self {
        Self::new(location)
    }
}
