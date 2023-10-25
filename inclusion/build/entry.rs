use std::io::Write;

use quick_xml::{events, Writer};
use time::format_description::well_known::Iso8601;

use crate::build::{Builder, InnerBuilder, CONFIG};
use crate::record::*;
use crate::{Error, Result};

/// Sitemap builder for the versatile XML file with an optional support of extensions.
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
/// use sitemapo::build::{Builder, EntryBuilder};
/// use sitemapo::record::Entry;
///
/// fn main() -> sitemapo::Result<()> {
///     let buf = Vec::new();
///     let url = "https://example.com/".try_into().unwrap();
///     let rec = Entry::new(url);
///
///     let mut builder = EntryBuilder::new(buf)?;
///     builder.write(&rec)?;
///     let _buf = builder.close()?;
///     Ok(())
/// }
/// ```
pub struct EntryBuilder<W> {
    inner: InnerBuilder<W, Entry>,
}

impl<W> EntryBuilder<W> {
    /// Creates a new instance with the given writer.
    pub(crate) fn from_writer(writer: W) -> Self {
        let inner = InnerBuilder::from_writer(writer);
        Self::from_inner(inner)
    }

    /// Creates a new instance with the given inner parser.
    pub(crate) fn from_inner(inner: InnerBuilder<W, Entry>) -> Self {
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

    pub(crate) fn create_entry_open(&mut self) -> Result<Vec<u8>> {
        self.inner.create_open_tag(URL_SET)
    }

    pub(crate) fn create_entry_record(&mut self, record: &Entry) -> Result<Vec<u8>> {
        if self.inner.records + 1 > RECORD_LIMIT {
            return Err(Error::EntryLimit { over: 1 });
        }

        let format = &Iso8601::<{ CONFIG }>;
        let location = record.location().to_string();
        let modified = record.modified().map(|u| u.format(format).unwrap());
        let priority = record.priority().map(|u| u.to_string());
        let frequency = record.frequency().map(|u| u.to_string());

        let mut temp = Writer::new(Vec::new());
        let element = temp.create_element(URL);
        element.write_inner_content(|writer| -> quick_xml::Result<()> {
            let tag = writer.create_element(LOCATION);
            tag.write_text_content(events::BytesText::new(&location))?;

            if let Some(modified) = modified {
                let tag = writer.create_element(LAST_MODIFIED);
                tag.write_text_content(events::BytesText::new(&modified))?;
            }

            if let Some(priority) = priority {
                let tag = writer.create_element(PRIORITY);
                tag.write_text_content(events::BytesText::new(&priority))?;
            }

            if let Some(frequency) = frequency {
                let tag = writer.create_element(CHANGE_FREQUENCY);
                tag.write_text_content(events::BytesText::new(&frequency))?;
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

    pub(crate) fn create_entry_close(&mut self) -> Result<Vec<u8>> {
        self.inner.create_close_tag(URL_SET)
    }
}

impl<W> std::fmt::Debug for EntryBuilder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntryBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<W: Write> Builder<W, Entry> for EntryBuilder<W> {
    type Error = Error;

    fn new(writer: W) -> Result<Self> {
        let mut this = Self::from_writer(writer);
        let temp = this.create_entry_open()?;
        this.inner.writer.write_all(&temp)?;
        Ok(this)
    }

    fn write(&mut self, record: &Entry) -> Result<()> {
        let temp = self.create_entry_record(record)?;
        self.inner.writer.write_all(&temp)?;
        self.inner.records += 1;
        Ok(())
    }

    fn close(mut self) -> Result<W> {
        let temp = self.create_entry_close()?;
        self.inner.writer.write_all(&temp)?;
        Ok(self.into_inner())
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
mod tokio {
    use async_trait::async_trait;
    use tokio::io::{AsyncWrite, AsyncWriteExt};

    use crate::build::{AsyncBuilder, EntryBuilder};
    use crate::record::Entry;
    use crate::{Error, Result};

    #[async_trait]
    impl<W: AsyncWrite + Unpin + Send> AsyncBuilder<W, Entry> for EntryBuilder<W> {
        type Error = Error;

        async fn new(writer: W) -> Result<Self> {
            let mut this = Self::from_writer(writer);
            let temp = this.create_entry_open()?;
            this.inner.writer.write_all(&temp).await?;
            Ok(this)
        }

        async fn write(&mut self, record: &Entry) -> Result<()> {
            let temp = self.create_entry_record(record)?;
            self.inner.writer.write_all(&temp).await?;
            self.inner.records += 1;
            Ok(())
        }

        async fn close(mut self) -> Result<W> {
            let temp = self.create_entry_close()?;
            self.inner.writer.write_all(&temp).await?;
            Ok(self.into_inner())
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;
    use url::Url;

    use crate::build::{Builder, EntryBuilder};
    use crate::record::Entry;
    use crate::Result;

    #[test]
    fn synk() -> Result<()> {
        let buf = Vec::new();
        let mut builder = EntryBuilder::new(buf)?;

        let url = Url::parse("https://example.com/").unwrap();
        let rec = Entry::new(url);
        builder.write(&rec)?;
        let _buf = builder.close()?;

        Ok(())
    }

    #[test]
    fn synk_with_buf() -> Result<()> {
        let buf = BufWriter::new(Vec::new());
        let mut builder = EntryBuilder::new(buf)?;

        let url = Url::parse("https://example.com/").unwrap();
        let rec = Entry::new(url);
        builder.write(&rec)?;
        let _buf = builder.close()?;

        Ok(())
    }
}

#[cfg(feature = "tokio")]
#[cfg(test)]
mod tokio_test {
    use tokio::io::{AsyncWriteExt, BufWriter};
    use url::Url;

    use crate::build::{AsyncBuilder, EntryBuilder};
    use crate::{record::Entry, Result};

    #[tokio::test]
    async fn asynk() -> Result<()> {
        let buf = Vec::new();
        let mut builder = EntryBuilder::new(buf).await?;

        let url = Url::parse("https://example.com/").unwrap();
        let rec = Entry::new(url);
        builder.write(&rec).await?;
        let _buf = builder.close().await?;

        Ok(())
    }

    #[tokio::test]
    async fn asynk_with_buf() -> Result<()> {
        let buf = BufWriter::new(Vec::new());
        let mut builder = EntryBuilder::new(buf).await?;

        let url = Url::parse("https://example.com/").unwrap();

        let rec = Entry::new(url);
        builder.write(&rec).await?;
        let mut buf = builder.close().await?;

        let _ = buf.flush().await?;

        Ok(())
    }
}
