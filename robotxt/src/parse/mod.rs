//! Parsing and matching of `robots.txt` files.

pub use access::AccessResult;
pub use agent::Agent;
pub use robots::{BYTE_LIMIT, Robots};
pub use rule::Rule;

mod access;
mod agent;
mod lexer;
mod parser;
mod robots;
mod rule;
