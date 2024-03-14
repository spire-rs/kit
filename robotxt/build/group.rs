use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use crate::build::format_comment;
use crate::paths::normalize_path;

/// The single formatted `user-agent` group.
///
/// See [crate::RobotsBuilder::group].
#[derive(Debug, Default, Clone)]
pub struct GroupBuilder {
    user_agents: HashSet<String>,
    rules_disallow: Vec<String>,
    rules_allow: Vec<String>,
    delay: Option<u16>,

    header: Option<String>,
    footer: Option<String>,
}

impl GroupBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a local header, usually used for rule notes.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["*"], |u| u.allow("/"))
    ///     .group(["foobot"], |u| {
    ///         u.header("Note: Bad Bot!")
    ///             .disallow("/")
    ///             .allow("/bad/bot.txt")
    ///     });
    /// ```
    pub fn header(mut self, header: &str) -> Self {
        self.header = Some(header.to_string());
        self
    }

    /// Adds an `Allow` directive.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["foobot"], |u| {
    ///         u.allow("/").disallow("/secret.txt")
    ///     });
    /// ```
    pub fn allow(mut self, rule: &str) -> Self {
        let rule = normalize_path(rule);
        self.rules_allow.push(rule);
        self
    }

    /// Adds a `Disallow` directive.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["foobot"], |u| {
    ///         u.allow("/").disallow("/secret.txt")
    ///     });
    /// ```
    pub fn disallow(mut self, rule: &str) -> Self {
        let rule = normalize_path(rule);
        self.rules_disallow.push(rule);
        self
    }

    /// Adds a `Crawl-Delay` directive.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["foobot"], |u| {
    ///         u.crawl_delay(5)
    ///     });
    /// ```
    pub fn crawl_delay(mut self, delay: u16) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Adds a local footer, usually used for rule notes.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["foobot"], |u| {
    ///         u.footer("Note: Bad Bot!")
    ///             .disallow("/")
    ///             .allow("/bad/bot.txt")
    ///     });
    /// ```
    pub fn footer(mut self, footer: &str) -> Self {
        self.footer = Some(footer.to_string());
        self
    }
}

impl<'ua> FromIterator<&'ua str> for GroupBuilder {
    fn from_iter<T: IntoIterator<Item = &'ua str>>(iter: T) -> Self {
        let uas = iter.into_iter().map(|ua| ua.trim().to_string());
        Self {
            user_agents: uas.collect(),
            ..Self::default()
        }
    }
}

impl Display for GroupBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let header = self.header.as_ref().map(|h| format_comment(h));
        let footer = self.footer.as_ref().map(|f| format_comment(f));
        let delay = self.delay.map(|d| format!("Crawl-Delay: {d}"));

        let agents = if self.user_agents.is_empty() {
            Some("User-Agent: *".to_string())
        } else {
            let uas = self.user_agents.iter();
            let uas = uas.map(|ua| format!("User-Agent: {ua}"));
            Some(uas.collect::<Vec<_>>().join("\n"))
        };

        let disallows = if self.rules_disallow.is_empty() {
            None
        } else {
            let rd = self.rules_disallow.iter();
            let rd = rd.map(|r| format!("Disallow: {r}"));
            Some(rd.collect::<Vec<_>>().join("\n"))
        };

        let allows = if self.rules_allow.is_empty() {
            // Explicit Allow: * if no Disallows.
            // Used to interrupt the user-group i.e.
            // user-agent: a ..no rules.. user-agent: b
            match self.rules_disallow.is_empty() {
                true => Some("Allow: *".to_string()),
                false => None,
            }
        } else {
            let rd = self.rules_allow.iter();
            let rd = rd.map(|r| format!("Allow: {r}"));
            Some(rd.collect::<Vec<_>>().join("\n"))
        };

        let result = [header, agents, delay, disallows, allows, footer];
        let result = result.iter().filter_map(|u| u.clone());
        let result = result.collect::<Vec<_>>().join("\n");
        write!(f, "{}", result.as_str())
    }
}

#[cfg(test)]
mod builder {
    use super::*;

    #[test]
    fn empty_uas() {
        let r = GroupBuilder::new().disallow("/foo").to_string();
        assert!(r.contains("User-Agent: *"));
    }

    #[test]
    fn no_rules() {
        let r = GroupBuilder::from_iter(["foobot"]).to_string();
        assert!(r.contains("Allow: *"));
    }
}
