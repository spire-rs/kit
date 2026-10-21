use url::Url;

/// A localized version of the page, from the [`hreflang` sitemap extension]:
/// an `<xhtml:link rel="alternate" hreflang="..." href="..."/>` element.
///
/// Every version of the page, including the page itself, should be listed.
///
/// [`hreflang` sitemap extension]: https://developers.google.com/search/docs/specialty/international/localized-versions#sitemap
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alternate {
    /// The language (and optionally region) code, e.g. `de` or `en-GB`, or
    /// `x-default` for the version of unmatched languages.
    pub language: String,
    /// The URL of the localized version.
    pub location: Url,
}

impl Alternate {
    /// Creates the localized version for the language.
    pub fn new(language: impl Into<String>, location: Url) -> Self {
        let language = language.into();
        Self { language, location }
    }
}
