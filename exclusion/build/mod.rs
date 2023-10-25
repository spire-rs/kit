mod builder;
mod group;

pub use builder::*;
pub use group::*;

/// Splits multiline comments into lines and prefixes them with `#`.
pub(crate) fn format_comment(txt: &str) -> String {
    txt.lines()
        .map(|txt| txt.trim())
        .filter(|txt| !txt.is_empty())
        .map(|txt| {
            if txt.starts_with('#') {
                txt.to_owned()
            } else {
                format!("# {txt}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
