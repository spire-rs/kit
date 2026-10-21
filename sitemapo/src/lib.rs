#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

// Re-exports
pub use build::{Builder, EntryBuilder, IndexBuilder, TextBuilder};
pub use error::{Error, Result};
pub use jiff;
pub use parse::{Format, Parser};
pub use record::{BYTE_LIMIT, RECORD_LIMIT, Record};
pub use url;

mod error;
mod limit;

/// Builders of text sitemaps, XML sitemaps and XML sitemap indexes.
pub mod build;
/// The parser of every sitemap format.
pub mod parse;
/// Sitemap records and their attributes.
pub mod record;

#[doc(hidden)]
pub mod prelude;
