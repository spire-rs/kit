//! Converts the element trees into records, skipping the invalid ones.

use std::str::FromStr;
use std::time::Duration;

use jiff::Timestamp;
use url::Url;

use crate::parse::element::{Element, Ns};
use crate::record::names::{CHANGE_FREQUENCY, LAST_MODIFIED, LOCATION, PRIORITY};
use crate::record::{
    Alternate, Entry, Image, News, Platform, Platforms, Publication, Relationship, Restriction,
    Sitemap, Uploader, Video, datetime,
};

fn url(text: Option<&str>) -> Option<Url> {
    Url::parse(text?).ok()
}

fn timestamp(text: Option<&str>) -> Option<Timestamp> {
    datetime::parse(text?)
}

fn parse<T: FromStr>(text: Option<&str>) -> Option<T> {
    text?.parse().ok()
}

/// Parses the `yes` and `no` values of the video extension.
fn yes_no(text: Option<&str>) -> Option<bool> {
    match text? {
        text if text.eq_ignore_ascii_case("yes") => Some(true),
        text if text.eq_ignore_ascii_case("no") => Some(false),
        _ => None,
    }
}

/// Converts a `<url>` element.
pub fn entry(element: &Element) -> Option<Entry> {
    let mut entry = Entry::new(url(element.child_text(Ns::Sitemap, LOCATION))?);
    entry.modified = timestamp(element.child_text(Ns::Sitemap, LAST_MODIFIED));
    entry.frequency = parse(element.child_text(Ns::Sitemap, CHANGE_FREQUENCY));
    entry.priority = parse(element.child_text(Ns::Sitemap, PRIORITY));
    entry.images = element
        .children(Ns::Image, "image")
        .filter_map(image)
        .collect();
    entry.videos = element
        .children(Ns::Video, "video")
        .filter_map(video)
        .collect();
    entry.news = element.child(Ns::News, "news").and_then(news);
    let alternates = element.children(Ns::Xhtml, "link");
    entry.alternates = alternates.filter_map(alternate).collect();
    Some(entry)
}

/// Converts a `<sitemap>` element.
pub fn sitemap(element: &Element) -> Option<Sitemap> {
    let mut sitemap = Sitemap::new(url(element.child_text(Ns::Sitemap, LOCATION))?);
    sitemap.modified = timestamp(element.child_text(Ns::Sitemap, LAST_MODIFIED));
    Some(sitemap)
}

/// Converts an `<image:image>` element.
fn image(element: &Element) -> Option<Image> {
    url(element.child_text(Ns::Image, LOCATION)).map(Image::new)
}

/// Converts an `<xhtml:link rel="alternate">` element.
fn alternate(element: &Element) -> Option<Alternate> {
    let rel = element.attribute("rel")?;
    if !rel.eq_ignore_ascii_case("alternate") {
        return None;
    }

    let language = element
        .attribute("hreflang")
        .filter(|language| !language.is_empty())?;
    let location = url(element.attribute("href"))?;
    Some(Alternate::new(language, location))
}

/// Converts a `<news:news>` element.
fn news(element: &Element) -> Option<News> {
    let publication = element.child(Ns::News, "publication")?;
    let name = publication.child_text(Ns::News, "name")?;
    let language = publication.child_text(Ns::News, "language")?;
    let publication = Publication::new(name, language);

    let published = timestamp(element.child_text(Ns::News, "publication_date"))?;
    let title = element.child_text(Ns::News, "title")?;
    Some(News::new(publication, published, title))
}

/// Converts a `<video:video>` element.
fn video(element: &Element) -> Option<Video> {
    let text = |name: &str| element.child_text(Ns::Video, name);

    let thumbnail = url(text("thumbnail_loc"))?;
    let mut video = Video::new(thumbnail, text("title")?, text("description")?);
    video.content = url(text("content_loc"));
    video.player = url(text("player_loc"));
    if video.content.is_none() && video.player.is_none() {
        return None;
    }

    video.duration = parse(text("duration")).map(Duration::from_secs);
    video.expires = timestamp(text("expiration_date"));
    video.rating = parse(text("rating")).filter(|rating| (0.0..=5.0).contains(rating));
    video.views = parse(text("view_count"));
    video.published = timestamp(text("publication_date"));
    video.family_friendly = yes_no(text("family_friendly"));
    video.requires_subscription = yes_no(text("requires_subscription"));
    video.live = yes_no(text("live"));

    video.restriction = element
        .child(Ns::Video, "restriction")
        .and_then(|restriction| {
            let relationship = Relationship::parse(restriction.attribute("relationship")?)?;
            let countries = restriction.text().split_whitespace();
            let countries = countries.map(str::to_ascii_uppercase).collect();
            Some(Restriction {
                relationship,
                countries,
            })
        });

    video.platforms = element.child(Ns::Video, "platform").and_then(|platform| {
        let relationship = Relationship::parse(platform.attribute("relationship")?)?;
        let platforms = platform.text().split_whitespace();
        let platforms = platforms.filter_map(Platform::parse).collect();
        Some(Platforms {
            relationship,
            platforms,
        })
    });

    video.uploader = element.child(Ns::Video, "uploader").and_then(|uploader| {
        let name = Some(uploader.text()).filter(|name| !name.is_empty())?;
        let info = url(uploader.attribute("info"));
        Some(Uploader {
            name: name.to_owned(),
            info,
        })
    });

    let tags = element.children(Ns::Video, "tag").map(Element::text);
    video.tags = tags
        .filter(|tag| !tag.is_empty())
        .map(str::to_owned)
        .collect();
    Some(video)
}
