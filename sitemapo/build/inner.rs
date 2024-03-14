use std::{marker::PhantomData, num::NonZeroU8};

use countio::Counter;
use quick_xml::{events, Writer};
use time::format_description::well_known::iso8601;

use crate::Error;

pub(crate) const CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
    .set_time_precision(iso8601::TimePrecision::Second {
        decimal_digits: NonZeroU8::new(2),
    })
    .encode();

pub(crate) struct InnerBuilder<W, D> {
    pub(crate) record: PhantomData<D>,
    pub(crate) writer: Counter<W>,
    pub(crate) records: usize,
}

impl<W, D> InnerBuilder<W, D> {
    /// Creates a new instance with a provided writer.
    pub fn from_writer(writer: W) -> Self {
        Self {
            record: PhantomData,
            writer: Counter::new(writer),
            records: 0,
        }
    }

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

    pub fn create_open_tag(&mut self, tag: &str) -> Result<Vec<u8>, Error> {
        let mut temp = Writer::new(Vec::new());
        temp.write_bom()?;

        // <?xml version="1.0" encoding="UTF-8"?>
        let decl = events::BytesDecl::new("1.0", Some("UTF-8"), None);
        temp.write_event(events::Event::Decl(decl))?;

        // <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        // <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        const XMLNS: [(&str, &str); 1] = [("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")];

        let tag = events::BytesStart::new(tag);
        let tag = tag.with_attributes(XMLNS);
        temp.write_event(events::Event::Start(tag))?;

        Ok(temp.into_inner())
    }

    pub fn create_close_tag(&mut self, tag: &str) -> Result<Vec<u8>, Error> {
        let mut temp = Writer::new(Vec::new());

        // </urlset>
        // </sitemapindex>
        let tag = events::BytesEnd::new(tag);
        temp.write_event(events::Event::End(tag))?;

        Ok(temp.into_inner())
    }
}

impl<W, D> std::fmt::Debug for InnerBuilder<W, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XmlBuilder")
            .field("bytes", &self.writer.writer_bytes())
            .field("records", &self.records)
            .finish()
    }
}
