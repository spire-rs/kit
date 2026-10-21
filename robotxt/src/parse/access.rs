//! The result of fetching a `robots.txt` file.

/// The result of an attempt to fetch the `robots.txt` file, as described in
/// RFC 9309, section 2.3.1.
///
/// See [`Robots::from_access`].
///
/// [`Robots::from_access`]: crate::Robots::from_access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessResult<'a> {
    /// The file was downloaded, so its rules must be followed.
    Successful(&'a [u8]),

    /// The file was not reached within five consecutive redirects.
    /// The crawler may treat it as [`AccessResult::Unavailable`].
    ///
    /// Redirects should be followed, even across hosts, and a file reached
    /// within five redirects applies to the initial host.
    TooManyRedirects,

    /// The server reported that the file does not exist or is not
    /// accessible, e.g. with a 4xx status code. Every URL may be crawled.
    Unavailable,

    /// The file could not be fetched due to server or network errors, e.g.
    /// with a 5xx status code. No URL may be crawled.
    ///
    /// If the file stays unreachable for a long time (e.g. 30 days), the
    /// crawler may treat it as [`AccessResult::Unavailable`] or keep using a
    /// cached copy.
    Unreachable,
}
