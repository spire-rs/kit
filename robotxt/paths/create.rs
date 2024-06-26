use url::Url;

use crate::{Error, Result};

/// Returns the expected path to the `robots.txt` file
/// as the `url::`[`Url`].
///
/// # Errors
///
/// Returns the error if the provided [`Url`] cannot be a base,
/// does not have a host or the schema is not `http` or `https`.
///
/// # Examples
///
/// ```rust
/// use url::Url;
/// use robotxt::create_url;
///
/// let path = "https://user:pass@example.com/foo/sample.txt";
/// let path = Url::parse(path).unwrap();
/// let robots = create_url(&path).unwrap().to_string();
/// assert_eq!(robots, "https://example.com/robots.txt")
/// ```
pub fn create_url(path: &Url) -> Result<Url> {
    let mut path = path.clone();

    if path.cannot_be_a_base() {
        return Err(Error::CannotBeBase);
    }

    if path.host().is_none() {
        return Err(Error::NoHost);
    }

    if path.scheme() != "http" && path.scheme() != "https" {
        return Err(Error::WrongScheme {
            scheme: path.scheme().to_string(),
        });
    }

    if !path.username().is_empty() {
        path.set_username("").expect("should pass base/host tests");
    }

    if path.password().is_some() {
        path.set_password(None)
            .expect("should pass base/host tests");
    }

    path.join("/robots.txt").map_err(Into::into)
}

#[cfg(test)]
mod test {
    use crate::{create_url, url::Url, Result};

    #[test]
    fn from_url() -> Result<()> {
        let path = "https://user:pass@example.com/foo/sample.txt";
        let path = Url::parse(path).unwrap();

        let robots = create_url(&path)?.to_string();
        assert_eq!(robots, "https://example.com/robots.txt");

        Ok(())
    }
}
