mod entry;
mod frequency;
mod index;
mod priority;

pub use entry::*;
pub use frequency::*;
pub use index::*;
pub use priority::*;

/// All formats limit a single sitemap to 50,000 URLs.
/// See [Build and submit a Sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap#sitemap-best-practices).
pub const RECORD_LIMIT: usize = 50_000;

/// All formats limit a single sitemap to 50MB (uncompressed).
/// See [Build and submit a Sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap#sitemap-best-practices).
pub const BYTE_LIMIT: usize = 52_428_800;

/// De facto limit is of 2000 characters, but browsers (e.g. Chrome) support longer anchors.
///
/// Used to prevent the missing newline vulnerability in text sitemaps.
pub const URL_LEN_LIMIT: usize = 65_536;

pub(crate) const LOCATION: &str = "loc";
pub(crate) const LAST_MODIFIED: &str = "lastmod";
pub(crate) const CHANGE_FREQUENCY: &str = "changefreq";
pub(crate) const PRIORITY: &str = "priority";

pub(crate) const URL_SET: &str = "urlset";
pub(crate) const URL: &str = "url";

pub(crate) const SITEMAP_INDEX: &str = "sitemapindex";
pub(crate) const SITEMAP: &str = "sitemap";
