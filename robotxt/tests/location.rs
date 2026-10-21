use robotxt::{Error, robots_url};
use url::Url;

#[test]
fn location() {
    let url = Url::parse("https://user:pass@example.com:8080/a/b?q=1#f").unwrap();
    let robots = robots_url(&url).unwrap();
    assert_eq!(robots.as_str(), "https://example.com:8080/robots.txt");
}

#[test]
fn errors() {
    let url = Url::parse("mailto:user@example.com").unwrap();
    assert_eq!(robots_url(&url), Err(Error::CannotBeBase));

    let url = Url::parse("ftp://example.com/").unwrap();
    let scheme = "ftp".to_owned();
    assert_eq!(robots_url(&url), Err(Error::UnsupportedScheme { scheme }));
}
