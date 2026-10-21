//! Error and result types.

/// Failure to derive the `robots.txt` location from a URL.
///
/// See [`robots_url`].
///
/// [`robots_url`]: crate::robots_url
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The URL cannot be a base, e.g. `mailto:` or `data:` addresses.
    #[error("url cannot be a base")]
    CannotBeBase,

    /// The URL does not have a host.
    #[error("url does not have a host")]
    NoHost,

    /// The URL scheme is neither `http` nor `https`.
    #[error("unsupported scheme `{scheme}`, expected `http` or `https`")]
    UnsupportedScheme {
        /// The scheme of the provided URL.
        scheme: String,
    },
}

/// A specialized [`Result`] type for [`robotxt`] operations.
///
/// [`Result`]: std::result::Result
/// [`robotxt`]: crate
pub type Result<T, E = Error> = std::result::Result<T, E>;
