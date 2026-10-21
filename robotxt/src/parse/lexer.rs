//! Splits `robots.txt` content into directives.

use std::fmt;

use bstr::ByteSlice;

/// A single `robots.txt` line with its trimmed value.
// TODO: Attach position.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Directive<'a> {
    UserAgent(&'a [u8]),
    Allow(&'a [u8]),
    Disallow(&'a [u8]),
    CrawlDelay(&'a [u8]),
    Sitemap(&'a [u8]),
    Unknown(&'a [u8]),
}

impl fmt::Debug for Directive<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (label, slice) = match self {
            Self::UserAgent(x) => ("User-Agent", x),
            Self::Allow(x) => ("Allow", x),
            Self::Disallow(x) => ("Disallow", x),
            Self::CrawlDelay(x) => ("Crawl-Delay", x),
            Self::Sitemap(x) => ("Sitemap", x),
            Self::Unknown(x) => ("Unknown", x),
        };

        f.debug_tuple(label).field(&slice.as_bstr()).finish()
    }
}

/// The UTF-8 byte order mark.
const BOM: &[u8] = b"\xef\xbb\xbf";

/// The directive a key spelling maps to.
#[derive(Clone, Copy)]
enum Key {
    UserAgent,
    Allow,
    Disallow,
    CrawlDelay,
    Sitemap,
}

impl Key {
    /// Creates the directive with the given value.
    const fn into_directive(self, value: &[u8]) -> Directive<'_> {
        match self {
            Self::UserAgent => Directive::UserAgent(value),
            Self::Allow => Directive::Allow(value),
            Self::Disallow => Directive::Disallow(value),
            Self::CrawlDelay => Directive::CrawlDelay(value),
            Self::Sitemap => Directive::Sitemap(value),
        }
    }
}

/// Every accepted spelling of every directive, including common misspellings.
const SPELLINGS: &[(&str, Key)] = &[
    ("user-agent", Key::UserAgent),
    ("user agent", Key::UserAgent),
    ("useragent", Key::UserAgent),
    ("allow", Key::Allow),
    ("alow", Key::Allow),
    ("allaw", Key::Allow),
    ("disallow", Key::Disallow),
    ("dissallow", Key::Disallow),
    ("dissalow", Key::Disallow),
    ("disalow", Key::Disallow),
    ("diasllow", Key::Disallow),
    ("disallaw", Key::Disallow),
    ("crawl-delay", Key::CrawlDelay),
    ("crawl delay", Key::CrawlDelay),
    ("crawldelay", Key::CrawlDelay),
    ("sitemap", Key::Sitemap),
    ("site-map", Key::Sitemap),
    ("site map", Key::Sitemap),
];

/// Returns true if the byte is a space or a tab.
const fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Removes leading spaces and tabs.
fn trim_spaces(input: &[u8]) -> &[u8] {
    let start = input.iter().position(|&c| !is_space(c));
    start.map_or(&[], |start| &input[start..])
}

/// Splits the input into lines terminated by a run of `\r` optionally
/// followed by `\n`, or by a single `\n` or `\0`.
fn lines(mut input: &[u8]) -> impl Iterator<Item = &[u8]> {
    std::iter::from_fn(move || {
        if input.is_empty() {
            return None;
        }

        let end = input.find_byteset(b"\r\n\0").unwrap_or(input.len());
        let (line, mut rest) = input.split_at(end);
        while let Some(tail) = rest.strip_prefix(b"\r") {
            rest = tail;
        }

        input = match rest.first() {
            Some(b'\n' | b'\0') => &rest[1..],
            _ => rest,
        };

        Some(line)
    })
}

/// Splits the input into directives, one per line.
///
/// Never fails: lines that are not directives become [`Directive::Unknown`].
pub fn tokenize(input: &[u8]) -> Vec<Directive<'_>> {
    let input = input.strip_prefix(BOM).unwrap_or(input);
    lines(input).map(tokenize_line).collect()
}

/// Tokenizes a single line without its line ending.
fn tokenize_line(line: &[u8]) -> Directive<'_> {
    tokenize_directive(line).unwrap_or(Directive::Unknown(line))
}

/// Attempts to tokenize a `key: value # comment` (or `key value`) line.
fn tokenize_directive(line: &[u8]) -> Option<Directive<'_>> {
    let line = trim_spaces(line);
    let (spelling, key) = SPELLINGS.iter().find(|(spelling, _)| {
        line.get(..spelling.len())
            .is_some_and(|key| key.eq_ignore_ascii_case(spelling.as_bytes()))
    })?;

    // The key is followed either by a colon or by at least one space.
    let rest = &line[spelling.len()..];
    let after_spaces = trim_spaces(rest);
    let value = match after_spaces.strip_prefix(b":") {
        Some(value) => value,
        None if after_spaces.len() < rest.len() => after_spaces,
        None => return None,
    };

    let value = value.find_byte(b'#').map_or(value, |end| &value[..end]);
    Some(key.into_directive(value.trim()))
}

#[cfg(test)]
mod test {
    use super::{Directive, tokenize};

    #[test]
    fn single() {
        let r = tokenize(b"user-agent: robotxt");
        assert_eq!(r, vec![Directive::UserAgent(b"robotxt")]);
    }

    #[test]
    fn empty() {
        let r = tokenize(
            b"
            user-agent: robotxt\n
            user-agent: robotxt",
        );

        let ua = Directive::UserAgent(b"robotxt");
        let em = Directive::Unknown(b"");
        assert_eq!(r, vec![em, ua, em, ua]);
    }

    #[test]
    fn line_endings() {
        let r = tokenize(b"allow: /a\r\nallow: /b\rallow: /c\r\r\nallow: /d\0allow: /e\n");
        let expected = [b"/a", b"/b", b"/c", b"/d", b"/e"].map(|x| Directive::Allow(x));
        assert_eq!(r, expected);
    }

    #[test]
    fn byte_order_mark() {
        let r = tokenize(b"\xef\xbb\xbfsitemap: https://example.com/");
        assert_eq!(r, vec![Directive::Sitemap(b"https://example.com/")]);
    }

    #[test]
    fn separators() {
        let r = tokenize(b"allow:/a\nallow : /b\nallow /c\n\tallow\t/d");
        let expected = [b"/a", b"/b", b"/c", b"/d"].map(|x| Directive::Allow(x));
        assert_eq!(r, expected);
    }

    #[test]
    fn spellings() {
        let r = tokenize(b"USER AGENT: a\nDissallow: /b\ncrawldelay: 5\nsite-map: c");
        let expected = [
            Directive::UserAgent(b"a"),
            Directive::Disallow(b"/b"),
            Directive::CrawlDelay(b"5"),
            Directive::Sitemap(b"c"),
        ];
        assert_eq!(r, expected);
    }

    #[test]
    fn comments() {
        let r = tokenize(b"# header\nallow: /a # trailing\ndisallow: # empty");
        let expected = [
            Directive::Unknown(b"# header"),
            Directive::Allow(b"/a"),
            Directive::Disallow(b""),
        ];
        assert_eq!(r, expected);
    }

    #[test]
    fn unknown() {
        let r = tokenize(b"host: example.com\nuser-agentx: a\nallow");
        let expected = [
            Directive::Unknown(b"host: example.com"),
            Directive::Unknown(b"user-agentx: a"),
            Directive::Unknown(b"allow"),
        ];
        assert_eq!(r, expected);
    }
}
