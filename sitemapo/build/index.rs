use std::io::Write;

use quick_xml::{events, Writer};
use time::format_description::well_known::Iso8601;

use crate::build::{Builder, InnerBuilder, CONFIG};
use crate::record::*;
use crate::{Error, Result};

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
/// ```rust
/// use sitemapo::build::{Builder, IndexBuilder};
/// use sitemapo::record::Index;
///
/// fn main() -> sitemapo::Result<()> {
///     let buf = Vec::new();
///     let url = "https://example.com/".try_into().unwrap();
///     let rec = Index::new(url);
///
///     let mut builder = IndexBuilder::new(buf)?;
///     builder.write(&rec)?;
///     let _buf = builder.close()?;
///     Ok(())
/// }
/// ```
pub struct IndexBuilder<W> {
    inner: InnerBuilder<W, Index>,
}

impl<W> IndexBuilder<W> {
    /// Creates a new instance with the given writer.
    pub(crate) fn from_writer(writer: W) -> Self {
        let inner = InnerBuilder::from_writer(writer);
        Self::from_inner(inner)
    }

    /// Creates a new instance with the given inner parser.
    pub(crate) fn from_inner(inner: InnerBuilder<W, Index>) -> Self {
        Self { inner }
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.get_mut()
    }

    /// Returns an underlying writer.
    pub fn into_inner(self) -> W {
        self.inner.into_inner()
    }

    pub(crate) fn create_index_open(&mut self) -> Result<Vec<u8>> {
        self.inner.create_open_tag(SITEMAP_INDEX)
    }

    pub(crate) fn create_index_record(&mut self, record: &Index) -> Result<Vec<u8>> {
        if self.inner.records + 1 > RECORD_LIMIT {
            return Err(Error::EntryLimit { over: 1 });
        }

        let format = &Iso8601::<{ CONFIG }>;
        let location = record.location.to_string();
        let modified = record.modified.map(|u| u.format(format).unwrap());

        let mut temp = Writer::new(Vec::new());
        let element = temp.create_element(SITEMAP);
        element.write_inner_content(|writer| -> quick_xml::Result<()> {
            let tag = writer.create_element(LOCATION);
            tag.write_text_content(events::BytesText::new(&location))?;

            if let Some(modified) = modified {
                let tag = writer.create_element(LAST_MODIFIED);
                tag.write_text_content(events::BytesText::new(&modified))?;
            }

            Ok(())
        })?;

        let buf = temp.into_inner();
        if buf.len() > BYTE_LIMIT {
            let over_limit = buf.len() - BYTE_LIMIT;
            return Err(Error::ByteLimit { over: over_limit });
        }

        Ok(buf)
    }

    pub(crate) fn create_index_close(&mut self) -> Result<Vec<u8>> {
        self.inner.create_close_tag(SITEMAP_INDEX)
    }
}

impl<W> std::fmt::Debug for IndexBuilder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<W: Write> Builder<W, Index> for IndexBuilder<W> {
    type Error = Error;

    fn new(writer: W) -> Result<Self> {
        let mut this = Self::from_writer(writer);
        let temp = this.create_index_open()?;
        this.inner.writer.write_all(&temp)?;
        Ok(this)
    }

    fn write(&mut self, record: &Index) -> Result<()> {
        let temp = self.create_index_record(record)?;
        self.inner.writer.write_all(&temp)?;
        self.inner.records += 1;
        Ok(())
    }

    fn close(mut self) -> Result<W> {
        let temp = self.create_index_close()?;
        self.inner.writer.write_all(&temp)?;
        Ok(self.into_inner())
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
mod tokio {
    use async_trait::async_trait;
    use tokio::io::{AsyncWrite, AsyncWriteExt};

    use crate::build::{AsyncBuilder, IndexBuilder};
    use crate::record::Index;
    use crate::{Error, Result};

    #[async_trait]
    impl<W: AsyncWrite + Unpin + Send> AsyncBuilder<W, Index> for IndexBuilder<W> {
        type Error = Error;

        async fn new(writer: W) -> Result<Self> {
            let mut this = Self::from_writer(writer);
            let temp = this.create_index_open()?;
            this.inner.writer.write_all(&temp).await?;
            Ok(this)
        }

        async fn write(&mut self, record: &Index) -> Result<()> {
            let temp = self.create_index_record(record)?;
            self.inner.writer.write_all(&temp).await?;
            self.inner.records += 1;
            Ok(())
        }

        async fn close(mut self) -> Result<W> {
            let temp = self.create_index_close()?;
            self.inner.writer.write_all(&temp).await?;
            Ok(self.into_inner())
        }
    }
}
