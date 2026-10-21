//! A reader that stops at the sitemap byte limit.

use std::io::{self, BufRead, Read};

use crate::record::BYTE_LIMIT;

/// Wraps a buffered reader to end it after [`BYTE_LIMIT`] bytes, and
/// remembers whether more bytes were left, so a single oversized line or
/// text node never has to be buffered.
#[derive(Debug)]
pub struct Limited<R> {
    inner: R,
    remaining: usize,
    exceeded: bool,
}

impl<R> Limited<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            remaining: BYTE_LIMIT,
            exceeded: false,
        }
    }

    /// Returns true if the reader was ended with bytes left.
    pub const fn is_exceeded(&self) -> bool {
        self.exceeded
    }

    fn advance(&mut self, amount: usize) {
        self.remaining -= amount.min(self.remaining);
    }
}

impl<R: BufRead> Read for Limited<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let available = self.fill_buf()?;
        let amount = available.len().min(buf.len());
        buf[..amount].copy_from_slice(&available[..amount]);
        self.consume(amount);
        Ok(amount)
    }
}

impl<R: BufRead> BufRead for Limited<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let available = self.inner.fill_buf()?;
        if self.remaining == 0 && !available.is_empty() {
            self.exceeded = true;
        }

        Ok(&available[..available.len().min(self.remaining)])
    }

    fn consume(&mut self, amount: usize) {
        self.advance(amount);
        self.inner.consume(amount);
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufRead, Read};

    use super::Limited;
    use crate::record::BYTE_LIMIT;

    #[test]
    fn within_limit() {
        let mut reader = Limited::new(b"abc".as_slice());
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, b"abc");
        assert!(!reader.is_exceeded());
    }

    #[test]
    fn exact_limit() {
        let txt = vec![b'a'; BYTE_LIMIT];
        let mut reader = Limited::new(txt.as_slice());
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap();
        assert_eq!(buf.len(), BYTE_LIMIT);
        assert!(!reader.is_exceeded());
    }

    #[test]
    fn over_limit() {
        let txt = vec![b'a'; BYTE_LIMIT + 1];
        let mut reader = Limited::new(txt.as_slice());
        let mut buf = Vec::new();
        reader.read_until(b'\n', &mut buf).unwrap();
        assert_eq!(buf.len(), BYTE_LIMIT);
        assert!(reader.is_exceeded());
    }
}
