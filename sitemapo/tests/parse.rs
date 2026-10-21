use sitemapo::record::{Entry, Frequency, Priority, Sitemap};
use sitemapo::{BYTE_LIMIT, Error, Format, Parser, RECORD_LIMIT, Record};

fn parse(txt: &[u8]) -> (Option<Format>, Vec<Record>) {
    let mut parser = Parser::new(txt);
    let records = parser.by_ref().collect::<Result<_, _>>().unwrap();
    (parser.format(), records)
}

fn entry(location: &str) -> Record {
    Record::Entry(Entry::new(location.parse().unwrap()))
}

#[test]
fn text() {
    let txt = b"\xef\xbb\xbf  https://example.com/1\r\n\nnot a url\nhttps://example.com/2";
    let (format, records) = parse(txt);
    assert_eq!(format, Some(Format::Text));
    assert_eq!(
        records,
        [
            entry("https://example.com/1"),
            entry("https://example.com/2")
        ]
    );
}

#[test]
fn empty() {
    assert_eq!(parse(b""), (None, Vec::new()));
    assert_eq!(parse(b" \n "), (None, Vec::new()));
    assert_eq!(parse(b"\nnot a url\n"), (Some(Format::Text), Vec::new()));
}

#[test]
fn url_set() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url>
                <loc>https://example.com/?a=1&amp;b=2</loc>
                <lastmod>2022-09-08</lastmod>
                <changefreq>Daily</changefreq>
                <priority>0.6</priority>
            </url>
            <url><loc>not a url</loc></url>
            <url><lastmod>2022-09-08</lastmod></url>
            <url>
                <loc><![CDATA[https://example.com/2]]></loc>
                <lastmod>yesterday</lastmod>
                <priority>2</priority>
            </url>
        </urlset>"#;

    let (format, records) = parse(xml);
    assert_eq!(format, Some(Format::UrlSet));

    let first = Entry::new("https://example.com/?a=1&b=2".parse().unwrap())
        .with_modified("2022-09-08T00:00:00Z".parse().unwrap())
        .with_frequency(Frequency::Daily)
        .with_priority(Priority::new(0.6).unwrap());
    assert_eq!(
        records,
        [Record::Entry(first), entry("https://example.com/2")]
    );
}

#[test]
fn index() {
    let xml = br#"
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap>
                <loc>https://example.com/sitemap1.xml.gz</loc>
                <lastmod>2004-10-01T18:23:17+00:00</lastmod>
            </sitemap>
            <sitemap><loc>https://example.com/sitemap2.xml.gz</loc></sitemap>
        </sitemapindex>"#;

    let (format, records) = parse(xml);
    assert_eq!(format, Some(Format::Index));

    let first = Sitemap::new("https://example.com/sitemap1.xml.gz".parse().unwrap())
        .with_modified("2004-10-01T18:23:17Z".parse().unwrap());
    let second = Sitemap::new("https://example.com/sitemap2.xml.gz".parse().unwrap());
    assert_eq!(records, [Record::Sitemap(first), Record::Sitemap(second)]);
}

#[test]
fn without_namespace() {
    let xml = b"<urlset><url><loc>https://example.com/</loc></url></urlset>";
    assert_eq!(parse(xml).1, [entry("https://example.com/")]);
}

#[test]
fn ignores_unknown_elements() {
    let xml = br#"
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:x="urn:x">
            <x:meta>ignored</x:meta>
            <url>
                <x:loc>https://example.com/ignored</x:loc>
                <loc>https://example.com/</loc>
                <x:a><x:b><x:c><x:d><x:e><x:f><x:g><x:h><x:i>deep</x:i></x:h></x:g></x:f></x:e></x:d></x:c></x:b></x:a>
            </url>
        </urlset>"#;

    assert_eq!(parse(xml).1, [entry("https://example.com/")]);
}

#[test]
fn outdated_namespace() {
    let xml = br#"<urlset xmlns="http://www.google.com/schemas/sitemap/0.84">
        <url><loc>https://example.com/</loc></url></urlset>"#;
    assert_eq!(parse(xml).1, [entry("https://example.com/")]);
}

#[test]
fn unknown_root() {
    let mut parser = Parser::new(b"<rss><channel/></rss>".as_slice());
    let name = "rss".to_owned();
    assert!(matches!(parser.next(), Some(Err(Error::UnknownRoot { name: n })) if n == name));
    assert!(parser.next().is_none());
}

#[test]
fn malformed() {
    let xml = b"<urlset><url><loc>https://example.com/</loc></url></wrong>";
    let mut parser = Parser::new(xml.as_slice());
    assert_eq!(
        parser.next().unwrap().unwrap(),
        entry("https://example.com/")
    );
    assert!(matches!(parser.next(), Some(Err(Error::Xml(_)))));
    assert!(parser.next().is_none());
}

#[test]
fn record_limit() {
    let txt = "https://example.com/\n".repeat(RECORD_LIMIT + 1);
    let mut parser = Parser::new(txt.as_bytes());
    assert_eq!(
        parser.by_ref().take(RECORD_LIMIT).flatten().count(),
        RECORD_LIMIT
    );
    assert!(matches!(parser.next(), Some(Err(Error::RecordLimit))));
    assert!(parser.next().is_none());
}

#[test]
fn byte_limit() {
    let mut txt = b"https://example.com/\n".to_vec();
    txt.resize(BYTE_LIMIT + 1, b' ');
    let mut parser = Parser::new(txt.as_slice());
    assert_eq!(
        parser.next().unwrap().unwrap(),
        entry("https://example.com/")
    );
    assert!(matches!(parser.next(), Some(Err(Error::ByteLimit))));

    let mut xml = b"<urlset><url><loc>https://example.com/</loc></url>".to_vec();
    xml.resize(BYTE_LIMIT + 1, b' ');
    xml.extend_from_slice(b"</urlset>");
    let mut parser = Parser::new(xml.as_slice());
    assert_eq!(
        parser.next().unwrap().unwrap(),
        entry("https://example.com/")
    );
    assert!(matches!(parser.next(), Some(Err(Error::ByteLimit))));
}
