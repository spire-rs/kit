<div align="center">

<img src=".github/assets/logo.png" alt="Spire" width="104" height="104" />

# Kit

**The robots.txt and sitemap protocols for Rust.**

Standalone crates that parse and build the files crawlers read before they
crawl, used by the [Spire](https://github.com/spire-rs/spire) crawler framework.

[![Build](https://img.shields.io/github/actions/workflow/status/spire-rs/kit/build.yml?branch=main&label=build&style=flat-square)](https://github.com/spire-rs/kit/actions/workflows/build.yml)
[![Security](https://img.shields.io/github/actions/workflow/status/spire-rs/kit/security.yml?branch=main&label=security&style=flat-square)](https://github.com/spire-rs/kit/actions/workflows/security.yml)

[**crates.io/crates/robotxt**](https://crates.io/crates/robotxt) · [**crates.io/crates/sitemapo**](https://crates.io/crates/sitemapo) · [**github.com/spire-rs**](https://github.com/spire-rs)

</div>

## Crates

| Crate | Version | What it does |
| --- | --- | --- |
| [robotxt](robotxt/README.md) | [![robotxt](https://img.shields.io/crates/v/robotxt.svg?style=flat-square)](https://crates.io/crates/robotxt) | Parses and builds robots.txt files following RFC 9309, with the `crawl-delay` and `sitemap` extensions. |
| [sitemapo](sitemapo/README.md) | [![sitemapo](https://img.shields.io/crates/v/sitemapo.svg?style=flat-square)](https://crates.io/crates/sitemapo) | Parses and builds text and XML sitemaps and sitemap indexes, with the image, video, news and `hreflang` extensions. |

## Features

- **RFC 9309 matching**: exact user-agent matching by default, or by prefix, with `*` and `$` wildcards, percent-encoding normalization and longest-match precedence.
- **Bounded on untrusted input**: robotxt reads at most 500 KiB and sitemapo at most 50 MiB, and invalid lines and records are skipped instead of failing the whole file.
- **Every sitemap format**: text sitemaps, XML `urlset`s and `sitemapindex`es are detected from the first bytes and read through one parser.
- **Builders**: write robots.txt files and sitemaps in a deterministic order; the sitemap builders keep each file within 50,000 records and 50 MiB.
- **Few dependencies**: no async runtime, regex or parser combinator library.

## Requirements

- **Rust + Cargo**: 1.90+, Edition 2024

## Quick start

Add the crates you need:

```toml
[dependencies]
robotxt = "0.7"
sitemapo = "0.3"
```

Check whether a crawler may fetch a page, then read the sitemaps the
robots.txt file lists:

```rust
use robotxt::Robots;
use sitemapo::{Parser, Record};

fn main() -> Result<(), sitemapo::Error> {
    let txt = b"
        User-Agent: foobot
        Disallow: /private
        Sitemap: https://example.com/sitemap.xml
    ";

    let robots = Robots::parse(txt, "foobot");
    assert!(robots.is_allowed("/public/page.html"));
    assert!(!robots.is_allowed("/private/page.html"));

    for _sitemap in robots.sitemaps() {
        // Fetch the sitemap, then parse its bytes.
        let body = b"https://example.com/public/page.html\n";
        for record in Parser::new(body.as_slice()) {
            if let Record::Entry(entry) = record? {
                println!("{}", entry.location);
            }
        }
    }

    Ok(())
}
```

Each crate's README covers its features, builders and more examples.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines, and
each crate's `CHANGELOG.md` for release notes.

## License

MIT License, see [LICENSE](LICENSE).
