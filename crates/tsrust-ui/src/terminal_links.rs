//! File locations from terminal output; parsing never opens URLs or touches disk.
#[derive(Debug, PartialEq)]
pub(crate) struct FileLink {pub path: String, pub line: usize, pub column: usize}

pub(crate) fn file_link(text: &str, hovered: usize) -> Option<FileLink> {
    if text.len() > 8192 || hovered >= text.len() {return None;}
    let mut previous_end = 0;
    for (colon, _) in text.match_indices(':') {
        if colon < previous_end {continue;}
        let digits = text[colon + 1..].bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {continue;}
        let line = text[colon + 1..colon + 1 + digits].parse::<usize>().ok();
        let mut end = colon + 1 + digits;
        let mut column = Some(1);
        if text.as_bytes().get(end) == Some(&b':') {
            let count = text[end + 1..].bytes().take_while(u8::is_ascii_digit).count();
            if count > 0 {column = text[end + 1..end + 1 + count].parse().ok(); end += count + 1;}
        }
        if text[end..].chars().next().is_some_and(|c| !c.is_whitespace() && !matches!(c, ':' | ')' | ']' | ',' | '\'' | '"')) {continue;}
        let prefix = text[previous_end..colon].trim_start();
        previous_end = end;
        let (Some(line), Some(column)) = (line, column) else {continue;};
        if line == 0 || column == 0 {continue;}
        let diagnostic = prefix.strip_prefix("-->");
        let candidate = diagnostic.unwrap_or(prefix).trim_start();
        let quoted = candidate.chars().last().filter(|c| matches!(c, '\'' | '"')).and_then(|quote| {
            let body = candidate.strip_suffix(quote)?;
            body.strip_prefix(quote).or_else(|| body.rsplit_once(quote).map(|(_, path)| path))
        });
        let path = if let Some(path) = quoted {path}
            else if diagnostic.is_some() || candidate.starts_with('/') || candidate.starts_with("./") || candidate.starts_with("../") || candidate.as_bytes().get(1) == Some(&b':') {candidate}
            else {candidate.rsplit_once(char::is_whitespace).map_or(candidate, |(_, path)| path)};
        let start = colon - path.len() - usize::from(quoted.is_some());
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
            ("'./Sean's files/main.tsr':4", "./Sean's files/main.tsr", 4, 1),
            ("C:\\repo\\main.tsr:9:2", "C:\\repo\\main.tsr", 9, 2),
        ] {
            let hover = text.find(".tsr").unwrap();
            assert_eq!(file_link(text, hover), Some(FileLink {path: path.into(), line, column}));
            assert_eq!(file_link(text, text.len()), None);
        }
        for text in [
            " --> src/my project/日本.tsr:8:4",
            "error in 'src/my project/日本.tsr':8:4",
            "bad.tsr:0:9 'src/my project/日本.tsr':8:4",
            "/tmp/old.tsr:7:2 'src/my project/日本.tsr':8:4",
        ] {
            for hover in [text.find("src/").unwrap(), text.find("日本").unwrap(), text.len() - 1] {
                assert_eq!(file_link(text, hover), Some(FileLink {path: "src/my project/日本.tsr".into(), line: 8, column: 4}));
            }
        }
        for text in ["https://example.com/file.tsr:3:4", "src/main.tsr:0:2", "nothing here", "file.tsr:2oops"] {assert_eq!(file_link(text, 2), None);}
        assert_eq!(file_link(" --> src/main.tsr:4:2 error", 0), None);
    }
}
