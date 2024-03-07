use bytes::Bytes;
use countio::Counter;
use quick_xml::{events::Event, Reader};

use crate::parse::try_if_readable;
use crate::Result;

pub(crate) enum Output<T> {
    /// Next record.
    Some(T),
    /// The event didn't result into the new record.
    None,
    /// End of file.
    End,
}

impl<T> From<Option<T>> for Output<T> {
    fn from(value: Option<T>) -> Self {
        value.map(Output::Some).unwrap_or(Output::End)
    }
}

pub(crate) struct InnerParser<R, D> {
    pub(crate) record: Option<D>,
    pub(crate) reader: Reader<Counter<R>>,
    pub(crate) records: usize,
    pub(crate) path: Vec<Bytes>,
}

impl<R, D> InnerParser<R, D> {
    /// Creates a new instance with the given reader.
    pub fn from_reader(reader: R) -> Self {
        Self {
            record: None,
            reader: Reader::from_reader(Counter::new(reader)),
            records: 0,
            path: Vec::default(),
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn get_ref(&self) -> &R {
        self.reader.get_ref().get_ref()
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.reader.get_mut().get_mut()
    }

    /// Returns an underlying reader.
    pub fn into_inner(self) -> R {
        self.reader.into_inner().into_inner()
    }

    pub fn try_if_readable(&mut self) -> Result<()> {
        try_if_readable(self.records, self.reader.get_ref().reader_bytes())
    }

    /// TODO: Desc.
    pub fn write_event<F>(&mut self, next: Event, tag: &[u8], apply: F) -> Result<Output<D>>
    where
        D: Default,
        F: FnOnce(&mut Self, &str),
    {
        match next {
            // Replace the old record builder with the new one.
            Event::Start(bytes) => {
                let name = bytes.name().into_inner();
                if name.eq_ignore_ascii_case(tag) {
                    self.records += 1;
                    let instance = D::default();
                    self.record.replace(instance);
                }

                self.path.push(name.to_vec().into());
            }

            // Try to apply changes to the current record.
            Event::Text(bytes) => {
                let text = bytes.unescape()?;
                apply(self, &text);
            }

            // Return the current record if the closing tag is matched.
            Event::End(bytes) => {
                let name = bytes.name().into_inner().to_vec();
                if self.path.pop() != Some(name.clone().into()) {
                    // TODO: Skip til next start tag.
                }

                if name.eq_ignore_ascii_case(tag) {
                    let rec = self.record.take();
                    return Ok(rec.into());
                }
            }

            // Try to return the last entry or None as EOF.
            Event::Eof => {
                let rec = self.record.take();
                return Ok(rec.into());
            }

            _ => {} // Ignore.
        }

        Ok(Output::None)
    }
}

impl<R, D> std::fmt::Debug for InnerParser<R, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerParser")
            .field("bytes", &self.reader.get_ref().reader_bytes())
            .field("records", &self.records)
            .finish()
    }
}
