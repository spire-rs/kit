use jiff::Timestamp;

/// A news article, from the [news sitemap extension].
///
/// News sitemaps should only list articles published in the last two days.
///
/// [news sitemap extension]: https://developers.google.com/search/docs/crawling-indexing/sitemaps/news-sitemap
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct News {
    /// The publication of the article.
    pub publication: Publication,
    /// The time the article was published.
    pub published: Timestamp,
    /// The title of the article.
    pub title: String,
}

impl News {
    /// Creates the news article.
    pub fn new(publication: Publication, published: Timestamp, title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            publication,
            published,
            title,
        }
    }
}

/// The publication of a [`News`] article.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Publication {
    /// The name of the publication, exactly as it appears on its articles.
    pub name: String,
    /// The ISO 639 language code of the publication, e.g. `en`, or `zh-cn`
    /// and `zh-tw` for Chinese.
    pub language: String,
}

impl Publication {
    /// Creates the publication.
    pub fn new(name: impl Into<String>, language: impl Into<String>) -> Self {
        let name = name.into();
        let language = language.into();
        Self { name, language }
    }
}
