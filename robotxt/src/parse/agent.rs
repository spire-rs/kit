//! The crawler's name and how it is matched against `User-Agent` values.

use crate::ANY_AGENT;
use crate::parse::lexer::Directive;

/// The crawler's name and how it is matched against `User-Agent` values.
///
/// A plain `&str` converts into [`Agent::Exact`]. Every strategy ignores
/// case and falls back to the [`ANY_AGENT`] group when no other group matches.
///
/// ```rust
/// use robotxt::{Agent, Robots};
///
/// let txt = b"
///     User-Agent: foobot
///     Disallow: /
/// ";
///
/// let r = Robots::parse(txt, "foobot-images");
/// assert_eq!(r.user_agent(), "*");
///
/// let r = Robots::parse(txt, Agent::Prefix("foobot-images"));
/// assert_eq!(r.user_agent(), "foobot");
/// ```
///
/// [`ANY_AGENT`]: crate::ANY_AGENT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Agent<'a> {
    /// The `User-Agent` value must equal the crawler's name, as RFC 9309
    /// requires.
    Exact(&'a str),

    /// The longest `User-Agent` value the crawler's name starts with, e.g.
    /// a `foobot` group applies to the `foobot-images` crawler.
    Prefix(&'a str),
}

impl<'a> Agent<'a> {
    /// Returns the crawler's name.
    #[must_use]
    pub const fn name(&self) -> &'a str {
        match self {
            Self::Exact(name) | Self::Prefix(name) => name,
        }
    }

    /// Returns the `User-Agent` value of the groups that apply, or
    /// [`ANY_AGENT`] if no other group matches.
    pub(crate) fn select<'d>(self, directives: &[Directive<'d>]) -> &'d [u8] {
        let name = self.name().trim().as_bytes();
        let mut values = directives
            .iter()
            .filter_map(|directive| match *directive {
                Directive::UserAgent(value) => Some(value),
                _ => None,
            })
            .filter(|value| !value.is_empty() && *value != ANY_AGENT.as_bytes());

        let selected = match self {
            Self::Exact(_) => values.find(|value| value.eq_ignore_ascii_case(name)),
            Self::Prefix(_) => values
                .filter(|value| {
                    name.get(..value.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(value))
                })
                .max_by_key(|value| value.len()),
        };

        selected.unwrap_or(ANY_AGENT.as_bytes())
    }
}

impl<'a> From<&'a str> for Agent<'a> {
    fn from(name: &'a str) -> Self {
        Self::Exact(name)
    }
}

impl<'a> From<&'a String> for Agent<'a> {
    fn from(name: &'a String) -> Self {
        Self::Exact(name)
    }
}
