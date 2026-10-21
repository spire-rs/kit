use std::io::Write;
use std::marker::PhantomData;

use url::Url;

use crate::record::{BYTE_LIMIT, Entry, RECORD_LIMIT, Sitemap};
use crate::{Error, Result};

mod xml;

/// The builder of text sitemaps.
pub type TextBuilder<W> = Builder<W, Text>;
/// The builder of XML `urlset` sitemaps.
pub type EntryBuilder<W> = Builder<W, UrlSet>;
/// The builder of XML `sitemapindex` sitemaps.
pub type IndexBuilder<W> = Builder<W, Index>;

mod private {
    pub trait Sealed {}
}

/// A sitemap format a [`Builder`] writes.
pub trait Format: private::Sealed {
    /// The record of the format.
    type Record;

    #[doc(hidden)]
    const HEADER: &'static str;
    #[doc(hidden)]
    const FOOTER: &'static str;

    #[doc(hidden)]
    fn encode(record: &Self::Record, output: &mut String);
}

/// A text file with one URL per line.
#[derive(Debug)]
pub enum Text {}

/// An XML `urlset` of pages, which declares the namespaces of every
/// supported extension.
#[derive(Debug)]
pub enum UrlSet {}

/// An XML `sitemapindex` of sitemaps.
#[derive(Debug)]
pub enum Index {}

impl private::Sealed for Text {}
impl private::Sealed for UrlSet {}
impl private::Sealed for Index {}

impl Format for Text {
    type Record = Url;

    const FOOTER: &'static str = "";
    const HEADER: &'static str = "";

    fn encode(record: &Url, output: &mut String) {
        output.push_str(record.as_str());
        output.push('\n');
    }
}

impl Format for UrlSet {
    type Record = Entry;

    const FOOTER: &'static str = xml::URL_SET_FOOTER;
    const HEADER: &'static str = xml::URL_SET_HEADER;

    fn encode(record: &Entry, output: &mut String) {
        xml::entry(record, output);
    }
}

impl Format for Index {
    type Record = Sitemap;

    const FOOTER: &'static str = xml::INDEX_FOOTER;
    const HEADER: &'static str = xml::INDEX_HEADER;

    fn encode(record: &Sitemap, output: &mut String) {
        xml::sitemap(record, output);
    }
}

/// The builder of a sitemap in the format `F`, see [`TextBuilder`],
/// [`EntryBuilder`] and [`IndexBuilder`].
///
/// A record that would take the sitemap over [`RECORD_LIMIT`] records or
/// [`BYTE_LIMIT`] bytes, including the closing tag, is rejected with
/// [`Error::RecordLimit`] or [`Error::ByteLimit`] without being written, and
/// the sitemap can still be finished. [`Builder::records`] and
/// [`Builder::bytes`] help to split the records into several sitemaps.
///
/// ```rust
/// use sitemapo::EntryBuilder;
/// use sitemapo::record::Entry;
///
/// let mut builder = EntryBuilder::new(Vec::new());
/// builder.write(&Entry::new("https://example.com/".parse().unwrap()))?;
/// let xml = String::from_utf8(builder.finish()?).unwrap();
/// assert!(xml.contains("<loc>https://example.com/</loc>"));
/// # Ok::<(), sitemapo::Error>(())
/// ```
#[derive(Debug)]
pub struct Builder<W, F> {
    writer: W,
    bytes: usize,
    records: usize,
    format: PhantomData<F>,
}

impl<W, F: Format> Builder<W, F> {
    /// Creates the builder of the sitemap into the writer.
    /// Nothing is written until the first record or [`Builder::finish`].
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            bytes: 0,
            records: 0,
            format: PhantomData,
        }
    }

    /// Returns the number of records written.
    pub const fn records(&self) -> usize {
        self.records
    }

    /// Returns the number of bytes written.
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// Returns a reference to the writer.
    pub const fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the writer.
    pub const fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Returns the bytes of the record, preceded by the header if it was not
    /// written yet, or the error of the limit it would go over.
    fn encode(&self, record: &F::Record) -> Result<String> {
        if self.records >= RECORD_LIMIT {
            return Err(Error::RecordLimit);
        }

        let mut chunk = String::new();
        if self.records == 0 {
            chunk.push_str(F::HEADER);
        }

        F::encode(record, &mut chunk);
        if self.bytes + chunk.len() + F::FOOTER.len() > BYTE_LIMIT {
            return Err(Error::ByteLimit);
        }

        Ok(chunk)
    }

    fn commit(&mut self, chunk: &str) {
        self.bytes += chunk.len();
        self.records += 1;
    }

    /// Returns the bytes that close the sitemap.
    fn closing(&self) -> String {
        let header = if self.records == 0 { F::HEADER } else { "" };
        [header, F::FOOTER].concat()
    }
}

impl<W: Write, F: Format> Builder<W, F> {
    /// Writes the record.
    ///
    /// # Errors
    ///
    /// Returns an error if the record would go over a limit, or if writing
    /// fails.
    pub fn write(&mut self, record: &F::Record) -> Result<()> {
        let chunk = self.encode(record)?;
        self.writer.write_all(chunk.as_bytes())?;
        self.commit(&chunk);
        Ok(())
    }

    /// Closes the sitemap, flushes the writer and returns it.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn finish(mut self) -> Result<W> {
        self.writer.write_all(self.closing().as_bytes())?;
        self.writer.flush()?;
        Ok(self.writer)
    }
}
