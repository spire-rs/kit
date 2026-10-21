#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// The README examples use both the parser and the builder.
#![cfg_attr(all(feature = "builder", feature = "parser"), doc = include_str!("../README.md"))]

// Re-exports
#[cfg(feature = "builder")]
pub use build::{GroupBuilder, RobotsBuilder};
pub use error::{Error, Result};
pub use location::robots_url;
#[cfg(feature = "parser")]
pub use parse::{AccessResult, Agent, BYTE_LIMIT, Robots, Rule};
pub use url;

mod error;
mod location;

#[cfg(any(feature = "builder", feature = "parser"))]
mod path;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
mod build;

#[cfg(feature = "parser")]
#[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
mod parse;

#[doc(hidden)]
pub mod prelude;

/// The `User-Agent` value of the group that applies to every crawler
/// without a group of its own.
pub const ANY_AGENT: &str = "*";
