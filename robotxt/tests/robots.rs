use robotxt::{AccessResult, BYTE_LIMIT, Robots};
use url::Url;

fn robots(rules: &str) -> Robots {
    Robots::parse(format!("User-Agent: *\n{rules}").as_bytes(), "foobot")
}

#[test]
fn precedence() {
    assert!(robots("Allow: /p\nDisallow: /").is_allowed("/page"));
    assert!(robots("Allow: /folder\nDisallow: /folder").is_allowed("/folder/page"));
    assert!(robots("Allow: /page\nDisallow: /*.ph").is_allowed("/page.php5"));
    assert!(!robots("Allow: /page\nDisallow: /*.htm").is_allowed("/page.htm"));
    assert!(robots("Allow: /$\nDisallow: /").is_allowed("/"));
    assert!(!robots("Allow: /$\nDisallow: /").is_allowed("/page.htm"));
}

#[test]
fn matching_rule() {
    let r = robots("Allow: /example/\nDisallow: /example/nope.txt");
    let rule = r.matching_rule("/example/nope.txt").unwrap();
    assert_eq!(rule.pattern(), "/example/nope.txt");
    assert!(!rule.is_allow());
    assert!(r.matching_rule("/other").is_none());
}

#[test]
fn always() {
    assert_eq!(Robots::parse(b"", "foobot").always(), Some(true));
    assert_eq!(robots("Allow: /\nAllow: /foo").always(), Some(true));
    assert_eq!(robots("Disallow: /*\nDisallow: /foo").always(), Some(false));
    assert_eq!(robots("Disallow: /*\nAllow: /foo").always(), None);
    assert_eq!(robots("Disallow: /foo").always(), None);
}

#[test]
fn empty_rules() {
    assert_eq!(robots("Disallow: /\nAllow:").always(), Some(false));
    assert_eq!(robots("Disallow:").always(), Some(true));
}

#[test]
fn robots_txt() {
    assert!(Robots::disallow_all().is_allowed("/robots.txt"));
    assert!(robots("Disallow: /").is_allowed("/robots.txt"));
    assert!(robots("Disallow: /robots").is_allowed("/robots.txt"));
}

#[test]
fn urls() {
    let r = robots("Disallow: /*?id=");
    let url = Url::parse("https://example.com/page?id=1").unwrap();
    assert!(!r.is_url_allowed(&url));

    let url = Url::parse("https://example.com/page#?id=1").unwrap();
    assert!(r.is_url_allowed(&url));
}

#[test]
fn percent_encoding() {
    let r = robots("Disallow: /%61");
    assert!(!r.is_allowed("/a"));
    assert!(!r.is_url_allowed(&Url::parse("https://example.com/%61").unwrap()));
    assert!(!robots("Disallow: /ä").is_allowed("/%c3%a4"));
}

#[test]
fn inner_anchor_is_literal() {
    let r = robots("Disallow: /a$b");
    assert!(!r.is_allowed("/a$b/c"));
    assert!(r.is_allowed("/a"));
}

#[test]
fn crawl_delay() {
    let r = robots("Crawl-Delay: 5\nCrawl-Delay: 2.5\nCrawl-Delay: -1");
    assert_eq!(
        r.crawl_delay(),
        Some(std::time::Duration::from_millis(2500))
    );
}

#[test]
fn sitemaps() {
    let txt = b"Sitemap: https://a.com/1.xml\nUser-Agent: b\nSitemap: https://a.com/2.xml\nSitemap: invalid";
    assert_eq!(Robots::parse(txt, "foobot").sitemaps().len(), 2);
}

#[test]
fn access() {
    let txt = b"User-Agent: *\nDisallow: /private";
    let r = Robots::from_access(AccessResult::Successful(txt), "foobot");
    assert!(!r.is_allowed("/private"));

    for access in [AccessResult::TooManyRedirects, AccessResult::Unavailable] {
        assert_eq!(Robots::from_access(access, "foobot").always(), Some(true));
    }

    let r = Robots::from_access(AccessResult::Unreachable, "foobot");
    assert_eq!(r.always(), Some(false));
}

/// Returns the file whose last line ends `cut` bytes past the limit.
fn padded(last: &[u8], cut: isize) -> Vec<u8> {
    let mut txt = b"User-Agent: *\nDisallow: /\n".to_vec();
    let end = BYTE_LIMIT.checked_add_signed(cut).unwrap();
    txt.resize(end - last.len(), b'\n');
    txt.extend_from_slice(last);
    txt
}

#[test]
fn byte_limit() {
    // The cut line `Allow: /` would allow every URL.
    let txt = padded(b"Allow: /private/area\n", 12);
    let r = Robots::parse(&txt, "foobot");
    assert_eq!(r.always(), Some(false));
    assert_eq!(Robots::from_reader(txt.as_slice(), "foobot").unwrap(), r);

    // A line ending right at the limit is complete.
    let txt = padded(b"Allow: /public\n", 1);
    let r = Robots::parse(&txt, "foobot");
    assert!(r.is_allowed("/public"));
    assert_eq!(Robots::from_reader(txt.as_slice(), "foobot").unwrap(), r);

    // A file of exactly the limit is parsed whole.
    let txt = padded(b"Allow: /public", 0);
    assert!(Robots::parse(&txt, "foobot").is_allowed("/public"));
}

#[test]
fn from_reader() {
    let txt = b"User-Agent: *\nDisallow: /".as_slice();
    let r = Robots::from_reader(txt, "foobot").unwrap();
    assert!(!r.is_allowed("/"));
}

#[test]
fn disallow_all() {
    let r = Robots::disallow_all();
    assert_eq!(r.always(), Some(false));
    assert!(!r.matching_rule("/page").unwrap().is_allow());
}
