# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-10-21

### Added

- `Agent` to choose how the crawler's name is matched: `Agent::Exact` (RFC 9309,
  the default for a plain `&str`) or `Agent::Prefix`.
- `Robots::is_url_allowed`, `Robots::matching_rule`, `Robots::rules`,
  `Robots::allow_all` and `Robots::disallow_all`.
- `Rule`, with its pattern, permission and matching.

### Changed

- `Robots::from_bytes` is now `Robots::parse`, and it, `Robots::from_reader`
  and `Robots::from_access` take an `Agent` (or `&str`).
- `Robots::is_relative_allowed` is now `Robots::is_allowed`,
  `Robots::is_absolute_allowed` is now `Robots::is_url_allowed`, and
  `Robots::is_always` is now `Robots::always`.
- `Robots::from_always` is now `Robots::allow_all` and `Robots::disallow_all`.
- `ALL_UAS` is now `ANY_AGENT`.
- `create_url` is now `robots_url`, and `Error::WrongScheme` is now
  `Error::UnsupportedScheme`.
- `AccessResult::Redirect` is now `AccessResult::TooManyRedirects`.
- `GroupBuilder::crawl_delay` takes a `Duration`.
- User-agents are matched exactly by default, instead of by prefix.
- Paths and patterns are normalized as RFC 9309 describes: percent-encoded
  unreserved characters are decoded, so `/%61` and `/a` are the same path.
- Builder output keeps the order of the user-agents and rules, includes the
  sitemaps, and ends every line with a newline.
- The minimum supported Rust version is 1.90, and the edition is 2024.

### Removed

- The `optimal` and `full` features: allowing or disallowing everything is
  always detected.
- `Robots::try_is_relative_allowed` and `Robots::try_is_absolute_allowed`;
  use `Robots::matching_rule`.
- `Robots::len` and `Robots::is_empty`; use `Robots::rules`.
- `AccessResult::as_str`, and its `Deref` and `Display` implementations.
- `Error::Url`, as building the `robots.txt` URL can no longer fail.
- The `regex`, `nom` and `percent-encoding` dependencies.

### Fixed

- A rule cut at the 500 KiB limit no longer applies: an `Allow: /private`
  line cut to `Allow: /` used to allow every URL.
- Fragments are no longer matched against rules.
- A `$` that is not the last character is literal, instead of dropping the rule.
- Empty rules are ignored: an empty `Allow:` used to cancel `Disallow: /`.
- Rules before the first `User-Agent` line no longer apply.
- `/robots.txt` is allowed under `Robots::disallow_all`.
- A stray leading `\xef`, `\xbb` or `\xbf` byte no longer corrupts the first line.
- Patterns and user-agents with line breaks can't inject lines into built files.
- Loading cached rules with `serde` sorts and normalizes them again.
