use std::io::Write;

use countio::Counter;
use url::Url;

use crate::build::Builder;
use crate::record::*;
use crate::{Error, Result};

/// Sitemap builder for the simple TXT file that contains one URL per line.
///
/// For example:
///
/// ```txt
/// https://www.example.com/file1.html
/// https://www.example.com/file2.html
/// ```
///
/// Enforces [total written/read bytes](BYTE_LIMIT) and [total records](RECORD_LIMIT) limits.
/// See [Error].
///
/// ```rust
/// use sitemapo::build::{Builder, PlainBuilder};
///
/// fn main() -> sitemapo::Result<()> {
///     let buf = Vec::new();
///     let rec = "https://example.com/".try_into().unwrap();
///
///     let mut builder = PlainBuilder::new(buf)?;
///     builder.write(&rec)?;
///     let _buf = builder.close()?;
///     Ok(())
/// }
/// ```
pub struct PlainBuilder<W> {
    writer: Counter<W>,
    records: usize,
}

impl<W> PlainBuilder<W> {
    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        self.writer.get_ref()
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        self.writer.get_mut()
    }

    /// Returns an underlying writer.
    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }
}

impl<W> PlainBuilder<W> {
    /// Creates a new instance with a provided writer.
    pub(crate) fn from_writer(writer: W) -> Self {
        Self {
            writer: Counter::new(writer),
            records: 0,
        }
    }

    pub(crate) fn create_next_line(&mut self, url: &Url) -> Result<Vec<u8>> {
        const NEWLINE: &str = "\n";

        if self.records + 1 > RECORD_LIMIT {
            return Err(Error::EntryLimit { over: 1 });
        }

        let record = url.to_string();
        let record_bytes = record.len() + NEWLINE.len();
        let total_bytes = self.writer.writer_bytes() + record_bytes;
        if total_bytes > BYTE_LIMIT {
            let over_limit = total_bytes - BYTE_LIMIT;
            return Err(Error::ByteLimit { over: over_limit });
        }

        Ok((record + NEWLINE).into_bytes())
    }
}

impl<W: Write> Builder<W, Url> for PlainBuilder<W> {
    type Error = Error;

    fn new(writer: W) -> Result<Self> {
        Ok(Self::from_writer(writer))
    }

    fn write(&mut self, record: &Url) -> Result<()> {
        let record = self.create_next_line(record)?;
        self.writer.write_all(&record)?;
        self.records += 1;
        Ok(())
    }

    fn close(self) -> Result<W> {
        Ok(self.into_inner())
    }
}

impl<W> std::fmt::Debug for PlainBuilder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TxtBuilder")
            .field("bytes", &self.writer.writer_bytes())
            .field("records", &self.records)
            .finish()
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
mod tokio {
    use async_trait::async_trait;
    use tokio::io::{AsyncWrite, AsyncWriteExt};
    use url::Url;

    use crate::build::{AsyncBuilder, PlainBuilder};
    use crate::{Error, Result};

    #[async_trait]
    impl<W: AsyncWrite + Unpin + Send> AsyncBuilder<W, Url> for PlainBuilder<W> {
        type Error = Error;

        async fn new(writer: W) -> Result<Self> {
            Ok(Self::from_writer(writer))
        }

        async fn write(&mut self, record: &Url) -> Result<()> {
            let record = self.create_next_line(record)?;
            self.writer.write_all(&record).await?;
            self.records += 1;
            Ok(())
        }

        async fn close(self) -> Result<W> {
            Ok(self.into_inner())
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufWriter;
    use url::Url;

    use crate::build::{Builder, PlainBuilder};
    use crate::Result;

    #[test]
    fn synk() -> Result<()> {
        let buf = Vec::new();
        let mut builder = PlainBuilder::new(buf).unwrap();

        let url = Url::parse("https://example.com/").unwrap();
        builder.write(&url).unwrap();
        let buf = builder.close().unwrap();

        let exp = String::from_utf8(buf).unwrap();
        assert_eq!(url.to_string() + "\n", exp);

        Ok(())
    }

    #[test]
    fn synk_with_buf() -> Result<()> {
        let buf = BufWriter::new(Vec::new());
        let mut builder = PlainBuilder::new(buf)?;

        let url = Url::parse("https://example.com/").unwrap();
        builder.write(&url)?;
        let buf = builder.close()?;

        let buf = buf.into_inner().unwrap();
        let exp = String::from_utf8(buf).unwrap();
        assert_eq!(url.to_string() + "\n", exp);

        Ok(())
    }
}

#[cfg(feature = "tokio")]
#[cfg(test)]
mod tokio_test {
    use tokio::io::{AsyncWriteExt, BufWriter};
    use url::Url;

    use crate::build::{AsyncBuilder, PlainBuilder};
    use crate::Result;

    #[tokio::test]
    async fn asynk() -> Result<()> {
        let buf = Vec::new();
        let mut builder = PlainBuilder::new(buf).await?;

        let url = Url::parse("https://example.com/").unwrap();
        builder.write(&url).await?;
        let buf = builder.close().await?;

        let exp = String::from_utf8(buf);
        assert_eq!(Ok(url.to_string() + "\n"), exp);

        Ok(())
    }

    #[tokio::test]
    async fn asynk_with_buf() -> Result<()> {
        let buf = BufWriter::new(Vec::new());
        let mut builder = PlainBuilder::new(buf).await?;

        let url = Url::parse("https://example.com/").unwrap();
        builder.write(&url).await?;
        let mut buf = builder.close().await?;

        let _ = buf.flush().await?;
        let buf = buf.into_inner();
        let exp = String::from_utf8(buf);
        assert_eq!(Ok(url.to_string() + "\n"), exp);

        Ok(())
    }
}
