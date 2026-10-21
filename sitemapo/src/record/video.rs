use std::time::Duration;

use jiff::Timestamp;
use url::Url;

/// A video of the page, from the [video sitemap extension].
///
/// Either the [`content`] or the [`player`] location is required.
///
/// ```rust
/// use sitemapo::record::Video;
///
/// let video = Video::new(
///     "https://example.com/thumbnail.jpg".parse().unwrap(),
///     "Grilling steaks for summer",
///     "Alkis shows you how to get perfectly done steaks every time.",
/// )
/// .with_content("https://example.com/video.mp4".parse().unwrap());
/// ```
///
/// [video sitemap extension]: https://developers.google.com/search/docs/crawling-indexing/sitemaps/video-sitemaps
/// [`content`]: Video::content
/// [`player`]: Video::player
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    /// The URL of the thumbnail image.
    pub thumbnail: Url,
    /// The title of the video.
    pub title: String,
    /// The description of the video, up to 2,048 characters.
    pub description: String,
    /// The URL of the video file.
    pub content: Option<Url>,
    /// The URL of the video player.
    pub player: Option<Url>,
    /// The duration of the video, from 1 second to 8 hours.
    pub duration: Option<Duration>,
    /// The time after which the video is no longer available.
    pub expires: Option<Timestamp>,
    /// The rating of the video, from `0.0` to `5.0`.
    pub rating: Option<f32>,
    /// The number of times the video was viewed.
    pub views: Option<u64>,
    /// The time the video was first published.
    pub published: Option<Timestamp>,
    /// Whether the video is suitable for all ages.
    pub family_friendly: Option<bool>,
    /// The countries where the video may or may not be shown.
    pub restriction: Option<Restriction>,
    /// The platforms where the video may or may not be shown.
    pub platforms: Option<Platforms>,
    /// Whether a subscription is required to view the video.
    pub requires_subscription: Option<bool>,
    /// The uploader of the video.
    pub uploader: Option<Uploader>,
    /// Whether the video is a live stream.
    pub live: Option<bool>,
    /// The tags of the video, up to 32.
    pub tags: Vec<String>,
}

impl Video {
    /// Creates the video with its required metadata. Either the content or
    /// the player location must also be set.
    pub fn new(thumbnail: Url, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            thumbnail,
            title: title.into(),
            description: description.into(),
            content: None,
            player: None,
            duration: None,
            expires: None,
            rating: None,
            views: None,
            published: None,
            family_friendly: None,
            restriction: None,
            platforms: None,
            requires_subscription: None,
            uploader: None,
            live: None,
            tags: Vec::new(),
        }
    }

    /// Sets the URL of the video file.
    pub fn with_content(mut self, content: Url) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets the URL of the video player.
    pub fn with_player(mut self, player: Url) -> Self {
        self.player = Some(player);
        self
    }
}

/// Whether a [`Restriction`] or [`Platforms`] lists the allowed or the
/// denied values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relationship {
    /// Only the listed values are allowed.
    Allow,
    /// Only the listed values are denied.
    Deny,
}

impl Relationship {
    /// Returns the value of the `relationship` attribute.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }

    pub(crate) fn parse(relationship: &str) -> Option<Self> {
        [Self::Allow, Self::Deny]
            .into_iter()
            .find(|known| known.as_str().eq_ignore_ascii_case(relationship.trim()))
    }
}

/// The countries where a [`Video`] may or may not be shown.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Restriction {
    /// Whether the countries are allowed or denied.
    pub relationship: Relationship,
    /// The ISO 3166 country codes, e.g. `US` or `DE`.
    pub countries: Vec<String>,
}

/// A platform a [`Video`] may or may not be shown on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Desktop and laptop browsers.
    Web,
    /// Mobile browsers.
    Mobile,
    /// TV browsers.
    Tv,
}

impl Platform {
    /// Returns the value listed in the `platform` element.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Mobile => "mobile",
            Self::Tv => "tv",
        }
    }

    pub(crate) fn parse(platform: &str) -> Option<Self> {
        [Self::Web, Self::Mobile, Self::Tv]
            .into_iter()
            .find(|known| known.as_str().eq_ignore_ascii_case(platform))
    }
}

/// The platforms a [`Video`] may or may not be shown on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Platforms {
    /// Whether the platforms are allowed or denied.
    pub relationship: Relationship,
    /// The platforms.
    pub platforms: Vec<Platform>,
}

/// The uploader of a [`Video`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uploader {
    /// The name of the uploader.
    pub name: String,
    /// The URL of the page with more information about the uploader.
    pub info: Option<Url>,
}
