mod auto;
mod entry;
mod index;
mod inner;
mod plain;

pub use auto::*;
pub use entry::*;
pub use index::*;
pub(crate) use inner::*;
pub use plain::*;

/// Core trait for the parser implementation.
pub trait Parser<R: std::io::Read, D>: Sized {
    type Error: std::error::Error;

    // Creates a new `Parser` instance.
    fn new(reader: R) -> Result<Self, Self::Error>;

    /// Reads another record from the underlying reader.
    fn read(&mut self) -> Result<Option<D>, Self::Error>;

    /// Closes tags if needed and releases the reader.
    fn close(self) -> Result<R, Self::Error>;
}

/// Core trait for the async parser implementation.
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[async_trait::async_trait]
pub trait AsyncParser<R: tokio::io::AsyncRead, D>: Sized {
    type Error: std::error::Error;

    // Creates a new `AsyncParser` instance.
    async fn new(reader: R) -> Result<Self, Self::Error>;

    /// Reads another record from the underlying reader.
    async fn read(&mut self) -> Result<Option<D>, Self::Error>;

    /// Closes tags if needed and releases the reader.
    async fn close(self) -> Result<R, Self::Error>;
}

pub(crate) fn try_if_readable(records: usize, bytes: usize) -> crate::Result<()> {
    use crate::record::{BYTE_LIMIT, RECORD_LIMIT};

    if records + 1 > RECORD_LIMIT {
        return Err(crate::Error::EntryLimit { over: 1 });
    }

    if bytes > BYTE_LIMIT {
        let over = bytes - BYTE_LIMIT;
        return Err(crate::Error::ByteLimit { over });
    }

    Ok(())
}
