use std::fmt::Write;

/// Half-open UTF-8 byte offsets in the original source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// One exact suggested source edit. Consumers must validate against their snapshot.
#[derive(Clone, Debug)]
pub struct SuggestedFix {
    pub title: String,
    pub span: Span,
    pub replacement: String,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub related: Vec<(Span, String)>,
    pub fixes: Vec<SuggestedFix>,
}

impl Diagnostic {
    pub fn is_warning(&self) -> bool { self.code == "unused-task" }
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            related: Vec::new(),
            fixes: Vec::new(),
        }
    }

    pub fn fix(
        mut self,
        title: impl Into<String>,
        span: Span,
        replacement: impl Into<String>,
    ) -> Self {
        self.fixes.push(SuggestedFix {
            title: title.into(),
            span,
            replacement: replacement.into(),
        });
        self
    }

    pub fn related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push((span, message.into()));
        self
    }

    pub fn to_json(&self, source: &str) -> String {
        let related = self
            .related
            .iter()
            .map(|(span, message)| {
                format!(
                    "{{\"message\":{},\"location\":{}}}",
                    json_string(message),
                    location_json(source, *span)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{{}\"code\":{},\"message\":{},\"location\":{},\"related\":[{}]}}",
            if self.is_warning(){"\"severity\":\"warning\","}else{""},
            json_string(self.code),
            json_string(&self.message),
            location_json(source, self.span),
            related
        )
    }

    pub fn render(&self, path: &str, source: &str) -> String {
        self.render_terminal(path, source, false)
    }

    /// Human-readable excerpts; color is explicitly selected by the CLI.
    pub fn render_terminal(&self, path: &str, source: &str, color: bool) -> String {
        let paint = |text: &str, style: &str| {
            if color {
                format!("\x1b[{style}m{text}\x1b[0m")
            } else {
                text.to_owned()
            }
        };
        let mut output = format!(
            "{}: {}\n",
            paint(&format!("{}[{}]", if self.is_warning(){"warning"}else{"error"}, self.code), if self.is_warning(){"1;33"}else{"1;31"}),
            self.message
        );
        let primary = (self.span, if self.is_warning(){"Task can be abandoned here."}else{"Invalid use here."}.to_owned());
        for (index, (span, label)) in std::iter::once(&primary)
            .chain(self.related.iter())
            .enumerate()
        {
            let (line_number, column) = line_column(source, *span);
            let line = source.lines().nth(line_number - 1).unwrap_or("");
            let line_start = source[..span.start].rfind('\n').map_or(0, |i| i + 1);
            let prefix = source.get(line_start..span.start).unwrap_or("");
            let highlighted = source
                .get(span.start..span.end)
                .unwrap_or("")
                .split('\n')
                .next()
                .unwrap_or("");
            let visible_prefix = terminal_text(prefix);
            let visible_line = terminal_text(line);
            let padding = terminal_width(&visible_prefix);
            let width = terminal_width(&terminal_text(highlighted)).max(1);
            let digits = line_number.to_string().len();
            let style = if index == 0 { "1;31" } else { "1;36" };
            let _ = writeln!(
                output,
                " {} {}",
                paint("-->", "1;34"),
                paint(&format!("{path}:{line_number}:{column}"), "1")
            );
            let _ = writeln!(output, " {} {}", " ".repeat(digits), paint("|", "34"));
            let _ = writeln!(
                output,
                " {} {} {visible_line}",
                paint(&line_number.to_string(), "34"),
                paint("|", "34")
            );
            let _ = writeln!(
                output,
                " {} {} {}{} {}\n",
                " ".repeat(digits),
                paint("|", "34"),
                " ".repeat(padding),
                paint(&"^".repeat(width), style),
                paint(label, style)
            );
        }
        output
    }
}

pub fn line_column(source: &str, span: Span) -> (usize, usize) {
    let prefix = &source[..span.start];
    (
        prefix.bytes().filter(|b| *b == b'\n').count() + 1,
        prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
    )
}

fn location_json(source: &str, span: Span) -> String {
    let (line, column) = line_column(source, span);
    format!(
        "{{\"line\":{line},\"column\":{column},\"start\":{},\"end\":{}}}",
        span.start, span.end
    )
}

pub fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn terminal_text(text: &str) -> String {
    let mut output = String::new();
    let mut width = 0;
    for ch in text.chars() {
        if ch == '\t' {
            let spaces = 4 - width % 4;
            output.push_str(&" ".repeat(spaces));
            width += spaces;
        } else if ch.is_control() {
            let escaped = format!("\\u{{{:x}}}", ch as u32);
            width += escaped.len();
            output.push_str(&escaped);
        } else {
            output.push(ch);
            width += character_width(ch);
        }
    }
    output
}
fn character_width(ch: char) -> usize {
    match ch as u32 {
        0x0300..=0x036f | 0xfe00..=0xfe0f | 0x200d => 0,
        0x1100..=0x115f
        | 0x2e80..=0xa4cf
        | 0xac00..=0xd7a3
        | 0xf900..=0xfaff
        | 0xfe10..=0xfe19
        | 0xfe30..=0xfe6f
        | 0xff01..=0xff60
        | 0xffe0..=0xffe6
        | 0x1f300..=0x1faff => 2,
        _ => 1,
    }
}
fn terminal_width(text: &str) -> usize {
    text.chars().map(character_width).sum()
}
