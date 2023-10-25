use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::normalize_path;
use crate::parse::rule::Rule;

/// The [`AlwaysRules`] enum determines if the [Rules::is_allowed] results
/// from the set of [`Rule`]s or the single provided global rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlwaysRules {
    Rules(Vec<Rule>),
    Always(bool),
}

/// The [`Rules`] struct provides convenient and efficient storage for
/// the data associated with certain user-agent for further matching.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rules {
    #[cfg_attr(feature = "serde", serde(flatten))]
    rules: AlwaysRules,
    delay: Option<Duration>,
}

impl Rules {
    /// Creates a new `Rules` with the specified rules and delay.
    pub fn from_rules(mut rules: Vec<Rule>, delay: Option<Duration>) -> Self {
        // TODO: Remove overlapping rules.

        // Empty or all allow.
        #[cfg(feature = "optimal")]
        if rules.is_empty() || rules.iter().all(|r| r.is_allowed()) {
            return Self::from_always(true, delay);
        }

        // Rules are sorted by length and permission i.e.
        // 5 > 4, 5 allow > 5 disallow.
        rules.sort();

        // All disallow + universal disallow.
        // Universal rule should be one of the smallest, so reverse the iter.
        #[cfg(feature = "optimal")]
        if rules.iter().all(|r| !r.is_allowed()) && rules.iter().rev().any(|r| r.is_universal()) {
            return Self::from_always(false, delay);
        }

        let rules = AlwaysRules::Rules(rules);
        Self { rules, delay }
    }

    /// Creates a new `Rules` from the global rule.
    pub fn from_always(always: bool, delay: Option<Duration>) -> Self {
        let rules = AlwaysRules::Always(always);
        Self { rules, delay }
    }

    /// Returns true if the relative path is allowed for this set of rules.
    /// NOTE: Expects relative path.
    pub fn is_allowed(&self, path: &str) -> bool {
        match &self.rules {
            AlwaysRules::Always(always) => *always,
            AlwaysRules::Rules(rules) => {
                let path = normalize_path(path);
                if path.eq("/robots.txt") {
                    return true;
                }

                let path = path.as_str();
                match rules.iter().find(|r| r.is_match(path)) {
                    Some(rule) => rule.is_allowed(),
                    None => true,
                }
            }
        }
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

    /// Returns the total amount of applied rules.
    pub fn len(&self) -> Option<usize> {
        match &self.rules {
            AlwaysRules::Rules(vec) => Some(vec.len()),
            AlwaysRules::Always(_) => None,
        }
    }
}

#[cfg(feature = "optimal")]
#[cfg(test)]
mod always {
    use super::*;

    #[test]
    fn from() {
        let r = Rules::from_always(true, None);
        assert_eq!(r.is_always(), Some(true));
        let r = Rules::from_always(false, None);
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn empty() {
        let r = Rules::from_rules(Vec::new(), None);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn allow() {
        let r = Rule::new("/", true).unwrap();
        let r = Rules::from_rules(vec![r.clone(), r.clone()], None);
        assert_eq!(r.is_always(), Some(true));
    }

    #[test]
    fn disallow_ok() {
        let universal = Rule::new("/*", false).unwrap();
        let disallow = Rule::new("/foo", false).unwrap();
        let r = Rules::from_rules(vec![universal, disallow], None);
        assert_eq!(r.is_always(), Some(false));
    }

    #[test]
    fn disallow_err() {
        let universal = Rule::new("/*", false).unwrap();
        let allow = Rule::new("/foo", true).unwrap();
        let r = Rules::from_rules(vec![universal, allow], None);
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
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(rules.is_allowed("/page"));
    }

    #[test]
    fn restrictive() {
        let allow = Rule::new("/folder", true).unwrap();
        let disallow = Rule::new("/folder", false).unwrap();
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(rules.is_allowed("/folder/page"));
    }

    #[test]
    fn restrictive2() {
        let allow = Rule::new("/page", true).unwrap();
        let disallow = Rule::new("/*.ph", false).unwrap();
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(rules.is_allowed("/page.php5"));
    }

    #[test]
    fn longer() {
        let allow = Rule::new("/page", true).unwrap();
        let disallow = Rule::new("/*.htm", false).unwrap();
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(!rules.is_allowed("/page.htm"));
    }

    #[test]
    fn specific() {
        let allow = Rule::new("/$", true).unwrap();
        let disallow = Rule::new("/", false).unwrap();
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(rules.is_allowed("/"));
    }

    #[test]
    fn specific2() {
        let allow = Rule::new("/$", true).unwrap();
        let disallow = Rule::new("/", false).unwrap();
        let rules = Rules::from_rules(vec![allow, disallow], None);

        assert!(!rules.is_allowed("/page.htm"));
    }
}
