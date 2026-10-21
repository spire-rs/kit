use url::Url;

/// An image of the page, from the [image sitemap extension].
///
/// A page can list up to 1,000 images. Google no longer reads the caption,
/// title, geo location and license tags, so only the location is kept.
///
/// [image sitemap extension]: https://developers.google.com/search/docs/crawling-indexing/sitemaps/image-sitemaps
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Image {
    /// The URL of the image.
    pub location: Url,
}

impl Image {
    /// Creates the image with the given location.
    pub const fn new(location: Url) -> Self {
        Self { location }
    }
}

impl From<Url> for Image {
    fn from(location: Url) -> Self {
        Self::new(location)
    }
}
