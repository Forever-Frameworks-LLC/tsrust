//! File locations from terminal output; parsing never opens URLs or touches disk.
#[derive(Debug, PartialEq)]
pub(crate) struct FileLink {pub path: String, pub line: usize, pub column: usize}

pub(crate) fn file_link(text: &str, hovered: usize) -> Option<FileLink> {
    if text.len() > 8192 || hovered >= text.len() {return None;}
    for (colon, _) in text.match_indices(':') {
        let digits = text[colon + 1..].bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {continue;}
        let line = text[colon + 1..colon + 1 + digits].parse::<usize>().ok().filter(|line| *line > 0)?;
        let mut end = colon + 1 + digits;
        let mut column = 1;
        if text.as_bytes().get(end) == Some(&b':') {
            let count = text[end + 1..].bytes().take_while(u8::is_ascii_digit).count();
            if count > 0 {column = text[end + 1..end + 1 + count].parse().ok().filter(|column| *column > 0)?; end += count + 1;}
        }
        if text[end..].chars().next().is_some_and(|c| !c.is_whitespace() && !matches!(c, ':' | ')' | ']' | ',' | '\'' | '"')) {continue;}
        let prefix = &text[..colon];
        let candidate = prefix.trim_start().strip_prefix("-->").unwrap_or(prefix.trim_start()).trim_start();
        let candidate = if candidate.starts_with(['\'', '"']) {&candidate[1..]} else {candidate};
        let candidate = candidate.trim_end_matches(['\'', '"']);
        let path = if candidate.starts_with('/') || candidate.starts_with("./") || candidate.starts_with("../") || candidate.as_bytes().get(1) == Some(&b':') {candidate}
            else {candidate.rsplit_once(char::is_whitespace).map_or(candidate, |(_, path)| path)};
        let start = colon - path.len() - usize::from(prefix.ends_with(['\'', '"']));
        if hovered < start || hovered >= end || path.is_empty() || path.contains("://") || !path.contains(['/', '\\', '.']) {continue;}
        return Some(FileLink {path: path.into(), line, column});
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_file_locations() {
        for (text, path, line, column) in [
            ("  --> /tmp/my project/日本.tsr:11:10 [error]", "/tmp/my project/日本.tsr", 11, 10),
            ("src/main.tsr:2:3 error", "src/main.tsr", 2, 3),
            ("'./my project/file.tsr':4", "./my project/file.tsr", 4, 1),
            ("C:\\repo\\main.tsr:9:2", "C:\\repo\\main.tsr", 9, 2),
        ] {
            let hover = text.find(".tsr").unwrap();
            assert_eq!(file_link(text, hover), Some(FileLink {path: path.into(), line, column}));
            assert_eq!(file_link(text, text.len()), None);
        }
        for text in ["https://example.com/file.tsr:3:4", "src/main.tsr:0:2", "nothing here", "file.tsr:2oops"] {assert_eq!(file_link(text, 2), None);}
        assert_eq!(file_link(" --> src/main.tsr:4:2 error", 0), None);
    }
}
