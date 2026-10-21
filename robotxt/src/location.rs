//! The `robots.txt` location.

use url::Url;

use crate::{Error, Result};

/// Returns the location of the `robots.txt` file that governs the URL.
///
/// The credentials, path, query and fragment of the URL are discarded.
///
/// # Errors
///
/// Returns an error if the URL cannot be a base, does not have a host, or
/// its scheme is neither `http` nor `https`.
///
/// # Examples
///
/// ```rust
/// use url::Url;
/// use robotxt::robots_url;
///
/// let url = Url::parse("https://user:pass@example.com/foo/sample.txt?q#f").unwrap();
/// let robots = robots_url(&url).unwrap();
/// assert_eq!(robots.as_str(), "https://example.com/robots.txt");
/// ```
pub fn robots_url(url: &Url) -> Result<Url> {
    if url.cannot_be_a_base() {
        return Err(Error::CannotBeBase);
    }

    if !matches!(url.scheme(), "http" | "https") {
        let scheme = url.scheme().to_owned();
        return Err(Error::UnsupportedScheme { scheme });
    }

    if url.host().is_none() {
        return Err(Error::NoHost);
    }

    // Credentials can always be removed from an `http(s)` URL with a host.
    let mut url = url.clone();
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_path("/robots.txt");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}
