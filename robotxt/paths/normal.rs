use std::sync::OnceLock;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Returns the prefixed & percent-encoded path.
/// NOTE: Expects relative path.
pub(crate) fn normalize_path(path: &str) -> String {
    static FRAGMENT: OnceLock<AsciiSet> = OnceLock::new();
    let fragment = FRAGMENT.get_or_init(|| CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>'));
    let path = utf8_percent_encode(path, fragment).to_string();

    // Url::make_relative strips leading and trailing /
    // https://github.com/servo/rust-url/issues/772
    // https://github.com/servo/rust-url/issues/766
    if !path.starts_with('/') {
        '/'.to_string() + &path
    } else {
        path
    }
}
