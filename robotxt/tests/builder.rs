use std::time::Duration;

use robotxt::{GroupBuilder, RobotsBuilder};
use url::Url;

#[test]
fn empty() {
    assert_eq!(RobotsBuilder::new().to_string(), "");
}

#[test]
fn comments() {
    let txt = RobotsBuilder::new()
        .header("# a\n\n  b  ")
        .footer("c")
        .to_string();
    assert_eq!(txt, "# a\n# b\n\n# c\n");
}

#[test]
fn sitemaps() {
    let url: Url = "https://example.com/1.xml".parse().unwrap();
    let txt = RobotsBuilder::new()
        .sitemap(url.clone())
        .sitemap(url)
        .to_string();
    assert_eq!(txt, "Sitemap: https://example.com/1.xml\n");
}

#[test]
fn any_agent() {
    let txt = GroupBuilder::new([]).disallow("/foo").to_string();
    assert_eq!(txt, "User-Agent: *\nDisallow: /foo\n");
}

#[test]
fn no_rules() {
    let txt = GroupBuilder::new(["foobot"]).to_string();
    assert_eq!(txt, "User-Agent: foobot\nDisallow:\n");
}

#[test]
fn ordered() {
    let txt = GroupBuilder::new(["b", "a", " b ", ""])
        .crawl_delay(Duration::from_millis(1500))
        .disallow("/")
        .allow("*.html")
        .allow("")
        .to_string();

    let expected =
        "User-Agent: b\nUser-Agent: a\nCrawl-Delay: 1.5\nDisallow: /\nAllow: /*.html\nAllow:\n";
    assert_eq!(txt, expected);
}

#[test]
fn line_injection() {
    let txt = GroupBuilder::new(["a", "b\nDisallow: /"])
        .allow("/a\nDisallow: /")
        .to_string();
    assert_eq!(txt, "User-Agent: a\nAllow: /a%0ADisallow:%20/\n");
}

#[test]
fn round_trip() {
    let builder = RobotsBuilder::new()
        .group(["foobot"], |u| u.allow("/a").disallow("/"))
        .group([], |u| u.disallow("/b"))
        .sitemap("https://example.com/1.xml".parse().unwrap());

    let r = builder.parse("foobot");
    assert!(r.is_allowed("/a"));
    assert!(!r.is_allowed("/b"));
    assert_eq!(r.sitemaps().len(), 1);

    let r = builder.parse("barbot");
    assert!(r.is_allowed("/a"));
    assert!(!r.is_allowed("/b"));
}
