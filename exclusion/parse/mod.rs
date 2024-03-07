use std::io::{BufReader, Read};
use std::sync::Arc;

use url::Url;

use crate::BYTE_LIMIT;
pub use access::AccessResult;
use inner::RobotsInner;

mod access;
mod inner;
mod lexer;
mod parser;
mod rule;

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
    #[cfg_attr(feature = "serde", serde(flatten))]
    inner: Arc<RobotsInner>,
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
        let inner = RobotsInner::from_bytes(robots, user_agent);
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Creates a new instance from the generic reader.
    ///
    /// ```rust
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
    /// ```rust
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
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let r = Robots::from_always(true, "foobot");
    /// assert!(r.is_relative_allowed("/example/yeah.txt"));
    /// assert!(r.is_relative_allowed("/example/nope.txt"));
    /// ```
    pub fn from_always(always: bool, user_agent: &str) -> Self {
        let inner = RobotsInner::from_always(always, None, user_agent);
        Self {
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
    /// Returns `Some(true)` if there is an explicit `allow` or the global rule.
    /// NOTE: Expects relative path.
    ///
    /// ```rust
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// assert_eq!(r.try_is_relative_allowed("/example/yeah.txt"), Some(true));
    /// assert_eq!(r.try_is_relative_allowed("/example/nope.txt"), Some(false));
    /// assert_eq!(r.try_is_relative_allowed("/invalid/path.txt"), None);
    /// ```
    pub fn try_is_relative_allowed(&self, addr: &str) -> Option<bool> {
        self.inner.try_is_allowed(addr)
    }

    /// Returns `true` if the path is allowed for the user-agent.
    /// NOTE: Expects relative path.
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
    pub fn is_relative_allowed(&self, addr: &str) -> bool {
        self.inner.is_allowed(addr)
    }

    /// Returns `Some(true)` if there is an explicit `allow` or the global rule.
    /// NOTE: Expects relative path.
    ///
    /// ```rust
    /// use url::Url;
    /// use robotxt::Robots;
    ///
    /// let txt = r#"
    ///     User-Agent: foobot
    ///     Allow: /example/
    ///     Disallow: /example/nope.txt
    /// "#.as_bytes();
    ///
    /// let r = Robots::from_bytes(txt, "foobot");
    /// let base = Url::parse("https://example.com/").unwrap();
    /// assert_eq!(r.try_is_absolute_allowed(&base.join("/example/yeah.txt").unwrap()), Some(true));
    /// assert_eq!(r.try_is_absolute_allowed(&base.join("/example/nope.txt").unwrap()), Some(false));
    /// assert_eq!(r.try_is_absolute_allowed(&base.join("/invalid/path.txt").unwrap()), None);
    /// ```
    pub fn try_is_absolute_allowed(&self, addr: &Url) -> Option<bool> {
        let path = addr.path().to_owned();

        let query = addr
            .query()
            .map(|u| "?".to_owned() + u)
            .unwrap_or("".to_owned());

        let frag = addr
            .fragment()
            .map(|u| "#".to_owned() + u)
            .unwrap_or("".to_owned());

        let relative = path + &query + &frag;
        self.inner.try_is_allowed(&relative)
    }

    /// Returns true if the path is allowed for the user-agent.
    /// NOTE: Ignores different host.
    ///
    /// ```rust
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
        self.try_is_absolute_allowed(addr).unwrap_or(true)
    }

    /// Returns `Some(_)` if the site is fully allowed or disallowed.
    ///
    /// ```rust
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
    /// ```rust
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
        self.inner.user_agent()
    }

    /// Returns the crawl-delay of the user-agent if specified.
    ///
    /// ```rust
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
    /// ```rust
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
    /// with (or optimized to) the global rule.
    pub fn len(&self) -> Option<usize> {
        self.inner.len()
    }

    /// Returns true if there are no applied rules i.e. it is constructed
    /// with (or optimized to) the global rule.
    pub fn is_empty(&self) -> Option<bool> {
        self.inner.is_empty()
    }
}
