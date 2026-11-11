//! Import declarations move as complete source slices; bare imports remain order barriers.
use super::{FormatOptions, trivia};
use crate::{Diagnostic, syntax::{Token, TokenKind}};

struct Import {
    first: usize,
    path: usize,
    last: usize,
    bare: bool,
}

fn declarations(tokens: &[Token]) -> Vec<Import> {
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut at = 0;
    while at < tokens.len() {
        let token = &tokens[at];
        if depth == 0 && token.kind == TokenKind::Name && token.text == "import" {
            let mut path = at + 1;
            while path < tokens.len() && !matches!(tokens[path].text.as_str(), ";" | "(") {
                if tokens[path].kind == TokenKind::String { break; }
                path += 1;
            }
            if path < tokens.len() && tokens[path].kind == TokenKind::String
                && (path == at + 1 || tokens[path - 1].text == "from") {
                let last = if tokens.get(path + 1).is_some_and(|token| token.text == ";") { path + 1 } else { path };
                result.push(Import { first: at, path, last, bare: path == at + 1 });
                at = last + 1;
                continue;
            }
        }
        if token.kind == TokenKind::Symbol {
            match token.text.as_str() {
                "{" | "(" | "[" => depth += 1,
                "}" | ")" | "]" => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        at += 1;
    }
    result
}

pub(super) fn organize(source: &str, options: &FormatOptions) -> Result<String, Diagnostic> {
    if !options.organize_imports { return Ok(source.to_owned()); }
    let tokens = crate::syntax::lex(source)?;
    let imports = declarations(&tokens);
    let comments = trivia::comment_spans(source)?;
    let mut output = String::new();
    let mut offset = 0;
    let mut at = 0;
    while at < imports.len() {
        if imports[at].bare { at += 1; continue; }
        let first = at;
        while at + 1 < imports.len() && !imports[at + 1].bare && imports[at + 1].first == imports[at].last + 1 { at += 1; }
        let mut start = tokens[imports[first].first].span.start;
        let gap_start = imports[first].first.checked_sub(1).map_or(0, |i| tokens[i].span.end).max(offset);
        let gap = &source[gap_start..start];
        let attached = gap.rfind("\n\n").map_or(gap_start, |i| gap_start + i + 2);
        if let Some(comment) = comments.iter().find(|span| span.start >= attached && span.end <= start) {
            start = comment.start;
        }
        output.push_str(&source[offset..start]);
        let mut entries = Vec::new();
        let mut begin = start;
        for import in &imports[first..=at] {
            let mut end = tokens[import.last].span.end;
            // A same-line comment belongs to this declaration, not the next one.
            let next_comment = comments.partition_point(|span| span.start < end);
            for comment in &comments[next_comment..] {
                let gap = &source[end..comment.start];
                if !gap.trim().is_empty() || gap.contains('\n') { break; }
                end = comment.end;
            }
            entries.push((!tokens[import.path].text.starts_with("std:"), tokens[import.path].text.as_str(), source[begin..end].trim()));
            begin = end;
        }
        entries.sort_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));
        for (index, entry) in entries.iter().enumerate() {
            if index > 0 { output.push_str(if entries[index - 1].0 != entry.0 { "\n\n" } else { "\n" }); }
            output.push_str(entry.2);
        }
        offset = begin;
        at += 1;
    }
    output.push_str(&source[offset..]);
    Ok(output)
}

pub(super) fn wrap(source: &str, options: &FormatOptions) -> Result<String, Diagnostic> {
    if options.import_wrap_threshold == 0 { return Ok(source.to_owned()); }
    let tokens = crate::syntax::lex(source)?;
    let mut edits = Vec::new();
    for import in declarations(&tokens) {
        let Some(open) = (import.first + 1..import.path).find(|&i| tokens[i].text == "{") else { continue; };
        let Some(close) = (open + 1..import.path).find(|&i| tokens[i].text == "}") else { continue; };
        let commas: Vec<_> = (open + 1..close).filter(|&i| tokens[i].text == ",").collect();
        let trailing = tokens.get(close.wrapping_sub(1)).is_some_and(|token| token.text == ",");
        let count = usize::from(close > open + 1) + commas.len() - usize::from(trailing);
        if count <= options.import_wrap_threshold { continue; }
        for left in std::iter::once(open).chain(commas) {
            let right = left + 1;
            let gap = &source[tokens[left].span.end..tokens[right].span.start];
            let indent = " ".repeat(if right == close { 0 } else { options.indent_width });
            let replacement = if right == close && !gap.trim().is_empty() {
                format!("{}\n", gap.trim_end())
            } else {
                format!("\n{indent}{}", gap.trim_start())
            };
            edits.push((tokens[left].span.end, tokens[right].span.start, replacement));
        }
        if !trailing {
            let gap = &source[tokens[close - 1].span.end..tokens[close].span.start];
            edits.push((tokens[close - 1].span.end, tokens[close].span.start, format!("{}\n", gap.trim_end())));
        }
    }
    let mut output = source.to_owned();
    for (start, end, text) in edits.into_iter().rev() { output.replace_range(start..end, &text); }
    Ok(output)
}

#[cfg(test)]
mod tests {
use crate::format;

#[test]
fn import_rules_group_sort_wrap_and_preserve_declarations() {
    let source = "// module header\n\nimport { Z } from './z.tsr'; // z tail\n// a lead\nimport { A as Alias } from './a.tsr';\nimport { join } from 'std:path';\nimport { A, B as Bee, C, D, E, F } from 'std:fs';\nfunction main() {}\n";
    let result = format(source).unwrap();
    assert!(result.starts_with("// module header\n\nimport {\n  A,\n  B as Bee,\n  C,\n  D,\n  E,\n  F\n} from 'std:fs';\nimport { join } from 'std:path';\n\n// a lead\nimport { A as Alias } from './a.tsr';"), "{result}");
    assert!(result.contains("import { Z } from './z.tsr'; // z tail"), "{result}");
    assert_eq!(format(&result).unwrap(), result);
    let tokens = |text: &str| {
        let mut tokens = crate::syntax::lex(text).unwrap().into_iter().map(|token| format!("{:?}:{}", token.kind, token.text)).collect::<Vec<_>>();
        tokens.sort(); tokens
    };
    assert_eq!(tokens(source), tokens(&result));
    for source in [
        "import { A, B, C, D, E } from './a.tsr';",
        "import { A, B, C, D, E, F, } from './a.tsr';",
        "import { A, /* retain */ B, C, D, E, F } from './a.tsr';",
        "import { A, // retained line\n B, C, D, E, F } from './a.tsr';",
        "import { z } from './z.tsr'; import './side-effect.tsr'; import { a } from 'std:a';",
        "import Z from './z.json'; import A from './a.json';",
        "import { z } from './z.tsr'; const n = 1; import { a } from 'std:a';",
        "import { z } from './z.tsr'; /* tail */ /* more */\nimport { a } from './a.tsr';",
    ] {
        let result = format(source).unwrap();
        assert_eq!(tokens(source), tokens(&result), "{result}");
        assert_eq!(format(&result).unwrap(), result, "{source}");
        if source.contains("/* retain */") { assert!(result.contains("\n  /* retain */ B,"), "{result}"); }
        if source.contains("side-effect") { assert!(result.find("./z.tsr").unwrap() < result.find("side-effect").unwrap() && result.find("side-effect").unwrap() < result.find("std:a").unwrap()); }
    }
    let mut options = crate::FormatOptions::default();
    options.organize_imports = false;
    options.import_wrap_threshold = 0;
    let result = crate::format_with_options(source, &options).unwrap();
    assert!(result.find("./z.tsr").unwrap() < result.find("std:fs").unwrap());
    assert!(!result.contains("import {\n"));
    for value in [serde_json::json!({"importWrapThreshold":101}), serde_json::json!({"importWrapThreshold":-1}), serde_json::json!({"organizeImports":"yes"})] {
        assert!(crate::FormatOptions::overlay(&options, &value).is_err());
    }
}

}
