/// Splits multiline comments into lines and prefixes them with `#`.
pub fn format_comment(txt: &str) -> String {
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
