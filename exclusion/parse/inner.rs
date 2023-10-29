use std::cmp::min;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use url::Url;

use crate::parse::lexer::{into_directives, Directive};
use crate::parse::rule::Rule;
use crate::paths::normalize_path;
use crate::{ALL_UAS, BYTE_LIMIT};

#[derive(Debug, Default)]
struct Builder {
    captures_group: bool,
    captures_rules: bool,

    pub longest_match: String,
    pub rules: Vec<Rule>,
    pub crawl_delay: Option<Duration>,
    pub sitemaps: Vec<Url>,
}

impl Builder {
    /// Creates a new [`Builder`] from the list of directives.
    pub fn from_directives(directives: &[Directive], user_agent: &str) -> Self {
        let mut state = Builder::from_longest_match(directives, user_agent);

        for directive in directives {
            match directive {
                Directive::UserAgent(user_agent) => {
                    if let Some(user_agent) = Self::try_user_agent(user_agent) {
                        if !state.captures_group || !state.captures_rules {
                            state.captures_rules = user_agent == state.longest_match;
                        }
                    }

                    state.captures_group = true;
                }

                Directive::Allow(data) | Directive::Disallow(data) => {
                    state.captures_group = false;
                    if state.captures_rules {
                        let allow = matches!(directive, Directive::Allow(_));
                        state.try_rule(data, allow);
                    }
                }

                Directive::CrawlDelay(data) => {
                    state.captures_group = false;
                    if state.captures_rules {
                        state.try_delay(data);
                    }
                }

                Directive::Sitemap(data) => state.try_sitemap(data),
                Directive::Unknown(_) => continue,
            }
        }

        state
    }

    /// Creates initial [`Builder`] state from the list of directives.
    fn from_longest_match(directives: &[Directive], user_agent: &str) -> Self {
        let (longest_match, captures_rules) = Self::find_longest_match(directives, user_agent);
        Self {
            longest_match,
            captures_rules,
            ..Self::default()
        }
    }

    /// Finds the longest matching user-agent and if the parser should check non-assigned rules.
    fn find_longest_match(directives: &[Directive], user_agent: &str) -> (String, bool) {
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

    /// Attempts to parse the `User-Agent`.
    fn try_user_agent(data: &[u8]) -> Option<String> {
        let data = String::from_utf8(data.to_vec()).ok()?;
        Some(data.trim().to_lowercase())
    }

    /// Attempts to parse and store the valid matching `Rule`.
    pub fn try_rule(&mut self, data: &[u8], allow: bool) {
        let data = String::from_utf8(data.to_vec()).ok();
        let rule = data.map(|data| Rule::new(&data, allow).ok());
        if let Some(rule) = rule.flatten() {
            self.rules.push(rule);
        }
    }

    /// Attempts to parse and store the valid `Duration` as a `crawl-delay`.
    pub fn try_delay(&mut self, data: &[u8]) {
        let data = String::from_utf8(data.to_vec()).ok();
        self.crawl_delay = data
            .and_then(|data| data.parse::<f64>().ok())
            .and_then(|secs| Duration::try_from_secs_f64(secs).ok())
            .map(|curr| (self.crawl_delay.unwrap_or(curr), curr))
            .map(|(prev, curr)| prev.min(curr));
    }

    /// Attempts to parse and store the valid `Url` address as a `sitemap`.
    pub fn try_sitemap(&mut self, data: &[u8]) {
        let data = String::from_utf8(data.to_vec()).ok();
        let addr = data.and_then(|data| Url::parse(data.as_str()).ok());
        if let Some(addr) = addr {
            self.sitemaps.push(addr);
        }
    }
}

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

        let directives = into_directives(robots.as_slice());
        Self::from_directives(directives.as_slice(), user_agent)
    }

    /// Creates a new [`RobotsInner`] from the list of directives.
    // TODO: Remove overlapping rules.
    fn from_directives(directives: &[Directive], user_agent: &str) -> Self {
        let mut state = Builder::from_directives(directives, user_agent);

        // Rules are sorted by length and permission i.e.
        // 5 > 4, 5 allow > 5 disallow.
        state.rules.sort();

        Self {
            user_agent: state.longest_match,
            rules: Self::optimize(state.rules),
            crawl_delay: state.crawl_delay,
            sitemaps: state.sitemaps,
        }
    }

    // TODO: Desc.
    fn optimize(rules: Vec<Rule>) -> Rules {
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

    /// Returns `Some(true)` if there is an `allow` or global rule.
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
    /// with (or reduced to) the global rule.
    pub fn len(&self) -> Option<usize> {
        match &self.rules {
            Rules::Rules(vec) => Some(vec.len()),
            Rules::Always(_) => None,
        }
    }

    /// Returns true if there are no applied rules i.e. it is constructed
    /// with (or reduced to) the global rule.
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|len| len == 0)
    }
}

#[cfg(test)]
#[cfg(feature = "optimal")]
mod optimal_output {
    use super::*;

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

    fn create(allow: &str, disallow: &str) -> RobotsInner {
        let txt = format!("Allow: {} \n Disallow: {}", allow, disallow);
        RobotsInner::from_bytes(txt.as_bytes(), ALL_UAS)
    }

    #[test]
    fn simple() {
        let r = create("/p", "/");
        assert!(r.is_allowed("/page"));
    }

    #[test]
    fn restrictive() {
        let r = create("/folder", "/folder");
        assert!(r.is_allowed("/folder/page"));
    }

    #[test]
    fn restrictive2() {
        let r = create("/page", "/*.ph");
        assert!(r.is_allowed("/page.php5"));
    }

    #[test]
    fn longer() {
        let r = create("/page", "/*.htm");
        assert!(!r.is_allowed("/page.htm"));
    }

    #[test]
    fn specific() {
        let r = create("/$", "/");
        assert!(r.is_allowed("/"));
    }

    #[test]
    fn specific2() {
        let r = create("/$", "/");
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
