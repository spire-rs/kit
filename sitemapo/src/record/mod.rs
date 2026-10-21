pub use alternate::Alternate;
pub use entry::Entry;
pub use frequency::{Frequency, FrequencyError};
pub use image::Image;
pub use news::{News, Publication};
pub use priority::{Priority, PriorityError};
pub use sitemap::Sitemap;
pub use video::{Platform, Platforms, Relationship, Restriction, Uploader, Video};

mod alternate;
mod entry;
mod frequency;
mod image;
mod news;
mod priority;
mod sitemap;
mod video;

pub(crate) mod datetime;

/// The maximum number of records of a single sitemap.
///
/// See [Build and submit a sitemap] on Google Search Central.
///
/// [Build and submit a sitemap]: https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap#sitemap-best-practices
pub const RECORD_LIMIT: usize = 50_000;

/// The maximum size of a single (uncompressed) sitemap: 50 MiB.
///
/// See [Build and submit a sitemap] on Google Search Central.
///
/// [Build and submit a sitemap]: https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap#sitemap-best-practices
pub const BYTE_LIMIT: usize = 50 * 1024 * 1024;

/// A single record of a sitemap.
#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    /// A page of a text sitemap or of an XML `urlset`.
    Entry(Entry),
    /// A sitemap listed in an XML `sitemapindex`.
    Sitemap(Sitemap),
}

impl From<Entry> for Record {
    fn from(entry: Entry) -> Self {
        Self::Entry(entry)
    }
}

impl From<Sitemap> for Record {
    fn from(sitemap: Sitemap) -> Self {
        Self::Sitemap(sitemap)
    }
}

/// XML element and namespace names.
pub(crate) mod names {
    #[cfg(test)]
    pub const SITEMAP_NS: &str = "http://www.sitemaps.org/schemas/sitemap/0.9";
    pub const IMAGE_NS: &str = "http://www.google.com/schemas/sitemap-image/1.1";
    pub const VIDEO_NS: &str = "http://www.google.com/schemas/sitemap-video/1.1";
    pub const NEWS_NS: &str = "http://www.google.com/schemas/sitemap-news/0.9";
    pub const XHTML_NS: &str = "http://www.w3.org/1999/xhtml";

    pub const URL_SET: &str = "urlset";
    pub const URL: &str = "url";
    pub const SITEMAP_INDEX: &str = "sitemapindex";
    pub const SITEMAP: &str = "sitemap";

    pub const LOCATION: &str = "loc";
    pub const LAST_MODIFIED: &str = "lastmod";
    pub const CHANGE_FREQUENCY: &str = "changefreq";
    pub const PRIORITY: &str = "priority";
}
