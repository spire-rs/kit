## robotxt

[![Build Status][action-badge]][action-url]
[![Crate Docs][docs-badge]][docs-url]
[![Crate Version][crates-badge]][crates-url]

**Check out other `spire` projects [here](https://github.com/spire-rs).**

[action-badge]: https://img.shields.io/github/actions/workflow/status/spire-rs/kit/build.yml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/spire-rs/kit/actions/workflows/build.yml
[crates-badge]: https://img.shields.io/crates/v/robotxt.svg?logo=rust&style=flat-square
[crates-url]: https://crates.io/crates/robotxt
[docs-badge]: https://img.shields.io/docsrs/robotxt?logo=Docs.rs&style=flat-square
[docs-url]: https://docs.rs/robotxt

The implementation of the robots.txt (or URL exclusion) protocol in the Rust
programming language, following [RFC 9309], with the support of the
`crawl-delay` and `sitemap` extensions.

### Features

- `parser` to enable `robotxt::{Robots}`. **Enabled by default**.
- `builder` to enable `robotxt::{RobotsBuilder, GroupBuilder}`. **Enabled by
  default**.
- `serde` to enable `serde::{Deserialize, Serialize}` implementations, allowing
  the caching of parsed rules.

### Examples

- parse the rules that apply to the crawler:

```rust
use robotxt::Robots;

let txt = b"
  User-Agent: foobot
  Disallow: *
  Allow: /example/
  Disallow: /example/nope.txt
";

let r = Robots::parse(txt, "foobot");
assert!(r.is_allowed("/example/yeah.txt"));
assert!(!r.is_allowed("/example/nope.txt"));
assert!(!r.is_allowed("/invalid/path.txt"));
```

- match the crawler's name by prefix instead of exactly, so a `foobot` group
  also applies to the `foobot-images` crawler:

```rust
use robotxt::{Agent, Robots};

let txt = b"
  User-Agent: foobot
  Disallow: /
";

let r = Robots::parse(txt, Agent::Prefix("foobot-images"));
assert!(!r.is_allowed("/"));
```

- build a new `robots.txt` file:

```rust
use std::time::Duration;
use robotxt::RobotsBuilder;

let txt = RobotsBuilder::new()
    .header("Robots.txt: Start")
    .group(["foobot"], |u| {
        u.crawl_delay(Duration::from_secs(5))
            .header("Rules for Foobot: Start")
            .allow("/example/yeah.txt")
            .disallow("/example/nope.txt")
            .footer("Rules for Foobot: End")
    })
    .group(["barbot", "nombot"], |u| {
        u.crawl_delay(Duration::from_secs(2))
            .disallow("/example/yeah.txt")
            .disallow("/example/nope.txt")
    })
    .sitemap("https://example.com/sitemap_1.xml".parse().unwrap())
    .sitemap("https://example.com/sitemap_2.xml".parse().unwrap())
    .footer("Robots.txt: End");

println!("{txt}");
```

### Links

- [Request for Comments: 9309][RFC 9309] on RFC-Editor.com
- [Introduction to Robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/intro)
  on Google.com
- [How Google interprets Robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/robots_txt)
  on Google.com
- [What is Robots.txt file](https://moz.com/learn/seo/robotstxt) on Moz.com

### Notes

- Only `User-Agent`, `Allow`, `Disallow`, `Crawl-Delay` and `Sitemap` are
  understood. Other lines, e.g. the non-standard `Host` or `Clean-Param`
  directives, are ignored.

[RFC 9309]: https://www.rfc-editor.org/rfc/rfc9309.html
