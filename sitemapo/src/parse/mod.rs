use std::io::BufRead;

use quick_xml::NsReader;
use quick_xml::events::Event;
use url::Url;

use crate::limit::Limited;
use crate::parse::element::Decoder;
use crate::record::{Entry, RECORD_LIMIT, Record};
use crate::{Error, Result};

mod convert;
mod element;

/// The format of a sitemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// A text file with one URL per line.
    Text,
    /// An XML `urlset` of pages.
    UrlSet,
    /// An XML `sitemapindex` of sitemaps.
    Index,
}

/// The parser of every sitemap format.
///
/// The format is detected from the first bytes: XML documents start with
/// `<`, anything else is read as a text sitemap. Text sitemaps and XML
/// `urlset`s yield [`Record::Entry`], XML `sitemapindex`es yield
/// [`Record::Sitemap`], whose sitemaps the caller may fetch and parse in turn.
///
/// Records without a valid location are skipped, as are invalid optional
/// values. Parsing fails with [`Error::RecordLimit`] or [`Error::ByteLimit`]
/// once the sitemap goes over a limit, after the records within the limits
/// were returned. The parser returns nothing after an error.
///
/// Compressed sitemaps (`.xml.gz`) should be decompressed by the reader.
///
/// ```rust
/// use sitemapo::{Parser, Record};
///
/// let xml = br#"
///     <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
///         <url><loc>https://example.com/</loc></url>
///     </urlset>
/// "#;
///
/// for record in Parser::new(xml.as_slice()) {
///     let Record::Entry(entry) = record? else { continue };
///     assert_eq!(entry.location.as_str(), "https://example.com/");
/// }
/// # Ok::<(), sitemapo::Error>(())
/// ```
#[derive(Debug)]
pub struct Parser<R> {
    state: State<R>,
    format: Option<Format>,
    records: usize,
}

#[derive(Debug)]
enum State<R> {
    Detect(Limited<R>),
    Text(Limited<R>, Vec<u8>),
    Xml(Box<Xml<R>>),
    Done,
}

#[derive(Debug)]
struct Xml<R> {
    reader: NsReader<Limited<R>>,
    decoder: Decoder,
    buf: Vec<u8>,
}

/// Returns the first byte that tells the format apart, skipping whitespace
/// and the byte order mark.
fn first_significant(buf: &[u8]) -> Option<u8> {
    let is_skipped = |byte: &u8| byte.is_ascii_whitespace() || matches!(byte, 0xef | 0xbb | 0xbf);
    buf.iter().copied().find(|byte| !is_skipped(byte))
}

/// Parses a line of a text sitemap.
fn parse_line(line: &[u8]) -> Option<Url> {
    let line = std::str::from_utf8(line).ok()?;
    let line = line.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    Url::parse(line).ok().filter(|_| !line.is_empty())
}

impl<R> Parser<R> {
    /// Creates the parser of the sitemap from the reader.
    pub const fn new(reader: R) -> Self {
        Self {
            state: State::Detect(Limited::new(reader)),
            format: None,
            records: 0,
        }
    }

    /// Returns the format of the sitemap, once detected by the first read.
    pub const fn format(&self) -> Option<Format> {
        self.format
    }

    /// Switches to the format told apart by the first significant byte, or
    /// ends the parser of an empty sitemap.
    fn detected(&mut self, first: Option<u8>) {
        let State::Detect(reader) = std::mem::replace(&mut self.state, State::Done) else {
            return;
        };

        self.state = match first {
            Some(b'<') => State::Xml(Box::new(Xml {
                reader: NsReader::from_reader(reader),
                decoder: Decoder::default(),
                buf: Vec::new(),
            })),
            Some(_) => {
                self.format = Some(Format::Text);
                State::Text(reader, Vec::new())
            }
            None => State::Done,
        };
    }

    /// Counts the record, and stops the parser after an error.
    fn emit(&mut self, record: Result<Option<Record>>) -> Option<Result<Record>> {
        if let State::Xml(xml) = &self.state {
            self.format = self.format.or(xml.decoder.format());
        }

        let record = record.and_then(|record| {
            if record.is_some() {
                self.records += 1;
                if self.records > RECORD_LIMIT {
                    return Err(Error::RecordLimit);
                }
            }

            Ok(record)
        });

        if !matches!(record, Ok(Some(_))) {
            self.state = State::Done;
        }

        record.transpose()
    }
}

/// Returns [`Error::ByteLimit`] if the reader ended at the limit.
fn end<R>(reader: &Limited<R>) -> Result<Option<Record>> {
    if reader.is_exceeded() {
        Err(Error::ByteLimit)
    } else {
        Ok(None)
    }
}

/// Returns [`Error::ByteLimit`] instead of the XML error caused by the
/// document being cut at the limit.
fn xml_error<R>(reader: &Limited<R>, error: quick_xml::Error) -> Error {
    if reader.is_exceeded() {
        Error::ByteLimit
    } else {
        error.into()
    }
}

impl<R: BufRead> Parser<R> {
    fn read(&mut self) -> Result<Option<Record>> {
        loop {
            match &mut self.state {
                State::Detect(reader) => {
                    let first = loop {
                        let buf = reader.fill_buf()?;
                        if buf.is_empty() {
                            break None;
                        }

                        if let Some(byte) = first_significant(buf) {
                            break Some(byte);
                        }

                        let amount = buf.len();
                        reader.consume(amount);
                    };

                    self.detected(first);
                }
                State::Text(reader, line) => loop {
                    line.clear();
                    if reader.read_until(b'\n', line)? == 0 {
                        return end(reader);
                    }

                    if let Some(location) = parse_line(line) {
                        return Ok(Some(Entry::new(location).into()));
                    }
                },
                State::Xml(xml) => loop {
                    let Xml {
                        reader,
                        decoder,
                        buf,
                    } = xml.as_mut();
                    buf.clear();
                    let (ns, event) = match reader.read_resolved_event_into(buf) {
                        Ok(resolved) => resolved,
                        Err(error) => return Err(xml_error(reader.get_ref(), error)),
                    };

                    if matches!(event, Event::Eof) {
                        return end(reader.get_ref());
                    }

                    if let Some(record) = decoder.decode(&ns, event)? {
                        return Ok(Some(record));
                    }
                },
                State::Done => return Ok(None),
            }
        }
    }
}

impl<R: BufRead> Iterator for Parser<R> {
    type Item = Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        let record = self.read();
        self.emit(record)
    }
}
