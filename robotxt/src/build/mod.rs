//! Building `robots.txt` files.

use std::fmt;

use url::Url;

pub use crate::build::group::GroupBuilder;

mod group;

/// Returns the non-empty lines of the comment text, without leading `#`.
fn comment_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(|line| line.trim().trim_start_matches('#').trim_start())
        .filter(|line| !line.is_empty())
}

/// Writes every non-empty line of the text as a `#` comment.
fn write_comment(f: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    comment_lines(text).try_for_each(|line| writeln!(f, "# {line}"))
}

/// The builder of `robots.txt` files.
///
/// Lines are written in the order they were added, each `Allow` and
/// `Disallow` pattern is normalized, and every line ends with a newline.
///
/// ```rust
/// use robotxt::RobotsBuilder;
///
/// let txt = RobotsBuilder::new()
///     .header("Robots.txt: Start")
///     .group(["foobot"], |u| {
///         u.crawl_delay(std::time::Duration::from_secs(5))
///             .allow("/example/yeah.txt")
///             .disallow("/example/nope.txt")
///     })
///     .group(["barbot", "nombot"], |u| u.disallow("/"))
///     .sitemap("https://example.com/sitemap.xml".parse().unwrap())
///     .footer("Robots.txt: End");
///
/// assert_eq!(txt.to_string(), "\
/// ## Robots.txt: Start
///
/// User-Agent: foobot
/// Crawl-Delay: 5
/// Allow: /example/yeah.txt
/// Disallow: /example/nope.txt
///
/// User-Agent: barbot
/// User-Agent: nombot
/// Disallow: /
///
/// Sitemap: https://example.com/sitemap.xml
///
/// ## Robots.txt: End
/// ");
/// ```
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct RobotsBuilder {
    header: Option<String>,
    groups: Vec<GroupBuilder>,
    sitemaps: Vec<Url>,
    footer: Option<String>,
}

impl RobotsBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the comment at the top of the file, e.g. for legal notices.
    pub fn header(mut self, header: &str) -> Self {
        self.header = Some(header.to_owned());
        self
    }

    /// Adds the group for the user-agents, or for [`ANY_AGENT`] if there
    /// are none, with the rules added by the closure.
    ///
    /// ```rust
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::new()
    ///     .group(["*"], |u| u.disallow("/"))
    ///     .group(["foobot"], |u| u.allow("/"));
    /// ```
    ///
    /// [`ANY_AGENT`]: crate::ANY_AGENT
    pub fn group<'a>(
        mut self,
        user_agents: impl IntoIterator<Item = &'a str>,
        rules: impl FnOnce(GroupBuilder) -> GroupBuilder,
    ) -> Self {
        let group = GroupBuilder::from_iter(user_agents);
        self.groups.push(rules(group));
        self
    }

    /// Adds the `Sitemap` URL, unless it was already added.
    pub fn sitemap(mut self, sitemap: Url) -> Self {
        if !self.sitemaps.contains(&sitemap) {
            self.sitemaps.push(sitemap);
        }

        self
    }

    /// Sets the comment at the bottom of the file.
    pub fn footer(mut self, footer: &str) -> Self {
        self.footer = Some(footer.to_owned());
        self
    }

    /// Parses the built file for the crawler.
    /// See [`Robots::parse`].
    ///
    /// ```rust
    /// use robotxt::RobotsBuilder;
    ///
    /// let r = RobotsBuilder::new()
    ///     .group(["foobot"], |u| u.disallow("/"))
    ///     .parse("foobot");
    /// assert!(!r.is_allowed("/"));
    /// ```
    ///
    /// [`Robots::parse`]: crate::Robots::parse
    #[cfg(feature = "parser")]
    #[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
    pub fn parse<'a>(&self, agent: impl Into<crate::Agent<'a>>) -> crate::Robots {
        crate::Robots::parse(self.to_string().as_bytes(), agent)
    }
}

impl fmt::Display for RobotsBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let has_lines = |text: &&str| comment_lines(text).next().is_some();
        let header = self.header.as_deref().filter(has_lines);
        let footer = self.footer.as_deref().filter(has_lines);

        // Sections are separated by a blank line.
        let mut first = true;
        let mut separate = |f: &mut fmt::Formatter<'_>| {
            if std::mem::take(&mut first) {
                Ok(())
            } else {
                f.write_str("\n")
            }
        };

        if let Some(header) = header {
            separate(f)?;
            write_comment(f, header)?;
        }

        for group in &self.groups {
            separate(f)?;
            write!(f, "{group}")?;
        }

        if !self.sitemaps.is_empty() {
            separate(f)?;
            for sitemap in &self.sitemaps {
                writeln!(f, "Sitemap: {sitemap}")?;
            }
        }

        if let Some(footer) = footer {
            separate(f)?;
            write_comment(f, footer)?;
        }

        Ok(())
    }
}
