use std::fmt;

use bstr::ByteSlice;
use nom::branch::{alt, Alt};
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::{space0, space1};
use nom::combinator::{eof, opt};
use nom::error::{Error as NomError, ParseError as NomParseError};
use nom::multi::many_till;
use nom::sequence::preceded;
use nom::{Err as NomErr, IResult as NomResult};

/// The `Directive` enum represents every supported `robots.txt` directive.
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

const CARRIAGE: u8 = b'\r';
const NEWLINE: u8 = b'\n';
const COMMENT: u8 = b'#';

/// Returns true if the character code is neither a newline nor a carriage return.
pub fn not_line_ending(c: u8) -> bool {
    c != NEWLINE && c != CARRIAGE
}

/// Returns true if the character code is neither a newline, a carriage return,
/// nor a comment character.
pub fn not_line_ending_or_comment(c: u8) -> bool {
    c != NEWLINE && c != CARRIAGE && c != COMMENT
}

/// Consumes every character until a newline.
pub fn consume_newline(input: &[u8]) -> NomResult<&[u8], Option<&[u8]>> {
    let (input, _) = take_while(|i| i == CARRIAGE)(input)?;
    let (input, output) = opt(tag(b"\n"))(input)?;
    Ok((input, output))
}

/// Attempts to match `spellings` to the `input` slice.
/// Used to simplify individual directive parsers.
fn builder<'a, O, E: NomParseError<&'a [u8]>>(
    input: &'a [u8],
    spellings: impl Alt<&'a [u8], O, E>,
) -> NomResult<&'a [u8], &'a [u8]>
where
    NomErr<NomError<&'a [u8]>>: From<NomErr<E>>,
{
    // Tries to match to the spelling list.
    let (input, _) = preceded(space0, alt(spellings))(input)?;
    // Tries to match the separator (colon or spaces).
    let (input, _) = alt((preceded(space0, tag(b":")), space1))(input)?;
    // Tries to retrieve the value of the kv pair.
    let (input, line) = take_while(not_line_ending_or_comment)(input)?;

    // Skips the rest.
    let (input, _) = opt(preceded(tag(b"#"), take_while(not_line_ending)))(input)?;
    let (input, _) = consume_newline(input)?;

    let line = line.trim();
    Ok((input, line))
}

/// Attempts to parse the `user-agent` directive.
fn user_agent(input: &[u8]) -> NomResult<&[u8], Directive> {
    let spellings = (
        tag_no_case("user-agent"),
        tag_no_case("user agent"),
        tag_no_case("useragent"),
    );

    let (input, agent) = builder(input, spellings)?;
    Ok((input, Directive::UserAgent(agent)))
}

/// Attempts to parse the `allow` directive.
fn allow(input: &[u8]) -> NomResult<&[u8], Directive> {
    let spellings = (
        tag_no_case("allow"),
        tag_no_case("alow"),
        tag_no_case("allaw"),
    );

    let (input, rule) = builder(input, spellings)?;
    Ok((input, Directive::Allow(rule)))
}

/// Attempts to parse the `disallow` directive.
fn disallow(input: &[u8]) -> NomResult<&[u8], Directive> {
    let spellings = (
        tag_no_case("disallow"),
        tag_no_case("dissallow"),
        tag_no_case("dissalow"),
        tag_no_case("disalow"),
        tag_no_case("diasllow"),
        tag_no_case("disallaw"),
    );

    // Empty disallow is equivalent to allow all.
    // https://moz.com/learn/seo/robotstxt
    let (input, rule) = builder(input, spellings)?;
    if rule.is_empty() {
        Ok((input, Directive::Allow(b"/")))
    } else {
        Ok((input, Directive::Disallow(rule)))
    }
}

/// Attempts to parse the `crawl-delay` directive.
fn crawl_delay(input: &[u8]) -> NomResult<&[u8], Directive> {
    let spellings = (
        tag_no_case("crawl-delay"),
        tag_no_case("crawl delay"),
        tag_no_case("crawldelay"),
    );

    let (input, delay) = builder(input, spellings)?;
    Ok((input, Directive::CrawlDelay(delay)))
}

/// Attempts to parse the `sitemap` directive.
fn sitemap(input: &[u8]) -> NomResult<&[u8], Directive> {
    let spellings = (
        tag_no_case("sitemap"),
        tag_no_case("site-map"),
        tag_no_case("site map"),
    );

    let (input, sitemap) = builder(input, spellings)?;
    Ok((input, Directive::Sitemap(sitemap)))
}

/// Consumes the rest of the line as no directives were found here.
fn unknown(input: &[u8]) -> NomResult<&[u8], Directive> {
    let (input, unknown) = take_while(not_line_ending)(input)?;
    let (input, _) = consume_newline(input)?;
    Ok((input, Directive::Unknown(unknown)))
}

/// Parses the input slice into the list of directives.
fn lex(input: &[u8]) -> NomResult<&[u8], Vec<Directive>> {
    // Removes the byte order mark (BOM).
    let (input, _) = opt(tag(b"\xef"))(input)?;
    let (input, _) = opt(tag(b"\xbb"))(input)?;
    let (input, _) = opt(tag(b"\xbf"))(input)?;

    // Creates and runs the matcher.
    let matcher = alt((user_agent, allow, disallow, crawl_delay, sitemap, unknown));
    let (input, (directives, _)) = many_till(matcher, eof)(input)?;

    Ok((input, directives))
}

/// Parses the input slice into the list of directives.
///
/// # Safety
///
/// Discards the possibility of any error as [`unknown`] consumes anything.
pub fn into_directives(input: &[u8]) -> Vec<Directive> {
    match lex(input) {
        Ok((_, directives)) => directives,
        Err(_) => unreachable!(), // Vec::default()
    }
}

#[cfg(test)]
mod parsing {
    use super::*;

    #[test]
    fn single() {
        let r = b"user-agent: robotxt";
        let r = into_directives(r);

        let ua = b"robotxt";
        let ua = Directive::UserAgent(ua);
        assert_eq!(r, vec![ua]);
    }

    #[test]
    fn empty() {
        let r = b"
            user-agent: robotxt\n
            user-agent: robotxt";
        let r = into_directives(r);

        let ua = b"robotxt";
        let ua = Directive::UserAgent(ua);
        let em = Directive::Unknown(b"");
        assert_eq!(r, vec![em, ua, em, ua]);
    }
}
