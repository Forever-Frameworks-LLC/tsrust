// Completion edit planning shared with the native editor kernel.
use std::{collections::BTreeMap, ops::Range};

#[derive(Debug)]
pub(crate) struct Snippet {
    pub text: String,
    pub stops: Vec<(u32, Range<usize>)>,
    pub active: u32,
}

impl Snippet {
    // shortcut: numeric stops/defaults only; add variables/transforms with external server support.
    pub fn parse(source: &str, offset: usize) -> Option<Self> {
        let mut text = String::new();
        let mut stops = Vec::new();
        let mut defaults = BTreeMap::new();
        let mut chars = source.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                let next = chars.next()?;
                if !matches!(next, '$' | '}' | '\\') { text.push(ch); }
                text.push(next);
                continue;
            }
            if ch != '$' { text.push(ch); continue; }
            let braced = chars.peek() == Some(&'{');
            if braced { chars.next(); }
            let mut number = String::new();
            while chars.peek().is_some_and(char::is_ascii_digit) { number.push(chars.next()?); }
            let number: u32 = number.parse().ok()?;
            let mut default = String::new();
            if braced {
                if chars.peek() == Some(&':') {
                    chars.next();
                    loop {
                        match chars.next()? {
                            '}' => break,
                            '\\' => default.push(chars.next()?),
                            '$' | '{' => return None,
                            ch => default.push(ch),
                        }
                    }
                    defaults.insert(number, default.clone());
                } else if chars.next()? != '}' { return None; }
            }
            if default.is_empty() { default = defaults.get(&number).cloned().unwrap_or_default(); }
            let start = offset + text.len();
            text.push_str(&default);
            stops.push((number, start..offset + text.len()));
        }
        if !stops.iter().any(|(number, _)| *number == 0) {
            stops.push((0, offset + text.len()..offset + text.len()));
        }
        let active = stops.iter().map(|(number, _)| *number).filter(|number| *number != 0).min().unwrap_or(0);
        Some(Self { text, stops, active })
    }

    pub fn contains(&self, selection: &Range<usize>) -> bool {
        self.stops.iter().any(|(number, range)| *number == self.active && range.start <= selection.start && selection.end <= range.end)
    }

    pub fn adjust(&mut self, edit: &Range<usize>, length: usize) -> bool {
        if !self.contains(edit) { return false; }
        let shift = |value: usize| value - (edit.end - edit.start) + length;
        for (number, range) in &mut self.stops {
            if *number == self.active && range.start <= edit.start && edit.end <= range.end {
                range.end = shift(range.end);
            } else if range.start >= edit.end {
                range.start = shift(range.start);
                range.end = shift(range.end);
            } else if range.end > edit.start { return false; }
        }
        true
    }

    pub fn advance(&mut self, backwards: bool) -> bool {
        let mut numbers = self.stops.iter().map(|(number, _)| *number).collect::<Vec<_>>();
        numbers.sort_unstable_by_key(|number| if *number == 0 { u64::MAX } else { u64::from(*number) });
        numbers.dedup();
        let Some(index) = numbers.iter().position(|number| *number == self.active) else { return false; };
        let next = if backwards { index.saturating_sub(1) } else { index + 1 };
        let Some(number) = numbers.get(next) else { return false; };
        self.active = *number;
        true
    }
}


use serde_json::Value;

pub(crate) struct CompletionPlan {
    pub edits: Vec<(Range<usize>, String)>,
    pub insertion: Range<usize>,
    pub snippet: Option<Snippet>,
}
impl CompletionPlan {
    pub fn text(&self, before: &str) -> String {
        let mut after = before.to_owned();
        for (range, text) in self.edits.iter().rev() { after.replace_range(range.clone(), text); }
        after
    }
}

// The bridge converts wire UTF-16 positions to scalar columns before this shared planner.
fn offset(text: &str, position: &Value) -> Result<usize, String> {
    let row = position.get("line").and_then(Value::as_u64).and_then(|n|usize::try_from(n).ok()).ok_or("Invalid completion line.")?;
    let column = position.get("character").and_then(Value::as_u64).and_then(|n|usize::try_from(n).ok()).ok_or("Invalid completion column.")?;
    let mut lines = text.split('\n');
    let mut start = 0;
    for _ in 0..row { start += lines.next().ok_or("Completion line is outside the document.")?.len() + 1; }
    let line = lines.next().ok_or("Completion line is outside the document.")?.trim_end_matches('\r');
    let byte = line.char_indices().map(|(byte,_)|byte).chain(std::iter::once(line.len())).nth(column).ok_or("Completion column is outside the line.")?;
    Ok(start + byte)
}
fn edit_range(text: &str, value: &Value) -> Result<Range<usize>,String> {
    let start = offset(text, &value["start"])?;
    let end = offset(text, &value["end"])?;
    if start > end { return Err("Completion range is reversed.".into()); }
    Ok(start..end)
}
pub(crate) fn plan(text: &str, item: &Value, fallback: Range<usize>) -> Result<CompletionPlan,String> {
    if fallback.start > fallback.end || !text.is_char_boundary(fallback.start) || !text.is_char_boundary(fallback.end) { return Err("Invalid completion fallback range.".into()); }
    let mut primary = fallback;
    let mut inserted = item.get("insertText").filter(|value|!value.is_null()).or_else(||item.get("label")).and_then(Value::as_str).ok_or("Completion text is missing.")?.to_owned();
    if let Some(edit) = item.get("textEdit").filter(|value|!value.is_null()) {
        inserted = edit.get("newText").and_then(Value::as_str).ok_or("Completion edit text is missing.")?.to_owned();
        primary = edit_range(text, edit.get("range").or_else(||edit.get("replace")).ok_or("Completion edit range is missing.")?)?;
        if let Some(insert) = edit.get("insert") {
            let insert = edit_range(text, insert)?;
            if insert.start != primary.start || insert.end > primary.end { return Err("Invalid completion insert/replace ranges.".into()); }
        }
    }
    let mut edits = Vec::new();
    if let Some(additional) = item.get("additionalTextEdits").filter(|value|!value.is_null()) {
        for edit in additional.as_array().ok_or("Invalid additional completion edits.")? {
            edits.push((edit_range(text, &edit["range"])?, edit.get("newText").and_then(Value::as_str).ok_or("Additional completion text is missing.")?.to_owned()));
        }
    }
    let mut start = primary.start;
    for (range, replacement) in &edits {
        if range.end <= primary.start { start = start.checked_add(replacement.len()).and_then(|value|value.checked_sub(range.len())).ok_or("Completion offset overflow.")?; }
    }
    let snippet = match item.get("insertTextFormat").filter(|value|!value.is_null()).map(|value|value.as_u64().ok_or("Invalid completion text format.")).transpose()?.unwrap_or(1) {
        1 => None,
        2 => { let snippet = Snippet::parse(&inserted,start).ok_or("Unsupported or invalid completion snippet.")?; inserted.clone_from(&snippet.text); Some(snippet) },
        _ => return Err("Invalid completion text format.".into()),
    };
    let insertion = start..start.checked_add(inserted.len()).ok_or("Completion length overflow.")?;
    edits.push((primary, inserted));
    edits.sort_by_key(|(range,_)|(range.start,range.end));
    for pair in edits.windows(2) {
        if pair[0].0.end > pair[1].0.start || pair[0].0.start == pair[1].0.start { return Err("Completion edits overlap.".into()); }
    }
    Ok(CompletionPlan { edits, insertion, snippet })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn import_edits_and_unicode_snippet_stops_share_one_plan() {
        let text="//🦀\r\npackage main\r\n\r\nfunc main(){ pri }";
        let begin=text.find("pri").unwrap();
        let item=json!({"label":"Println","insertText":"Println(${1:é})$0","insertTextFormat":2,"additionalTextEdits":[{"range":{"start":{"line":2,"character":0},"end":{"line":2,"character":0}},"newText":"import \"fmt\"\r\n"}]});
        let plan=plan(text,&item,begin..begin+3).unwrap();
        assert!(plan.text(text).contains("import \"fmt\"\r\n"));
        assert!(plan.text(text).ends_with("func main(){ Println(é) }"));
        let snippet=plan.snippet.as_ref().unwrap();
        let range=snippet.stops.iter().find(|(number,_)|*number==1).unwrap().1.clone();
        assert_eq!(&plan.text(text)[range],"é");
        assert_eq!(plan.insertion.start,begin+"import \"fmt\"\r\n".len());
    }
    #[test]
    fn invalid_and_overlapping_edits_never_produce_a_partial_plan() {
        let text="é word";
        let range=|a,b|json!({"start":{"line":0,"character":a},"end":{"line":0,"character":b}});
        for edit in [range(3,6),range(2,2),range(7,7),range(3,2)] {
            let item=json!({"label":"value","additionalTextEdits":[{"range":edit,"newText":"x"}]});
            assert!(plan(text,&item,3..7).is_err());
        }
        assert!(plan(text,&json!({"label":"x"}),1..1).is_err());
        assert_eq!(plan(text,&json!({"label":"x"}),3..7).unwrap().text(text),"é x");
    }
}
#[cfg(test)]
mod snippet_tests {
    use super::*;
    #[test]
    fn numeric_snippets_track_unicode_edits_and_stop_order() {
        let mut snippet = Snippet::parse("Ok(${2:é}) => {${1:value}; $0}", 10).unwrap();
        assert_eq!(snippet.text, "Ok(é) => {value; }");
        assert_eq!(snippet.active, 1);
        let range = snippet.stops.iter().find(|(n, _)| *n == 1).unwrap().1.clone();
        assert!(snippet.adjust(&range, "日本語".len()));
        assert!(snippet.advance(false)); assert_eq!(snippet.active, 2);
        assert!(snippet.advance(false)); assert_eq!(snippet.active, 0);
        assert!(!snippet.advance(false));
        assert!(!snippet.adjust(&(0..0), 1));
        let mirrored = Snippet::parse("${1:name} $1 \\$0 $0", 0).unwrap();
        assert_eq!(mirrored.text, "name name $0 ");
        assert_eq!(mirrored.stops[0].1, 0..4);
        assert_eq!(mirrored.stops[1].1, 5..9);
        assert!(Snippet::parse("${1:unterminated", 0).is_none());
        assert!(Snippet::parse("${VAR}", 0).is_none());
    }
}
