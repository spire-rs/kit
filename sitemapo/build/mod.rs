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

// TODO: Make builders take BufWrite.

/// Core trait for the builder implementation.
pub trait Builder<W: std::io::Write, D>: Sized {
    type Error: std::error::Error;

    // Creates a new `Builder` instance.
    fn new(writer: W) -> Result<Self, Self::Error>;

    /// Writes another record into the underlying writer.
    fn write(&mut self, record: &D) -> Result<(), Self::Error>;

    /// Closes tags if needed and releases the writer.
    fn close(self) -> Result<W, Self::Error>;
}

/// Core trait for the async builder implementation.
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[async_trait::async_trait]
pub trait AsyncBuilder<W: tokio::io::AsyncWrite, D>: Sized {
    type Error: std::error::Error;

    // Creates a new `AsyncBuilder` instance.
    async fn new(writer: W) -> Result<Self, Self::Error>;

    /// Writes another record into the underlying writer.
    async fn write(&mut self, record: &D) -> Result<(), Self::Error>;

    /// Closes tags if needed and releases the writer.
    async fn close(self) -> Result<W, Self::Error>;
}
