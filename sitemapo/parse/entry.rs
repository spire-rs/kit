use quick_xml::events;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::Url;

use crate::parse::{InnerParser, Output, Parser};
use crate::record::*;
use crate::{Error, Result};

/// [`Entry`] builder.
#[derive(Debug, Clone, Default)]
pub(crate) struct EntryFactory {
    location: Option<Url>,
    modified: Option<OffsetDateTime>,
    priority: Option<Priority>,
    frequency: Option<Frequency>,
}

impl EntryFactory {
    /// Attempts to construct the new record.
    pub fn build(self) -> Option<Entry> {
        self.location.map(|u| {
            let mut rec = Entry::new(u);
            rec.modified = self.modified;
            rec.priority = self.priority;
            rec.frequency = self.frequency;
            rec
        })
    }
}

/// Sitemap parser for the versatile XML file with an optional support of extensions.
///
/// For example:
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
///     <url>
///         <loc>https://www.example.com/foo.html</loc>
///         <lastmod>2022-06-04</lastmod>
///     </url>
/// </urlset>
/// ```
///
/// Enforces total written/read bytes and total records limits.
/// See [Error].
///
/// ```rust
/// use sitemapo::parse::{Parser, EntryParser};
///
/// fn main() -> sitemapo::Result<()> {
///     let buf = // "<urlset>...</urlset>".as_bytes();
///     # r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
///     #         <url>
///     #             <loc>https://www.example.com/file1.html</loc>
///     #             <lastmod>2022-09-08T10:43:13.000-04:00</lastmod>
///     #             <changefreq>daily</changefreq>
///     #             <priority>0.6</priority>
///     #         </url>
///     #     </urlset>
///     # "#.as_bytes();
///
///     let mut parser = EntryParser::new(buf)?;
///     let _rec = parser.read()?;
///     let _buf = parser.close()?;
///     Ok(())
/// }
/// ```
pub struct EntryParser<R> {
    inner: InnerParser<R, EntryFactory>,
}

impl<R> EntryParser<R> {
    /// Creates a new instance with the given reader.
    pub(crate) fn from_reader(reader: R) -> Self {
        let inner = InnerParser::from_reader(reader);
        Self::from_inner(inner)
    }

    /// Creates a new instance with the given inner parser.
    pub(crate) fn from_inner(inner: InnerParser<R, EntryFactory>) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    /// Returns an underlying reader.
    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }

    fn apply_inner(inner: &mut InnerParser<R, EntryFactory>, text: &str) {
        static LOC: [&str; 3] = [URL_SET, URL, LOCATION];
        static MOD: [&str; 3] = [URL_SET, URL, LAST_MODIFIED];
        static FRQ: [&str; 3] = [URL_SET, URL, CHANGE_FREQUENCY];
        static PRI: [&str; 3] = [URL_SET, URL, PRIORITY];

        if let Some(rec) = &mut inner.record {
            match inner.path.as_slice() {
                x if x == LOC => rec.location = Url::parse(text).ok(),
                x if x == MOD => rec.modified = OffsetDateTime::parse(text, &Iso8601::PARSING).ok(),
                x if x == FRQ => rec.frequency = Frequency::parse(text).ok(),
                x if x == PRI => rec.priority = Priority::parse(text).ok(),
                _ => {}
            }
        }
    }

    pub(crate) fn write_event(&mut self, event: events::Event) -> Result<Output<Entry>> {
        let tag = URL.as_bytes();
        let builder = self.inner.write_event(event, tag, Self::apply_inner);

        if let Ok(Output::Some(r)) = builder {
            if let Some(record) = r.build() {
                return Ok(Output::Some(record));
            }
        }

        Ok(Output::None)
    }
}

impl<R> std::fmt::Debug for EntryParser<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryParser")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<R: std::io::BufRead> Parser<R, Entry> for EntryParser<R> {
    type Error = Error;

    fn new(reader: R) -> Result<Self> {
        // TODO: events::Event::Decl.
        Ok(Self::from_reader(reader))
    }

    fn read(&mut self) -> Result<Option<Entry>> {
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
mod async_parser {
    use tokio::io::AsyncBufRead;

    use crate::parse::{AsyncParser, EntryParser, Output};
    use crate::record::Entry;
    use crate::{Error, Result};

    #[async_trait::async_trait]
    impl<R: AsyncBufRead + Unpin + Send> AsyncParser<R, Entry> for EntryParser<R> {
        type Error = Error;

        async fn new(reader: R) -> Result<Self> {
            // TODO: events::Event::Decl.
            Ok(Self::from_reader(reader))
        }

        async fn read(&mut self) -> Result<Option<Entry>> {
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

#[cfg(test)]
mod test {
    use url::Url;

    use crate::parse::EntryParser;
    use crate::record::Entry;
    use crate::Result;

    const EXAMPLE: &'static str = r#"
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url>
            <loc>https://www.example.com/file1.html</loc>
            <lastmod>2022-09-08T10:43:13.000-04:00</lastmod>
            <changefreq>daily</changefreq>
            <priority>0.6</priority>
        </url>
    </urlset>"#;

    #[test]
    fn synk() -> Result<()> {
        use crate::parse::Parser;

        let buf = EXAMPLE.as_bytes();
        let mut parser = EntryParser::new(buf)?;
        let record: Entry = parser.read()?.unwrap();
        parser.close()?;

        let exp = Url::parse("https://www.example.com/file1.html");
        assert_eq!(record.location, exp.unwrap());

        Ok(())
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn asynk() -> Result<()> {
        use crate::parse::AsyncParser;

        let buf = EXAMPLE.as_bytes();
        let mut parser = EntryParser::new(buf).await?;
        let record: Entry = parser.read().await?.unwrap();
        parser.close().await?;

        let exp = Url::parse("https://www.example.com/file1.html");
        assert_eq!(record.location, exp.unwrap());

        Ok(())
    }
}
