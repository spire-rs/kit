### sitemapo

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]
[![Crate Coverage][coverage-badge]][coverage-url]

**Also check out other `xwde` projects [here](https://github.com/xwde).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/xwde/kit/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/xwde/kit/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/sitemapo.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/sitemapo
[docs-badge]: https://img.shields.io/docsrs/sitemapo?logo=Docs.rs&style=flat-square
[docs-url]: http://docs.rs/sitemapo
[coverage-badge]: https://img.shields.io/codecov/c/github/xwde/kit?logo=codecov&logoColor=white&style=flat-square
[coverage-url]: https://app.codecov.io/gh/xwde/kit

The implementation of the Sitemap (or URL inclusion) protocol in the Rust
programming language with the support of `txt` & `xml` formats, and `video`,
`image`, `news` extensions (according to the Google's spec).

### Features

> **Warning** : `extension` are not yet implemented.

- `extension` to enable all XML sitemap extensions. **Enabled by default**.
- `tokio` to enable asynchronous parsers & builders.

### Examples

- automatic parser: `AutoParser`.

```rust
#[derive(Debug, thiserror::Error)]
enum CustomError {
    // ..
    #[error("sitemap error: {0}")]
    Sitemap(#[from] sitemapo::Error),
    //..
}

fn main() -> Result<(), CustomError> {
    type SyncReader = std::io::BufReader<std::io::Cursor<Vec<u8>>>;
    fn fetch(_: url::Url) -> Result<SyncReader, CustomError> {
        // ..
        unreachable!()
    }

    // Sitemaps listed in the robots.txt file.
    let sitemaps = Vec::default();

    let mut parser = sitemapo::parse::AutoParser::new(sitemaps);
    while let Some(_record) = parser.try_sync(fetch)? {
        // ..
    }

    Ok(())
}
```

- automatic builder: `AutoBuilder`.

```rust
#[derive(Debug, thiserror::Error)]
enum CustomError {
    // ..
    #[error("sitemap error: {0}")]
    Sitemap(#[from] sitemapo::Error),
    //..
}

fn main() -> Result<(), CustomError> {
    Ok(())
}
```

- Also includes parsers: `parse::{TxtParser, XmlParser}` and builders:
  `build:{TxtBuilder, XmlBuilder}`.

### Links

- [Sitemaps Overview](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview)
  on Google.com
- [Sitemaps Best Practice](https://developers.google.com/search/blog/2014/10/best-practices-for-xml-sitemaps-rssatom)
  on Google.com
- [Sitemaps Format](https://www.sitemaps.org/protocol.html) on Sitemap.org
- [Sitemaps FAQ](https://www.sitemaps.org/faq.htm) on Sitemap.org

### Crates

- [svmk/rust-sitemap](https://crates.io/crates/sitemap)
- [goddtriffin/sitemap-rs](https://crates.io/crates/sitemap-rs)
