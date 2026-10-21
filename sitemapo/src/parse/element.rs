//! Decodes XML events into the element trees of the sitemap records.

use quick_xml::XmlVersion;
use quick_xml::escape::unescape;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};

use crate::parse::{Format, convert};
use crate::record::Record;
use crate::record::names::{
    IMAGE_NS, NEWS_NS, SITEMAP, SITEMAP_INDEX, URL, URL_SET, VIDEO_NS, XHTML_NS,
};
use crate::{Error, Result};

/// The number of nested elements kept below each record element. Deeper
/// elements are skipped, so that a deeply nested document is never stored.
const MAX_DEPTH: usize = 8;

/// The namespace of an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ns {
    Sitemap,
    Image,
    Video,
    News,
    Xhtml,
    Other,
}

impl Ns {
    /// Resolves the namespace of an element below the root.
    ///
    /// The sitemap namespace is the one of the root element, whatever it is,
    /// as sitemaps often declare an outdated one, and elements without a
    /// namespace belong to it too.
    fn resolve(result: &ResolveResult<'_>, root: Option<&str>) -> Self {
        match result {
            ResolveResult::Bound(Namespace(namespace)) => match *namespace {
                IMAGE_NS => Self::Image,
                VIDEO_NS => Self::Video,
                NEWS_NS => Self::News,
                XHTML_NS => Self::Xhtml,
                namespace if Some(namespace) == root => Self::Sitemap,
                _ => Self::Other,
            },
            // An undeclared prefix is matched by its conventional name.
            ResolveResult::Unknown(prefix) => match prefix.as_str() {
                "image" => Self::Image,
                "video" => Self::Video,
                "news" => Self::News,
                "xhtml" => Self::Xhtml,
                _ => Self::Other,
            },
            ResolveResult::Unbound => Self::Sitemap,
        }
    }
}

/// An element of a record, with its text and child elements.
#[derive(Debug)]
pub struct Element {
    ns: Ns,
    name: String,
    attributes: Vec<(String, String)>,
    text: String,
    children: Vec<Element>,
}

impl Element {
    fn new(ns: Ns, start: &BytesStart<'_>) -> Self {
        let attributes = start.attributes().flatten().filter_map(|attribute| {
            let name = attribute.key.local_name().as_ref().to_owned();
            let value = attribute.normalized_value(XmlVersion::Implicit1_0).ok()?;
            let value = value.into_owned();
            Some((name, value))
        });

        Self {
            ns,
            name: start.local_name().as_ref().to_owned(),
            attributes: attributes.collect(),
            text: String::new(),
            children: Vec::new(),
        }
    }

    /// Returns true if the element has the namespace and the local name.
    pub fn is(&self, ns: Ns, name: &str) -> bool {
        self.ns == ns && self.name == name
    }

    /// Returns the trimmed text of the element.
    pub fn text(&self) -> &str {
        self.text.trim()
    }

    /// Returns the value of the attribute with the local name.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        let mut attributes = self.attributes.iter();
        attributes
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.trim())
    }

    /// Returns the child elements with the namespace and the local name.
    pub fn children(&self, ns: Ns, name: &str) -> impl Iterator<Item = &Self> {
        self.children.iter().filter(move |child| child.is(ns, name))
    }

    /// Returns the first child element with the namespace and the local name.
    pub fn child(&self, ns: Ns, name: &str) -> Option<&Self> {
        self.children(ns, name).next()
    }

    /// Returns the non-empty text of the first child element with the
    /// namespace and the local name.
    pub fn child_text(&self, ns: Ns, name: &str) -> Option<&str> {
        self.child(ns, name)
            .map(Self::text)
            .filter(|text| !text.is_empty())
    }
}

/// Collects the elements of each record and converts them once complete.
#[derive(Debug, Default)]
pub struct Decoder {
    format: Option<Format>,
    namespace: Option<String>,
    depth: usize,
    stack: Vec<Element>,
}

impl Decoder {
    /// Returns the format declared by the root element, once it is read.
    pub const fn format(&self) -> Option<Format> {
        self.format
    }

    /// Processes the next event, returning the record it completes, if any.
    pub fn decode(&mut self, ns: &ResolveResult<'_>, event: Event<'_>) -> Result<Option<Record>> {
        match event {
            Event::Start(start) => self.open(ns, &start).map(|()| None),
            Event::Empty(start) => self.open(ns, &start).map(|()| self.close()),
            Event::End(_) => Ok(self.close()),
            Event::Text(text) => {
                self.push_text(&text.xml10_content());
                Ok(None)
            }
            Event::CData(data) => {
                self.push_text(&data.xml10_content());
                Ok(None)
            }
            Event::GeneralRef(reference) => {
                self.push_reference(&reference);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Returns true if the elements at the depth are stored: the record
    /// elements, right below the root, and their descendants.
    const fn is_stored(depth: usize) -> bool {
        depth >= 2 && depth < 2 + MAX_DEPTH
    }

    fn open(&mut self, ns: &ResolveResult<'_>, start: &BytesStart<'_>) -> Result<()> {
        self.depth += 1;
        if self.depth == 1 {
            if let ResolveResult::Bound(Namespace(namespace)) = ns {
                self.namespace = Some((*namespace).to_owned());
            }

            let name = start.local_name();
            self.format = Some(match name.as_ref() {
                URL_SET => Format::UrlSet,
                SITEMAP_INDEX => Format::Index,
                name => {
                    let name = name.to_owned();
                    return Err(Error::UnknownRoot { name });
                }
            });
        } else if Self::is_stored(self.depth) {
            let ns = Ns::resolve(ns, self.namespace.as_deref());
            self.stack.push(Element::new(ns, start));
        }

        Ok(())
    }

    fn close(&mut self) -> Option<Record> {
        let depth = self.depth;
        self.depth = depth.saturating_sub(1);
        if !Self::is_stored(depth) {
            return None;
        }

        let element = self.stack.pop()?;
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(element);
            return None;
        }

        match self.format? {
            Format::UrlSet if element.is(Ns::Sitemap, URL) => {
                convert::entry(&element).map(Record::Entry)
            }
            Format::Index if element.is(Ns::Sitemap, SITEMAP) => {
                convert::sitemap(&element).map(Record::Sitemap)
            }
            _ => None,
        }
    }

    /// Appends the text to the innermost element, if it is stored.
    fn push_text(&mut self, text: &str) {
        if self.depth == self.stack.len() + 1
            && let Some(element) = self.stack.last_mut()
        {
            element.text.push_str(text);
        }
    }

    /// Appends the character or entity reference, or the reference itself if
    /// it is unknown.
    fn push_reference(&mut self, reference: &BytesRef<'_>) {
        if let Ok(Some(character)) = reference.resolve_char_ref() {
            self.push_text(character.encode_utf8(&mut [0; 4]));
            return;
        }

        let reference = format!("&{};", &**reference);
        match unescape(&reference) {
            Ok(text) => self.push_text(&text),
            Err(_) => self.push_text(&reference),
        }
    }
}
