//! A convenience module that re-exports commonly used items.
//!
//! This module is intended to be glob-imported for convenience:
//!
//! ```ignore
//! use robotxt::prelude::*;
//! ```

#[cfg(feature = "builder")]
#[cfg_attr(docsrs, doc(cfg(feature = "builder")))]
pub use crate::build::{GroupBuilder, RobotsBuilder};
pub use crate::location::robots_url;
#[cfg(feature = "parser")]
#[cfg_attr(docsrs, doc(cfg(feature = "parser")))]
pub use crate::parse::{AccessResult, Agent, BYTE_LIMIT, Robots, Rule};
pub use crate::{ANY_AGENT, Error, Result};
