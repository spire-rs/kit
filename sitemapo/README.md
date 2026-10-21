### sitemapo

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]

**Check out other `spire` projects [here](https://github.com/spire-rs).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/spire-rs/kit/build.yml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/spire-rs/kit/actions/workflows/build.yml
[crates-badge]: https://img.shields.io/crates/v/sitemapo.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/sitemapo
[docs-badge]: https://img.shields.io/docsrs/sitemapo?logo=Docs.rs&style=flat-square
[docs-url]: https://docs.rs/sitemapo

The implementation of the Sitemap (or URL inclusion) protocol in the Rust
programming language, with the support of text and XML sitemaps, sitemap
indexes, and the image, video, news and `hreflang` extensions.

### Examples

- parse a sitemap of any format, and collect the nested sitemaps of an index:

```rust
use sitemapo::{Parser, Record};

let xml = br#"
  <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <sitemap><loc>https://example.com/sitemap.xml</loc></sitemap>
  </sitemapindex>
"#;

let mut nested = Vec::new();
for record in Parser::new(xml.as_slice()) {
    match record? {
        Record::Entry(entry) => println!("{}", entry.location),
        // Fetch and parse each nested sitemap in turn.
        Record::Sitemap(sitemap) => nested.push(sitemap.location),
    }
}

assert_eq!(nested.len(), 1);
# Ok::<(), sitemapo::Error>(())
```

- build a sitemap with the extensions:

```rust
use sitemapo::EntryBuilder;
use sitemapo::record::{Alternate, Entry, Frequency, Image};

let location: url::Url = "https://example.com/".parse().unwrap();
let entry = Entry::new(location.clone())
    .with_modified("2024-01-01T00:00:00Z".parse().unwrap())
    .with_frequency(Frequency::Daily)
    .with_image(Image::new("https://example.com/image.png".parse().unwrap()))
    .with_alternate(Alternate::new("en", location));

let mut builder = EntryBuilder::new(Vec::new());
builder.write(&entry)?;
let xml = String::from_utf8(builder.finish()?).unwrap();
assert!(xml.contains("<image:loc>https://example.com/image.png</image:loc>"));
# Ok::<(), sitemapo::Error>(())
```

- `TextBuilder` and `IndexBuilder` build text sitemaps and sitemap indexes.

- in asynchronous code, fetch the sitemap first, then parse the bytes, which
  never blocks. Gzipped sitemaps can be decompressed while they are parsed,
  e.g. with `flate2::read::GzDecoder`:

```rust,ignore
let body = reqwest::get("https://example.com/sitemap.xml").await?.bytes().await?;
for record in sitemapo::Parser::new(&body[..]) {
    // ..
}
```

### Links

- [Sitemaps Overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview)
  on Google.com
- [Image](https://developers.google.com/search/docs/crawling-indexing/sitemaps/image-sitemaps),
  [Video](https://developers.google.com/search/docs/crawling-indexing/sitemaps/video-sitemaps)
  and [News](https://developers.google.com/search/docs/crawling-indexing/sitemaps/news-sitemap)
  sitemaps on Google.com
- [Localized Versions](https://developers.google.com/search/docs/specialty/international/localized-versions#sitemap)
  on Google.com
- [Sitemaps Format](https://www.sitemaps.org/protocol.html) on Sitemaps.org

### Notes

- Compressed (`.xml.gz`) sitemaps should be decompressed by the reader.
- RSS and Atom feeds, which Google also accepts as sitemaps, are not supported.
