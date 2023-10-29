use std::cmp::min;
use std::time::Duration;
use url::Url;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::normalize_path;
use crate::parse::rule::Rule;

#[derive(Debug, Clone, Default)]
pub struct RobotsInnerBuilder {
    rules: Vec<Rule>,
    delay: Option<Duration>,
    sitemaps: Vec<Url>,
}

impl RobotsInnerBuilder {
    /// Attempts to parse and store the valid matching `Rule`.
    pub fn try_rule(&mut self, data: &[u8], allow: bool) -> Option<()> {
        let data = String::from_utf8(data.to_vec()).ok()?;
        let rule = Rule::new(data.as_str(), allow).ok()?;
        self.rules.push(rule);
        Some(())
    }

    /// Attempts to parse and store the valid `Duration` as a `crawl-delay`.
    pub fn try_delay(&mut self, data: &[u8]) -> Option<()> {
        let data = String::from_utf8(data.to_vec()).ok()?;
        let seconds = data.parse::<f64>().ok()?;
        let new = Duration::try_from_secs_f64(seconds).ok()?;
        let min = self.delay.map(|now| min(now, new));
        self.delay = min.or(Some(new));
        Some(())
    }

    /// Attempts to parse and store the valid `Url` address as a `sitemap`.
    pub fn try_sitemap(&mut self, data: &[u8]) -> Option<()> {
        let data = String::from_utf8(data.to_vec()).ok()?;
        let addr = Url::parse(data.as_str()).ok()?;
        self.sitemaps.push(addr);
        Some(())
    }

    /// Creates a new [`RobotsInner`].
    pub fn build(self) -> RobotsInner {
        RobotsInner::build(self.rules, self.delay).with_sitemap(self.sitemaps)
    }
}

/// The [`AlwaysRules`] enum determines if the [RobotsInner::is_allowed] results
/// from the set of [`Rule`]s or the single provided global rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlwaysRules {
    Rules(Vec<Rule>),
    Always(bool),
}

// TODO: Wrap into Arc in Robots.

/// The [`RobotsInner`] struct provides convenient and efficient storage for
/// the data associated with certain user-agent for further matching.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RobotsInner {
    #[cfg_attr(feature = "serde", serde(flatten))]
    rules: AlwaysRules,
    delay: Option<Duration>,
    sitemaps: Vec<Url>,
}

impl RobotsInner {
    /// Creates a new `[RobotsInner`] with the given data.
    pub fn build(mut rules: Vec<Rule>, delay: Option<Duration>) -> Self {
        // TODO: Remove overlapping rules.

        // Empty or all allow.
        #[cfg(feature = "optimal")]
        if rules.is_empty() || rules.iter().all(|r| r.is_allowed()) {
            return Self::new(true, delay);
        }

        // Rules are sorted by length and permission i.e.
        // 5 > 4, 5 allow > 5 disallow.
        rules.sort();

        // All disallow + universal disallow.
        // Universal rule should be one of the smallest, so reverse the iter.
        #[cfg(feature = "optimal")]
        if rules.iter().all(|r| !r.is_allowed()) && rules.iter().rev().any(|r| r.is_universal()) {
            return Self::new(false, delay);
        }

        Self {
            rules: AlwaysRules::Rules(rules),
            delay,
            sitemaps: Vec::default(),
        }
    }

    /// Adds the given set of sitemaps to the [`RobotsInner`].
    pub fn with_sitemap(mut self, sitemaps: Vec<Url>) -> Self {
        // TODO: Deduplicate and merge.
        self.sitemaps = sitemaps;
        self
    }

    /// Creates a new [`RobotsInner`] from the global rule.
    pub fn new(always: bool, delay: Option<Duration>) -> Self {
        Self {
            rules: AlwaysRules::Always(always),
            delay,
            sitemaps: Vec::default(),
        }
    }

    /// Returns `Some(true)` if there is an `allow` or global rule.
    /// NOTE: Expects relative path.
    pub fn try_is_allowed(&self, path: &str) -> Option<bool> {
        match self.rules {
            AlwaysRules::Always(always) => Some(always),
            AlwaysRules::Rules(ref rules) => match normalize_path(path).as_str() {
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
            AlwaysRules::Rules(_) => None,
            AlwaysRules::Always(always) => Some(*always),
        }
    }

    /// Returns the specified crawl-delay.
    pub fn crawl_delay(&self) -> Option<Duration> {
        self.delay
    }

    /// Returns all collected sitemaps.
    pub fn sitemaps(&self) -> &[Url] {
        self.sitemaps.as_slice()
    }

    /// Returns the total amount of applied rules unless constructed
    /// with (or reduced to) the global rule.
    pub fn len(&self) -> Option<usize> {
        match &self.rules {
            AlwaysRules::Rules(vec) => Some(vec.len()),
            AlwaysRules::Always(_) => None,
        }
    }

    /// Returns true if there are no applied rules i.e. it is constructed
    /// with (or reduced to) the global rule.
    pub fn is_empty(&self) -> Option<bool> {
        self.len().map(|len| len == 0)
    }
}

#[cfg(feature = "optimal")]
#[cfg(test)]
mod always {
    use super::*;

    #[test]
    fn from() {
        let r = RobotsInner::new(true, None);
        assert_eq!(r.is_always(), Some(true));
        let r = RobotsInner::new(false, None);
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn empty() {
        let r = RobotsInner::build(Vec::new(), None);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn allow() {
        let r = Rule::new("/", true).unwrap();
        let r = RobotsInner::build(vec![r.clone(), r.clone()], None);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn disallow_ok() {
        let universal = Rule::new("/*", false).unwrap();
        let disallow = Rule::new("/foo", false).unwrap();
        let r = RobotsInner::build(vec![universal, disallow], None);
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn disallow_err() {
        let universal = Rule::new("/*", false).unwrap();
        let allow = Rule::new("/foo", true).unwrap();
        let r = RobotsInner::build(vec![universal, allow], None);
        assert_eq!(r.is_always(), None);
    }
}

#[cfg(test)]
mod precedence {
    use super::*;

    #[test]
    fn simple() {
        let allow = Rule::new("/p", true).unwrap();
        let disallow = Rule::new("/", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(rules.is_allowed("/page"));
    }

    #[test]
    fn restrictive() {
        let allow = Rule::new("/folder", true).unwrap();
        let disallow = Rule::new("/folder", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(rules.is_allowed("/folder/page"));
    }

    #[test]
    fn restrictive2() {
        let allow = Rule::new("/page", true).unwrap();
        let disallow = Rule::new("/*.ph", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(rules.is_allowed("/page.php5"));
    }

    #[test]
    fn longer() {
        let allow = Rule::new("/page", true).unwrap();
        let disallow = Rule::new("/*.htm", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(!rules.is_allowed("/page.htm"));
    }

    #[test]
    fn specific() {
        let allow = Rule::new("/$", true).unwrap();
        let disallow = Rule::new("/", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(rules.is_allowed("/"));
    }

    #[test]
    fn specific2() {
        let allow = Rule::new("/$", true).unwrap();
        let disallow = Rule::new("/", false).unwrap();
        let rules = RobotsInner::build(vec![allow, disallow], None);

        assert!(!rules.is_allowed("/page.htm"));
    }
}
