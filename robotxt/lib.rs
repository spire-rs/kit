#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("./README.md")]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

// Re-exports
pub use url;

#[cfg(feature = "builder")]
pub use build::{GroupBuilder, RobotsBuilder};
#[cfg(feature = "parser")]
pub use parse::{AccessResult, Robots, ALL_UAS};
pub use paths::{create_url, BYTE_LIMIT};

/// Unrecoverable failure during `robots.txt` building or parsing.
///
/// This may be extended in the future so exhaustive matching is discouraged.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Unable to create the expected path to the `robots.txt` file:
    /// cannot be a base url.
    #[error("cannot be a base url")]
    CannotBeBase,

    /// Unable to create the expected path to the `robots.txt` file:
    /// unexpected address scheme, expected `http` or `https`.
    #[error("addr scheme: `{scheme}`, expected `http` or `https`")]
    WrongScheme { scheme: String },

    /// Unable to create the expected path to the `robots.txt` file:
    /// unexpected parsing error.
    // TODO: Remove url::ParseError.
    #[error("url parsing error: {0}")]
    Url(#[from] url::ParseError),
}

/// A specialized [`Result`] type for [`robotxt`] operations.
///
/// [`Result`]: std::result::Result
/// [`robotxt`]: crate
pub type Result<T, E = Error> = std::result::Result<T, E>;

mod paths;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
mod build;

#[cfg(feature = "parser")]
#[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
mod parse;

#[doc(hidden)]
pub mod prelude {
    #[cfg(feature = "builder")]
    pub use super::build::{GroupBuilder, RobotsBuilder};
    #[cfg(feature = "parser")]
    pub use super::parse::{AccessResult, Robots, ALL_UAS};
    pub use super::paths::{create_url, BYTE_LIMIT};
    pub use super::{Error, Result};
}
