//! Error and result types.

use std::io;

use crate::record::{BYTE_LIMIT, RECORD_LIMIT};

/// Failure to parse or build a sitemap.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The sitemap has more than [`RECORD_LIMIT`] records.
    #[error("sitemap has more than {} records", RECORD_LIMIT)]
    RecordLimit,

    /// The sitemap is larger than [`BYTE_LIMIT`] bytes.
    #[error("sitemap is larger than {} bytes", BYTE_LIMIT)]
    ByteLimit,

    /// The XML document is neither a `urlset` nor a `sitemapindex`.
    #[error("unexpected root element `{name}`, expected `urlset` or `sitemapindex`")]
    UnknownRoot {
        /// The local name of the root element.
        name: String,
    },

    /// The underlying reader or writer failed.
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    /// The XML document is malformed.
    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),
}

/// A specialized [`Result`] type for [`sitemapo`] operations.
///
/// [`Result`]: std::result::Result
/// [`sitemapo`]: crate
pub type Result<T, E = Error> = std::result::Result<T, E>;
