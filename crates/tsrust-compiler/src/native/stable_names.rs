//! Keep generated item names stable across edits for incremental native builds.
//!
//! The emitter numbers records, unions, functions, closures and several helper
//! enums program-wide in creation order. Adding one function or type renumbers
//! everything created after it, so an edit to one module rewrites generated
//! code across the whole program and rustc's incremental cache (and, later,
//! per-module crates) cannot reuse it. On Studio, adding one function with one
//! closure rewrote 35% of the generated Rust.
//!
//! This pass renames those numbers through a ledger kept beside the rustc
//! incremental session. Each item gets a key from the checked program (source
//! names, literal values, type arguments, member types, enclosing function) or,
//! for helper enums and data statics, from the first item that uses it. The
//! ledger remembers the number assigned to each key, so existing items keep
//! their numbers and only new items get new ones. Labels and temporaries that
//! are numbered program-wide are renumbered within each top-level item.
//!
//! Renaming is a bijection within each number space and skips string literals
//! and comments, so program behavior never depends on key quality: keys only
//! decide how much generated code stays byte-identical between builds.
use crate::analysis::CheckedProgram;
use crate::syntax::Type;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Stable identity of each checked record, union, function and closure.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Keys {
    pub records: Vec<String>,
    pub unions: Vec<String>,
    pub functions: Vec<String>,
    pub closures: Vec<String>,
    /// Source modules in dependency order, and each function's and closure's
    /// module index (None: made by the compiler), for per-module crates.
    #[serde(default)]
    pub modules: Vec<String>,
    #[serde(default)]
    pub function_modules: Vec<Option<usize>>,
    #[serde(default)]
    pub closure_modules: Vec<Option<usize>>,
}

/// Number assigned to each key, per number space; persisted between builds.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ledger {
    spaces: BTreeMap<String, BTreeMap<String, Vec<u64>>>,
}

impl Ledger {
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }
    pub fn save(&self, path: &std::path::Path) {
        if let Ok(bytes) = serde_json::to_vec(self) {
            let pending = path.with_extension(format!("{}.tmp", std::process::id()));
            if std::fs::write(&pending, bytes)
                .and_then(|()| std::fs::rename(&pending, path))
                .is_err()
            {
                let _ = std::fs::remove_file(&pending);
            }
        }
    }
}

fn digest(text: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(text.as_bytes())
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Replace linked module indices with module paths and drop creation counters.
fn normalize(name: &str, modules: &[String]) -> String {
    let mut out = String::with_capacity(name.len());
    let mut rest = name;
    while let Some(index) = rest.find("__tsrust_module_") {
        out.push_str(&rest[..index]);
        let tail = &rest[index + "__tsrust_module_".len()..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        match tail[..digits]
            .parse::<usize>()
            .ok()
            .and_then(|module| modules.get(module))
        {
            Some(path) => {
                out.push_str(path);
                out.push_str("::");
                rest = tail[digits..].strip_prefix('_').unwrap_or(&tail[digits..]);
            }
            None => {
                out.push_str("__tsrust_module_");
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    // `<specialization#7>` is a creation counter; type arguments identify it instead.
    let mut cleaned = String::with_capacity(out.len());
    let mut chars = out.chars().peekable();
    while let Some(c) = chars.next() {
        cleaned.push(c);
        if c == '#' {
            while chars.peek().is_some_and(char::is_ascii_digit) {
                chars.next();
            }
        }
    }
    cleaned
}

struct Context<'a> {
    records: &'a [String],
    unions: &'a [String],
    closures: &'a [String],
}

fn type_text(ty: Type, context: &Context<'_>) -> String {
    let record = |id: usize| context.records.get(id).map_or("?", String::as_str);
    match ty {
        Type::Record(id) => format!("R{}", record(id)),
        Type::Enum(id, _) => format!("E{}", record(id)),
        Type::Literal(id, _) => format!("L{}", record(id)),
        Type::Set(id) => format!("S{}", record(id)),
        Type::Map(id) => format!("M{}", record(id)),
        Type::Array(element) => format!("[{}]", type_text(element.ty(), context)),
        Type::Union(id) => format!(
            "U{}",
            context.unions.get(id.index()).map_or("?", String::as_str)
        ),
        Type::BorrowedUnion(id) => format!(
            "&U{}",
            context.unions.get(id.index()).map_or("?", String::as_str)
        ),
        Type::Closure(id) => format!("C{}", context.closures.get(id).map_or("?", String::as_str)),
        // Contract and family numbers are themselves creation counters.
        Type::Callable(_) => "callable".into(),
        Type::Callback(_) => "callback".into(),
        Type::StoredCallable(_) => "stored".into(),
        Type::BorrowedCallable(_) => "&stored".into(),
        other => format!("{other:?}"),
    }
}

/// Derive keys for the checked program. `modules` lists linked module paths by index.
pub fn keys(program: &CheckedProgram, modules: &[String]) -> Keys {
    let record_base = program
        .records
        .iter()
        .map(|record| {
            let mut base = normalize(&record.name.text, modules);
            if let Some(native) = record.native_type {
                base.push_str(&format!("|native:{native:?}"));
            }
            if let Some(literal) = &record.literal_definition {
                base.push_str(&format!("|literal:{literal:?}"));
            }
            if let Some(definition) = &record.enum_definition {
                base.push_str("|enum:");
                for member in &definition.members {
                    base.push_str(&member.name.text);
                    base.push(',');
                }
            }
            base
        })
        .collect::<Vec<_>>();
    let mut records = record_base
        .iter()
        .map(|base| digest(base))
        .collect::<Vec<_>>();
    let mut unions = vec![String::new(); program.unions.len()];
    let no_closures = Vec::new();
    // Refine records and unions together so recursive and nested shapes settle.
    for _ in 0..4 {
        let context = Context {
            records: &records,
            unions: &unions,
            closures: &no_closures,
        };
        let next_unions = program
            .unions
            .iter()
            .map(|union| {
                digest(
                    &union
                        .members
                        .iter()
                        .map(|member| type_text(*member, &context))
                        .collect::<Vec<_>>()
                        .join("|"),
                )
            })
            .collect::<Vec<_>>();
        let next_records = program
            .records
            .iter()
            .zip(&record_base)
            .map(|(record, base)| {
                let mut text = base.clone();
                for field in &record.fields {
                    text.push_str(&format!(
                        "|{}:{}",
                        field.name.text,
                        field.ty.map_or("?".into(), |ty| type_text(ty, &context))
                    ));
                }
                if let Some(application) = &record.application {
                    for argument in &application.resolved_arguments {
                        text.push_str(&format!("|arg:{}", type_text(*argument, &context)));
                    }
                }
                if let Some(map) = &record.map_definition {
                    text.push_str(&format!("|map:{}", type_text(map.value, &context)));
                }
                digest(&text)
            })
            .collect::<Vec<_>>();
        unions = next_unions;
        records = next_records;
    }
    let context = Context {
        records: &records,
        unions: &unions,
        closures: &no_closures,
    };
    let signature = |function: &crate::analysis::CheckedFunction| {
        let parameters = function
            .bindings
            .iter()
            .take(function.parameter_count)
            .map(|binding| type_text(binding.ty, &context))
            .collect::<Vec<_>>()
            .join(",");
        format!("({parameters})->{}", type_text(function.result, &context))
    };
    let functions = program
        .functions
        .iter()
        .map(|function| {
            digest(&format!(
                "{}{}",
                normalize(&function.name, modules),
                signature(function)
            ))
        })
        .collect::<Vec<_>>();
    let mut ordinals: HashMap<String, usize> = HashMap::new();
    let closures = program
        .closures
        .iter()
        .map(|closure| {
            let enclosing = normalize(&closure.function.name, modules);
            let ordinal = ordinals.entry(enclosing.clone()).or_default();
            let key = digest(&format!(
                "{enclosing}#{ordinal}{}",
                signature(&closure.function)
            ));
            *ordinal += 1;
            key
        })
        .collect::<Vec<_>>();
    Keys {
        records,
        unions,
        functions,
        closures,
        ..Keys::default()
    }
}

/// Number spaces: every prefix in a space shares the same number for one item.
const SPACES: &[(&str, &[&str])] = &[
    (
        "record",
        &[
            "Record",
            "RecordArray",
            "Map",
            "Set",
            "tsr_console_record_",
            "decode_schema",
        ],
    ),
    ("union", &["Union", "ViewUnion"]),
    (
        "function",
        &["function", "tsr_result_function", "TsrFunctionError"],
    ),
    ("closure", &["Closure", "TsrClosureError"]),
    ("storage", &["StoredCallable", "TsrStorageError"]),
    ("callback", &["TsrCallbackError"]),
    ("region", &["RegionError", "RegionIncoming"]),
    ("data", &["TSR_DATA"]),
];
/// Identifiers numbered program-wide but only meaningful inside one item.
const LOCAL: &[&str] = &[
    "conditional_temporary_",
    "optional_slot",
    "spread_position",
    "spread_rest",
    "spread_count",
    "spread_segment",
    "spread_source",
];
const LABELS: &[&str] = &["exception_region", "match_value", "loop_", "body_"];

#[derive(Clone, Copy, PartialEq)]
enum Token<'a> {
    /// A numbered program-wide identifier: (space index, prefix, number).
    Numbered(usize, &'a str, u64),
    /// A numbered item-local identifier or label: (prefix, number, is_label).
    Local(&'a str, u64, bool),
}

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn classify(word: &str, label: bool) -> Option<Token<'_>> {
    let digits = word.bytes().rev().take_while(u8::is_ascii_digit).count();
    if digits == 0 || digits > 18 {
        return None;
    }
    let (prefix, number) = word.split_at(word.len() - digits);
    let number = number.parse().ok()?;
    if label {
        return LABELS
            .contains(&prefix)
            .then_some(Token::Local(prefix, number, true));
    }
    for (space, (_, prefixes)) in SPACES.iter().enumerate() {
        if prefixes.contains(&prefix) {
            return Some(Token::Numbered(space, prefix, number));
        }
    }
    LOCAL
        .contains(&prefix)
        .then_some(Token::Local(prefix, number, false))
}

/// Visit numbered tokens outside strings, characters and comments.
fn scan<'a>(text: &'a str, mut visit: impl FnMut(usize, usize, Token<'a>)) {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        let c = bytes[i];
        if c == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i = text[i..].find('\n').map_or(n, |end| i + end);
        } else if c == b'/' && bytes.get(i + 1) == Some(&b'*') {
            let mut depth = 1;
            i += 2;
            while depth > 0 && i < n {
                if bytes[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if c == b'"' {
            i += 1;
            while i < n && bytes[i] != b'"' {
                i += if bytes[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
        } else if c == b'r'
            && (i == 0 || !identifier_byte(bytes[i - 1]))
            && matches!(bytes.get(i + 1), Some(b'"' | b'#'))
        {
            let hashes = bytes[i + 1..]
                .iter()
                .take_while(|byte| **byte == b'#')
                .count();
            if bytes.get(i + 1 + hashes) == Some(&b'"') {
                let close = format!("\"{}", "#".repeat(hashes));
                i = text[i + 2 + hashes..]
                    .find(&close)
                    .map_or(n, |end| i + 2 + hashes + end + close.len());
            } else {
                i += 1;
            }
        } else if c == b'\'' {
            // Character literal, or a label/lifetime identifier.
            let rest = &text[i + 1..];
            let mut chars = rest.char_indices();
            match chars.next() {
                Some((_, '\\')) => {
                    i = rest[2..].find('\'').map_or(n, |end| i + 1 + 2 + end + 1);
                }
                Some((_, first)) if matches!(chars.next(), Some((_, '\''))) => {
                    i += 1 + first.len_utf8() + 1
                }
                Some(_) => {
                    let start = i + 1;
                    let end = start
                        + text[start..]
                            .bytes()
                            .take_while(|byte| identifier_byte(*byte))
                            .count();
                    if let Some(token) = classify(&text[start..end], true) {
                        visit(start, end, token);
                    }
                    i = end.max(i + 1);
                }
                None => i += 1,
            }
        } else if (c.is_ascii_alphabetic() || c == b'_')
            && (i == 0 || !identifier_byte(bytes[i - 1]))
        {
            let end = i + text[i..]
                .bytes()
                .take_while(|byte| identifier_byte(*byte))
                .count();
            if let Some(token) = classify(&text[i..end], false) {
                visit(i, end, token);
            }
            i = end;
        } else {
            i += 1;
        }
    }
}

/// Rename generated numbers in `application` through `ledger`.
pub fn stabilize(application: &str, keys: &Keys, ledger: &mut Ledger) -> String {
    stabilize_with_origins(application, keys, ledger).0
}

/// Compiler numbers of renamed items: stable number to the number the compiler
/// created (which indexes `Keys` and crate-plan provenance).
#[derive(Default)]
pub struct Origins {
    pub functions: HashMap<u64, u64>,
    pub closures: HashMap<u64, u64>,
}

/// As [`stabilize`], also returning where each function and closure number came from.
pub fn stabilize_with_origins(application: &str, keys: &Keys, ledger: &mut Ledger) -> (String, Origins) {
    let Some(items) = super::codegen_units::top_level_items(application) else {
        return (application.to_owned(), Origins::default());
    };
    // Owner of each item: the numbered item it defines, if any.
    let item_owner = |item: &super::codegen_units::Item| -> Option<(usize, u64)> {
        let head = &application[item.head..item.end];
        let mut owner = None;
        let line = head.split('\n').next().unwrap_or_default();
        let declared = line
            .split(['{', '(', '<', '=', ':'])
            .next()
            .unwrap_or_default();
        scan(declared, |_, _, token| {
            if let (None, Token::Numbered(space, _, number)) = (owner, token) {
                owner = Some((space, number));
            }
        });
        owner
    };
    let compiler_key = |space: usize, number: u64| -> Option<&String> {
        let list = match SPACES[space].0 {
            "record" => &keys.records,
            "union" => &keys.unions,
            "function" => &keys.functions,
            "closure" => &keys.closures,
            _ => return None,
        };
        list.get(usize::try_from(number).ok()?)
    };
    // Key every number that appears; anchored spaces key by first using item.
    let mut key: HashMap<(usize, u64), String> = HashMap::new();
    let mut anchored_ordinal: HashMap<String, usize> = HashMap::new();
    for item in &items {
        let text = &application[item.start..item.end];
        let owner = item_owner(item);
        let owner_key = match owner {
            Some((space, number)) => compiler_key(space, number)
                .cloned()
                .unwrap_or_else(|| format!("{}:{number}", SPACES[space].0)),
            None => digest(text.split('\n').next().unwrap_or_default()),
        };
        scan(text, |_, _, token| {
            let Token::Numbered(space, _, number) = token else {
                return;
            };
            if key.contains_key(&(space, number)) || owner == Some((space, number)) {
                return;
            }
            let derived = match compiler_key(space, number) {
                Some(stable) => stable.clone(),
                None => {
                    let ordinal = anchored_ordinal
                        .entry(format!("{space}:{owner_key}"))
                        .or_default();
                    let derived = format!("{owner_key}#{}", *ordinal);
                    *ordinal += 1;
                    derived
                }
            };
            key.insert((space, number), derived);
        });
        // A number only seen in its own definition still needs a key.
        if let Some((space, number)) = owner {
            key.entry((space, number)).or_insert_with(|| {
                compiler_key(space, number)
                    .cloned()
                    .unwrap_or_else(|| digest(text))
            });
        }
    }
    // Assign numbers: reuse the ledger's number for a key, else take a fresh one.
    let mut mapping: HashMap<(usize, u64), u64> = HashMap::new();
    for (space, (name, _)) in SPACES.iter().enumerate() {
        let mut numbers = key
            .keys()
            .filter(|(s, _)| *s == space)
            .map(|(_, number)| *number)
            .collect::<Vec<_>>();
        numbers.sort_unstable();
        let book = ledger.spaces.entry((*name).to_owned()).or_default();
        let mut next = book
            .values()
            .flatten()
            .copied()
            .max()
            .map_or(0, |max| max + 1);
        let mut used = std::collections::HashSet::new();
        let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        for number in numbers {
            groups
                .entry(key[&(space, number)].clone())
                .or_default()
                .push(number);
        }
        let mut updated = BTreeMap::new();
        for (group_key, members) in groups {
            let prior = book.get(&group_key).cloned().unwrap_or_default();
            let mut assigned = Vec::with_capacity(members.len());
            for (index, number) in members.into_iter().enumerate() {
                let chosen = match prior.get(index) {
                    Some(previous) if !used.contains(previous) => *previous,
                    _ => {
                        let fresh = next;
                        next += 1;
                        fresh
                    }
                };
                used.insert(chosen);
                mapping.insert((space, number), chosen);
                assigned.push(chosen);
            }
            updated.insert(group_key, assigned);
        }
        *book = updated;
    }
    // Rewrite: program-wide numbers through the mapping, local ones per item.
    let mut out = String::with_capacity(application.len());
    let mut last = 0;
    for item in &items {
        let text = &application[item.start..item.end];
        let mut locals: HashMap<(&str, bool, u64), u64> = HashMap::new();
        let mut counters: HashMap<(&str, bool), u64> = HashMap::new();
        let mut edits = Vec::new();
        scan(text, |start, end, token| {
            let replacement = match token {
                Token::Numbered(space, prefix, number) => format!(
                    "{prefix}{}",
                    mapping.get(&(space, number)).copied().unwrap_or(number)
                ),
                Token::Local(prefix, number, label) => {
                    let local = *locals.entry((prefix, label, number)).or_insert_with(|| {
                        let counter = counters.entry((prefix, label)).or_default();
                        let value = *counter;
                        *counter += 1;
                        value
                    });
                    format!("{prefix}{local}")
                }
            };
            edits.push((start, end, replacement));
        });
        out.push_str(&application[last..item.start]);
        let mut cursor = 0;
        for (start, end, replacement) in edits {
            out.push_str(&text[cursor..start]);
            out.push_str(&replacement);
            cursor = end;
        }
        out.push_str(&text[cursor..]);
        last = item.end;
    }
    out.push_str(&application[last..]);
    let mut origins = Origins::default();
    for ((space, number), stable) in &mapping {
        match SPACES[*space].0 {
            "function" => {
                origins.functions.insert(*stable, *number);
            }
            "closure" => {
                origins.closures.insert(*stable, *number);
            }
            _ => {}
        }
    }
    (stabilize_family_variants(&out), origins)
}

/// Family enums name variants with a program-wide counter (`V1446(Closure1147)`),
/// so one new closure anywhere renames variants of unrelated families and every
/// function matching on them. Rename each family's variants from their payload
/// (`V_1147` for `Closure1147`, after closure numbers are stable), rewriting the
/// definition, `StoredCallableN::Vk` paths, and `Self::Vk` in the family's impls.
fn stabilize_family_variants(text: &str) -> String {
    let Some(items) = super::codegen_units::top_level_items(text) else {
        return text.to_owned();
    };
    let family_of = |item: &super::codegen_units::Item| -> Option<u64> {
        let head = text[item.head..item.end]
            .split('\n')
            .next()
            .unwrap_or_default();
        let at = head.find("StoredCallable")?;
        let before = &head[..at];
        if !(before.trim_end().ends_with("enum") || before.contains("impl")) {
            return None;
        }
        head[at + "StoredCallable".len()..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count()
            .checked_sub(0)
            .and_then(|digits| head[at + "StoredCallable".len()..][..digits].parse().ok())
    };
    // Variant renames per family, from each enum definition.
    let mut renames: HashMap<u64, HashMap<u64, String>> = HashMap::new();
    for item in &items {
        let head = text[item.head..item.end]
            .split('\n')
            .next()
            .unwrap_or_default();
        if !head.trim_start().starts_with("enum StoredCallable")
            && !head.contains(" enum StoredCallable")
        {
            continue;
        }
        let Some(family) = family_of(item) else {
            continue;
        };
        let body = &text[item.head..item.end];
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut map = HashMap::new();
        let bytes = body.as_bytes();
        let mut index = 0;
        while let Some(found) = body[index..].find('V') {
            let at = index + found;
            index = at + 1;
            if at > 0 && identifier_byte(bytes[at - 1]) {
                continue;
            }
            let digits = body[at + 1..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            if digits == 0 || bytes.get(at + 1 + digits) != Some(&b'(') {
                continue;
            }
            let Ok(number) = body[at + 1..at + 1 + digits].parse::<u64>() else {
                continue;
            };
            let payload_start = at + 2 + digits;
            let payload = body[payload_start..]
                .split(['(', ')', ',', '<'])
                .next()
                .unwrap_or_default()
                .trim();
            let base = match payload.strip_prefix("Closure") {
                Some(rest)
                    if !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()) =>
                {
                    format!("V_{rest}")
                }
                _ => format!("V_{}", digest(payload)),
            };
            let count = seen.entry(base.clone()).or_default();
            let name = if *count == 0 {
                base
            } else {
                format!("{base}_{count}")
            };
            *count += 1;
            map.entry(number).or_insert(name);
        }
        if !map.is_empty() {
            renames.insert(family, map);
        }
    }
    if renames.is_empty() {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for item in &items {
        let body = &text[item.start..item.end];
        let own = family_of(item).and_then(|family| renames.get(&family));
        let bytes = body.as_bytes();
        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        let mut index = 0;
        while let Some(found) = body[index..].find('V') {
            let at = index + found;
            index = at + 1;
            if at > 0 && identifier_byte(bytes[at - 1]) {
                continue;
            }
            let digits = body[at + 1..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            if digits == 0
                || bytes
                    .get(at + 1 + digits)
                    .is_some_and(|byte| identifier_byte(*byte))
            {
                continue;
            }
            let Ok(number) = body[at + 1..at + 1 + digits].parse::<u64>() else {
                continue;
            };
            // Which family does this variant belong to?
            let prefix = &body[..at];
            let family = if let Some(path) = prefix.strip_suffix("::") {
                let digits_before = path.bytes().rev().take_while(u8::is_ascii_digit).count();
                if path[..path.len() - digits_before].ends_with("StoredCallable")
                    && digits_before > 0
                {
                    path[path.len() - digits_before..]
                        .parse::<u64>()
                        .ok()
                        .and_then(|family| renames.get(&family))
                } else if path.ends_with("Self") {
                    own
                } else {
                    None
                }
            } else if own.is_some() && text[item.head..item.end].trim_start().starts_with("enum")
                || own.is_some() && body.contains(" enum StoredCallable")
            {
                own
            } else {
                None
            };
            if let Some(name) = family.and_then(|map| map.get(&number)) {
                edits.push((at, at + 1 + digits, name.clone()));
            }
        }
        out.push_str(&text[last..item.start]);
        let mut cursor = 0;
        for (start, end, name) in edits {
            out.push_str(&body[cursor..start]);
            out.push_str(&name);
            cursor = end;
        }
        out.push_str(&body[cursor..]);
        last = item.end;
    }
    out.push_str(&text[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(functions: &[&str], records: &[&str]) -> Keys {
        Keys {
            functions: functions.iter().map(|key| (*key).into()).collect(),
            records: records.iter().map(|key| (*key).into()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn existing_items_keep_numbers_when_new_items_are_inserted_before_them() {
        let first = "struct Record0 { field0: i64 }\nfn function0(value0: Record0) -> i64 { value0.field0 }\nfn main() { function0(Record0 { field0: 1 }); }\n";
        let mut ledger = Ledger::default();
        let before = stabilize(first, &keys(&["f:a"], &["r:point"]), &mut ledger);
        // A new function and record are created first, shifting compiler numbers.
        let second = "struct Record0 { field0: bool }\nstruct Record1 { field0: i64 }\nfn function0() -> bool { true }\nfn function1(value0: Record1) -> i64 { value0.field0 }\nfn main() { function1(Record1 { field0: 1 }); }\n";
        let after = stabilize(
            second,
            &keys(&["f:new", "f:a"], &["r:flag", "r:point"]),
            &mut ledger,
        );
        assert!(
            before.contains("struct Record0 { field0: i64 }")
                && before.contains("fn function0(value0: Record0)")
        );
        assert!(after.contains("struct Record0 { field0: i64 }"), "{after}");
        assert!(
            after.contains("fn function0(value0: Record0) -> i64"),
            "{after}"
        );
        assert!(
            after.contains("struct Record1 { field0: bool }")
                && after.contains("fn function1() -> bool"),
            "{after}"
        );
    }

    #[test]
    fn renaming_skips_strings_comments_and_hex_and_renumbers_locals_per_item() {
        let text = "fn function5() { let conditional_temporary_40 = 0xff1; 'exception_region9: { break 'exception_region9 \"function5 Record3\"; } // function5\n let c = 'a'; }\nfn function6() { let conditional_temporary_41 = 1; }\n";
        let mut ledger = Ledger::default();
        let out = stabilize(
            text,
            &keys(&["", "", "", "", "", "f5", "f6"], &[]),
            &mut ledger,
        );
        assert!(
            out.contains("fn function0()") && out.contains("fn function1()"),
            "{out}"
        );
        assert!(
            out.contains("let conditional_temporary_0 = 0xff1;"),
            "{out}"
        );
        assert!(
            out.contains("'exception_region0: { break 'exception_region0 \"function5 Record3\""),
            "{out}"
        );
        assert!(
            out.contains("// function5") && out.contains("let c = 'a';"),
            "{out}"
        );
        assert!(
            out.contains("fn function1() { let conditional_temporary_0 = 1; }"),
            "{out}"
        );
    }

    #[test]
    fn family_variants_are_named_from_their_payloads() {
        let text = "enum StoredCallable3 {\n    V40(Closure7),\n    V41(Closure9<'a>),\n}\nimpl StoredCallable3 { fn call(self) { match self { Self::V40(e) => e.call(), Self::V41(e) => e.call() } } }\nfn function0(x: StoredCallable3) { if let StoredCallable3::V41(_) = x {} let y = Union1::V41(1); }\n";
        let out = stabilize_family_variants(text);
        assert!(
            out.contains("V_7(Closure7),") && out.contains("V_9(Closure9<'a>),"),
            "{out}"
        );
        assert!(
            out.contains("Self::V_7(e)")
                && out.contains("Self::V_9(e)")
                && out.contains("StoredCallable3::V_9(_)"),
            "{out}"
        );
        assert!(
            out.contains("Union1::V41(1)"),
            "other enums keep their variants: {out}"
        );
    }

    #[test]
    fn normalization_uses_module_paths_and_drops_creation_counters() {
        let modules = vec!["a.tsr".to_owned(), "b.tsr".to_owned()];
        assert_eq!(
            normalize("__tsrust_module_1_computed<specialization#7>", &modules),
            "b.tsr::computed<specialization#>"
        );
        assert_eq!(
            normalize("__tsrust_module_0_@data-record:0:3", &modules),
            "a.tsr::@data-record:0:3"
        );
    }
}
