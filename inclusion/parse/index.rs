use quick_xml::events;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::Url;

use crate::parse::{InnerParser, Output, Parser};
use crate::record::*;
use crate::{Error, Result};

/// [`Index`] builder.
#[derive(Debug, Clone, Default)]
pub(crate) struct IndexFactory {
    pub(crate) location: Option<Url>,
    pub(crate) modified: Option<OffsetDateTime>,
}

impl IndexFactory {
    /// Attempts to construct the new record.
    pub fn build(self) -> Option<Index> {
        self.location.map(|u| {
            let mut rec = Index::new(u);
            rec.modified = self.modified;
            rec
        })
    }
}

/// Sitemap index parser for the versatile XML file.
///
/// For example:
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
///    <sitemap>
///       <loc>http://www.example.com/sitemap.xml.gz</loc>
///       <lastmod>2004-10-01T18:23:17+00:00</lastmod>
///    </sitemap>
/// </sitemapindex>
/// ```
///
/// Enforces total written/read bytes and total records limits.
/// See [Error].
///
pub struct IndexParser<R> {
    inner: InnerParser<R, IndexFactory>,
}

impl<R> IndexParser<R> {
    /// Creates a new instance with the given reader.
    pub(crate) fn from_reader(reader: R) -> Self {
        let inner = InnerParser::from_reader(reader);
        Self::from_inner(inner)
    }

    /// Creates a new instance with the given inner parser.
    pub(crate) fn from_inner(inner: InnerParser<R, IndexFactory>) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        self.inner.reader.get_ref().get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.reader.get_mut().get_mut()
    }

    /// Returns an underlying reader.
    pub fn into_inner(self) -> R {
        self.inner.reader.into_inner().into_inner()
    }

    fn apply_inner(inner: &mut InnerParser<R, IndexFactory>, text: &str) {
        static LOC: [&str; 3] = [SITEMAP_INDEX, SITEMAP, LOCATION];
        static MOD: [&str; 3] = [SITEMAP_INDEX, SITEMAP, LAST_MODIFIED];

        if let Some(rec) = &mut inner.record {
            match inner.path.as_slice() {
                x if x == LOC => rec.location = Url::parse(text).ok(),
                x if x == MOD => rec.modified = OffsetDateTime::parse(text, &Iso8601::PARSING).ok(),
                _ => {}
            }
        }
    }

    pub(crate) fn write_event(&mut self, event: events::Event) -> Result<Output<Index>> {
        let tag = SITEMAP.as_bytes();
        let builder = self.inner.write_event(event, tag, Self::apply_inner);

        if let Ok(Output::Some(r)) = builder {
            if let Some(record) = r.build() {
                return Ok(Output::Some(record));
            }
        }

        Ok(Output::None)
    }
}

impl<R> std::fmt::Debug for IndexParser<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexParser")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<R: std::io::BufRead> Parser<R, Index> for IndexParser<R> {
    type Error = Error;

    fn new(reader: R) -> Result<Self> {
        // TODO: events::Event::Decl.
        Ok(Self::from_reader(reader))
    }

    fn read(&mut self) -> Result<Option<Index>> {
        let mut buf = Vec::new();
        loop {
            self.inner.try_if_readable()?;
            let event = self.inner.reader.read_event_into(&mut buf)?;
            match self.write_event(event)? {
                Output::Some(record) => return Ok(Some(record)),
                Output::None => {}
                Output::End => return Ok(None),
            }
        }
    }

    fn close(self) -> Result<R> {
        Ok(self.into_inner())
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
mod tokio {
    use tokio::io::AsyncBufRead;

    use crate::parse::{AsyncParser, IndexParser, Output};
    use crate::record::*;
    use crate::{Error, Result};

    #[async_trait::async_trait]
    impl<R: AsyncBufRead + Unpin + Send> AsyncParser<R, Index> for IndexParser<R> {
        type Error = Error;

        async fn new(reader: R) -> Result<Self> {
            // TODO: events::Event::Decl.
            Ok(Self::from_reader(reader))
        }

        async fn read(&mut self) -> Result<Option<Index>> {
            let mut buf = Vec::new();
            loop {
                self.inner.try_if_readable()?;
                let event = self.inner.reader.read_event_into_async(&mut buf).await?;
                match self.write_event(event)? {
                    Output::Some(record) => return Ok(Some(record)),
                    Output::None => {}
                    Output::End => return Ok(None),
                }
            }
        }

        async fn close(self) -> Result<R> {
            Ok(self.into_inner())
        }
    }
}
