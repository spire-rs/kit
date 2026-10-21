use std::io::{self, Write};

use sitemapo::record::{Entry, Sitemap};
use sitemapo::{
    BYTE_LIMIT, EntryBuilder, Error, IndexBuilder, Parser, RECORD_LIMIT, Record, TextBuilder,
};
use url::Url;

fn url(location: &str) -> Url {
    location.parse().unwrap()
}

#[test]
fn text() {
    let mut builder = TextBuilder::new(Vec::new());
    builder.write(&url("https://example.com/1")).unwrap();
    builder.write(&url("https://example.com/2")).unwrap();
    assert_eq!(builder.records(), 2);

    let txt = builder.finish().unwrap();
    assert_eq!(txt, b"https://example.com/1\nhttps://example.com/2\n");
}

#[test]
fn url_set() {
    let entry = Entry::new(url("https://example.com/?a=1&b=<2>"))
        .with_modified("2024-01-02T03:04:05Z".parse().unwrap());

    let mut builder = EntryBuilder::new(Vec::new());
    builder.write(&entry).unwrap();
    let xml = String::from_utf8(builder.finish().unwrap()).unwrap();

    assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
    assert!(xml.contains("  <url>\n    <loc>https://example.com/?a=1&amp;b=%3C2%3E</loc>\n"));
    assert!(xml.contains("<lastmod>2024-01-02T03:04:05Z</lastmod>"));
    assert!(xml.ends_with("  </url>\n</urlset>\n"));
}

#[test]
fn index() {
    let mut builder = IndexBuilder::new(Vec::new());
    builder
        .write(&Sitemap::new(url("https://example.com/sitemap.xml")))
        .unwrap();
    let xml = String::from_utf8(builder.finish().unwrap()).unwrap();
    assert!(
        xml.contains("<sitemap>\n    <loc>https://example.com/sitemap.xml</loc>\n  </sitemap>")
    );
}

#[test]
fn empty() {
    let xml = EntryBuilder::new(Vec::new()).finish().unwrap();
    assert_eq!(Parser::new(xml.as_slice()).count(), 0);
    assert_eq!(TextBuilder::new(Vec::new()).finish().unwrap(), b"");
}

#[test]
fn round_trip() {
    let entries: Vec<_> = (0..3)
        .map(|i| Entry::new(url(&format!("https://example.com/{i}"))))
        .collect();

    let mut builder = EntryBuilder::new(Vec::new());
    entries
        .iter()
        .try_for_each(|entry| builder.write(entry))
        .unwrap();
    let xml = builder.finish().unwrap();

    let records: Vec<_> = Parser::new(xml.as_slice())
        .collect::<Result<_, _>>()
        .unwrap();
    let expected: Vec<_> = entries.into_iter().map(Record::Entry).collect();
    assert_eq!(records, expected);
}

#[test]
fn record_limit() {
    let mut builder = TextBuilder::new(io::sink());
    let location = url("https://example.com/");
    (0..RECORD_LIMIT).for_each(|_| builder.write(&location).unwrap());
    assert!(matches!(builder.write(&location), Err(Error::RecordLimit)));
    assert_eq!(builder.records(), RECORD_LIMIT);
    builder.finish().unwrap();
}

#[test]
fn byte_limit() {
    let path = "a".repeat(BYTE_LIMIT / 4);
    let location = url(&format!("https://example.com/{path}"));

    let mut builder = EntryBuilder::new(Vec::new());
    for _ in 0..3 {
        builder.write(&Entry::new(location.clone())).unwrap();
    }

    let entry = Entry::new(location);
    assert!(matches!(builder.write(&entry), Err(Error::ByteLimit)));
    assert!(builder.bytes() <= BYTE_LIMIT);

    let xml = builder.finish().unwrap();
    assert!(xml.len() <= BYTE_LIMIT);
    assert_eq!(Parser::new(xml.as_slice()).count(), 3);
}

/// A writer that fails after the first write.
struct Failing(bool);

impl Write for Failing {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if std::mem::replace(&mut self.0, true) {
            return Err(io::Error::other("failed"));
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn io_error() {
    let mut builder = TextBuilder::new(Failing(false));
    builder.write(&url("https://example.com/1")).unwrap();
    assert!(matches!(
        builder.write(&url("https://example.com/2")),
        Err(Error::Io(_))
    ));
    assert_eq!(builder.records(), 1);
}
