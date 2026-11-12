//! Numeric LSP snippet stops, tracked in UTF-8 document offsets.
use std::{collections::BTreeMap, ops::Range};

#[derive(Debug)]
pub(super) struct Snippet {
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

#[cfg(test)]
mod tests {
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
