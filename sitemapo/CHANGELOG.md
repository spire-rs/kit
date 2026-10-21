# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-10-21

### Added

- The image, video, news and `hreflang` extensions: `Entry::images`,
  `Entry::videos`, `Entry::news` and `Entry::alternates`.
- `Parser`, which detects text sitemaps, XML `urlset`s and XML
  `sitemapindex`es, and iterates over their `Record`s.
- `Builder`, with the `TextBuilder`, `EntryBuilder` and `IndexBuilder` aliases,
  and `Builder::records` and `Builder::bytes` to split large sitemaps.
- `Frequency::next_change`, in calendar months and years.
- `Error::UnknownRoot`, for XML documents that are not sitemaps.

### Changed

- Dates are `jiff::Timestamp` instead of `time::OffsetDateTime`.
- `record::Index` is now `record::Sitemap`.
- `Priority::new_fallback` is now `Priority::clamped`, `Priority::as_inner` is
  now `Priority::get`, and `Priority::parse` and `Frequency::parse` are now
  `FromStr` implementations.
- `Frequency::next_date` is now `Frequency::next_change`.
- Parsers are iterators instead of implementing the `Parser` trait, and
  builders write with `Builder::write` and `Builder::finish`.
- `Error::EntryLimit { over }` is now `Error::RecordLimit`, and
  `Error::ByteLimit { over }` is now `Error::ByteLimit`.
- The sitemap namespace is the one of the root element, so sitemaps with an
  outdated namespace parse.
- The minimum supported Rust version is 1.90, and the edition is 2024.

### Removed

- `AutoParser`, `PlainParser`, `EntryParser`, `IndexParser` and their
  `Parser` and `AsyncParser` traits; parse with `Parser` and fetch the sitemaps
  of an index yourself.
- `PlainBuilder` and the `Builder` and `AsyncBuilder` traits.
- The `tokio` and `extension` features, and the `countio`, `bytes`, `time`,
  `isolang` and `async-trait` dependencies.
- `Frequency::is_outdated`; compare `Frequency::next_change` with the time.
- `URL_LEN_LIMIT`.

### Fixed

- Parsing no longer loops forever at the end of a sitemap, or on text sitemaps.
- Date-only `lastmod` values like `2022-06-04` are no longer dropped.
- The XML builders enforce the 50 MiB limit for the whole file, closing tag
  included, instead of for each entry.
- Built sitemaps no longer start with a byte order mark, and a `lastmod`
  value can no longer panic the builder.
- Priorities are written exactly: `0.25` instead of `0.2`.
- A single oversized line or text node is never buffered past the 50 MiB limit.
