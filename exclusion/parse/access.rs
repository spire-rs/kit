/// The `AccessResult` enum represents the result of the
/// `robots.txt` retrieval attempt.
///
/// See [`crate::Robots::from_access`].
/// Also see 2.3.1. Access Results in the specification.
#[derive(Debug)]
pub enum AccessResult<'a> {
    /// 2.3.1.1.  Successful Access
    ///
    /// If the crawler successfully downloads the robots.txt file, the
    /// crawler MUST follow the parseable rules.
    Successful(&'a [u8]),
    /// 2.3.1.2.  Redirects
    ///
    /// It's possible that a server responds to a robots.txt fetch request
    /// with a redirect, such as HTTP 301 or HTTP 302 in the case of HTTP.
    /// The crawlers SHOULD follow at least five consecutive redirects, even
    /// across authorities (for example, hosts in the case of HTTP).
    ///
    /// If a robots.txt file is reached within five consecutive redirects,
    /// the robots.txt file MUST be fetched, parsed, and its rules followed
    /// in the context of the initial authority.
    ///
    /// If there are more than five consecutive redirects, crawlers MAY
    /// assume that the robots.txt file is unavailable.
    Redirect,
    /// 2.3.1.3.  "Unavailable" Status
    ///
    /// "Unavailable" means the crawler tries to fetch the robots.txt file
    /// and the server responds with status codes indicating that the
    /// resource in question is unavailable.  For example, in the context of
    /// HTTP, such status codes are in the 400-499 range.
    ///
    /// If a server status code indicates that the robots.txt file is
    /// unavailable to the crawler, then the crawler MAY access any resources
    /// on the server.
    Unavailable,
    /// 2.3.1.4.  "Unreachable" Status
    ///
    /// If the robots.txt file is unreachable due to server or network
    /// errors, this means the robots.txt file is undefined and the crawler
    /// MUST assume complete disallow.  For example, in the context of HTTP,
    /// server errors are identified by status codes in the 500-599 range.
    ///
    /// If the robots.txt file is undefined for a reasonably long period of
    /// time (for example, 30 days), crawlers MAY assume that the robots.txt
    /// file is unavailable as defined in Section 2.3.1.3 or continue to use
    /// cached copy.
    Unreachable,
}

impl AccessResult<'_> {
    /// Returns the textual representation of a status.
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessResult::Successful(_) => "Successful",
            AccessResult::Redirect => "Redirect",
            AccessResult::Unavailable => "Unavailable",
            AccessResult::Unreachable => "Unreachable",
        }
    }
}

impl std::fmt::Display for AccessResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
