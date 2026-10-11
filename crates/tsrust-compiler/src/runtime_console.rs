// Node-compatible console formatting for the native printable value domain.
enum TsrConsoleValue {
    Text(std::string::String),
    Int(i64),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
    Array(Vec<TsrConsoleValue>),
    Set(Vec<TsrConsoleValue>),
    Map(Vec<(TsrConsoleValue,TsrConsoleValue)>),
    Object(Vec<(std::string::String, TsrConsoleValue)>),
    Truncated,
}
fn tsr_console_quote(text: &str, json: bool) -> std::string::String {
    let quote = if json || (text.contains('\'') && !text.contains('"')) {
        '"'
    } else if text.contains('\'')
        && text.contains('"')
        && !text.contains('`')
        && !text.contains("${")
    {
        '`'
    } else {
        '\''
    };
    let mut output = std::string::String::new();
    output.push(quote);
    for c in text.chars() {
        match c {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{8}' => output.push_str("\\b"),
            '\u{c}' => output.push_str("\\f"),
            c if c == quote => {
                output.push('\\');
                output.push(c);
            }
            c if (c as u32) < 32 || (!json && c == '\u{7f}') => {
                use std::fmt::Write as _;
                if json {
                    write!(output, "\\u{:04x}", c as u32).unwrap();
                } else {
                    write!(output, "\\x{:02X}", c as u32).unwrap();
                }
            }
            c => output.push(c),
        }
    }
    output.push(quote);
    output
}
fn tsr_console_number(value: f64, negative_zero: bool) -> std::string::String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == f64::INFINITY {
        return "Infinity".into();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if value == 0.0 {
        return if negative_zero && value.is_sign_negative() {
            "-0"
        } else {
            "0"
        }
        .into();
    }
    if !(1e-6..1e21).contains(&value.abs()) {
        let mut text = format!("{value:e}");
        tsr_number_tie(value, &mut text, 0);
        let (digits, exponent) = text.split_once('e').unwrap();
        let exponent = exponent.parse::<i32>().unwrap();
        format!("{digits}e{exponent:+}")
    } else {
        let mut text = format!("{value}");
        tsr_number_tie(value, &mut text, 0);
        text
    }
}
impl TsrConsoleValue {
    fn inspect(&self, depth: i32, hidden: bool) -> std::string::String {
        self.inspect_options(depth, hidden, false)
    }
    fn inspect_options(&self, depth: i32, hidden: bool, colors: bool) -> std::string::String {
        let style = |text: std::string::String, code: &str| {
            if colors {
                format!("\x1b[{code}m{text}\x1b[39m")
            } else {
                text
            }
        };
        match self {
            Self::Text(text) => style(tsr_console_quote(text, false), "32"),
            Self::Int(value) => style(value.to_string(), "33"),
            Self::Number(value) => style(tsr_console_number(*value, true), "33"),
            Self::Boolean(value) => style(value.to_string(), "33"),
            Self::Null => {
                if colors {
                    "\x1b[1mnull\x1b[22m".into()
                } else {
                    "null".into()
                }
            }
            Self::Undefined => style("undefined".into(), "90"),
            Self::Truncated => style("[Object]".into(), "36"),
            Self::Map(values) => {
                if depth < 0 { return style("[Map]".into(),"36"); }
                if values.is_empty() { return "Map(0) {}".into(); }
                let cells=values.iter().map(|(key,value)|format!("{} => {}",key.inspect_options(depth-1,hidden,colors),value.inspect_options(depth-1,hidden,colors))).collect::<Vec<_>>();
                format!("Map({}) {{ {} }}",values.len(),cells.join(", "))
            }
            Self::Set(values) => {
                if depth < 0 { return style("[Set]".into(), "36"); }
                if values.is_empty() { return "Set(0) {}".into(); }
                let cells = values.iter().take(100).map(|v| v.inspect_options(depth - 1, hidden, colors)).collect::<Vec<_>>();
                format!("Set({}) {{ {} }}", values.len(), cells.join(", "))
            }
            Self::Array(values) => {
                if depth < 0 {
                    return style("[Array]".into(), "36");
                }
                if values.is_empty() && !hidden {
                    return "[]".into();
                }
                let mut cells = values
                    .iter()
                    .take(100)
                    .map(|v| v.inspect_options(depth - 1, hidden, colors))
                    .collect::<Vec<_>>();
                if values.len() > 100 {
                    cells.push(format!("... {} more items", values.len() - 100));
                }
                if hidden {
                    cells.push(format!(
                        "[length]: {}",
                        style(values.len().to_string(), "33")
                    ));
                }
                format!("[ {} ]", cells.join(", "))
            }
            Self::Object(values) => {
                if depth < 0 {
                    return style("[Object]".into(), "36");
                }
                if values.is_empty() {
                    return "{}".into();
                }
                let cells = values
                    .iter()
                    .map(|(key, v)| {
                        let plain = !key.is_empty()
                            && key.chars().enumerate().all(|(i, c)| {
                                c == '_'
                                    || c == '$'
                                    || c.is_ascii_alphabetic()
                                    || (i > 0 && c.is_ascii_digit())
                            });
                        format!(
                            "{}: {}",
                            if plain {
                                key.clone()
                            } else {
                                style(tsr_console_quote(key, false), "32")
                            },
                            v.inspect_options(depth - 1, hidden, colors)
                        )
                    })
                    .collect::<Vec<_>>();
                format!("{{ {} }}", cells.join(", "))
            }
        }
    }
    fn plain(&self) -> std::string::String {
        match self {
            Self::Text(s) => s.clone(),
            _ => self.inspect(2, false),
        }
    }
    fn json(&self) -> std::string::String {
        match self {
            Self::Text(s) => tsr_console_quote(s, true),
            Self::Int(v) => v.to_string(),
            Self::Number(v) => {
                if v.is_finite() {
                    tsr_console_number(*v, false)
                } else {
                    "null".into()
                }
            }
            Self::Boolean(v) => v.to_string(),
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
            Self::Array(v) => format!(
                "[{}]",
                v.iter()
                    .map(|v| if matches!(v, Self::Undefined) {
                        "null".into()
                    } else {
                        v.json()
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Set(_) | Self::Map(_) => "{}".into(),
            Self::Object(v) => format!(
                "{{{}}}",
                v.iter()
                    .filter(|(_, v)| !matches!(v, Self::Undefined))
                    .map(|(k, v)| format!("{}:{}", tsr_console_quote(k, true), v.json()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Self::Truncated => "null".into(),
        }
    }
    fn numeric(&self, prefix: bool, integer: bool) -> f64 {
        let text = match self {
            Self::Text(s) => s.clone(),
            Self::Null => return if prefix { f64::NAN } else { 0.0 },
            Self::Undefined => return f64::NAN,
            Self::Boolean(v) => {
                return if prefix {
                    f64::NAN
                } else {
                    f64::from(u8::from(*v))
                };
            }
            Self::Int(v) => {
                return if integer {
                    (*v as f64).trunc()
                } else {
                    *v as f64
                };
            }
            Self::Number(v) => {
                if prefix {
                    tsr_console_number(*v, false)
                } else {
                    return *v;
                }
            }
            Self::Array(v) => v
                .iter()
                .map(|v| {
                    if matches!(v, Self::Null | Self::Undefined) {
                        std::string::String::new()
                    } else {
                        v.label()
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
            _ => return f64::NAN,
        };
        let text = text.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
        let unsigned = text
            .strip_prefix('-')
            .or_else(|| text.strip_prefix('+'))
            .unwrap_or(text);
        let sign = if text.starts_with('-') { -1.0 } else { 1.0 };
        if integer || !prefix {
            let radix = if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
                16
            } else if !prefix && (unsigned.starts_with("0b") || unsigned.starts_with("0B")) {
                2
            } else if !prefix && (unsigned.starts_with("0o") || unsigned.starts_with("0O")) {
                8
            } else {
                10
            };
            if integer || radix != 10 {
                if !prefix && unsigned != text {
                    return f64::NAN;
                }
                let digits = if radix == 10 {
                    unsigned
                } else {
                    &unsigned[2..]
                };
                let mut value = 0.0;
                let mut count = 0;
                for c in digits.chars() {
                    if let Some(digit) = c.to_digit(radix) {
                        value = value * f64::from(radix) + f64::from(digit);
                        count += 1;
                    } else {
                        break;
                    }
                }
                if count == 0 || (!prefix && count != digits.len()) {
                    return f64::NAN;
                }
                return sign * value;
            }
        }
        if !prefix {
            return if text.is_empty() {
                0.0
            } else if unsigned == "Infinity" {
                sign * f64::INFINITY
            } else if matches!(unsigned, "inf" | "NaN") {
                f64::NAN
            } else if text
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
            {
                text.parse().unwrap_or(f64::NAN)
            } else {
                f64::NAN
            };
        }
        if unsigned.starts_with("Infinity") {
            return sign * f64::INFINITY;
        }
        let end = text
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E'))
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8());
        for end in (1..=end).rev() {
            if text.is_char_boundary(end)
                && let Ok(value) = text[..end].parse::<f64>()
            {
                return value;
            }
        }
        f64::NAN
    }
}
fn tsr_console_format(values: &[TsrConsoleValue]) -> std::string::String {
    let Some(first) = values.first() else {
        return std::string::String::new();
    };
    let TsrConsoleValue::Text(format) = first else {
        return values
            .iter()
            .map(TsrConsoleValue::plain)
            .collect::<Vec<_>>()
            .join(" ");
    };
    if values.len() == 1 {
        return format.clone();
    }
    let mut output = std::string::String::new();
    let mut argument = 1;
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%'
            && let Some(&specifier) = chars.peek()
        {
            if specifier == '%' {
                chars.next();
                output.push('%');
                continue;
            }
            if argument < values.len()
                && matches!(specifier, 's' | 'd' | 'i' | 'f' | 'j' | 'o' | 'O' | 'c')
            {
                chars.next();
                let value = &values[argument];
                argument += 1;
                let text = match specifier {
                    's' => match value {
                        TsrConsoleValue::Array(_) | TsrConsoleValue::Object(_) => {
                            value.inspect(0, false)
                        }
                        _ => value.plain(),
                    },
                    'd' => tsr_console_number(value.numeric(false, false), true),
                    'i' => tsr_console_number(value.numeric(true, true), true),
                    'f' => tsr_console_number(value.numeric(true, false), true),
                    'j' => value.json(),
                    'o' => value.inspect(4, true),
                    'O' => value.inspect(2, false),
                    'c' => std::string::String::new(),
                    _ => unreachable!(),
                };
                output.push_str(&text);
                continue;
            }
        }
        output.push(c);
    }
    for value in &values[argument..] {
        output.push(' ');
        output.push_str(&value.plain());
    }
    output
}
fn tsr_console_width(text: &str) -> usize {
    text.chars()
        .map(|c| {
            let n = c as u32;
            if (0x300..=0x36f).contains(&n) || n == 0x200d || (0xfe00..=0xfe0f).contains(&n) {
                0
            } else if (0x1100..=0x115f).contains(&n)
                || (0x2e80..=0xa4cf).contains(&n)
                || (0xac00..=0xd7a3).contains(&n)
                || (0xf900..=0xfaff).contains(&n)
                || (0xff01..=0xff60).contains(&n)
                || (0x1f300..=0x1faff).contains(&n)
            {
                2
            } else {
                1
            }
        })
        .sum()
}
fn tsr_console_table(
    data: &TsrConsoleValue,
    selected: Option<Vec<std::string::String>>,
) -> std::string::String {
    let rows = match data {
        TsrConsoleValue::Array(values) => values
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v))
            .collect::<Vec<_>>(),
        TsrConsoleValue::Object(values) => values.iter().map(|(k, v)| (k.clone(), v)).collect(),
        _ => return format!("{}\n", data.plain()),
    };
    let mut columns = selected.clone().unwrap_or_default();
    let mut has_values = false;
    if selected.is_none() {
        for (_, value) in &rows {
            match value {
                TsrConsoleValue::Object(fields) => {
                    for (key, _) in fields {
                        if !columns.contains(key) {
                            columns.push(key.clone());
                        }
                    }
                }
                TsrConsoleValue::Array(items) => {
                    for i in 0..items.len() {
                        let key = i.to_string();
                        if !columns.contains(&key) {
                            columns.push(key);
                        }
                    }
                }
                _ => has_values = true,
            }
        }
    }
    if has_values {
        columns.push("Values".into());
    }
    let mut matrix = vec![vec![std::string::String::from("(index)")]];
    matrix[0].extend(columns.iter().cloned());
    for (index, value) in rows {
        let mut row = vec![index];
        for key in &columns {
            let cell = match value {
                TsrConsoleValue::Object(fields) => {
                    fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
                }
                TsrConsoleValue::Array(items) => {
                    key.parse::<usize>().ok().and_then(|i| items.get(i))
                }
                _ => {
                    if key == "Values" {
                        Some(value)
                    } else {
                        None
                    }
                }
            };
            row.push(cell.map_or_else(std::string::String::new, |v| v.inspect(0, false)));
        }
        matrix.push(row);
    }
    let widths = (0..matrix[0].len())
        .map(|column| {
            matrix
                .iter()
                .map(|row| tsr_console_width(&row[column]))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    let border = |left: char, middle: char, right: char| {
        format!(
            "{left}{}{right}\n",
            widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join(&middle.to_string())
        )
    };
    let mut output = border('┌', '┬', '┐');
    for (i, row) in matrix.iter().enumerate() {
        output.push('│');
        for (cell, width) in row.iter().zip(&widths) {
            output.push(' ');
            output.push_str(cell);
            output.push_str(&" ".repeat(width - tsr_console_width(cell) + 1));
            output.push('│');
        }
        output.push('\n');
        if i == 0 {
            output.push_str(&border('├', '┼', '┤'));
        }
    }
    output.push_str(&border('└', '┴', '┘'));
    output
}
#[derive(Default)]
struct TsrConsoleState {
    indent: usize,
    counts: std::collections::BTreeMap<std::string::String, u64>,
    timers: std::collections::BTreeMap<std::string::String, std::time::Instant>,
}
std::thread_local! {static TSR_CONSOLE_STATE: std::cell::RefCell<TsrConsoleState> = std::cell::RefCell::new(TsrConsoleState::default());}
fn tsr_console_write(error: bool, text: &str) {
    if tsr_console_test_capture(error,text) {return;}
    use std::io::Write as _;
    TSR_CONSOLE_STATE.with(|state| {
        let indent = state.borrow().indent;
        fn write(destination: &mut impl std::io::Write, text: &str, indent: usize) {
            for line in text.split_inclusive('\n') {
                if indent > 0 {
                    let _ = write!(destination, "{:width$}", "", width = indent);
                }
                let _ = destination.write_all(line.as_bytes());
            }
        }
        // Node's default console ignores destination write errors.
        if error {
            write(&mut std::io::stderr().lock(), text, indent);
        } else {
            write(&mut std::io::stdout().lock(), text, indent);
        }
    });
}
fn tsr_console_scalar(error: bool, value: impl std::fmt::Display) {
    if TSR_CONSOLE_TEST.with(|slot|slot.borrow().name.is_some()) && tsr_console_test_capture(error,&format!("{value}\n")) {return;}
    use std::io::Write as _;
    TSR_CONSOLE_STATE.with(|state| {
        let indent = state.borrow().indent;
        if error {
            let _ = writeln!(
                std::io::stderr().lock(),
                "{:width$}{value}",
                "",
                width = indent
            );
        } else {
            let _ = writeln!(
                std::io::stdout().lock(),
                "{:width$}{value}",
                "",
                width = indent
            );
        }
    });
}
fn tsr_console_text(error: bool, value: &str) {
    if TSR_CONSOLE_TEST.with(|slot|slot.borrow().name.is_some()) && tsr_console_test_capture(error,&format!("{value}\n")) {return;}
    use std::io::Write as _;
    TSR_CONSOLE_STATE.with(|state| {
        let indent = state.borrow().indent;
        fn write(destination: &mut impl std::io::Write, value: &str, indent: usize) {
            for (index, line) in value.split('\n').enumerate() {
                if index > 0 {
                    let _ = destination.write_all(b"\n");
                }
                let _ = write!(destination, "{:width$}{line}", "", width = indent);
            }
            let _ = destination.write_all(b"\n");
        }
        if error {
            write(&mut std::io::stderr().lock(), value, indent);
        } else {
            write(&mut std::io::stdout().lock(), value, indent);
        }
    });
}

impl TsrConsoleValue {
    fn truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Boolean(v) => *v,
            Self::Int(v) => *v != 0,
            Self::Number(v) => *v != 0.0 && !v.is_nan(),
            Self::Text(v) => !v.is_empty(),
            _ => true,
        }
    }
    fn label(&self) -> std::string::String {
        match self {
            Self::Text(v) => v.clone(),
            Self::Number(v) => tsr_console_number(*v, false),
            Self::Set(_) => "[object Set]".into(),
            Self::Map(_) => "[object Map]".into(),
            Self::Object(_) | Self::Truncated => "[object Object]".into(),
            Self::Array(v) => v
                .iter()
                .map(|v| {
                    if matches!(v, Self::Null | Self::Undefined) {
                        std::string::String::new()
                    } else {
                        v.label()
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
            _ => self.plain(),
        }
    }
}
fn tsr_console_call(method: &str, values: &[TsrConsoleValue]) {
    let line = |error, text: std::string::String| tsr_console_write(error, &format!("{text}\n"));
    match method {
        "log" | "info" | "debug" | "dirxml" => line(false, tsr_console_format(values)),
        "warn" | "error" => line(true, tsr_console_format(values)),
        "assert" => {
            if !values.first().is_some_and(TsrConsoleValue::truthy) {
                let message = tsr_console_format(values.get(1..).unwrap_or(&[]));
                line(
                    true,
                    if values.len() <= 1 {
                        "Assertion failed".into()
                    } else if matches!(values.get(1), Some(TsrConsoleValue::Text(_))) {
                        format!("Assertion failed: {message}")
                    } else {
                        format!("Assertion failed {message}")
                    },
                );
            }
        }
        "group" | "groupCollapsed" => {
            if !values.is_empty() {
                line(false, tsr_console_format(values));
            }
            TSR_CONSOLE_STATE.with(|s| s.borrow_mut().indent += 2);
        }
        "groupEnd" => TSR_CONSOLE_STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.indent = s.indent.saturating_sub(2);
        }),
        "count" | "countReset" => {
            let label = values
                .first()
                .filter(|v| !matches!(v, TsrConsoleValue::Undefined))
                .map_or_else(|| "default".into(), TsrConsoleValue::label);
            let message = TSR_CONSOLE_STATE.with(|s| {
                let mut s = s.borrow_mut();
                if method == "count" {
                    let n = s.counts.entry(label.clone()).or_default();
                    *n += 1;
                    Some(format!("{label}: {n}"))
                } else if values.first().is_some_and(|v| {
                    !matches!(v, TsrConsoleValue::Text(_) | TsrConsoleValue::Undefined)
                }) || s.counts.remove(&label).is_none()
                {
                    Some(format!("Count for '{label}' does not exist"))
                } else {
                    None
                }
            });
            if let Some(message) = message {
                line(method == "countReset", message);
            }
        }
        "time" | "timeLog" | "timeEnd" => {
            let label = values
                .first()
                .filter(|v| !matches!(v, TsrConsoleValue::Undefined))
                .map_or_else(|| "default".into(), TsrConsoleValue::label);
            let message = TSR_CONSOLE_STATE.with(|s| {
                let mut s = s.borrow_mut();
                if method == "time" {
                    if s.timers.contains_key(&label) {
                        return Some((
                            true,
                            format!("Label '{label}' already exists for console.time()"),
                        ));
                    }
                    s.timers.insert(label.clone(), std::time::Instant::now());
                    None
                } else {
                    let Some(start) = s.timers.get(&label).copied() else {
                        return Some((
                            true,
                            format!("No such label '{label}' for console.{method}()"),
                        ));
                    };
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                    if method == "timeEnd" {
                        s.timers.remove(&label);
                    }
                    let mut text = format!("{label}: {elapsed:.3}ms");
                    while text.ends_with("0ms") {
                        text.truncate(text.len() - 3);
                        text.push_str("ms");
                    }
                    if text.ends_with(".ms") {
                        text.truncate(text.len() - 3);
                        text.push_str("ms");
                    }
                    if method == "timeLog" && values.len() > 1 {
                        text.push(' ');
                        text.push_str(
                            &values[1..]
                                .iter()
                                .map(TsrConsoleValue::plain)
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                    }
                    Some((false, text))
                }
            });
            if let Some((error, text)) = message {
                line(error, text);
            }
        }
        "dir" => {
            let mut depth = 2;
            let mut hidden = false;
            let mut colors = false;
            if let Some(TsrConsoleValue::Object(options)) = values.get(1) {
                for (name, value) in options {
                    match (name.as_str(), value) {
                        ("depth", TsrConsoleValue::Null) => depth = 128,
                        ("depth", TsrConsoleValue::Int(v)) => depth = (*v).clamp(-1, 128) as i32,
                        ("depth", TsrConsoleValue::Number(v)) => {
                            depth = v.clamp(-1.0, 128.0) as i32
                        }
                        ("showHidden", TsrConsoleValue::Boolean(v)) => hidden = *v,
                        ("colors", TsrConsoleValue::Boolean(v)) => colors = *v,
                        _ => {}
                    }
                }
            }
            line(
                false,
                values.first().map_or_else(
                    || "undefined".into(),
                    |v| v.inspect_options(depth, hidden, colors),
                ),
            );
        }
        "trace" => line(
            true,
            format!(
                "Trace: {}\n{}",
                tsr_console_format(values),
                std::backtrace::Backtrace::force_capture()
            ),
        ),
        // Inspector-only methods are no-ops without an attached inspector.
        "profile" | "profileEnd" | "timeStamp" => {}
        "clear" => {
            use std::io::{IsTerminal as _, Write as _};
            let mut stdout = std::io::stdout().lock();
            if stdout.is_terminal() && std::env::var("TERM").as_deref() != Ok("dumb") {
                let _ = stdout.write_all(b"\x1b[1;1H\x1b[0J");
            }
        }
        _ => unreachable!("checked console method"),
    }
}

// Test reporting captures console calls only while a pretty-report scope is active.
#[derive(Default)]
struct TsrConsoleTest { name: Option<std::string::String>, entries: Vec<(bool,std::string::String,std::string::String)> }
std::thread_local! {
    static TSR_CONSOLE_TEST: std::cell::RefCell<TsrConsoleTest> = std::cell::RefCell::new(TsrConsoleTest::default());
    static TSR_CONSOLE_LOCATION: std::cell::RefCell<std::string::String> = const { std::cell::RefCell::new(std::string::String::new()) };
}
struct TsrConsoleLocation(Option<std::string::String>);
impl TsrConsoleLocation {
    fn new(path:&str,line:usize,column:usize)->Self {
        if !TSR_CONSOLE_TEST.with(|slot|slot.borrow().name.is_some()) {return Self(None);}
        let cwd=std::env::current_dir().unwrap_or_default();
        let path=std::path::Path::new(path);
        let location=format!("{}:{line}:{column}",path.strip_prefix(cwd).unwrap_or(path).display());
        Self(Some(TSR_CONSOLE_LOCATION.with(|slot|slot.replace(location))))
    }
}
impl Drop for TsrConsoleLocation {fn drop(&mut self){if let Some(previous)=self.0.take(){TSR_CONSOLE_LOCATION.with(|slot|{slot.replace(previous);});}}}
fn tsr_console_test_begin(name:&str) {
    if std::env::var("TSRUST_TEST_REPORTER").as_deref()!=Ok("pretty") {return;}
    TSR_CONSOLE_TEST.with(|slot|slot.borrow_mut().name=Some(name.to_owned()));
}
fn tsr_console_test_capture(error:bool,text:&str)->bool {
    TSR_CONSOLE_TEST.with(|slot|{
        let mut state=slot.borrow_mut();if state.name.is_none(){return false;}
        let indent=TSR_CONSOLE_STATE.with(|state|state.borrow().indent);
        let text=text.split_inclusive('\n').map(|line|format!("{:indent$}{line}","")).collect();
        state.entries.push((error,TSR_CONSOLE_LOCATION.with(|slot|slot.borrow().clone()),text));true
    })
}
fn tsr_console_test_end(show_scope:bool) {
    let state=TSR_CONSOLE_TEST.with(|slot|std::mem::take(&mut *slot.borrow_mut()));
    let Some(name)=state.name else{return;};
    use std::io::Write as _;
    for (error,location,text) in state.entries {
        let stream=if error {"stderr"} else {"stdout"};
        let mut output=if show_scope {format!("     {stream} | {name}\n")} else {format!("     {stream}\n")};
        if !location.is_empty(){output.push_str(&format!("       at {location}\n"));}
        for line in text.lines(){output.push_str(&format!("         {line}\n"));}
        if error {let _=std::io::stderr().lock().write_all(output.as_bytes());}
        else {let _=std::io::stdout().lock().write_all(output.as_bytes());}
    }
}
