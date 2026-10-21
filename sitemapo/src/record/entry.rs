use jiff::Timestamp;
use url::Url;

use crate::record::{Alternate, Frequency, Image, News, Priority, Video};

/// A page listed in a text sitemap or an XML `urlset`.
///
/// Text sitemaps only provide the location.
///
/// ```rust
/// use sitemapo::record::{Entry, Frequency, Image, Priority};
///
/// let location = "https://example.com/".parse().unwrap();
/// let entry = Entry::new(location)
///     .with_modified("2024-01-01T00:00:00Z".parse().unwrap())
///     .with_frequency(Frequency::Daily)
///     .with_priority(Priority::MAX)
///     .with_image(Image::new("https://example.com/image.png".parse().unwrap()));
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The URL of the page.
    pub location: Url,
    /// The time the page was last modified.
    pub modified: Option<Timestamp>,
    /// How often the page is likely to change.
    pub frequency: Option<Frequency>,
    /// The priority of the page relative to other pages of the site.
    pub priority: Option<Priority>,
    /// The images of the page, see [`Image`].
    pub images: Vec<Image>,
    /// The videos of the page, see [`Video`].
    pub videos: Vec<Video>,
    /// The news article of the page, see [`News`].
    pub news: Option<News>,
    /// The localized versions of the page, see [`Alternate`].
    pub alternates: Vec<Alternate>,
}

impl Entry {
    /// Creates the entry of the page.
    pub const fn new(location: Url) -> Self {
        Self {
            location,
            modified: None,
            frequency: None,
            priority: None,
            images: Vec::new(),
            videos: Vec::new(),
            news: None,
            alternates: Vec::new(),
        }
    }

    /// Sets the time the page was last modified.
    pub const fn with_modified(mut self, modified: Timestamp) -> Self {
        self.modified = Some(modified);
        self
    }

    /// Sets how often the page is likely to change.
    pub const fn with_frequency(mut self, frequency: Frequency) -> Self {
        self.frequency = Some(frequency);
        self
    }

    /// Sets the priority of the page.
    pub const fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Adds an image of the page.
    pub fn with_image(mut self, image: Image) -> Self {
        self.images.push(image);
        self
    }

    /// Adds a video of the page.
    pub fn with_video(mut self, video: Video) -> Self {
        self.videos.push(video);
        self
    }

    /// Sets the news article of the page.
    pub fn with_news(mut self, news: News) -> Self {
        self.news = Some(news);
        self
    }

    /// Adds a localized version of the page.
    pub fn with_alternate(mut self, alternate: Alternate) -> Self {
        self.alternates.push(alternate);
        self
    }
}

impl From<Url> for Entry {
    fn from(location: Url) -> Self {
        Self::new(location)
    }
}
