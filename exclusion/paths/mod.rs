pub use create::create_url;
pub(crate) use normal::normalize_path;

mod create;
mod normal;

/// Google currently enforces a `robots.txt` file size limit of 500 kibibytes (KiB).
/// See [How Google interprets Robots.txt](https://developers.google.com/search/docs/crawling-indexing/robots/robots_txt).
pub const BYTE_LIMIT: usize = 512_000;
