use std::io::BufRead;

use countio::Counter;
use url::Url;

use crate::parse::{try_if_readable, Parser};
use crate::{Error, Result};

/// Sitemap parser for the simple TXT file that contains one URL per line.
///
/// For example:
///
/// ```txt
/// https://www.example.com/file1.html
/// https://www.example.com/file2.html
/// ```
///
/// Enforces total written/read bytes and total records limits.
/// See [Error].
///
/// ```rust
/// use sitemapo::{parse::{Parser, PlainParser}, Error};
///
/// fn main() -> Result<(), Error> {
///     let buf = "https://example.com/file1.html".as_bytes();
///
///     let mut parser = PlainParser::new(buf).unwrap();
///     let _rec = parser.read()?;
///     let _buf = parser.close()?;
///     Ok(())
/// }
/// ```
pub struct PlainParser<R> {
    reader: Counter<R>,
    records: usize,
}

impl<R> PlainParser<R> {
    /// Creates a new instance with a provided reader.
    pub(crate) fn from_reader(reader: R) -> Self {
        Self {
            reader: Counter::new(reader),
            records: 0,
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        self.reader.get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.reader.get_mut()
    }

    /// Returns an underlying reader.
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }

    pub(crate) fn try_if_readable(&mut self) -> Result<()> {
        try_if_readable(self.records, self.reader.reader_bytes())
    }

    pub(crate) fn try_next_sync(&mut self) -> Result<Option<Url>>
    where
        R: BufRead,
    {
        loop {
            self.try_if_readable()?;
            let mut buf = String::new();
            if self.reader.read_line(&mut buf)? == 0 {
                return Ok(None);
            }

            self.records += 1;
            match Url::parse(buf.as_str()) {
                Ok(address) => return Ok(Some(address)),
                Err(_) => continue,
            }
        }
    }
}

impl<R: BufRead> Parser<R, Url> for PlainParser<R> {
    type Error = Error;

    fn new(reader: R) -> Result<Self> {
        Ok(Self::from_reader(reader))
    }

    fn read(&mut self) -> Result<Option<Url>> {
        self.try_next_sync()
    }

    fn close(self) -> Result<R> {
        Ok(self.into_inner())
    }
}

impl<R> std::fmt::Debug for PlainParser<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TxtParser")
            .field("bytes", &self.reader.reader_bytes())
            .field("records", &self.records)
            .finish()
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
mod tokio {
    use tokio::io::{AsyncBufRead, AsyncBufReadExt};
    use url::Url;

    use crate::parse::{AsyncParser, PlainParser};
    use crate::{Error, Result};

    impl<R: AsyncBufRead + Unpin + Send> PlainParser<R> {
        pub(crate) async fn try_next_async(&mut self) -> Result<Option<Url>> {
            loop {
                self.try_if_readable()?;
                let mut buf = String::new();
                if self.reader.read_line(&mut buf).await? == 0 {
                    return Ok(None);
                }

                self.records += 1;
                match Url::parse(buf.as_str()) {
                    Ok(address) => return Ok(Some(address)),
                    Err(_) => continue,
                }
            }
        }
    }

    #[async_trait::async_trait]
    impl<R: AsyncBufRead + Unpin + Send> AsyncParser<R, Url> for PlainParser<R> {
        type Error = Error;

        async fn new(reader: R) -> Result<Self> {
            Ok(Self::from_reader(reader))
        }

        async fn read(&mut self) -> Result<Option<Url>> {
            self.try_next_async().await
        }

        async fn close(self) -> Result<R> {
            Ok(self.into_inner())
        }
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use crate::{parse::PlainParser, Error};

    #[test]
    fn synk() -> Result<(), Error> {
        use crate::parse::Parser;

        let buf = r#"https://www.example.com/file1.html
        https://www.example.com/file2.html"#
            .as_bytes();

        let mut parser = PlainParser::new(buf)?;
        let url = parser.read()?;
        parser.close()?;

        let exp = Url::parse("https://www.example.com/file1.html");
        assert_eq!(url, exp.ok());

        Ok(())
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn asynk() -> Result<(), Error> {
        use crate::parse::AsyncParser;

        let buf = r#"https://www.example.com/file1.html
        https://www.example.com/file2.html"#
            .as_bytes();

        let mut parser = PlainParser::new(buf).await?;
        let url = parser.read().await?;
        parser.close().await?;

        let exp = Url::parse("https://www.example.com/file1.html");
        assert_eq!(url, exp.ok());

        Ok(())
    }
}
