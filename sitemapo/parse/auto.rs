use bytes::Bytes;
use countio::Counter;
use quick_xml::{events, Reader};
use url::Url;

use crate::{parse::*, record::*, Error};

/// Sitemap type resolver.
// TODO: Check for the plain txt sitemaps.
pub enum Scanner<R> {
    Plain(PlainParser<R>),
    Entry(EntryParser<R>),
    Index(IndexParser<R>),
}

impl<R> Scanner<R> {
    fn try_if_readable(reader: &Reader<Counter<R>>) -> Result<(), Error> {
        try_if_readable(0, reader.get_ref().reader_bytes())
    }

    /// Returns `Some(_)` is the opening tag was found, `bool` is true if the sitemap is an index.
    fn is_xml_sitemap(event: events::Event) -> Option<bool> {
        if let events::Event::Start(bytes) = event {
            let name = bytes.name().into_inner();
            if name.eq_ignore_ascii_case(SITEMAP_INDEX.as_bytes()) {
                return Some(true);
            } else if name.eq_ignore_ascii_case(URL_SET.as_bytes()) {
                return Some(false);
            }
        }

        None
    }

    fn create_xml(is_index: bool, reader: Reader<Counter<R>>) -> Self {
        let reader = reader.into_inner().into_inner();
        if is_index {
            let mut reader = InnerParser::from_reader(reader);
            let bytes = Bytes::from(SITEMAP_INDEX.as_bytes().to_vec());
            reader.path = Vec::from([bytes]);
            Self::Index(IndexParser::from_inner(reader))
        } else {
            let mut reader = InnerParser::from_reader(reader);
            let bytes = Bytes::from(URL_SET.as_bytes().to_vec());
            reader.path = Vec::from([bytes]);
            Self::Entry(EntryParser::from_inner(reader))
        }
    }
}

impl<R: std::io::BufRead> Scanner<R> {
    /// Creates a new instance with the given reader.
    pub fn from_sync(reader: R) -> Result<Self, Error> {
        let mut reader = Reader::from_reader(Counter::new(reader));
        let mut buf = Vec::new();

        loop {
            Self::try_if_readable(&reader)?;
            let event = reader.read_event_into(&mut buf)?;
            if let Some(is_index) = Self::is_xml_sitemap(event) {
                return Ok(Self::create_xml(is_index, reader));
            }
        }
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<R: tokio::io::AsyncBufRead + Unpin + Send> Scanner<R> {
    /// Creates a new instance with the given reader.
    pub async fn from_async(reader: R) -> Result<Self, Error> {
        let mut reader = Reader::from_reader(Counter::new(reader));
        let mut buf = Vec::new();

        loop {
            Self::try_if_readable(&reader)?;
            let event = reader.read_event_into_async(&mut buf).await?;
            if let Some(is_index) = Self::is_xml_sitemap(event) {
                return Ok(Self::create_xml(is_index, reader));
            }
        }
    }
}

/// Automatic sitemap record resolver.
///
/// ```rust
/// #[derive(Debug, thiserror::Error)]
/// enum CustomError {
///     // ..
///     #[error("sitemap error: {0}")]
///     Sitemap(#[from] sitemapo::Error),
///     //..
/// }
///
/// fn main() -> Result<(), CustomError> {
///     type SyncReader = std::io::BufReader<std::io::Cursor<Vec<u8>>>;
///     fn fetch(_: url::Url) -> Result<SyncReader, CustomError> {
///         // ..
///         unreachable!()
///     }
///
///     // Sitemaps listed in the robots.txt file.
///     let sitemaps = Vec::default();
///
///     let mut parser = sitemapo::parse::AutoParser::new(sitemaps);
///     while let Some(_record) = parser.try_sync(fetch)? {
///         // ..
///     }
///
///     Ok(())
/// }
/// ```
pub struct AutoParser<R> {
    sitemaps: Vec<Url>,
    plain: Option<PlainParser<R>>,
    entry: Option<EntryParser<R>>,
    index: Option<IndexParser<R>>,
}

impl<R> AutoParser<R> {
    /// Creates a new instance from the `robots.txt` provided list of root sitemaps.
    pub fn new(sitemaps: impl IntoIterator<Item = Url>) -> Self {
        Self {
            sitemaps: sitemaps.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Replaces the currently stored parser.
    fn replace_parser(&mut self, detector: Scanner<R>) {
        match detector {
            Scanner::Plain(parser) => self.plain = Some(parser),
            Scanner::Entry(parser) => self.entry = Some(parser),
            Scanner::Index(parser) => self.index = Some(parser),
        }
    }

    /// Returns `true` if no more sitemaps left to parse.
    pub fn is_empty(&self) -> bool {
        self.sitemaps.is_empty()
            && self.plain.is_none()
            && self.index.is_none()
            && self.entry.is_none()
    }

    /// Returns minimal (no resolved indexes) total sitemaps amount.
    pub fn len(&self) -> usize {
        self.sitemaps.len()
            + self.plain.is_some() as usize
            + self.index.is_some() as usize
            + self.entry.is_some() as usize
    }
}

// TODO: Iterator.
impl<R> AutoParser<R>
where
    R: std::io::BufRead,
{
    /// TODO: Desc.
    ///
    /// Silently ignores errors, skips failed sitemaps.
    pub fn try_sync<E, A>(&mut self, fetcher: A) -> Result<Option<Entry>, E>
    where
        E: std::error::Error + From<Error>,
        A: Fn(Url) -> Result<R, E>,
    {
        while !self.is_empty() {
            if let Some(parser) = self.plain.as_mut() {
                if let Ok(Some(record)) = parser.read() {
                    return Ok(Some(record.into()));
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(parser) = self.entry.as_mut() {
                if let Ok(Some(record)) = parser.read() {
                    return Ok(Some(record));
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(parser) = self.index.as_mut() {
                if let Ok(Some(record)) = parser.read() {
                    let reader = (fetcher)(record.location.clone())?;
                    // Ignore nested sitemap index or error.
                    match Scanner::from_sync(reader).ok() {
                        Some(Scanner::Index(_)) | None => {}
                        Some(parser) => self.replace_parser(parser),
                    }
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(sitemap) = self.sitemaps.pop() {
                let reader = (fetcher)(sitemap)?;
                if let Ok(sitemap) = Scanner::from_sync(reader) {
                    self.replace_parser(sitemap)
                }
            }

            // ...
        }

        Ok(None)
    }
}

// TODO: AsyncIterator/Stream.
// https://tokio.rs/tokio/tutorial/streams
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<R> AutoParser<R>
where
    R: tokio::io::AsyncBufRead + Unpin + Send,
{
    /// TODO: Desc.
    ///
    /// Silently ignores errors, skips failed sitemaps.
    pub async fn try_async<E, A, F>(&mut self, fetcher: A) -> Result<Option<Entry>, E>
    where
        E: std::error::Error + From<Error>,
        F: std::future::Future<Output = Result<R, E>>,
        A: Fn(Url) -> F,
    {
        while !self.is_empty() {
            if let Some(parser) = self.plain.as_mut() {
                if let Ok(Some(record)) = parser.read().await {
                    return Ok(Some(record.into()));
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(parser) = self.entry.as_mut() {
                if let Ok(Some(record)) = parser.read().await {
                    return Ok(Some(record));
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(parser) = self.index.as_mut() {
                if let Ok(Some(record)) = parser.read().await {
                    let reader = (fetcher)(record.location.clone()).await?;
                    // Ignore nested sitemap index or error.
                    match Scanner::from_async(reader).await.ok() {
                        Some(Scanner::Index(_)) | None => {}
                        Some(parser) => self.replace_parser(parser),
                    }
                }

                self.plain.take(); // If EOF or Error.
            }

            if let Some(sitemap) = self.sitemaps.pop() {
                let reader = (fetcher)(sitemap).await?;
                if let Ok(parser) = Scanner::from_async(reader).await {
                    self.replace_parser(parser)
                }
            }

            // ...
        }

        Ok(None)
    }
}

impl<R> std::fmt::Debug for AutoParser<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoParser")
            .field("sitemaps", &self.sitemaps)
            .field("plain", &self.plain)
            .field("index", &self.index)
            .field("entry", &self.entry)
            .finish()
    }
}

impl<R> Default for AutoParser<R> {
    fn default() -> Self {
        Self {
            sitemaps: Vec::new(),
            plain: None,
            index: None,
            entry: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    enum CustomError {
        #[error("sitemap error: {0}")]
        Sitemap(#[from] Error),
    }

    #[test]
    fn synk() -> Result<(), CustomError> {
        type SyncReader = std::io::BufReader<std::io::Cursor<Vec<u8>>>;
        fn sync_fetcher(_: Url) -> Result<SyncReader, CustomError> {
            unreachable!()
        }

        let mut a: AutoParser<SyncReader> = AutoParser::new([]);
        let _ = a.try_sync(sync_fetcher);
        Ok(())
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn asynk() -> Result<(), CustomError> {
        type AsyncReader = tokio::io::BufReader<std::io::Cursor<Vec<u8>>>;
        async fn async_fetcher(_: Url) -> Result<AsyncReader, test::CustomError> {
            unreachable!()
        }

        let mut a: AutoParser<AsyncReader> = AutoParser::new([]);
        let _ = a.try_async(async_fetcher).await;
        Ok(())
    }

    #[derive(Debug)]
    struct ResourceState {
        sitemaps: Option<AutoParser<()>>,
    }

    #[test]
    fn state() -> Result<(), Error> {
        let mut state = ResourceState { sitemaps: None };
        state.sitemaps = Some(AutoParser::new([]));
        Ok(())
    }
}
