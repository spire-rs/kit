use std::cmp::min;
use std::io::{BufReader, Read};
use std::sync::Arc;

use crate::{parse::lexer::Directive, BYTE_LIMIT};
pub use access::AccessResult;
use url::Url;

mod access;
mod inner;
mod lexer;
mod rule;

#[doc(hidden)]
#[cfg(feature = "inner")]
pub use crate::parse::inner::{RobotsInner, RobotsInnerBuilder};
#[cfg(not(feature = "inner"))]
use crate::parse::inner::{RobotsInner, RobotsInnerBuilder};

#[cfg(feature = "serde")]
use ::serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
mod serde;

/// All user agents group, used as a default for user-agents that don't have
/// an explicitly defined matching group.
///
///
/// Also see 2.2.1.The User-Agent Line.
///
/// ...
/// If no matching group exists, crawlers MUST obey the group with a
/// user-agent line with the '*' value, if present.
/// ...
///
/// If no group matches the product token and there is no group with a
/// user-agent line with the "*" value, or no groups are present at all,
/// no rules apply.
pub const ALL_UAS: &str = "*";

/// The set of directives related to the specific `user-agent` in the provided `robots.txt` file.
///
/// # Example
///
/// ```text
/// User-Agent: foobot
/// Disallow: *
/// Allow: /example/
/// Disallow: /example/nope.txt
/// ```
///
/// # Usage
///
/// ```rust
/// use robotxt::Robots;
///
/// let txt = // "...".as_bytes()
/// # r#"
/// #     User-Agent: foobot
/// #     Disallow: *
/// #     Allow: /example/
/// #     Disallow: /example/nope.txt
/// # "#.as_bytes();
/// let r = Robots::from_bytes(txt, "foobot");
/// assert!(r.is_relative_allowed("/example/yeah.txt"));
/// assert!(!r.is_relative_allowed("/example/nope.txt"));
/// assert!(!r.is_relative_allowed("/invalid/path.txt"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Robots {
    user_agent: Arc<String>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: Arc<RobotsInner>,
}

impl Robots {
    /// Finds the longest matching user-agent and if the parser should check non-assigned rules.
    fn find_agent(directives: &[Directive], user_agent: &str) -> (String, bool) {
        // Collects all uas.
        let all_uas = directives.iter().filter_map(|ua2| match ua2 {
            Directive::UserAgent(ua2) => std::str::from_utf8(ua2).ok(),
            _ => None,
        });

        // Filters out non-acceptable uas.
        let user_agent = user_agent.trim().to_lowercase();
        let acceptable_uas = all_uas
            .map(|ua| ua.trim().to_lowercase())
            .filter(|ua| user_agent.starts_with(ua.as_str()));

        // Finds the longest ua in the acceptable pool.
        let selected_ua = acceptable_uas
            .max_by(|lhs, rhs| lhs.len().cmp(&rhs.len()))
            .unwrap_or(ALL_UAS.to_string());

        // Determines if it should check non-assigned rules.
        let check_non_assigned = selected_ua == ALL_UAS;
        (selected_ua, check_non_assigned)
    }

    /// Creates a new instance from the list of directives.
    fn from_directives(directives: &[Directive], user_agent: &str) -> Self {
        fn parse_user_agent(u: &[u8]) -> Option<String> {
            let u = String::from_utf8(u.to_vec()).ok()?;
            let u = u.trim().to_lowercase();
            Some(u)
        }

        let (user_agent, mut captures_rules) = Self::find_agent(directives, user_agent);
        let mut captures_group = false;

        let mut inner = RobotsInnerBuilder::default();
        for directive in directives {
            match directive {
                Directive::UserAgent(u) => {
                    if let Some(u) = parse_user_agent(u) {
                        if !captures_group || !captures_rules {
                            captures_rules = u == user_agent;
                        }
                    }

                    captures_group = true;
                }

                Directive::Sitemap(data) => {
                    let _ = inner.try_sitemap(data);
                }

                Directive::Allow(data) | Directive::Disallow(data) => {
                    captures_group = false;
                    if captures_rules {
                        let allow = matches!(directive, Directive::Allow(_));
                        let _ = inner.try_rule(data, allow);
                    }
                }

                Directive::CrawlDelay(data) => {
                    captures_group = false;
                    if captures_rules {
                        let _ = inner.try_delay(data);
                    }
                }

                Directive::Unknown(_) => continue,
            }
        }

        Self {
            user_agent: Arc::new(user_agent),
            inner: Arc::new(inner.build()),
        }
    }
}

impl Robots {
    /// Creates a new instance from the byte slice.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Disallow: *
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(!r.is_relative_allowed("/example/nope.txt"));
    /// assert!(!r.is_relative_allowed("/invalid/path.txt"));
    /// ```
    pub fn from_bytes(robots: &[u8], user_agent: &str) -> Self {
        // Limits the input to 500 kibibytes.
        let limit = min(robots.len(), BYTE_LIMIT);
        let robots = &robots[0..limit];

        // Replaces '\x00' with '\n'.
        let robots: Vec<_> = robots
            .iter()
            .map(|u| match u {
                b'\x00' => b'\n',
                v => *v,
            })
            .collect();

        let directives = lexer::into_directives(robots.as_slice());
        Self::from_directives(directives.as_slice(), user_agent)
    }

    /// Creates a new instance from the generic reader.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// // Let's pretend it's something that actually needs a reader.
    /// // The std::io::Read trait is implemented for &[u8].
    /// let reader = r#"
    ///     User-Agent: foobot
    ///     Disallow: *
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_reader(reader, "foobot").unwrap();
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(!r.is_relative_allowed("/example/nope.txt"));
    /// assert!(!r.is_relative_allowed("/invalid/path.txt"));
    /// ```
    pub fn from_reader<R: Read>(reader: R, user_agent: &str) -> Result<Self, std::io::Error> {
        let reader = reader.take(BYTE_LIMIT as u64);
        let mut reader = BufReader::new(reader);

        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let robots = buffer.as_slice();
        Ok(Self::from_bytes(robots, user_agent))
    }

    /// Creates a new instance from the `AccessResult`.
    ///
    /// ```
    /// use robotxt::{AccessResult, Robots};
    ///
    /// let r = Robots::from_access(AccessResult::Redirect, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(r.is_relative_allowed("/example/nope.txt"));
    ///
    /// let r = Robots::from_access(AccessResult::Unavailable, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(r.is_relative_allowed("/example/nope.txt"));
    ///
    /// let r = Robots::from_access(AccessResult::Unreachable, "foobot");
    /// assert!(!r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(!r.is_relative_allowed("/example/nope.txt"));
    /// ```
    pub fn from_access(access: AccessResult, user_agent: &str) -> Self {
        use AccessResult as AR;
        match access {
            AR::Successful(txt) => Self::from_bytes(txt, user_agent),
            AR::Redirect | AR::Unavailable => Self::from_always(true, user_agent),
            AR::Unreachable => Self::from_always(false, user_agent),
        }
    }

    /// Creates a new instance from the global rule.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// let r = Robots::from_always(true, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(r.is_relative_allowed("/example/nope.txt"));
    /// ```
    pub fn from_always(always: bool, user_agent: &str) -> Self {
        let inner = RobotsInner::new(always, None);
        Self {
            user_agent: Arc::new(user_agent.to_string()),
            inner: Arc::new(inner),
        }
    }

    /// Creates a new builder with default settings.
    /// See [`RobotsBuilder::new`].
    ///
    /// [`RobotsBuilder::new`]: crate::RobotsBuilder::new
    #[cfg(feature = "builder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
    pub fn builder() -> crate::RobotsBuilder {
        crate::RobotsBuilder::new()
    }
}

impl Robots {
    /// Returns true if the path is allowed for the user-agent.
    /// NOTE: Expects relative path.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Disallow: *
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(!r.is_relative_allowed("/example/nope.txt"));
    /// assert!(!r.is_relative_allowed("/invalid/path.txt"));
    /// ```
    pub fn is_relative_allowed(&self, addr: &str) -> bool {
        self.inner.is_allowed(addr)
    }

    /// Returns true if the path is allowed for the user-agent.
    /// NOTE: Ignores different host.
    ///
    /// ```
    /// use url::Url;
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Disallow: *
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// let base = Url::parse("https://example.com/").unwrap();
    /// assert!(r.is_absolute_allowed(&base.join("/example/yeah.txt").unwrap()));
    /// assert!(!r.is_absolute_allowed(&base.join("/example/nope.txt").unwrap()));
    /// assert!(!r.is_absolute_allowed(&base.join("/invalid/path.txt").unwrap()));
    /// ```
    pub fn is_absolute_allowed(&self, addr: &Url) -> bool {
        let path = addr.path().to_owned();
        let query = addr.query().map(|u| "?".to_owned() + u);
        let query = query.unwrap_or("".to_owned());
        let frag = addr.fragment().map(|u| "#".to_owned() + u);
        let frag = frag.unwrap_or("".to_owned());

        let rel = path + &query + &frag;
        self.is_relative_allowed(&rel)
    }

    /// Returns `Some(_)` if the site is fully allowed or disallowed.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// let r = Robots::from_always(true, "foobot");
    /// assert_eq!(r.is_always(), Some(true));
    ///
    /// let r = Robots::from_always(false, "foobot");
    /// assert_eq!(r.is_always(), Some(false));
    /// ```
    pub fn is_always(&self) -> Option<bool> {
        self.inner.is_always()
    }

    /// Returns the longest matching user-agent.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foo
    ///     User-Agent: foobot
    ///     User-Agent: foobot-images
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot-search");
    /// assert_eq!(r.user_agent(), "foobot");
    /// ```
    pub fn user_agent(&self) -> &str {
        self.user_agent.as_ref()
    }

    /// Returns the crawl-delay of the user-agent if specified.
    ///
    /// ```
    /// use std::time::Duration;
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Crawl-Delay: 5
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// assert_eq!(r.crawl_delay(), Some(Duration::from_secs(5)));
    /// ```
    pub fn crawl_delay(&self) -> Option<std::time::Duration> {
        self.inner.crawl_delay()
    }

    /// Returns all collected sitemaps.
    ///
    /// ```
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     Sitemap: https://example.com/sitemap_1.xml
    ///     Sitemap: https://example.com/sitemap_2.xml
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// assert_eq!(r.sitemaps().len(), 2);
    /// ```
    pub fn sitemaps(&self) -> &[Url] {
        self.inner.sitemaps()
    }

    /// Returns the total amount of applied rules unless constructed
    /// with (or reduced to) the global rule.
    pub fn len(&self) -> Option<usize> {
        self.inner.len()
    }

    /// Returns true if there are no applied rules i.e. it is constructed
    /// with (or reduced to) the global rule.
    pub fn is_empty(&self) -> Option<bool> {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod precedence {
    use super::*;

    static DIRECTIVES: &[Directive] = &[
        Directive::UserAgent(b"bot-robotxt"),
        Directive::Allow(b"/1"),
        Directive::Disallow(b"/"),
        Directive::UserAgent(b"*"),
        Directive::Allow(b"/2"),
        Directive::Disallow(b"/"),
        Directive::UserAgent(b"bot"),
        Directive::Allow(b"/3"),
        Directive::Disallow(b"/"),
    ];

    #[test]
    fn specific() {
        let r = Robots::from_directives(DIRECTIVES, "bot-robotxt");

        // Matches:
        assert!(r.is_relative_allowed("/1"));

        // Doesn't match:
        assert!(!r.is_relative_allowed("/2"));
        assert!(!r.is_relative_allowed("/3"));
    }

    #[test]
    fn strict() {
        let r = Robots::from_directives(DIRECTIVES, "bot");

        // Matches:
        assert!(r.is_relative_allowed("/3"));

        // Doesn't match:
        assert!(!r.is_relative_allowed("/1"));
        assert!(!r.is_relative_allowed("/2"));
    }

    #[test]
    fn missing() {
        let r = Robots::from_directives(DIRECTIVES, "super-bot");

        // Matches:
        assert!(r.is_relative_allowed("/2"));

        // Doesn't match:
        assert!(!r.is_relative_allowed("/1"));
        assert!(!r.is_relative_allowed("/3"));
    }

    #[test]
    fn partial() {
        let r = Robots::from_directives(DIRECTIVES, "bot-super");

        // Matches:
        assert!(r.is_relative_allowed("/3"));

        // Doesn't match:
        assert!(!r.is_relative_allowed("/1"));
        assert!(!r.is_relative_allowed("/2"));
    }
}
