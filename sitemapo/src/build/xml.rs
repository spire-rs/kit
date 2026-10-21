//! Encodes the records of XML sitemaps.

use std::fmt::{Display, Write};

use quick_xml::escape::escape;

use crate::record::names::{CHANGE_FREQUENCY, LAST_MODIFIED, LOCATION, PRIORITY, SITEMAP, URL};
use crate::record::{Entry, News, Sitemap, Video};

pub const URL_SET_HEADER: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    "\n",
    r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9""#,
    r#" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1""#,
    r#" xmlns:video="http://www.google.com/schemas/sitemap-video/1.1""#,
    r#" xmlns:news="http://www.google.com/schemas/sitemap-news/0.9""#,
    r#" xmlns:xhtml="http://www.w3.org/1999/xhtml">"#,
    "\n",
);

pub const URL_SET_FOOTER: &str = "</urlset>\n";

pub const INDEX_HEADER: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8"?>"#,
    "\n",
    r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    "\n",
);

pub const INDEX_FOOTER: &str = "</sitemapindex>\n";

/// Writes XML elements with two spaces of indentation per level.
struct Xml<'a> {
    output: &'a mut String,
    depth: usize,
}

impl Xml<'_> {
    fn indent(&mut self) {
        (0..self.depth).for_each(|_| self.output.push_str("  "));
    }

    fn start(&mut self, name: &str) {
        self.indent();
        _ = writeln!(self.output, "<{name}>");
        self.depth += 1;
    }

    fn end(&mut self, name: &str) {
        self.depth -= 1;
        self.indent();
        _ = writeln!(self.output, "</{name}>");
    }

    fn element(&mut self, name: &str, text: impl Display) {
        self.indent();
        let text = text.to_string();
        _ = writeln!(self.output, "<{name}>{}</{name}>", escape(&text));
    }

    fn element_with(&mut self, name: &str, attribute: (&str, &str), text: impl Display) {
        self.indent();
        let (key, value) = attribute;
        let (value, text) = (escape(value), text.to_string());
        _ = writeln!(
            self.output,
            r#"<{name} {key}="{value}">{}</{name}>"#,
            escape(&text)
        );
    }

    fn optional(&mut self, name: &str, text: Option<impl Display>) {
        if let Some(text) = text {
            self.element(name, text);
        }
    }
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// Encodes the `<url>` element of the entry.
pub fn entry(entry: &Entry, output: &mut String) {
    let mut xml = Xml { output, depth: 1 };
    xml.start(URL);
    xml.element(LOCATION, &entry.location);
    xml.optional(LAST_MODIFIED, entry.modified);
    xml.optional(CHANGE_FREQUENCY, entry.frequency);
    xml.optional(PRIORITY, entry.priority);

    for image in &entry.images {
        xml.start("image:image");
        xml.element("image:loc", &image.location);
        xml.end("image:image");
    }

    for video in &entry.videos {
        encode_video(&mut xml, video);
    }

    if let Some(news) = &entry.news {
        encode_news(&mut xml, news);
    }

    for alternate in &entry.alternates {
        xml.indent();
        let language = escape(&alternate.language);
        let location = escape(alternate.location.as_str());
        _ = writeln!(
            xml.output,
            r#"<xhtml:link rel="alternate" hreflang="{language}" href="{location}"/>"#
        );
    }

    xml.end(URL);
}

fn encode_video(xml: &mut Xml<'_>, video: &Video) {
    xml.start("video:video");
    xml.element("video:thumbnail_loc", &video.thumbnail);
    xml.element("video:title", &video.title);
    xml.element("video:description", &video.description);
    xml.optional("video:content_loc", video.content.as_ref());
    xml.optional("video:player_loc", video.player.as_ref());
    xml.optional(
        "video:duration",
        video.duration.map(|duration| duration.as_secs()),
    );
    xml.optional("video:expiration_date", video.expires);
    xml.optional(
        "video:rating",
        video.rating.map(|rating| format!("{rating:?}")),
    );
    xml.optional("video:view_count", video.views);
    xml.optional("video:publication_date", video.published);
    xml.optional("video:family_friendly", video.family_friendly.map(yes_no));

    if let Some(restriction) = &video.restriction {
        let relationship = ("relationship", restriction.relationship.as_str());
        let countries = restriction.countries.join(" ");
        xml.element_with("video:restriction", relationship, countries);
    }

    if let Some(platforms) = &video.platforms {
        let relationship = ("relationship", platforms.relationship.as_str());
        let names = platforms.platforms.iter().map(|platform| platform.as_str());
        let names = names.collect::<Vec<_>>().join(" ");
        xml.element_with("video:platform", relationship, names);
    }

    let requires_subscription = video.requires_subscription.map(yes_no);
    xml.optional("video:requires_subscription", requires_subscription);

    if let Some(uploader) = &video.uploader {
        match &uploader.info {
            Some(info) => {
                xml.element_with("video:uploader", ("info", info.as_str()), &uploader.name);
            }
            None => xml.element("video:uploader", &uploader.name),
        }
    }

    xml.optional("video:live", video.live.map(yes_no));
    for tag in &video.tags {
        xml.element("video:tag", tag);
    }

    xml.end("video:video");
}

fn encode_news(xml: &mut Xml<'_>, news: &News) {
    xml.start("news:news");
    xml.start("news:publication");
    xml.element("news:name", &news.publication.name);
    xml.element("news:language", &news.publication.language);
    xml.end("news:publication");
    xml.element("news:publication_date", news.published);
    xml.element("news:title", &news.title);
    xml.end("news:news");
}

/// Encodes the `<sitemap>` element of the sitemap.
pub fn sitemap(sitemap: &Sitemap, output: &mut String) {
    let mut xml = Xml { output, depth: 1 };
    xml.start(SITEMAP);
    xml.element(LOCATION, &sitemap.location);
    xml.optional(LAST_MODIFIED, sitemap.modified);
    xml.end(SITEMAP);
}

#[cfg(test)]
mod test {
    use super::{INDEX_HEADER, URL_SET_HEADER};
    use crate::record::names::{IMAGE_NS, NEWS_NS, SITEMAP_NS, VIDEO_NS, XHTML_NS};

    #[test]
    fn namespaces() {
        for namespace in [SITEMAP_NS, IMAGE_NS, VIDEO_NS, NEWS_NS, XHTML_NS] {
            assert!(URL_SET_HEADER.contains(namespace), "{namespace}");
        }

        assert!(INDEX_HEADER.contains(SITEMAP_NS));
    }
}
