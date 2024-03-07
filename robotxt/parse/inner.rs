use std::cmp::min;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use url::Url;

use crate::parse::lexer::Lexer;
use crate::parse::parser::Parser;
use crate::parse::rule::Rule;
use crate::paths::normalize_path;
use crate::BYTE_LIMIT;

/// The [`Rules`] enum determines if the [RobotsInner::is_allowed] results
/// from the set of [`Rule`]s or the single provided global rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Rules {
    Rules(Vec<Rule>),
    Always(bool),
}

/// The [`RobotsInner`] struct provides convenient and efficient storage for
/// the data associated with certain user-agent for further matching.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RobotsInner {
    user_agent: String,
    #[cfg_attr(feature = "serde", serde(flatten))]
    rules: Rules,
    crawl_delay: Option<Duration>,
    sitemaps: Vec<Url>,
}

impl RobotsInner {
    /// Creates a new [`RobotsInner`] from the byte slice.
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

        let directives = Lexer::parse_tokens(&robots);
        let state = Parser::parse_rules(&directives, user_agent);

        Self {
            user_agent: state.longest_match,
            rules: Self::optimize(state.rules),
            crawl_delay: state.crawl_delay,
            sitemaps: state.sitemaps,
        }
    }

    // Applies optimizations if enabled.
    fn optimize(rules: Vec<Rule>) -> Rules {
        // TODO: Remove overlapping rules.

        #[cfg(feature = "optimal")]
        if rules.is_empty() || rules.iter().all(|r| r.is_allowed()) {
            // Empty or all allow.
            return Rules::Always(true);
        } else if rules.iter().all(|r| !r.is_allowed())
            && rules.iter().rev().any(|r| r.is_universal())
        {
            // All disallow + universal disallow.
            // Universal rule should be one of the smallest, so reverse the iter.
            return Rules::Always(false);
        }

        Rules::Rules(rules)
    }

    /// Creates a new [`RobotsInner`] from the global rule.
    pub fn from_always(always: bool, crawl_delay: Option<Duration>, user_agent: &str) -> Self {
        Self {
            user_agent: user_agent.to_string(),
            rules: Rules::Always(always),
            crawl_delay,
            sitemaps: Vec::default(),
        }
    }

    /// Returns `Some(true)` if there is an explicit `allow` or the global rule.
    /// NOTE: Expects relative path.
    pub fn try_is_allowed(&self, path: &str) -> Option<bool> {
        match self.rules {
            Rules::Always(always) => Some(always),
            Rules::Rules(ref rules) => match normalize_path(path).as_str() {
                "/robots.txt" => Some(true),
                path => rules
                    .iter()
                    .find(|r| r.is_match(path))
                    .map(|rule| rule.is_allowed()),
            },
        }
    }

    /// Returns true if the relative path is allowed for this set of rules.
    /// NOTE: Expects relative path.
    pub fn is_allowed(&self, path: &str) -> bool {
        // Returns true is there is no rule matching the path.
        self.try_is_allowed(path).unwrap_or(true)
    }

    /// Returns `Some(_)` if the rules fully allow or disallow.
    pub fn is_always(&self) -> Option<bool> {
        match &self.rules {
            Rules::Rules(_) => None,
            Rules::Always(always) => Some(*always),
        }
    }

    /// Returns the longest matching user-agent.
    pub fn user_agent(&self) -> &str {
        self.user_agent.as_ref()
    }

    /// Returns the specified crawl-delay.
    pub fn crawl_delay(&self) -> Option<Duration> {
        self.crawl_delay
    }

    /// Returns all collected sitemaps.
    pub fn sitemaps(&self) -> &[Url] {
        self.sitemaps.as_slice()
    }

    /// Returns the total amount of applied rules unless constructed
    /// with (or optimized to) the global rule.
    pub fn len(&self) -> Option<usize> {
        match &self.rules {
            Rules::Rules(vec) => Some(vec.len()),
            Rules::Always(_) => None,
        }
    }

    /// Returns true if there are no applied rules i.e. it is constructed
    /// with (or optimized to) the global rule.
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|len| len == 0)
    }
}

#[cfg(test)]
#[cfg(feature = "optimal")]
mod optimal_output {
    use super::*;
    use crate::ALL_UAS;

    #[test]
    fn from() {
        let r = RobotsInner::from_always(true, None, "foo");
        assert_eq!(r.is_always(), Some(true));
        let r = RobotsInner::from_always(false, None, "foo");
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn empty() {
        let r = RobotsInner::from_bytes(b"", ALL_UAS);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn allow() {
        let t = b"Allow: / \n Allow: /foo";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn disallow_all() {
        let t = b"Disallow: /* \n Disallow: /foo";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn disallow_exc() {
        let t = b"Disallow: /* \n Allow: /foo";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert_eq!(r.is_always(), None);
    }
}

#[cfg(test)]
mod precedence_rules {
    use super::*;
    use crate::ALL_UAS;

    #[test]
    fn simple() {
        let t = b"Allow: /p \n Disallow: /";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(r.is_allowed("/page"));
    }

    #[test]
    fn restrictive() {
        let t = b"Allow: /folder \n Disallow: /folder";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(r.is_allowed("/folder/page"));
    }

    #[test]
    fn restrictive2() {
        let t = b"Allow: /page \n Disallow: /*.ph";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(r.is_allowed("/page.php5"));
    }

    #[test]
    fn longer() {
        let t = b"Allow: /page \n Disallow: /*.htm";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(!r.is_allowed("/page.htm"));
    }

    #[test]
    fn specific() {
        let t = b"Allow: /$ \n Disallow: /";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(r.is_allowed("/"));
    }

    #[test]
    fn specific2() {
        let t = b"Allow: /$ \n Disallow: /";
        let r = RobotsInner::from_bytes(t, ALL_UAS);
        assert!(!r.is_allowed("/page.htm"));
    }
}

#[cfg(test)]
mod precedence_agents {
    use super::*;

    static TXT: &[u8] = br#"""
        User-Agent: bot-robotxt
        Allow: /1
        Disallow: /

        User-Agent: *
        Allow: /2
        Disallow: /

        User-Agent: bot
        Allow: /3
        Disallow: /
    """#;

    #[test]
    fn specific() {
        let r = RobotsInner::from_bytes(TXT, "bot-robotxt");

        // Matches:
        assert!(r.is_allowed("/1"));

        // Doesn't match:
        assert!(!r.is_allowed("/2"));
        assert!(!r.is_allowed("/3"));
    }

    #[test]
    fn strict() {
        let r = RobotsInner::from_bytes(TXT, "bot");

        // Matches:
        assert!(r.is_allowed("/3"));

        // Doesn't match:
        assert!(!r.is_allowed("/1"));
        assert!(!r.is_allowed("/2"));
    }

    #[test]
    fn missing() {
        let r = RobotsInner::from_bytes(TXT, "super-bot");

        // Matches:
        assert!(r.is_allowed("/2"));

        // Doesn't match:
        assert!(!r.is_allowed("/1"));
        assert!(!r.is_allowed("/3"));
    }

    #[test]
    fn partial() {
        let r = RobotsInner::from_bytes(TXT, "bot-super");

        // Matches:
        assert!(r.is_allowed("/3"));

        // Doesn't match:
        assert!(!r.is_allowed("/1"));
        assert!(!r.is_allowed("/2"));
    }
}
