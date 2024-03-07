#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("./README.md")]

/// Unrecoverable failure during `sitemap.xml` building or parsing.
///
/// This may be extended in the future so exhaustive matching is discouraged.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Parsers/builders enforce byte limit.
    /// See [`BYTE_LIMIT`].
    ///
    /// [`BYTE_LIMIT`]: record::BYTE_LIMIT
    #[error("too many records: {over} entries over limit")]
    EntryLimit { over: usize },

    /// Parsers/builders enforce byte limit.
    /// See [`RECORD_LIMIT`].
    ///
    /// [`RECORD_LIMIT`]: record::RECORD_LIMIT
    #[error("too many bytes: {over} bytes over limit")]
    ByteLimit { over: usize },

    /// Underlying reader/writer IO failure.
    /// See [`std::io::Error`].
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parser/builder failure.
    /// See [`quick_xml::Error`].
    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),
}

/// A specialized [`Result`] type for [`sitemapo`] operations.
///
/// [`Result`]: std::result::Result
/// [`sitemapo`]: crate
pub type Result<T> = std::result::Result<T, Error>;

// Re-exports
pub use url;

/// Builder types: `AutoBuilder`, `TxtBuilder` & `XmlBuilder`.
pub mod build;
/// Parser types: `AutoParser`, `TxtParser` & `XmlParser`.
pub mod parse;
/// Record and attribute types.
pub mod record;

#[doc(hidden)]
pub mod prelude {
    pub use super::{Error, Result};

    pub use super::build::*;
    pub use super::parse::*;
    pub use super::record::*;
}
