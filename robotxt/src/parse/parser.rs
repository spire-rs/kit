//! Selects the groups that apply to the crawler and collects their rules.

use std::time::Duration;

use url::Url;

use crate::parse::agent::Agent;
use crate::parse::lexer::Directive;
use crate::parse::robots::Robots;
use crate::parse::rule::Rule;

/// Parses the `Crawl-Delay` value in (possibly fractional) seconds.
fn parse_delay(value: &[u8]) -> Option<Duration> {
    let secs = std::str::from_utf8(value).ok()?.parse::<f64>().ok()?;
    Duration::try_from_secs_f64(secs).ok()
}

/// Parses the `Sitemap` value as an absolute URL.
fn parse_sitemap(value: &[u8]) -> Option<Url> {
    Url::parse(std::str::from_utf8(value).ok()?).ok()
}

/// Merges the groups that apply to the crawler, and collects the sitemaps.
///
/// A group starts with one or more `User-Agent` lines and ends at the next
/// `User-Agent` line after a rule. Rules before the first group apply to no
/// one. `Sitemap` lines do not belong to any group.
pub fn parse(directives: &[Directive<'_>], agent: Agent<'_>) -> Robots {
    let user_agent = agent.select(directives);
    let mut rules = Vec::new();
    let mut crawl_delay: Option<Duration> = None;
    let mut sitemaps = Vec::new();

    let mut in_agents = false;
    let mut applies = false;
    for directive in directives {
        match *directive {
            Directive::UserAgent(value) => {
                if !in_agents {
                    in_agents = true;
                    applies = false;
                }

                applies |= value.eq_ignore_ascii_case(user_agent);
            }
            Directive::Allow(value) | Directive::Disallow(value) => {
                in_agents = false;
                let allow = matches!(directive, Directive::Allow(_));
                if applies && let Some(rule) = Rule::parse(value, allow) {
                    rules.push(rule);
                }
            }
            Directive::CrawlDelay(value) => {
                in_agents = false;
                if applies && let Some(delay) = parse_delay(value) {
                    crawl_delay = Some(crawl_delay.map_or(delay, |prev| prev.min(delay)));
                }
            }
            Directive::Sitemap(value) => sitemaps.extend(parse_sitemap(value)),
            Directive::Unknown(_) => {}
        }
    }

    let user_agent = String::from_utf8_lossy(user_agent).to_ascii_lowercase();
    Robots::new(user_agent, rules, crawl_delay, sitemaps)
}
