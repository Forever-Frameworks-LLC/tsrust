//! Bundled UTF-8 path operations; no filesystem access or path normalization.
/// Last path component, with platform separators and optional case-sensitive suffix removal.
/// Explicit platform selection permits portable boundary testing of both separator conventions.
pub fn basename(path: &str, suffix: Option<&str>, windows: bool) -> std::string::String {
    let separator = |ch| ch == '/' || (windows && ch == '\\');
    let bytes = path.as_bytes();
    let drive = windows && bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let mut start = if drive { 2 } else { 0 };
    let suffix = suffix.filter(|suffix| {
        !suffix.is_empty() && suffix.encode_utf16().count() <= path.encode_utf16().count()
    });
    if suffix == Some(path) {
        return std::string::String::new();
    }
    let mut expected = suffix.map(|suffix| suffix.chars().rev());
    let mut next = expected.as_mut().and_then(Iterator::next);
    let mut end = None;
    let mut component_end = None;
    for (offset, ch) in path
        .char_indices()
        .rev()
        .take_while(|(offset, _)| *offset >= start)
    {
        if separator(ch) {
            if component_end.is_some() {
                start = offset + ch.len_utf8();
                break;
            }
            continue;
        }
        let first_end = *component_end.get_or_insert(offset + ch.len_utf8());
        if suffix.is_none() {
            end.get_or_insert(first_end);
            continue;
        }
        if let Some(wanted) = next {
            if wanted == ch {
                next = expected.as_mut().and_then(Iterator::next);
                if next.is_none() {
                    end = Some(offset);
                }
            } else {
                next = None;
                end = Some(first_end);
            }
        }
    }
    let end = if suffix.is_some() {
        if end == Some(start) {
            component_end.unwrap_or(path.len())
        } else {
            end.unwrap_or(path.len())
        }
    } else {
        end.unwrap_or(start)
    };
    path[start..end].to_owned()
}
