## robotxt

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]
[![Crate Coverage][coverage-badge]][coverage-url]

**Also check out other `spire-rs` projects
[here](https://github.com/spire-rs).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/spire-rs/kit/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/spire-rs/kit/actions/workflows/build.yaml
[crates-badge]: https://img.shields.io/crates/v/robotxt.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/robotxt
[docs-badge]: https://img.shields.io/docsrs/robotxt?logo=Docs.rs&style=flat-square
[docs-url]: http://docs.rs/robotxt
[coverage-badge]: https://img.shields.io/codecov/c/github/spire-rs/kit?logo=codecov&logoColor=white&style=flat-square
[coverage-url]: https://app.codecov.io/gh/spire-rs/kit

The implementation of the robots.txt (or URL exclusion) protocol in the Rust
programming language with the support of `crawl-delay`, `sitemap` and universal
`*` match extensions (according to the RFC specification).

### Features

- `parser` to enable `robotxt::{Robots}`. **Enabled by default**.
- `builder` to enable `robotxt::{RobotsBuilder, GroupBuilder}`. **Enabled by
  default**.
- `optimal` to optimize overlapping and global rules, potentially improving
  matching speed at the cost of longer parsing times.
- `serde` to enable `serde::{Deserialize, Serialize}` implementation, allowing
  the caching of related rules.

### Examples

- parse the most specific `user-agent` in the provided `robots.txt` file:

```rust
use robotxt::Robots;

fn main() {
    let txt = r#"
      User-Agent: foobot
      Disallow: *
      Allow: /example/
      Disallow: /example/nope.txt
    "#;

    let r = Robots::from_bytes(txt.as_bytes(), "foobot");
    assert!(r.is_relative_allowed("/example/yeah.txt"));
    assert!(!r.is_relative_allowed("/example/nope.txt"));
    assert!(!r.is_relative_allowed("/invalid/path.txt"));
}
```

- build the new `robots.txt` file in a declarative manner:

```rust
use robotxt::RobotsBuilder;

fn main() -> Result<(), url::ParseError> {
    let txt = RobotsBuilder::default()
        .header("Robots.txt: Start")
        .group(["foobot"], |u| {
            u.crawl_delay(5)
                .header("Rules for Foobot: Start")
                .allow("/example/yeah.txt")
                .disallow("/example/nope.txt")
                .footer("Rules for Foobot: End")
        })
        .group(["barbot", "nombot"], |u| {
            u.crawl_delay(2)
                .disallow("/example/yeah.txt")
                .disallow("/example/nope.txt")
        })
        .sitemap("https://example.com/sitemap_1.xml".try_into()?)
        .sitemap("https://example.com/sitemap_1.xml".try_into()?)
        .footer("Robots.txt: End");

    println!("{}", txt.to_string());
    Ok(())
}
```

### Links

- [Request for Comments: 9309](https://www.rfc-editor.org/rfc/rfc9309.txt) on
  RFC-Editor.com
- [Introduction to Robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/intro)
  on Google.com
- [How Google interprets Robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/robots_txt)
  on Google.com
- [What is Robots.txt file](https://moz.com/learn/seo/robotstxt) on Moz.com

### Notes

- The parser is based on
  [Smerity/texting_robots](https://github.com/Smerity/texting_robots).
- The `Host` directive is not supported.
