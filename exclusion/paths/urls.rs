use crate::{Error, Result};

/// Returns the expected path to the `robots.txt` file
/// as the [`url::Url`].
///
/// ```rust
/// use url::Url;
/// use robotxt::create_url;
///
/// let path = "https://user:pass@example.com/sample.txt";
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

/// Returns the expected path to the `robots.txt` file
/// as the [`http::Uri`].
///
/// ```rust
/// use http::Uri;
/// use robotxt::create_uri;
///
/// let path = "https://user:pass@example.com/sample.txt";
/// let path = Uri::from_static(path);
/// let robots = create_uri(&path).unwrap().to_string();
/// assert_eq!(robots, "https://example.com/robots.txt")
/// ```
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub fn create_uri(path: &http::Uri) -> Result<http::Uri> {
    // TODO: How do I remove username/password with http::Uri?
    let url = url::Url::parse(&path.to_string())?;
    let robots = create_url(&url)?;
    let uri = robots.to_string().parse::<http::Uri>();
    uri.map_err(|e| http::Error::from(e).into())
}

/// Returns the retrieval request to the `robots.txt` file
/// as the [`http::Request`].
///
/// ```rust
/// use http::{Request, Uri};
/// use robotxt::create_request;
///
/// let path = "https://user:pass@example.com/sample.txt";
/// let path = Uri::from_static(path);
///
/// let request: Request<Vec<u8>> = create_request(&path).unwrap();
/// assert_eq!(request.uri().to_string(), "https://example.com/robots.txt");
/// assert_eq!(request.method(), http::method::Method::GET);
/// ```
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub fn create_request<Body>(path: &http::Uri) -> Result<http::Request<Body>>
where
    Body: Default,
{
    let path = create_uri(path)?;
    let request = http::request::Request::builder()
        .method(http::method::Method::GET)
        .uri(path)
        .body(Body::default())?;

    Ok(request)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_url() -> Result<()> {
        let path = "https://user:pass@example.com/sample.txt";
        let path = url::Url::parse(path).unwrap();

        let robots = create_url(&path)?.to_string();
        assert_eq!(robots, "https://example.com/robots.txt");

        Ok(())
    }

    #[test]
    #[cfg(feature = "http")]
    fn from_uri() -> Result<()> {
        let path = "https://user:pass@example.com/sample.txt";
        let path = http::Uri::from_static(path);

        let robots = create_uri(&path)?.to_string();
        assert_eq!(robots, "https://example.com/robots.txt");

        Ok(())
    }

    #[test]
    #[cfg(feature = "http")]
    fn from_request() -> Result<()> {
        let path = "https://user:pass@example.com/sample.txt";
        let path = http::Uri::from_static(path);

        let request: http::Request<Vec<u8>> = create_request(&path)?;
        assert_eq!(request.uri().to_string(), "https://example.com/robots.txt");
        assert_eq!(request.method(), http::method::Method::GET);

        Ok(())
    }
}
