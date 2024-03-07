use url::Url;

use crate::build::{EntryBuilder, IndexBuilder};
use crate::record::Entry;
use crate::Error;

/// TODO: Desc.
///
/// Automatic sitemap file constructor.
/// NOTE: Does not deduplicate records.
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
///     Ok(())
/// }
/// ```
pub struct AutoBuilder<W> {
    index: Option<IndexBuilder<W>>,
    entry: Vec<EntryBuilder<W>>,
    queue: Vec<Entry>,
    // factory: impl Fn() -> W,
}

impl<W> AutoBuilder<W> {
    /// TODO: Desc.
    pub fn new() -> Self {
        todo!()
    }
}

impl<W> AutoBuilder<W>
where
    W: std::io::Write,
{
    /// TODO: Desc.
    pub fn try_sync<E, A>(&mut self, fetcher: A) -> Result<(), E>
    where
        E: std::error::Error + From<Error>,
        A: Fn(Url) -> Result<Vec<Entry>, E>,
    {
        // if let Some(builder) = self.entry.as_mut() {
        //     builder.write(record)
        // }

        todo!()
    }
}

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
impl<W> AutoBuilder<W>
where
    W: tokio::io::AsyncWrite + Unpin + Send,
{
    /// TODO: Desc.
    pub async fn try_async(&mut self) -> Result<(), Error> {
        todo!()
    }
}

impl<W> std::fmt::Debug for AutoBuilder<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Debug.
        f.debug_struct("AutoBuilder").finish()
    }
}

// impl<W> Default for AutoBuilder<W> {
//     fn default() -> Self {
//         Self {
//             entry: None,
//             index: None,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sync() -> Result<(), Error> {
        // TODO: Test.
        Ok(())
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn asynk() -> Result<(), Error> {
        // TODO: Test.
        Ok(())
    }
}
