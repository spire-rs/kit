use std::time::Duration;

use sitemapo::record::{
    Alternate, Entry, Image, News, Platform, Platforms, Publication, Relationship, Restriction,
    Uploader, Video,
};
use sitemapo::{EntryBuilder, Parser, Record};
use url::Url;

fn url(location: &str) -> Url {
    location.parse().unwrap()
}

fn parse(xml: &[u8]) -> Vec<Entry> {
    let records = Parser::new(xml).map(Result::unwrap);
    let entries = records.map(|record| match record {
        Record::Entry(entry) => entry,
        Record::Sitemap(_) => panic!("unexpected sitemap"),
    });

    entries.collect()
}

/// The example of Google's image sitemap documentation.
#[test]
fn image() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
            xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">
          <url>
            <loc>https://example.com/sample1.html</loc>
            <image:image>
              <image:loc>https://example.com/image.jpg</image:loc>
            </image:image>
            <image:image>
              <image:loc>https://example.com/photo.jpg</image:loc>
            </image:image>
          </url>
        </urlset>"#;

    let images = &parse(xml)[0].images;
    let expected = [
        Image::new(url("https://example.com/image.jpg")),
        Image::new(url("https://example.com/photo.jpg")),
    ];
    assert_eq!(images, &expected);
}

/// The example of Google's video sitemap documentation.
#[test]
fn video() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
            xmlns:video="http://www.google.com/schemas/sitemap-video/1.1">
          <url>
            <loc>https://www.example.com/videos/some_video_landing_page.html</loc>
            <video:video>
              <video:thumbnail_loc>https://www.example.com/thumbs/123.jpg</video:thumbnail_loc>
              <video:title>Grilling steaks for summer</video:title>
              <video:description>
                Alkis shows you how to get perfectly done steaks every time
              </video:description>
              <video:content_loc>
                http://streamserver.example.com/video123.mp4
              </video:content_loc>
              <video:player_loc>
                https://www.example.com/videoplayer.php?video=123
              </video:player_loc>
              <video:duration>600</video:duration>
              <video:expiration_date>2021-11-05T19:20:30+08:00</video:expiration_date>
              <video:rating>4.2</video:rating>
              <video:view_count>12345</video:view_count>
              <video:publication_date>2007-11-05T19:20:30+08:00</video:publication_date>
              <video:family_friendly>yes</video:family_friendly>
              <video:restriction relationship="allow">IE GB US CA</video:restriction>
              <video:platform relationship="allow">web tv</video:platform>
              <video:requires_subscription>yes</video:requires_subscription>
              <video:uploader
                info="https://www.example.com/users/grillymcgrillerson">GrillyMcGrillerson
              </video:uploader>
              <video:live>no</video:live>
              <video:tag>steak</video:tag>
              <video:tag>meat</video:tag>
            </video:video>
            <video:video>
              <video:thumbnail_loc>https://www.example.com/thumbs/345.jpg</video:thumbnail_loc>
              <video:title>Missing both locations</video:title>
              <video:description>Skipped</video:description>
            </video:video>
          </url>
        </urlset>"#;

    let videos = &parse(xml)[0].videos;
    assert_eq!(videos.len(), 1);

    let video = &videos[0];
    assert_eq!(video.title, "Grilling steaks for summer");
    assert_eq!(
        video.description,
        "Alkis shows you how to get perfectly done steaks every time"
    );
    assert_eq!(
        video.content.as_ref().map(Url::as_str),
        Some("http://streamserver.example.com/video123.mp4")
    );
    assert_eq!(video.duration, Some(Duration::from_secs(600)));
    assert_eq!(
        video.expires.map(|t| t.to_string()).as_deref(),
        Some("2021-11-05T11:20:30Z")
    );
    assert_eq!(video.rating, Some(4.2));
    assert_eq!(video.views, Some(12345));
    assert_eq!(video.family_friendly, Some(true));
    assert_eq!(
        video.restriction,
        Some(Restriction {
            relationship: Relationship::Allow,
            countries: ["IE", "GB", "US", "CA"].map(String::from).to_vec(),
        })
    );
    assert_eq!(
        video.platforms,
        Some(Platforms {
            relationship: Relationship::Allow,
            platforms: vec![Platform::Web, Platform::Tv],
        })
    );
    assert_eq!(video.requires_subscription, Some(true));
    assert_eq!(
        video.uploader,
        Some(Uploader {
            name: "GrillyMcGrillerson".to_owned(),
            info: Some(url("https://www.example.com/users/grillymcgrillerson")),
        })
    );
    assert_eq!(video.live, Some(false));
    assert_eq!(video.tags, ["steak", "meat"]);
}

/// The example of Google's news sitemap documentation.
#[test]
fn news() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
            xmlns:news="http://www.google.com/schemas/sitemap-news/0.9">
          <url>
            <loc>http://www.example.org/business/article55.html</loc>
            <news:news>
              <news:publication>
                <news:name>The Example Times</news:name>
                <news:language>en</news:language>
              </news:publication>
              <news:publication_date>2008-12-23</news:publication_date>
              <news:title>Companies A, B in Merger Talks</news:title>
            </news:news>
          </url>
        </urlset>"#;

    let expected = News::new(
        Publication::new("The Example Times", "en"),
        "2008-12-23T00:00:00Z".parse().unwrap(),
        "Companies A, B in Merger Talks",
    );
    assert_eq!(parse(xml)[0].news, Some(expected));
}

/// The example of Google's localized versions documentation.
#[test]
fn alternates() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
            xmlns:xhtml="http://www.w3.org/1999/xhtml">
          <url>
            <loc>https://www.example.com/english/page.html</loc>
            <xhtml:link rel="alternate" hreflang="de" href="https://www.example.de/deutsch/page.html"/>
            <xhtml:link rel="alternate" hreflang="en" href="https://www.example.com/english/page.html"/>
            <xhtml:link rel="canonical" hreflang="fr" href="https://www.example.com/ignored.html"/>
          </url>
        </urlset>"#;

    let expected = [
        Alternate::new("de", url("https://www.example.de/deutsch/page.html")),
        Alternate::new("en", url("https://www.example.com/english/page.html")),
    ];
    assert_eq!(parse(xml)[0].alternates, expected);
}

#[test]
fn undeclared_prefixes() {
    let xml = b"<urlset><url><loc>https://example.com/</loc>\
        <image:image><image:loc>https://example.com/a.png</image:loc></image:image></url></urlset>";
    assert_eq!(parse(xml)[0].images.len(), 1);
}

#[test]
fn custom_prefixes() {
    let xml = br#"<s:urlset xmlns:s="http://www.sitemaps.org/schemas/sitemap/0.9"
            xmlns:i="http://www.google.com/schemas/sitemap-image/1.1">
        <s:url><s:loc>https://example.com/</s:loc>
        <i:image><i:loc>https://example.com/a.png</i:loc></i:image></s:url></s:urlset>"#;
    assert_eq!(parse(xml)[0].images.len(), 1);
}

#[test]
fn round_trip() {
    let mut video = Video::new(
        url("https://example.com/thumb.jpg"),
        "Title & <more>",
        "Description",
    )
    .with_content(url("https://example.com/video.mp4"))
    .with_player(url("https://example.com/player?id=1&x=2"));
    video.duration = Some(Duration::from_secs(90));
    video.expires = Some("2030-01-01T00:00:00Z".parse().unwrap());
    video.rating = Some(3.5);
    video.views = Some(7);
    video.published = Some("2024-01-01T12:30:00Z".parse().unwrap());
    video.family_friendly = Some(false);
    video.restriction = Some(Restriction {
        relationship: Relationship::Deny,
        countries: vec!["US".to_owned()],
    });
    video.platforms = Some(Platforms {
        relationship: Relationship::Allow,
        platforms: vec![Platform::Mobile],
    });
    video.requires_subscription = Some(false);
    video.uploader = Some(Uploader {
        name: "Uploader".to_owned(),
        info: None,
    });
    video.live = Some(true);
    video.tags = vec!["a".to_owned(), "b".to_owned()];

    let location = url("https://example.com/page");
    let entry = Entry::new(location.clone())
        .with_image(Image::new(url("https://example.com/image.png")))
        .with_video(video)
        .with_news(News::new(
            Publication::new("Example", "zh-cn"),
            "2024-01-01T00:00:00Z".parse().unwrap(),
            "News",
        ))
        .with_alternate(Alternate::new("x-default", location));

    let mut builder = EntryBuilder::new(Vec::new());
    builder.write(&entry).unwrap();
    let xml = builder.finish().unwrap();
    assert_eq!(parse(&xml), [entry]);
}
