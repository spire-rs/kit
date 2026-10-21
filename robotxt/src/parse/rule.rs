//! `Allow` and `Disallow` rules.

use std::cmp::Reverse;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::path::{normalize, normalize_path};

/// A single `Allow` or `Disallow` rule of the group that applies to the crawler.
///
/// Patterns are matched against the path and query of a URL as described in
/// RFC 9309: `*` matches any sequence of characters, `$` at the end of the
/// pattern matches the end of the URL, and any other pattern matches every
/// URL it is a prefix of.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rule {
    pattern: String,
    allow: bool,
}

impl Rule {
    /// Creates the rule from the raw `robots.txt` value, or returns `None`
    /// if the value is empty, as an empty rule matches nothing.
    pub(crate) fn parse(value: &[u8], allow: bool) -> Option<Self> {
        if value.is_empty() {
            return None;
        }

        let pattern = normalize(value).into_owned();
        Some(Self { pattern, allow })
    }

    /// Creates the `Disallow` rule that matches every URL.
    pub(crate) fn disallow_all() -> Self {
        let pattern = "/".to_owned();
        Self {
            pattern,
            allow: false,
        }
    }

    /// Returns the normalized pattern.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns true for `Allow` rules and false for `Disallow` rules.
    #[must_use]
    pub const fn is_allow(&self) -> bool {
        self.allow
    }

    /// Returns true if the pattern matches the relative URL (path and query).
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let r = Robots::parse(b"User-Agent: *\nDisallow: /*.php$", "foobot");
    /// let rule = &r.rules()[0];
    /// assert!(rule.is_match("/index.php"));
    /// assert!(!rule.is_match("/index.php?id=1"));
    /// ```
    #[must_use]
    pub fn is_match(&self, path: &str) -> bool {
        self.matches(&normalize_path(path))
    }

    /// Returns true if the pattern matches the normalized relative URL.
    pub(crate) fn matches(&self, path: &str) -> bool {
        matches_pattern(&self.pattern, path)
    }

    /// Returns true if the rule matches every URL.
    pub(crate) fn is_universal(&self) -> bool {
        let rest = self.pattern.trim_start_matches('/');
        rest.len() + 1 == self.pattern.len() && rest.bytes().all(|byte| byte == b'*')
    }

    /// Returns the sort key that puts the rule that wins first: the longest
    /// pattern, and `Allow` over `Disallow` for patterns of the same length.
    pub(crate) fn precedence(&self) -> (Reverse<usize>, Reverse<bool>) {
        (Reverse(self.pattern.len()), Reverse(self.allow))
    }
}

/// Returns true if the pattern matches the path.
///
/// Segments between `*` are matched left to right at their first occurrence,
/// which leaves the longest possible remainder for the segments after them.
fn matches_pattern(pattern: &str, path: &str) -> bool {
    let (pattern, anchored) = match pattern.strip_suffix('$') {
        Some(pattern) => (pattern, true),
        None => (pattern, false),
    };

    let mut segments = pattern.split('*');
    let first = segments.next().unwrap_or_default();
    let Some(mut rest) = path.strip_prefix(first) else {
        return false;
    };

    let Some(last) = segments.next_back() else {
        return !anchored || rest.is_empty();
    };

    for segment in segments {
        match rest.find(segment) {
            Some(index) => rest = &rest[index + segment.len()..],
            None => return false,
        }
    }

    if anchored {
        rest.ends_with(last)
    } else {
        rest.contains(last)
    }
}

#[cfg(test)]
mod test {
    use super::{Rule, matches_pattern};

    fn rule(pattern: &str) -> Rule {
        Rule::parse(pattern.as_bytes(), true).unwrap()
    }

    #[test]
    fn empty() {
        assert_eq!(Rule::parse(b"", true), None);
    }

    #[test]
    fn root() {
        assert!(rule("/").is_match("/fish"));
        assert!(rule("/*").is_match("//"));

        let r = rule("/$");
        assert!(r.is_match("/"));
        assert!(!r.is_match("/fish"));
        assert!(!r.is_match("//"));
    }

    #[test]
    fn prefix() {
        let r = rule("/fish");
        for path in [
            "/fish",
            "/fish.html",
            "/fish/salmon.html",
            "/fishheads",
            "/fish.php?id=x",
        ] {
            assert!(r.is_match(path), "{path}");
        }

        for path in ["/Fish.asp", "/catfish", "/?id=fish", "/desert/fish"] {
            assert!(!r.is_match(path), "{path}");
        }
    }

    #[test]
    fn folder() {
        let r = rule("/fish/");
        for path in ["/fish/", "/fish/?id=anything", "/fish/salmon.htm"] {
            assert!(r.is_match(path), "{path}");
        }

        for path in ["/fish", "/fish.html", "/animals/fish/", "/Fish/Salmon.asp"] {
            assert!(!r.is_match(path), "{path}");
        }
    }

    #[test]
    fn trailing_wildcard() {
        let r = rule("/fish*");
        assert!(r.is_match("/fish"));
        assert!(r.is_match("/fishheads/yummy.html"));
        assert!(!r.is_match("/catfish"));
    }

    #[test]
    fn inner_wildcard() {
        let r = rule("/*.php");
        for path in [
            "/index.php",
            "/folder/filename.php?parameters",
            "/a.php.file.html",
        ] {
            assert!(r.is_match(path), "{path}");
        }

        assert!(!r.is_match("/"));
        assert!(!r.is_match("/windows.PHP"));

        let r = rule("/fish*.php");
        assert!(r.is_match("/fishheads/catfish.php?parameters"));
        assert!(!r.is_match("/Fish.PHP"));
    }

    #[test]
    fn both_wildcards() {
        let r = rule("/*.php$");
        assert!(r.is_match("/filename.php"));
        assert!(r.is_match("/folder/filename.php"));

        for path in [
            "/filename.php?parameters",
            "/filename.php/",
            "/filename.php5",
        ] {
            assert!(!r.is_match(path), "{path}");
        }
    }

    #[test]
    fn inner_anchor_is_literal() {
        let r = rule("/a$b");
        assert!(r.is_match("/a$b/c"));
        assert!(!r.is_match("/a"));
    }

    #[test]
    fn overlapping_segments() {
        assert!(matches_pattern("/*ab*ab$", "/abab"));
        assert!(!matches_pattern("/*aba*ba$", "/abab"));
        assert!(matches_pattern("/**a", "/xa"));
    }

    #[test]
    fn universal() {
        assert!(rule("/").is_universal());
        assert!(rule("*").is_universal());
        assert!(rule("/**").is_universal());
        assert!(!rule("/a").is_universal());
        assert!(!rule("/$").is_universal());
        assert!(!rule("//").is_universal());
    }

    #[test]
    fn percent_encoding() {
        assert!(rule("/%61").is_match("/a"));
        assert!(rule("/ä").is_match("/%c3%a4"));
    }

    /// Checks the matcher against the `wildcard` crate on random inputs.
    #[test]
    fn oracle() {
        use wildcard::WildcardBuilder;

        const ALPHABET: &[u8] = b"ab/*$";
        let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = move |bound: usize| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            usize::try_from(seed % 1024).unwrap() % bound
        };

        for _ in 0..200_000 {
            let mut pattern = String::from("/");
            let mut path = String::from("/");
            (0..next(8)).for_each(|_| pattern.push(char::from(ALPHABET[next(ALPHABET.len())])));
            (0..next(10)).for_each(|_| path.push(char::from(ALPHABET[next(3)])));

            // The oracle has no anchor or prefix matching, so translate both:
            // a trailing `$` is dropped, otherwise a trailing `*` is added.
            // Any other `$` is a literal for both.
            let oracle = match pattern.strip_suffix('$') {
                Some(anchored) => anchored.as_bytes().to_vec(),
                None => format!("{pattern}*").into_bytes(),
            };

            let oracle = WildcardBuilder::from_owned(oracle)
                .without_one_metasymbol()
                .without_escape()
                .build()
                .unwrap();

            assert_eq!(
                matches_pattern(&pattern, &path),
                oracle.is_match(path.as_bytes()),
                "pattern: {pattern}, path: {path}",
            );
        }
    }
}
