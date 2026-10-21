//! The rules that apply to a single crawler.

use std::io::{self, Read};
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use url::{Position, Url};

use crate::ANY_AGENT;
use crate::parse::access::AccessResult;
use crate::parse::agent::Agent;
use crate::parse::lexer::tokenize;
use crate::parse::parser;
use crate::parse::rule::Rule;
use crate::path::normalize_path;

/// The maximum number of bytes of a `robots.txt` file that are parsed.
///
/// RFC 9309 requires crawlers to parse at least 500 kibibytes (KiB), which is
/// also the limit Google enforces. The rest of the file is ignored, including
/// the line that crosses the limit, so a cut rule never applies.
pub const BYTE_LIMIT: usize = 500 * 1024;

/// Cuts the file at the last line ending within [`BYTE_LIMIT`] bytes.
fn truncate(txt: &[u8]) -> &[u8] {
    // A line ending right at the limit still completes the last line.
    let Some(window) = txt.get(..=BYTE_LIMIT) else {
        return txt;
    };

    let end = window
        .iter()
        .rposition(|byte| matches!(byte, b'\n' | b'\r' | b'\0'));
    &txt[..end.unwrap_or(0)]
}

/// The rules of a `robots.txt` file that apply to a single crawler.
///
/// The file is parsed once for the crawler: the groups whose `User-Agent`
/// matches its name are merged, or the [`ANY_AGENT`] group is used if there
/// are none, and every other group is discarded.
///
/// ```rust
/// use robotxt::Robots;
///
/// let txt = b"
///     User-Agent: foobot
///     Disallow: *
///     Allow: /example/
///     Disallow: /example/nope.txt
/// ";
///
/// let r = Robots::parse(txt, "foobot");
/// assert!(r.is_allowed("/example/yeah.txt"));
/// assert!(!r.is_allowed("/example/nope.txt"));
/// assert!(!r.is_allowed("/invalid/path.txt"));
/// ```
///
/// [`ANY_AGENT`]: crate::ANY_AGENT
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "Unchecked"))]
pub struct Robots {
    user_agent: String,
    rules: Vec<Rule>,
    #[cfg_attr(feature = "serde", serde(skip))]
    always: Option<bool>,
    crawl_delay: Option<Duration>,
    sitemaps: Vec<Url>,
}

/// The serialized [`Robots`], which may have been edited or written by an
/// older version, so its rules are normalized and sorted again.
#[cfg(feature = "serde")]
#[derive(Deserialize)]
struct Unchecked {
    user_agent: String,
    rules: Vec<Rule>,
    crawl_delay: Option<Duration>,
    sitemaps: Vec<Url>,
}

#[cfg(feature = "serde")]
impl From<Unchecked> for Robots {
    fn from(robots: Unchecked) -> Self {
        let rules = robots.rules.iter();
        let rules =
            rules.filter_map(|rule| Rule::parse(rule.pattern().as_bytes(), rule.is_allow()));
        let rules = rules.collect();
        Self::new(
            robots.user_agent,
            rules,
            robots.crawl_delay,
            robots.sitemaps,
        )
    }
}

impl Robots {
    /// Creates the rules, putting the rule that wins first.
    pub(crate) fn new(
        user_agent: String,
        mut rules: Vec<Rule>,
        crawl_delay: Option<Duration>,
        sitemaps: Vec<Url>,
    ) -> Self {
        rules.sort_by_key(Rule::precedence);
        Self {
            user_agent,
            always: Self::always_of(&rules),
            rules,
            crawl_delay,
            sitemaps,
        }
    }

    /// Parses the rules that apply to the crawler.
    /// A plain `&str` name is matched exactly, see [`Agent`].
    ///
    /// Only the first [`BYTE_LIMIT`] bytes are parsed.
    pub fn parse<'a>(txt: &[u8], agent: impl Into<Agent<'a>>) -> Self {
        parser::parse(&tokenize(truncate(txt)), agent.into())
    }

    /// Reads and parses the rules that apply to the crawler.
    /// A plain `&str` name is matched exactly, see [`Agent`].
    ///
    /// Reading stops after [`BYTE_LIMIT`] bytes, see [`Robots::parse`].
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn from_reader<'a, R: Read>(reader: R, agent: impl Into<Agent<'a>>) -> io::Result<Self> {
        // One byte past the limit tells whether the last line was cut.
        let mut txt = Vec::new();
        let limit = u64::try_from(BYTE_LIMIT + 1).unwrap_or(u64::MAX);
        reader.take(limit).read_to_end(&mut txt)?;
        Ok(Self::parse(&txt, agent))
    }

    /// Creates the rules from the result of fetching the file, as described
    /// in RFC 9309, section 2.3.1.
    ///
    /// ```rust
    /// use robotxt::{AccessResult, Robots};
    ///
    /// let r = Robots::from_access(AccessResult::Unavailable, "foobot");
    /// assert!(r.is_allowed("/example"));
    ///
    /// let r = Robots::from_access(AccessResult::Unreachable, "foobot");
    /// assert!(!r.is_allowed("/example"));
    /// ```
    pub fn from_access<'a>(access: AccessResult<'_>, agent: impl Into<Agent<'a>>) -> Self {
        match access {
            AccessResult::Successful(txt) => Self::parse(txt, agent),
            AccessResult::TooManyRedirects | AccessResult::Unavailable => Self::allow_all(),
            AccessResult::Unreachable => Self::disallow_all(),
        }
    }

    /// Creates the rules that allow every URL.
    pub fn allow_all() -> Self {
        Self::new(ANY_AGENT.to_owned(), Vec::new(), None, Vec::new())
    }

    /// Creates the rules that disallow every URL but `/robots.txt`.
    pub fn disallow_all() -> Self {
        let rules = vec![Rule::disallow_all()];
        Self::new(ANY_AGENT.to_owned(), rules, None, Vec::new())
    }

    /// Returns `Some(_)` if the rules allow or disallow every URL alike.
    fn always_of(rules: &[Rule]) -> Option<bool> {
        if rules.iter().all(Rule::is_allow) {
            Some(true)
        } else if rules.iter().all(|rule| !rule.is_allow()) && rules.iter().any(Rule::is_universal)
        {
            Some(false)
        } else {
            None
        }
    }

    /// Creates a new builder of `robots.txt` files.
    /// See [`RobotsBuilder`].
    ///
    /// [`RobotsBuilder`]: crate::RobotsBuilder
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    pub fn builder() -> crate::RobotsBuilder {
        crate::RobotsBuilder::new()
    }
}

impl Robots {
    /// Returns true if the crawler may fetch the relative URL (path and
    /// query). The fragment is ignored, and `/robots.txt` is always allowed.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let txt = b"
    ///     User-Agent: *
    ///     Disallow: /private
    /// ";
    ///
    /// let r = Robots::parse(txt, "foobot");
    /// assert!(r.is_allowed("/public?q=1"));
    /// assert!(!r.is_allowed("/private/page.html"));
    /// assert!(r.is_allowed("/robots.txt"));
    /// ```
    #[must_use]
    pub fn is_allowed(&self, path: &str) -> bool {
        let path = normalize_path(path);
        if path == "/robots.txt" {
            return true;
        }

        if let Some(always) = self.always {
            return always;
        }

        self.find_rule(&path).is_none_or(Rule::is_allow)
    }

    /// Returns true if the crawler may fetch the URL.
    ///
    /// Only the path and query are matched: the rules are expected to come
    /// from the `robots.txt` file of the URL's host, see [`robots_url`].
    ///
    /// ```rust
    /// use url::Url;
    /// use robotxt::Robots;
    ///
    /// let r = Robots::parse(b"User-Agent: *\nDisallow: /private", "foobot");
    /// let url = Url::parse("https://example.com/private#public").unwrap();
    /// assert!(!r.is_url_allowed(&url));
    /// ```
    ///
    /// [`robots_url`]: crate::robots_url
    #[must_use]
    pub fn is_url_allowed(&self, url: &Url) -> bool {
        self.is_allowed(&url[Position::BeforePath..Position::AfterQuery])
    }

    /// Returns the rule that decides whether the relative URL may be
    /// fetched, or `None` if no rule matches it.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let txt = b"
    ///     User-Agent: *
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// ";
    ///
    /// let r = Robots::parse(txt, "foobot");
    /// let rule = r.matching_rule("/example/nope.txt").unwrap();
    /// assert_eq!(rule.pattern(), "/example/nope.txt");
    /// assert!(!rule.is_allow());
    /// assert!(r.matching_rule("/other").is_none());
    /// ```
    #[must_use]
    pub fn matching_rule(&self, path: &str) -> Option<&Rule> {
        self.find_rule(&normalize_path(path))
    }

    fn find_rule(&self, path: &str) -> Option<&Rule> {
        self.rules.iter().find(|rule| rule.matches(path))
    }

    /// Returns `Some(true)` if every URL is allowed, `Some(false)` if every
    /// URL but `/robots.txt` is disallowed, or `None` otherwise.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let r = Robots::parse(b"User-Agent: *\nDisallow: /", "foobot");
    /// assert_eq!(r.always(), Some(false));
    /// assert_eq!(Robots::allow_all().always(), Some(true));
    /// ```
    #[must_use]
    pub const fn always(&self) -> Option<bool> {
        self.always
    }

    /// Returns the normalized `User-Agent` value of the groups that apply
    /// to the crawler, or [`ANY_AGENT`] if no other group matches.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let r = Robots::parse(b"User-Agent: FooBot\nDisallow: /", "foobot");
    /// assert_eq!(r.user_agent(), "foobot");
    /// ```
    ///
    /// [`ANY_AGENT`]: crate::ANY_AGENT
    #[must_use]
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns the rules that apply to the crawler, in the order of precedence.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Returns the smallest `Crawl-Delay` of the groups that apply to the crawler.
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use robotxt::Robots;
    ///
    /// let r = Robots::parse(b"User-Agent: foobot\nCrawl-Delay: 5", "foobot");
    /// assert_eq!(r.crawl_delay(), Some(Duration::from_secs(5)));
    /// ```
    #[must_use]
    pub const fn crawl_delay(&self) -> Option<Duration> {
        self.crawl_delay
    }

    /// Returns every valid `Sitemap` URL of the file, regardless of groups.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let txt = b"
    ///     Sitemap: https://example.com/sitemap_1.xml
    ///     Sitemap: https://example.com/sitemap_2.xml
    /// ";
    ///
    /// let r = Robots::parse(txt, "foobot");
    /// assert_eq!(r.sitemaps().len(), 2);
    /// ```
    #[must_use]
    pub fn sitemaps(&self) -> &[Url] {
        &self.sitemaps
    }
}
