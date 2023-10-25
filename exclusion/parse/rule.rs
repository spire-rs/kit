use std::cmp::Ordering;
use std::sync::OnceLock;

use regex::{escape, Regex, RegexBuilder};

use crate::normalize_path;

/// An error type indicating that a `Wildcard` could not be parsed correctly.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("too many ending wildcards: {0}")]
    TooManyEndings(usize),
    #[error("unexpected ending wildcard position: {0}")]
    EndingPosition(usize),
    #[error("regex instantiation error: {0}")]
    Regex(#[from] regex::Error),
}

/// The `Wildcard` struct provides efficient pattern matching for wildcards.
#[derive(Debug, Clone)]
pub enum Wildcard {
    Ending(String),
    Universal(String),
    Both(Regex),
}

impl Wildcard {
    /// Creates a new [`Wildcard`] with the specified pattern or returns
    /// `None` if the specified pattern does not contain any wildcard.
    /// NOTE: Expects normalized relative path.
    pub fn new(pattern: &str) -> Result<Option<Self>, Error> {
        let contains_universal = pattern.contains('*');
        let endings_amount = pattern.chars().filter(|&c| c == '$').count();
        let contains_ending = endings_amount > 0;

        // None.
        if !contains_ending && !contains_universal {
            return Ok(None);
        }

        // Only '$'.
        match endings_amount {
            x if x > 1 => return Err(Error::TooManyEndings(x)),
            x if x == 1 && pattern.ends_with('$') && !contains_universal => {
                let pattern = pattern.strip_suffix('$').expect("should end with '$'");
                return Ok(Some(Self::Ending(pattern.to_string())));
            }
            x if x == 1 && !pattern.ends_with('$') => {
                let pos = pattern.find('$').expect("should contain '$'");
                return Err(Error::EndingPosition(pos));
            }
            _ => {} // x if x == 0 || contains_universal
        }

        static STAR_KILLER: OnceLock<Regex> = OnceLock::new();
        let star_killer = STAR_KILLER.get_or_init(|| Regex::new(r"\*+").expect("should compile"));
        let pattern = star_killer.replace_all(pattern, "*");

        // Only '*'.
        if contains_universal && !contains_ending {
            return Ok(Some(Self::Universal(pattern.to_string())));
        }

        // Both '$' and '*'.
        let regex = escape(&pattern).replace("\\*", ".*").replace("\\$", "$");
        let regex = '^'.to_string() + &regex;

        let regex = RegexBuilder::new(&regex)
            .dfa_size_limit(42 * (1 << 10))
            .size_limit(42 * (1 << 10))
            .build()?;

        Ok(Some(Self::Both(regex)))
    }

    /// Returns true if the path matches the ending pattern.
    fn match_ending(pattern: &str, path: &str) -> bool {
        path == pattern
    }

    /// Returns true if the path matches the universal pattern.
    fn match_universal(pattern: &str, path: &str) -> bool {
        let mut splits = pattern.split('*');
        let mut pos = 0;

        // The first split is special as it doesn't start with '*'.
        // i.e. pattern '/a*c' : path '/abc' should match '/a'.
        if let Some(first) = splits.next() {
            pos += first.len();
            if !path.starts_with(first) {
                return false;
            }
        }

        for split in splits {
            match path[pos..].find(split) {
                Some(idx) => pos += idx + split.len(),
                None => return false,
            }
        }

        true
    }

    /// Returns true if the path matches the wildcard pattern.
    pub fn is_match(&self, path: &str) -> bool {
        match &self {
            Self::Ending(p) => Self::match_ending(p.as_str(), path),
            Self::Universal(p) => Self::match_universal(p.as_str(), path),
            Self::Both(r) => r.is_match(path),
        }
    }
}

#[cfg(test)]
mod wildcard {
    use super::{Error, Wildcard};

    #[test]
    fn none() -> Result<(), Error> {
        let wildcard = Wildcard::new("/")?;
        assert!(wildcard.is_none());
        Ok(())
    }

    #[test]
    fn ending() -> Result<(), Error> {
        let wildcard = Wildcard::new("/$")?.unwrap();
        assert!(matches!(wildcard, Wildcard::Ending(u) if u == "/"));
        Ok(())
    }

    #[test]
    fn universal() -> Result<(), Error> {
        let wildcard = Wildcard::new("/*")?.unwrap();
        assert!(matches!(wildcard, Wildcard::Universal(u) if u == "/*"));
        Ok(())
    }

    #[test]
    fn both() -> Result<(), Error> {
        let wildcard = Wildcard::new("/*$")?.unwrap();
        assert!(matches!(wildcard, Wildcard::Both(u) if u.as_str() == "^/.*$"));
        Ok(())
    }
}

/// The `Rule` struct provides a convenient and efficient way to process
/// and to match `robots.txt` provided patterns with relative paths.
#[derive(Debug, Clone)]
pub struct Rule {
    pattern: String,
    allow: bool,
    wildcard: Option<Wildcard>,
}

impl Rule {
    /// Creates a new `Rule` with the specified pattern and permission.
    pub fn new(pattern: &str, allow: bool) -> Result<Self, Error> {
        let pattern = normalize_path(pattern);
        let wildcard = Wildcard::new(pattern.as_str())?;

        Ok(Self {
            pattern,
            allow,
            wildcard,
        })
    }

    #[cfg(feature = "serde")]
    /// Extracts a string slice containing the entire pattern.
    pub fn pattern(&self) -> &str {
        self.pattern.as_str()
    }

    /// Returns true if the path matches the pattern.
    /// NOTE: Expects normalized relative path.
    pub fn is_match(&self, path: &str) -> bool {
        match &self.wildcard {
            None => path.starts_with(self.pattern.as_str()),
            Some(wildcard) => wildcard.is_match(path),
        }
    }

    /// Returns true if allowed.
    pub fn is_allowed(&self) -> bool {
        self.allow
    }

    /// Returns true if matches everything.
    #[cfg(feature = "optimal")]
    pub fn is_universal(&self) -> bool {
        match &self.wildcard {
            None => self.pattern == "/",
            Some(Wildcard::Ending(_)) => false,
            Some(Wildcard::Universal(p)) => p == "/*",
            Some(Wildcard::Both(r)) => r.as_str() == "^/.*$",
        }
    }
}

impl PartialEq<Self> for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl Eq for Rule {}

impl PartialOrd<Self> for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> Ordering {
        let length = other.pattern.len().cmp(&self.pattern.len());
        length.then_with(|| other.allow.cmp(&self.allow))
    }
}

#[cfg(test)]
mod matching {
    use super::{Error, Rule};

    #[test]
    fn root_none() -> Result<(), Error> {
        let r = Rule::new("/", true)?;

        // Matches:
        assert!(r.is_match("/fish"));

        Ok(())
    }

    #[test]
    fn root_universal() -> Result<(), Error> {
        let r = Rule::new("/*", true)?;

        // Matches:
        assert!(r.is_match("/fish"));
        assert!(r.is_match("//"));

        Ok(())
    }

    #[test]
    fn root_ending() -> Result<(), Error> {
        let r = Rule::new("/$", true)?;

        // Matches:
        assert!(r.is_match("/"));

        // Doesn't match:
        assert!(!r.is_match("/fish"));
        assert!(!r.is_match("//"));
        assert!(!r.is_match("/$"));

        Ok(())
    }

    #[test]
    fn simple() -> Result<(), Error> {
        let r = Rule::new("/fish", true)?;

        // Matches:
        assert!(r.is_match("/fish"));
        assert!(r.is_match("/fish.html"));
        assert!(r.is_match("/fish/salmon.html"));
        assert!(r.is_match("/fishheads"));
        assert!(r.is_match("/fishheads/yummy.html"));
        assert!(r.is_match("/fish.php?id=anything"));

        // Doesn't match:
        assert!(!r.is_match("/Fish.asp"));
        assert!(!r.is_match("/catfish"));
        assert!(!r.is_match("/?id=fish"));
        assert!(!r.is_match("/desert/fish"));

        Ok(())
    }

    #[test]
    fn folder() -> Result<(), Error> {
        let r = Rule::new("/fish/", true)?;

        // Matches:
        assert!(r.is_match("/fish/"));
        assert!(r.is_match("/fish/?id=anything"));
        assert!(r.is_match("/fish/salmon.htm"));

        // Doesn't match:
        assert!(!r.is_match("/fish"));
        assert!(!r.is_match("/fish.html"));
        assert!(!r.is_match("/animals/fish/"));
        assert!(!r.is_match("/Fish/Salmon.asp"));

        Ok(())
    }

    #[test]
    fn universal_end() -> Result<(), Error> {
        let r = Rule::new("/fish*", true)?;

        // Matches:
        assert!(r.is_match("/fish"));
        assert!(r.is_match("/fish.html"));
        assert!(r.is_match("/fish/salmon.html"));
        assert!(r.is_match("/fishheads"));
        assert!(r.is_match("/fishheads/yummy.html"));
        assert!(r.is_match("/fish.php?id=anything"));

        // Doesn't match:
        assert!(!r.is_match("/Fish.asp"));
        assert!(!r.is_match("/catfish"));
        assert!(!r.is_match("/?id=fish"));
        assert!(!r.is_match("/desert/fish"));

        Ok(())
    }

    #[test]
    fn universal_mid() -> Result<(), Error> {
        let r = Rule::new("/*.php", true)?;

        // Matches:
        assert!(r.is_match("/index.php"));
        assert!(r.is_match("/filename.php"));
        assert!(r.is_match("/folder/filename.php"));
        assert!(r.is_match("/folder/filename.php?parameters"));
        assert!(r.is_match("/folder/any.php.file.html"));
        assert!(r.is_match("/filename.php/"));

        // Doesn't match:
        assert!(!r.is_match("/"));
        assert!(!r.is_match("/windows.PHP"));

        Ok(())
    }

    #[test]
    fn universal_mid2() -> Result<(), Error> {
        let r = Rule::new("/fish*.php", true)?;

        // Matches:
        assert!(r.is_match("/fish.php"));
        assert!(r.is_match("/fishheads/catfish.php?parameters"));

        // Doesn't match:
        assert!(!r.is_match("/Fish.PHP"));

        Ok(())
    }

    #[test]
    fn both_wildcards() -> Result<(), Error> {
        let r = Rule::new("/*.php$", true)?;

        // Matches:
        assert!(r.is_match("/filename.php"));
        assert!(r.is_match("/folder/filename.php"));

        // Doesn't match:
        assert!(!r.is_match("/filename.php?parameters"));
        assert!(!r.is_match("/filename.php/"));
        assert!(!r.is_match("/filename.php5"));
        assert!(!r.is_match("/windows.PHP"));

        Ok(())
    }
}
