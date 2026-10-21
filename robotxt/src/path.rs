//! Normalization of URL paths and rule patterns.

use std::borrow::Cow;

/// Returns true for the bytes that are not percent-encoded: printable ASCII
/// except for the characters the [`url`] crate also encodes in paths.
const fn is_plain(byte: u8) -> bool {
    byte.is_ascii_graphic() && !matches!(byte, b'"' | b'<' | b'>' | b'`' | b'{' | b'}')
}

/// Returns true for the unreserved characters of RFC 3986.
const fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Decodes the two hexadecimal digits of a percent-encoded octet.
fn decode_hex(digits: Option<&[u8]>) -> Option<u8> {
    let &[high, low] = digits? else {
        return None;
    };

    let high = char::from(high).to_digit(16)?;
    let low = char::from(low).to_digit(16)?;
    u8::try_from(high * 16 + low).ok()
}

/// Appends the percent-encoded octet with uppercase hexadecimal digits.
fn push_encoded(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    output.push('%');
    output.push(char::from(HEX[usize::from(byte >> 4)]));
    output.push(char::from(HEX[usize::from(byte & 0xf)]));
}

/// Normalizes a path or a rule pattern for matching, as described in
/// RFC 9309, section 2.2.2.
///
/// The result always starts with `/`. Percent-encoded unreserved characters
/// are decoded, other percent-encoded octets get uppercase hexadecimal digits,
/// and non-ASCII octets, controls and spaces are percent-encoded. The `*` and
/// `$` characters are left as is.
///
/// Borrows the input if it is already normalized, as paths taken from a
/// [`Url`] usually are.
///
/// [`Url`]: url::Url
pub(crate) fn normalize(input: &[u8]) -> Cow<'_, str> {
    if input.starts_with(b"/")
        && is_normalized(input)
        && let Ok(input) = std::str::from_utf8(input)
    {
        return Cow::Borrowed(input);
    }

    Cow::Owned(normalize_owned(input))
}

/// Returns true if [`normalize`] would leave everything but the leading
/// `/` unchanged.
fn is_normalized(input: &[u8]) -> bool {
    let mut index = 0;
    while let Some(&byte) = input.get(index) {
        let digits = input.get(index + 1..index + 3);
        if byte == b'%'
            && let Some(decoded) = decode_hex(digits)
        {
            let is_lowercase =
                digits.is_some_and(|digits| digits.iter().any(u8::is_ascii_lowercase));
            if is_unreserved(decoded) || is_lowercase {
                return false;
            }

            index += 3;
            continue;
        }

        if !is_plain(byte) {
            return false;
        }

        index += 1;
    }

    true
}

fn normalize_owned(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len() + 1);
    if !input.starts_with(b"/") {
        output.push('/');
    }

    let mut index = 0;
    while let Some(&byte) = input.get(index) {
        if byte == b'%'
            && let Some(decoded) = decode_hex(input.get(index + 1..index + 3))
        {
            if is_unreserved(decoded) {
                output.push(char::from(decoded));
            } else {
                push_encoded(&mut output, decoded);
            }

            index += 3;
            continue;
        }

        if is_plain(byte) {
            output.push(char::from(byte));
        } else {
            push_encoded(&mut output, byte);
        }

        index += 1;
    }

    output
}

/// Normalizes a relative URL (path and query) for matching.
/// The fragment is discarded, as it is never sent to the server.
#[cfg(feature = "parser")]
pub(crate) fn normalize_path(path: &str) -> Cow<'_, str> {
    let path = path.split_once('#').map_or(path, |(path, _)| path);
    normalize(path.as_bytes())
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use super::normalize;

    #[test]
    fn leading_slash() {
        assert_eq!(normalize(b""), "/");
        assert_eq!(normalize(b"*.php"), "/*.php");
        assert_eq!(normalize(b"/a"), "/a");
    }

    #[test]
    fn percent_encoding() {
        assert_eq!(normalize(b"/%61%7e"), "/a~");
        assert_eq!(normalize(b"/%2f%3F"), "/%2F%3F");
        assert_eq!(normalize("/ä b".as_bytes()), "/%C3%A4%20b");
        assert_eq!(normalize(b"/\xff"), "/%FF");
        assert_eq!(normalize(b"/100%"), "/100%");
        assert_eq!(normalize(b"/%zz"), "/%zz");
    }

    #[test]
    fn borrowed() {
        for input in ["/a/b?c=%2F", "/*.php$", "/100%"] {
            assert!(
                matches!(normalize(input.as_bytes()), Cow::Borrowed(_)),
                "{input}"
            );
        }

        for input in ["a", "/%61", "/%2f", "/a b"] {
            assert!(
                matches!(normalize(input.as_bytes()), Cow::Owned(_)),
                "{input}"
            );
        }
    }

    #[test]
    fn idempotent() {
        for input in ["a b", "/%61%2f", "/ä", "/100%", "*$"] {
            let once = normalize(input.as_bytes()).into_owned();
            assert_eq!(normalize(once.as_bytes()), once);
        }
    }

    #[test]
    fn wildcards() {
        assert_eq!(normalize(b"/*.php$"), "/*.php$");
    }

    #[cfg(feature = "parser")]
    #[test]
    fn fragment() {
        assert_eq!(super::normalize_path("/a?b#c"), "/a?b");
    }
}
