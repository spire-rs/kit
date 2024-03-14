use std::time::Duration;

use url::Url;

use crate::parse::lexer::Directive;
use crate::parse::rule::Rule;
use crate::ALL_UAS;

#[derive(Debug, Default)]
pub struct Parser {
    captures_group: bool,
    captures_rules: bool,

    pub longest_match: String,
    pub rules: Vec<Rule>,
    pub crawl_delay: Option<Duration>,
    pub sitemaps: Vec<Url>,
}

impl Parser {
    /// Creates a new [`Parser`] with all extracted data from the list of directives.
    pub fn parse_rules(directives: &[Directive], user_agent: &str) -> Self {
        let (longest_match, captures_rules) = Self::longest_match(directives, user_agent);
        let mut state = Self {
            longest_match,
            captures_rules,
            ..Self::default()
        };

        directives.iter().for_each(|directive| match directive {
            Directive::UserAgent(data) => state.try_user_agent(data),
            Directive::Allow(data) => state.try_rule(data, true),
            Directive::Disallow(data) => state.try_rule(data, false),
            Directive::CrawlDelay(data) => state.try_delay(data),
            Directive::Sitemap(data) => state.try_sitemap(data),
            Directive::Unknown(_) => {}
        });

        // Rules are sorted by length and permission i.e.
        // 5 > 4, 5 allow > 5 disallow.
        state.rules.sort();
        state
    }

    /// Finds the longest matching user-agent and if the parser should check non-assigned rules
    /// i.e. `Allow`/`Disallow`/`Crawl-Delay` before the first `User-Agent`.
    fn longest_match(directives: &[Directive], user_agent: &str) -> (String, bool) {
        // Collects all `User-Agent`s.
        let all_uas = directives.iter().filter_map(|ua2| match ua2 {
            Directive::UserAgent(ua2) => std::str::from_utf8(ua2).ok(),
            _ => None,
        });

        // Filters out non-acceptable `User-Agent`s.
        let user_agent = user_agent.trim().to_lowercase();
        let acceptable_uas = all_uas
            .map(|ua| ua.trim().to_lowercase())
            .filter(|ua| user_agent.starts_with(ua.as_str()));

        // Finds the longest `User-Agent` in the acceptable pool.
        let selected_ua = acceptable_uas
            .max_by(|lhs, rhs| lhs.len().cmp(&rhs.len()))
            .unwrap_or(ALL_UAS.to_string());

        // Determines if it should check non-assigned rules.
        let check_non_assigned = selected_ua == ALL_UAS;
        (selected_ua, check_non_assigned)
    }

    /// Attempts to parse and match the `User-Agent`.
    fn try_user_agent(&mut self, data: &[u8]) {
        let data = String::from_utf8(data.to_vec()).ok();
        let user_agent = data.map(|data| data.trim().to_lowercase());

        if let Some(user_agent) = user_agent {
            if !self.captures_group || !self.captures_rules {
                self.captures_rules = user_agent == self.longest_match;
            }
        }

        self.captures_group = true;
    }

    /// Attempts to parse and store the valid matching `Rule`.
    fn try_rule(&mut self, data: &[u8], allow: bool) {
        self.captures_group = false;
        if !self.captures_rules {
            return;
        }

        let data = String::from_utf8(data.to_vec()).ok();
        let rule = data.and_then(|data| Rule::new(&data, allow).ok());
        if let Some(rule) = rule {
            self.rules.push(rule);
        }
    }

    /// Attempts to parse and store the valid `Duration` as a `crawl-delay`.
    fn try_delay(&mut self, data: &[u8]) {
        self.captures_group = false;
        if !self.captures_rules {
            return;
        }

        let data = String::from_utf8(data.to_vec()).ok();
        self.crawl_delay = data
            .and_then(|data| data.parse::<f64>().ok())
            .and_then(|secs| Duration::try_from_secs_f64(secs).ok())
            .map(|curr| (self.crawl_delay.unwrap_or(curr), curr))
            .map(|(prev, curr)| prev.min(curr));
    }

    /// Attempts to parse and store the valid `Url` address as a `sitemap`.
    fn try_sitemap(&mut self, data: &[u8]) {
        let data = String::from_utf8(data.to_vec()).ok();
        let addr = data.and_then(|data| Url::parse(data.as_str()).ok());
        if let Some(addr) = addr {
            self.sitemaps.push(addr);
        }
    }
}
