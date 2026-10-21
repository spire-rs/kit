//! A convenience module that re-exports commonly used items.
//!
//! This module is intended to be glob-imported for convenience:
//!
//! ```ignore
//! use sitemapo::prelude::*;
//! ```

pub use crate::build::{Builder, EntryBuilder, IndexBuilder, TextBuilder};
pub use crate::parse::{Format, Parser};
pub use crate::record::{BYTE_LIMIT, Entry, RECORD_LIMIT, Record, Sitemap};
pub use crate::{Error, Result};
