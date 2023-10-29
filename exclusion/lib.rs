#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("./README.md")]

/// Unrecoverable failure during `robots.txt` building or parsing.
///
/// This may be extended in the future so exhaustive matching is discouraged.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Unable to create the expected path to the `robots.txt` file:
    /// cannot be a base url.
    #[error("cannot be a base url")]
    CannotBeBase,

    /// Unable to create the expected path to the `robots.txt` file:
    /// unexpected address scheme, expected `http` or `https`.
    #[error("unexpected addr scheme: `{scheme}`, expected `http` or `https`")]
    WrongScheme { scheme: String },

    /// Unable to create the expected path to the `robots.txt` file:
    /// unexpected parsing error.
    #[error("unexpected parsing error: {0}")]
    Url(#[from] url::ParseError),
}

/// A specialized [`Result`] type for [`robotxt`] operations.
///
/// [`Result`]: std::result::Result
/// [`robotxt`]: crate
pub type Result<T> = std::result::Result<T, Error>;

mod paths;
pub use paths::*;

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
mod build;
#[cfg(feature = "builder")]
pub use build::*;

#[cfg(feature = "parser")]
#[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
mod parse;
#[cfg(feature = "parser")]
pub use parse::*;

// Re-exports
pub use url;

#[doc(hidden)]
pub mod prelude {
    pub use super::paths::*;
    pub use super::{Error, Result};

    #[cfg(feature = "builder")]
    pub use super::build::*;
    #[cfg(feature = "parser")]
    pub use super::parse::*;
}
