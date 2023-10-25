use url::{ParseError, Url};

use crate::Error;

/// Returns the expected path to the `robots.txt` file.
///
/// ```rust
/// use url::Url;
/// use robotxt::create_path;
///
/// let path = "https://user:pass@example.com/sample.txt";
/// let path = Url::parse(path).unwrap();
/// let robots = create_path(&path).unwrap().to_string();
/// assert_eq!(robots, "https://example.com/robots.txt")
/// ```
pub fn create_path(path: &Url) -> Result<Url, Error> {
    let mut path: Url = path.clone();

    if path.cannot_be_a_base() {
        return Err(ParseError::SetHostOnCannotBeABaseUrl.into());
    }

    if path.scheme() != "http" && path.scheme() != "https" {
        return Err(ParseError::EmptyHost.into());
    }

    if !path.username().is_empty() {
        path.set_username("").unwrap();
    }

    if path.password().is_some() {
        path.set_password(None).unwrap();
    }

    path.join("/robots.txt").map_err(|e| e.into())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_url() {
        let path = "https://user:pass@example.com/sample.txt";
        let path = Url::parse(path).unwrap();
        let robots = create_path(&path).unwrap().to_string();
        assert_eq!(robots, "https://example.com/robots.txt")
    }
}
