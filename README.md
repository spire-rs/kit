## Kit

[![Build Status][action-badge]][action-url]
[![Crate Coverage][coverage-badge]][coverage-url]

[action-badge]: https://img.shields.io/github/actions/workflow/status/spire-rs/kit/build.yaml?branch=main&label=build&logo=github&style=flat-square
[action-url]: https://github.com/spire-rs/kit/actions/workflows/build.yaml
[coverage-badge]: https://img.shields.io/codecov/c/github/spire-rs/kit?logo=codecov&logoColor=white&style=flat-square
[coverage-url]: https://app.codecov.io/gh/spire-rs/kit

#### Crates:

- [countio](./countio/): The wrapper struct to enable byte counting for
  `std::io::{Read, Write, Seek}` and its async variants from `futures` and
  `tokio`.
- [robotxt](./robotxt/): The implementation of the Robots.txt (or URL exclusion)
  protocol with the support of `crawl-delay`, `sitemap` and universal `*` match
  extensions.
- [sitemapo](./sitemapo/): The implementation of the Sitemap (or URL inclusion)
  protocol with the support of txt, xml formats and video, image, and news
  extensions.
