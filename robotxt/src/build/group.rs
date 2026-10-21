//! Building a single `User-Agent` group.

use std::fmt;
use std::time::Duration;

use crate::ANY_AGENT;
use crate::build::write_comment;
use crate::path::normalize;

/// The builder of a single `User-Agent` group.
///
/// See [`RobotsBuilder::group`].
///
/// [`RobotsBuilder::group`]: crate::RobotsBuilder::group
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct GroupBuilder {
    header: Option<String>,
    user_agents: Vec<String>,
    crawl_delay: Option<Duration>,
    rules: Vec<(&'static str, String)>,
    footer: Option<String>,
}

impl GroupBuilder {
    /// Creates the group for the user-agents, or for [`ANY_AGENT`] if there
    /// are none. Blank, repeated and multi-line user-agents are skipped.
    ///
    /// [`ANY_AGENT`]: crate::ANY_AGENT
    pub fn new<'a>(user_agents: impl IntoIterator<Item = &'a str>) -> Self {
        let mut group = Self::default();
        for user_agent in user_agents.into_iter().map(str::trim) {
            let is_valid = !user_agent.is_empty() && !user_agent.contains(char::is_control);
            if is_valid && !group.user_agents.iter().any(|ua| ua == user_agent) {
                group.user_agents.push(user_agent.to_owned());
            }
        }

        group
    }

    /// Sets the comment above the group, e.g. for rule notes.
    pub fn header(mut self, header: &str) -> Self {
        self.header = Some(header.to_owned());
        self
    }

    /// Sets the `Crawl-Delay` of the group.
    pub const fn crawl_delay(mut self, delay: Duration) -> Self {
        self.crawl_delay = Some(delay);
        self
    }

    /// Adds an `Allow` rule with the normalized pattern.
    ///
    /// ```rust
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::new().group(["foobot"], |u| u.allow("/"));
    /// ```
    pub fn allow(self, pattern: &str) -> Self {
        self.rule("Allow", pattern)
    }

    /// Adds a `Disallow` rule with the normalized pattern.
    ///
    /// ```rust
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::new().group(["foobot"], |u| u.disallow("/secret.txt"));
    /// ```
    pub fn disallow(self, pattern: &str) -> Self {
        self.rule("Disallow", pattern)
    }

    fn rule(mut self, directive: &'static str, pattern: &str) -> Self {
        let pattern = pattern.trim();
        let pattern = if pattern.is_empty() {
            String::new()
        } else {
            normalize(pattern.as_bytes()).into_owned()
        };

        self.rules.push((directive, pattern));
        self
    }

    /// Sets the comment below the group.
    pub fn footer(mut self, footer: &str) -> Self {
        self.footer = Some(footer.to_owned());
        self
    }
}

impl<'a> FromIterator<&'a str> for GroupBuilder {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl fmt::Display for GroupBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(header) = &self.header {
            write_comment(f, header)?;
        }

        if self.user_agents.is_empty() {
            writeln!(f, "User-Agent: {ANY_AGENT}")?;
        }

        for user_agent in &self.user_agents {
            writeln!(f, "User-Agent: {user_agent}")?;
        }

        if let Some(delay) = self.crawl_delay {
            writeln!(f, "Crawl-Delay: {}", delay.as_secs_f64())?;
        }

        // An empty `Disallow` allows every URL and ends the group.
        if self.rules.is_empty() {
            writeln!(f, "Disallow:")?;
        }

        for (directive, pattern) in &self.rules {
            match pattern.as_str() {
                "" => writeln!(f, "{directive}:")?,
                pattern => writeln!(f, "{directive}: {pattern}")?,
            }
        }

        if let Some(footer) = &self.footer {
            write_comment(f, footer)?;
        }

        Ok(())
    }
}
