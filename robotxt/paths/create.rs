use crate::{Error, Result};

/// Returns the expected path to the `robots.txt` file
/// as the [`url::Url`].
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
pub fn create_url(path: &url::Url) -> Result<url::Url> {
    let mut path = path.clone();

    if path.cannot_be_a_base() {
        return Err(Error::CannotBeBase);
    }

    if path.scheme() != "http" && path.scheme() != "https" {
        return Err(Error::WrongScheme {
            scheme: path.scheme().to_string(),
        });
    }

    if !path.username().is_empty() {
        path.set_username("").unwrap();
    }

    if path.password().is_some() {
        path.set_password(None).unwrap();
    }

    path.join("/robots.txt").map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_url() -> Result<()> {
        let path = "https://user:pass@example.com/foo/sample.txt";
        let path = url::Url::parse(path).unwrap();

        let robots = create_url(&path)?.to_string();
        assert_eq!(robots, "https://example.com/robots.txt");

        Ok(())
    }
}
