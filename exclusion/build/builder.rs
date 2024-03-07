use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use url::Url;

use crate::build::format_comment;
use crate::GroupBuilder;

/// The set of formatted `user-agent` groups that can be written
/// in the `robots.txt` compliant format.
#[derive(Debug, Default, Clone)]
pub struct RobotsBuilder {
    groups: Vec<GroupBuilder>,
    sitemaps: HashSet<Url>,

    header: Option<String>,
    footer: Option<String>,
}

impl RobotsBuilder {
    /// Creates a new [`RobotsBuilder`] with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a global header, usually used for permissions or legal notices.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .header("Note: Stop right there!")
    ///     .group(["*"], |u| u.disallow("/"))
    ///     .group(["foobot"], |u| u.allow("/"));
    /// ```
    pub fn header(mut self, header: &str) -> Self {
        self.header = Some(header.to_string());
        self
    }

    /// Adds a new `user-agent` group from the provided list of user-agents.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["*"], |u| u.disallow("/"))
    ///     .group(["foobot"], |u| u.allow("/"));
    /// ```
    pub fn group<'a>(
        mut self,
        group: impl IntoIterator<Item = &'a str>,
        factory: impl FnOnce(GroupBuilder) -> GroupBuilder,
    ) -> Self {
        let section = GroupBuilder::from_iter(group);
        self.groups.push(factory(section));
        self
    }

    /// Adds the `Sitemap` directive from the URL address.
    ///
    /// ```
    /// use url::Url;
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .sitemap("https://example.com/sitemap_1.xml".try_into().unwrap())
    ///     .sitemap("https://example.com/sitemap_1.xml".try_into().unwrap());
    /// ```
    pub fn sitemap(mut self, sitemap: Url) -> Self {
        self.sitemaps.insert(sitemap);
        self
    }

    /// Adds a global footer, usually used for notices.
    ///
    /// ```
    /// use robotxt::RobotsBuilder;
    ///
    /// let txt = RobotsBuilder::default()
    ///     .group(["*"], |u| u.disallow("/"))
    ///     .group(["foobot"], |u| u.allow("/"))
    ///     .footer("Note: Have a nice day!");
    /// ```
    pub fn footer(mut self, footer: &str) -> Self {
        self.footer = Some(footer.to_string());
        self
    }

    /// Parses the constructed output.
    /// See [`Robots::from_bytes`].
    ///
    /// [`Robots`]: crate::Robots
    #[cfg(feature = "parser")]
    #[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
    pub fn parse(&self, user_agent: &str) -> crate::Robots {
        let txt = self.to_string();
        crate::Robots::from_bytes(txt.as_bytes(), user_agent)
    }
}

impl Display for RobotsBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let header = self.header.as_ref().map(|h| format_comment(h));
        let footer = self.footer.as_ref().map(|f| format_comment(f));

        let groups = self.groups.iter().map(|u| u.to_string());
        let groups = groups.collect::<Vec<_>>().join("\n\n");

        let result = [header, Some(groups), footer];
        let result = result.iter().filter_map(|u| u.clone());
        let result = result.collect::<Vec<_>>().join("\n\n");
        write!(f, "{}", result.as_str())
    }
}

#[cfg(test)]
mod builder {
    use crate::{Result, RobotsBuilder};

    #[test]
    fn readme() -> Result<()> {
        let txt = RobotsBuilder::default()
            .header("Robots.txt: Start")
            .group(["foobot"], |u| {
                u.crawl_delay(5)
                    .header("Rules for Foobot: Start")
                    .allow("/example/yeah.txt")
                    .disallow("/example/nope.txt")
                    .footer("Rules for Foobot: End")
            })
            .group(["barbot", "nombot"], |u| {
                u.crawl_delay(2)
                    .disallow("/example/yeah.txt")
                    .disallow("/example/nope.txt")
            })
            .sitemap("https://example.com/sitemap_1.xml".try_into()?)
            .sitemap("https://example.com/sitemap_2.xml".try_into()?)
            .footer("Robots.txt: End");

        println!("{}", txt.to_string());
        Ok(())
    }
}
