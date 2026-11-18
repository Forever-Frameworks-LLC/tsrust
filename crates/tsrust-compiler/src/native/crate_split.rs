//! One Rust crate per source module for development builds.
//!
//! A generated program is one crate today, so every edit re-checks and
//! re-links the whole program. This pass splits the (stably named) program
//! into crates that rustc builds separately:
//!
//! - `tsrust_types`: every item used from more than one crate (records,
//!   unions, families, shared closures, runtime glue) and functions the
//!   compiler made without a source module.
//! - one crate per source module: its generated functions, the closures and
//!   types only it uses, and the bodies of its synchronous closures whose
//!   structs had to move to `tsrust_types`.
//! - the program crate: `main`, and a compile-time check of every link-time
//!   declaration.
//!
//! Module crates depend only on `tsrust_types`, so they build in parallel.
//! A call from one module crate to a function of another goes through an
//! `unsafe extern "Rust"` declaration that the linker resolves to the
//! function's `#[unsafe(export_name)]` definition: the same direct call as in
//! one crate. A closure body that lives in a module crate while its struct is
//! in `tsrust_types` is the closure's original method, implemented through a
//! per-closure trait so the text is unchanged; `tsrust_types` calls it through
//! a link-time declaration. The program crate contains `[declared, defined]`
//! for every declaration, so a signature mismatch is a compile error, never
//! undefined behavior. Generated code is otherwise unchanged: no boxing, no
//! trait objects, no dynamic dispatch.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use super::codegen_units::{stable_hash, top_level_line_starts};

pub const TYPES_CRATE: &str = "tsrust_types";

/// One crate of a split program, in build order (types, modules, program last).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateSource {
    pub name: String,
    /// Crates this one is compiled against (`--extern name=lib<name>.rlib`).
    pub dependencies: Vec<String>,
    pub source: String,
    /// The program crate links every other crate into the executable.
    pub program: bool,
}

/// Where generated items come from: indices into `modules` by item number.
pub struct Provenance<'a> {
    pub modules: &'a [String],
    pub function_module: &'a dyn Fn(u64) -> Option<usize>,
    pub closure_module: &'a dyn Fn(u64) -> Option<usize>,
}

/// Crate name for a source module: stable across builds while its path is.
pub fn module_crate_name(module: &str) -> String {
    let tail = module
        .rsplit(['/', '\\', ':'])
        .next()
        .unwrap_or(module)
        .split('.')
        .next()
        .unwrap_or_default()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .take(24)
        .collect::<String>();
    format!("tsrust_m_{tail}_{:08x}", stable_hash(module) as u32)
}

const KEYWORDS: &[&str] = &[
    "fn", "struct", "enum", "union", "type", "trait", "mod", "const", "static", "impl", "use", "macro_rules!",
    "thread_local!",
];

fn identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Length of leading same-line attributes (`#[derive(Clone)] `).
fn attributes_length(line: &str) -> usize {
    let bytes = line.as_bytes();
    let mut index = 0;
    while bytes[index..].starts_with(b"#[") {
        let mut depth = 0;
        let mut end = None;
        for (offset, byte) in bytes[index..].iter().enumerate() {
            match byte {
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(index + offset + 1);
                        break;
                    }
                }
                b'\n' => break,
                _ => {}
            }
        }
        match end {
            Some(end) if bytes.get(end) == Some(&b' ') => index = end + 1,
            _ => break,
        }
    }
    index
}

/// The item keyword a line starts with (after attributes, visibility and qualifiers),
/// and the byte offset just after the keyword.
fn keyword(line: &str) -> Option<(&'static str, usize)> {
    let mut index = attributes_length(line);
    let rest = &line[index..];
    for visibility in ["pub(crate) ", "pub(super) ", "pub(self) ", "pub "] {
        if rest.starts_with(visibility) {
            index += visibility.len();
            break;
        }
    }
    loop {
        let rest = &line[index..];
        if let Some(stripped) = ["async ", "unsafe ", "safe "].iter().find(|q| rest.starts_with(**q)) {
            index += stripped.len();
        } else if rest.starts_with("extern \"") {
            let close = rest[8..].find('"')?;
            index += 8 + close + 1;
            if line[index..].starts_with(' ') {
                index += 1;
            }
        } else {
            break;
        }
    }
    let rest = &line[index..];
    if rest.starts_with("std::thread_local!") {
        return Some(("thread_local!", index + "std::thread_local!".len()));
    }
    KEYWORDS.iter().find_map(|keyword| {
        let after = rest.strip_prefix(keyword)?;
        let boundary = keyword.ends_with('!') || after.bytes().next().is_none_or(|byte| !identifier_byte(byte));
        boundary.then_some((*keyword, index + keyword.len()))
    })
}

/// Visit bytes outside comments and string, raw string and character literals.
/// The visitor receives each code byte index; literals and comments are skipped whole.
fn code_bytes(text: &str, mut visit: impl FnMut(usize)) {
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
        } else if (c == b'r' || c == b'b' || c == b'c')
            && (i == 0 || !identifier_byte(bytes[i - 1]))
            && raw_string_end(text, i).is_some()
        {
            i = raw_string_end(text, i).unwrap_or(n);
        } else if c == b'\'' {
            // Character literal, or a lifetime/label (visited as code).
            let rest = &text[i + 1..];
            let mut chars = rest.char_indices();
            match chars.next() {
                Some((_, '\\')) => i = rest[2..].find('\'').map_or(n, |end| i + 1 + 2 + end + 1),
                Some((_, first)) if matches!(chars.next(), Some((_, '\''))) => i += 1 + first.len_utf8() + 1,
                _ => {
                    visit(i);
                    i += 1;
                }
            }
        } else {
            visit(i);
            i += 1;
        }
    }
}

/// End of a raw string (`r"…"`, `r#"…"#`, `br"…"`) or byte string starting at `start`.
fn raw_string_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = start;
    if matches!(bytes[i], b'b' | b'c') {
        i += 1;
        if bytes.get(i) == Some(&b'"') {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'"' {
                j += if bytes[j] == b'\\' { 2 } else { 1 };
            }
            return Some(j + 1);
        }
    }
    if bytes.get(i) != Some(&b'r') {
        return None;
    }
    i += 1;
    let hashes = bytes[i..].iter().take_while(|byte| **byte == b'#').count();
    if bytes.get(i + hashes) != Some(&b'"') {
        return None;
    }
    let close = format!("\"{}", "#".repeat(hashes));
    let body = i + hashes + 1;
    Some(text[body..].find(&close).map_or(text.len(), |end| body + end + close.len()))
}

/// Identifiers in code (not in comments or literals).
fn identifiers(text: &str) -> HashSet<&str> {
    let bytes = text.as_bytes();
    let mut out = HashSet::new();
    let mut skip_until = 0;
    code_bytes(text, |i| {
        if i < skip_until {
            return;
        }
        let c = bytes[i];
        if (c.is_ascii_alphabetic() || c == b'_') && (i == 0 || !identifier_byte(bytes[i - 1])) {
            let end = i + bytes[i..].iter().take_while(|byte| identifier_byte(**byte)).count();
            out.insert(&text[i..end]);
            skip_until = end;
        }
    });
    out
}

/// Offsets just after a `}` that closes back to depth zero.
fn closes_at_depth_zero(text: &str) -> Vec<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i64;
    let mut out = Vec::new();
    code_bytes(text, |i| match bytes[i] {
        b'{' | b'(' | b'[' => depth += 1,
        b'}' | b')' | b']' => {
            depth -= 1;
            if depth == 0 && bytes[i] == b'}' {
                out.push(i + 1);
            }
        }
        _ => {}
    });
    out
}

/// Top-level items with leading attribute lines; inner attributes and comments stand alone.
fn split_items(text: &str) -> Option<Vec<String>> {
    let mut starts = top_level_line_starts(text)?;
    starts.push(text.len());
    let mut items: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;
    for window in starts.windows(2) {
        let line = &text[window[0]..window[1]];
        let stripped = line.trim();
        if stripped.starts_with("#[") && stripped.ends_with(']') && keyword(line).is_none() {
            pending.get_or_insert_with(String::new).push_str(line);
            continue;
        }
        let starts_item = keyword(line).is_some()
            || items.is_empty()
            || stripped.starts_with("#!")
            || (stripped.starts_with("//") && pending.is_none());
        let mut piece = pending.take().unwrap_or_default();
        piece.push_str(line);
        if starts_item {
            items.push(piece);
        } else if let Some(last) = items.last_mut() {
            last.push_str(&piece);
        }
    }
    if let Some(pending) = pending {
        items.push(pending);
    }
    // `…}fn next(…)`: another item starts on the line a previous one closes.
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let mut start = 0;
        for offset in closes_at_depth_zero(&item) {
            let rest = &item[offset..];
            let stripped = rest.trim_start_matches(' ');
            if stripped.is_empty() || stripped.starts_with('\n') || keyword(stripped).is_none() {
                continue;
            }
            let mut piece = item[start..offset].to_owned();
            piece.push('\n');
            out.push(piece);
            start = offset + (rest.len() - stripped.len());
        }
        out.push(item[start..].to_owned());
    }
    Some(out)
}

fn head_line(item: &str) -> &str {
    item.split('\n').find(|line| keyword(line).is_some()).unwrap_or_default()
}

/// Skip a leading `<…>` list, returning (inner text, rest).
/// Whether a generic parameter list declares a type (not only lifetimes).
fn type_parameters(generics: &str) -> bool {
    let mut depth = 0usize;
    let mut start = true;
    for character in generics.chars() {
        match character {
            '<' | '(' => depth += 1,
            '>' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => start = true,
            ' ' => {}
            _ if start => {
                if character != '\'' {
                    return true;
                }
                start = false;
            }
            _ => {}
        }
    }
    false
}

fn generic_list(text: &str) -> (&str, &str) {
    if !text.starts_with('<') {
        return ("", text);
    }
    let mut depth = 0;
    for (index, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' if index == 0 || &text[index - 1..index] != "-" => {
                depth -= 1;
                if depth == 0 {
                    return (&text[1..index], &text[index + 1..]);
                }
            }
            _ => {}
        }
    }
    ("", text)
}

fn first_identifier(text: &str) -> Option<&str> {
    let start = text.find(|c: char| c.is_ascii_alphabetic() || c == '_')?;
    let end = start + text[start..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(text.len() - start);
    Some(&text[start..end])
}

enum Kind {
    Prologue,
    Impl { target: Option<String>, trait_name: Option<String> },
    Named(String),
}

fn kind_name(item: &str) -> Kind {
    let line = head_line(item);
    let Some((keyword, after)) = keyword(line) else {
        return Kind::Prologue;
    };
    match keyword {
        "use" | "macro_rules!" => Kind::Prologue,
        "impl" => {
            let (_, rest) = generic_list(&line[after..]);
            let header = rest.split('{').next().unwrap_or(rest);
            let (trait_part, target) = match header.split_once(" for ") {
                Some((trait_part, target)) => (Some(trait_part), target),
                None => (None, header),
            };
            let mut cleaned = String::with_capacity(target.len());
            let mut chars = target.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\'' {
                    while chars.peek().is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_') {
                        chars.next();
                    }
                    cleaned.push(' ');
                } else if c == '&' {
                    cleaned.push(' ');
                } else {
                    cleaned.push(c);
                }
            }
            let cleaned = cleaned.replace(" mut ", " ").replace("dyn ", " ");
            let target = first_identifier(cleaned.trim_start().trim_start_matches("mut ")).map(str::to_owned);
            let trait_name = trait_part.and_then(|part| first_identifier(part.trim_start_matches("unsafe "))).map(str::to_owned);
            Kind::Impl { target, trait_name }
        }
        "thread_local!" => {
            // `thread_local! { static NAME: … }`: one static, owned by one crate.
            let at = item.find("static ").map(|at| at + 7);
            match at.and_then(|at| first_identifier(&item[at..])) {
                Some(name) => Kind::Named(name.to_owned()),
                None => Kind::Prologue,
            }
        }
        _ => match first_identifier(&line[after..]) {
            Some(name) => Kind::Named(name.to_owned()),
            None => Kind::Prologue,
        },
    }
}

/// Split at top-level commas (outside `()`, `<>`, `[]`).
fn split_top_commas(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'-' if bytes.get(i + 1) == Some(&b'>') => {
                i += 2;
                continue;
            }
            b'(' | b'<' | b'[' => depth += 1,
            b')' | b'>' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push(text[start..i].trim().to_owned());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        parts.push(tail.to_owned());
    }
    parts
}

#[derive(Clone)]
struct Signature {
    generics: String,
    params: Vec<String>,
    ret: String,
}

/// `fn NAME<G>(PARAMS) -> RET {` within a head line.
fn parse_signature(line: &str, name: &str) -> Option<Signature> {
    let pattern = format!("fn {name}");
    let mut index = 0;
    let found = loop {
        let at = index + line[index..].find(&pattern)?;
        let end = at + pattern.len();
        if line.as_bytes().get(end).is_none_or(|byte| !identifier_byte(*byte)) {
            break end;
        }
        index = end;
    };
    let (generics, rest) = generic_list(&line[found..]);
    let rest = rest.strip_prefix('(')?;
    let mut depth = 1;
    let mut close = None;
    let bytes = rest.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        match byte {
            b'(' | b'[' | b'<' => depth += 1,
            b')' | b']' => depth -= 1,
            b'>' if i == 0 || bytes[i - 1] != b'-' => depth -= 1,
            _ => {}
        }
        if depth == 0 {
            close = Some(i);
            break;
        }
    }
    let close = close?;
    let params = split_top_commas(&rest[..close]);
    let after = rest[close + 1..].trim();
    let ret = if let Some(ret) = after.strip_prefix("->") {
        let brace = if ret.ends_with('{') { ret.rfind('{') } else { ret.find("{ ").or_else(|| ret.find('{')) }?;
        let ret = ret[..brace].trim();
        if ret.contains(" where ") || ret.starts_with("where ") {
            return None;
        }
        ret.to_owned()
    } else if after.starts_with('{') {
        String::new()
    } else {
        return None;
    };
    Some(Signature { generics: generics.to_owned(), params, ret })
}

fn declaration_param(param: &str) -> &str {
    let param = param.trim();
    param.strip_prefix("mut ").unwrap_or(param)
}

fn param_name(param: &str) -> &str {
    declaration_param(param).split(':').next().unwrap_or_default().trim()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum Home {
    Types,
    Program,
    Module(usize),
}

struct Group {
    name: String,
    texts: Vec<String>,
    movable: bool,
    home: Option<Home>,
    refs: BTreeSet<usize>,
    signature: Option<Signature>,
    /// For a closure body split from its closure: the closure's group and the impl head.
    body_of: Option<(usize, String)>,
}

fn numbered(name: &str, prefixes: &[&str]) -> Option<u64> {
    prefixes.iter().find_map(|prefix| {
        let digits = name.strip_prefix(prefix)?;
        (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then(|| digits.parse().ok())?
    })
}

/// Split `application` (one generated program, not partitioned) into crates.
/// Returns `None` if the program cannot be split (unbalanced text or a function
/// called across crates whose signature cannot be declared).
pub fn split(application: &str, provenance: &Provenance<'_>) -> Option<Vec<CrateSource>> {
    let items = split_items(application)?;
    let mut prologue: Vec<String> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();
    let mut impls: Vec<(String, Option<String>, Option<String>)> = Vec::new();
    let new_group = |name: &str, groups: &mut Vec<Group>, index_of: &mut HashMap<String, usize>| -> usize {
        *index_of.entry(name.to_owned()).or_insert_with(|| {
            groups.push(Group {
                name: name.to_owned(),
                texts: Vec::new(),
                movable: false,
                home: None,
                refs: BTreeSet::new(),
                signature: None,
                body_of: None,
            });
            groups.len() - 1
        })
    };
    for item in items {
        match kind_name(&item) {
            Kind::Prologue => prologue.push(item),
            Kind::Impl { target, trait_name } => impls.push((item, target, trait_name)),
            Kind::Named(name) => {
                let index = new_group(&name, &mut groups, &mut index_of);
                groups[index].texts.push(item);
            }
        }
    }
    for (item, target, trait_name) in impls {
        let owner = target
            .as_ref()
            .and_then(|target| index_of.get(target))
            .or_else(|| trait_name.as_ref().and_then(|name| index_of.get(name)))
            .copied();
        match owner {
            Some(owner) => groups[owner].texts.push(item),
            None => prologue.push(item),
        }
    }
    let module_home = |module: Option<usize>| module.map_or(Home::Types, Home::Module);
    // Synchronous closure bodies may live apart from their struct.
    let closure_count = groups.len();
    for closure in 0..closure_count {
        let Some(number) = numbered(&groups[closure].name, &["Closure"]) else {
            continue;
        };
        let found = groups[closure].texts.iter().enumerate().find_map(|(position, item)| {
            let head = head_line(item);
            if !head.starts_with("impl") || head.split('{').next().unwrap_or_default().contains(" for ") {
                return None;
            }
            let method = item.split('\n').nth(1)?.trim();
            if !method.starts_with("fn call") {
                return None;
            }
            let signature = parse_signature(method, "call")?;
            if signature.ret.contains('&') || signature.ret.contains("'_") || signature.ret.contains("impl ") {
                return None;
            }
            // The body would be declared extern, which cannot take type parameters.
            let (impl_generics, _) = generic_list(head.strip_prefix("impl")?);
            if type_parameters(impl_generics) || type_parameters(&signature.generics) {
                return None;
            }
            Some((position, signature, head.to_owned()))
        });
        let Some((position, signature, head)) = found else {
            continue;
        };
        let text = groups[closure].texts.remove(position);
        let name = format!("{}#body", groups[closure].name);
        let body = new_group(&name, &mut groups, &mut index_of);
        groups[body].texts.push(text);
        groups[body].movable = true;
        groups[body].home = Some(module_home((provenance.closure_module)(number)));
        groups[body].signature = Some(signature);
        groups[body].body_of = Some((closure, head));
    }
    for group in &mut groups {
        if group.body_of.is_some() {
            continue;
        }
        if group.name == "main" {
            group.home = Some(Home::Program);
        } else if let Some(number) = numbered(&group.name, &["tsr_result_function", "function"]) {
            group.movable = true;
            group.home = Some(module_home((provenance.function_module)(number)));
        } else if let Some(number) = numbered(&group.name, &["Closure", "TsrClosureError"]) {
            group.home = Some(module_home((provenance.closure_module)(number)));
        } else if let Some(number) = numbered(&group.name, &["TsrFunctionError"]) {
            group.home = Some(module_home((provenance.function_module)(number)));
        }
    }
    for index in 0..groups.len() {
        let mut refs = BTreeSet::new();
        for text in &groups[index].texts {
            for word in identifiers(text) {
                if let Some(&target) = index_of.get(word)
                    && target != index
                {
                    refs.insert(target);
                }
            }
        }
        if let Some((closure, _)) = groups[index].body_of {
            refs.insert(closure);
        }
        groups[index].refs = refs;
    }
    let mut signature_refs: HashMap<usize, BTreeSet<usize>> = HashMap::new();
    for index in 0..groups.len() {
        if !groups[index].movable || groups[index].body_of.is_some() {
            continue;
        }
        let signature = parse_signature(head_line(&groups[index].texts[0]), &groups[index].name);
        // An extern declaration cannot be generic (callback parameters are): such a
        // function is placed like a type, beside its users or in the types crate.
        if signature.as_ref().is_some_and(|signature| type_parameters(&signature.generics)) {
            groups[index].movable = false;
            continue;
        }
        let text = signature.as_ref().map_or(String::new(), |s| format!("{} -> {}", s.params.join(","), s.ret));
        signature_refs.insert(
            index,
            identifiers(&text).into_iter().filter_map(|word| index_of.get(word).copied()).collect(),
        );
        groups[index].signature = signature;
    }
    // A closure body kept apart from its struct is declared in the struct's crate.
    let mut body_of_closure: HashMap<usize, usize> = HashMap::new();
    for index in 0..groups.len() {
        let Some((closure, _)) = groups[index].body_of else {
            continue;
        };
        body_of_closure.insert(closure, index);
        let text = groups[index].signature.as_ref().map_or(String::new(), |s| format!("{} -> {}", s.params.join(","), s.ret));
        signature_refs.insert(
            index,
            identifiers(&text).into_iter().filter_map(|word| index_of.get(word).copied()).collect(),
        );
    }
    let mut users: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); groups.len()];
    for (index, group) in groups.iter().enumerate() {
        for &target in &group.refs {
            users[target].insert(index);
        }
    }
    // Items used by exactly one crate's code stay there; others go to the types crate.
    for index in 0..groups.len() {
        if groups[index].home.is_some() {
            continue;
        }
        let homes = users[index]
            .iter()
            .filter(|user| groups[**user].movable || groups[**user].home == Some(Home::Program))
            .filter_map(|user| groups[*user].home)
            .collect::<BTreeSet<_>>();
        groups[index].home = Some(if homes.len() == 1 { *homes.iter().next()? } else { Home::Types });
    }
    let home = |groups: &[Group], index: usize| groups[index].home.unwrap_or(Home::Types);
    // A crate sees only itself and the types crate: anything used elsewhere moves up.
    loop {
        let mut changed = false;
        for index in 0..groups.len() {
            let own = home(&groups, index);
            let mut needs = groups[index].refs.clone();
            for &target in &groups[index].refs {
                if groups[target].movable && home(&groups, target) != own
                    && let Some(signature) = signature_refs.get(&target)
                {
                    needs.extend(signature.iter().copied());
                }
            }
            if let Some(&body) = body_of_closure.get(&index)
                && home(&groups, body) != own
                && let Some(signature) = signature_refs.get(&body)
            {
                needs.extend(signature.iter().copied());
            }
            for target in needs {
                let other = home(&groups, target);
                if !groups[target].movable && other != own && other != Home::Types {
                    groups[target].home = Some(Home::Types);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    // Every cross-crate call needs a declarable signature.
    let mut declarations: BTreeMap<Home, BTreeSet<usize>> = BTreeMap::new();
    for index in 0..groups.len() {
        let own = home(&groups, index);
        for &target in &groups[index].refs {
            let other = home(&groups, target);
            if groups[target].movable && groups[target].body_of.is_none() && other != own && other != Home::Types {
                groups[target].signature.as_ref()?;
                declarations.entry(own).or_default().insert(target);
            }
        }
    }
    let exported: HashSet<usize> = declarations.values().flatten().copied().collect();
    // Closure bodies whose struct stays in the same crate are emitted unchanged.
    let mut detours: BTreeMap<usize, usize> = BTreeMap::new(); // closure -> body group
    for index in 0..groups.len() {
        if let Some((closure, _)) = groups[index].body_of.clone() {
            if home(&groups, closure) == home(&groups, index) {
                let text = std::mem::take(&mut groups[index].texts);
                groups[closure].texts.extend(text);
                groups[index].home = None;
            } else {
                detours.insert(closure, index);
            }
        }
    }

    let crate_name = |home: Home| match home {
        Home::Types => TYPES_CRATE.to_owned(),
        Home::Program => "program".to_owned(),
        Home::Module(module) => module_crate_name(&provenance.modules[module]),
    };
    let mut members: BTreeMap<Home, Vec<usize>> = BTreeMap::new();
    for (index, group) in groups.iter().enumerate() {
        if let Some(home) = group.home {
            members.entry(home).or_default().push(index);
        }
    }
    members.entry(Home::Types).or_default();
    members.entry(Home::Program).or_default();
    let header = prologue
        .iter()
        .filter(|item| {
            let item = item.trim_start();
            item.starts_with("#!") || item.starts_with("//")
        })
        .cloned()
        .collect::<String>();
    let shared = prologue
        .iter()
        .filter(|item| {
            let item = item.trim_start();
            !(item.starts_with("#!") || item.starts_with("//"))
        })
        .cloned()
        .collect::<String>();
    let names: Vec<(Home, String)> = members
        .keys()
        .copied()
        .filter(|home| *home != Home::Program)
        .map(|home| (home, crate_name(home)))
        .collect();
    let mut module_order = names.clone();
    module_order.sort_by(|a, b| (a.0 != Home::Types, &a.1).cmp(&(b.0 != Home::Types, &b.1)));
    let mut checks: Vec<(Home, String, Home)> = Vec::new();
    let mut crates = Vec::new();
    for (own, name) in module_order.iter().cloned().chain(std::iter::once((Home::Program, crate_name(Home::Program)))) {
        let mut source = String::with_capacity(4096);
        source.push_str(&header);
        if own != Home::Types {
            source.push_str(&format!("#[allow(unused_imports)] use {TYPES_CRATE}::*;\n"));
        }
        source.push_str(&shared);
        if own == Home::Program {
            for (_, other) in &module_order {
                source.push_str(&format!("extern crate {other} as _;\n"));
            }
        }
        let declared = declarations.get(&own).cloned().unwrap_or_default();
        let shims: Vec<(usize, usize)> = detours
            .iter()
            .filter(|(closure, _)| home(&groups, **closure) == own)
            .map(|(closure, body)| (*closure, *body))
            .collect();
        if !declared.is_empty() || !shims.is_empty() {
            source.push_str("pub mod tsrust_extern {\n#[allow(unused_imports)] use super::*;\nunsafe extern \"Rust\" {\n");
            for &target in &declared {
                let group = &groups[target];
                let signature = group.signature.as_ref()?;
                let generic = if signature.generics.is_empty() { String::new() } else { format!("<{}>", signature.generics) };
                let params = signature.params.iter().map(|p| declaration_param(p)).collect::<Vec<_>>().join(", ");
                let arrow = if signature.ret.is_empty() { String::new() } else { format!(" -> {}", signature.ret) };
                source.push_str(&format!(
                    "#[link_name = \"tsrust_{0}\"] pub safe fn {0}{generic}({params}){arrow};\n",
                    group.name
                ));
                checks.push((own, group.name.clone(), home(&groups, target)));
            }
            for &(closure, body) in &shims {
                source.push_str(&detour_declaration(&groups[closure].name, &groups[body])?);
                source.push('\n');
                checks.push((own, format!("closure_body_{}", groups[closure].name), home(&groups, body)));
            }
            source.push_str("}\n}\n#[allow(unused_imports)] use tsrust_extern::*;\n");
        }
        let mut list = members.get(&own).cloned().unwrap_or_default();
        list.sort_by(|a, b| groups[*a].name.cmp(&groups[*b].name));
        for index in list {
            let group = &groups[index];
            let detoured_body = group.body_of.as_ref().is_some_and(|(closure, _)| detours.contains_key(closure));
            if !detoured_body {
                for text in &group.texts {
                    let mut text = publish(text);
                    if group.movable && exported.contains(&index) {
                        text = format!("#[unsafe(export_name = \"tsrust_{}\")]\n{text}", group.name);
                    }
                    source.push_str(&rewrite_paths(&text, own, &groups, &index_of));
                }
            }
            if let Some(&body) = detours.get(&index) {
                source.push_str(&detour_shim(&group.name, &groups[body])?);
            }
            if detoured_body {
                let (closure, _) = group.body_of.as_ref()?;
                source.push_str(&rewrite_paths(&detour_body(&groups[*closure].name, group)?, own, &groups, &index_of));
            }
        }
        if own == Home::Program {
            source.push_str("#[allow(dead_code)] fn tsrust_link_checks() {\n");
            for (declarer, symbol, definer) in &checks {
                let declared = if *declarer == Home::Program {
                    "crate::tsrust_extern".to_owned()
                } else {
                    format!("{}::tsrust_extern", crate_name(*declarer))
                };
                source.push_str(&format!("    let _ = [{declared}::{symbol}, {}::{symbol}];\n", crate_name(*definer)));
            }
            source.push_str("}\n");
        }
        let dependencies = match own {
            Home::Types => Vec::new(),
            Home::Program => module_order.iter().map(|(_, name)| name.clone()).collect(),
            Home::Module(_) => vec![TYPES_CRATE.to_owned()],
        };
        crates.push(CrateSource { name, dependencies, source, program: own == Home::Program });
    }
    Some(crates)
}

/// Rewrite `crate::NAME` for items that now live in another crate.
fn rewrite_paths(
    text: &str,
    own: Home,
    groups: &[Group],
    index_of: &HashMap<String, usize>,
) -> String {
    if !text.contains("crate::") {
        return text.to_owned();
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut edits = Vec::new();
    code_bytes(text, |i| {
        if bytes[i..].starts_with(b"crate::") && (i == 0 || !identifier_byte(bytes[i - 1])) {
            let start = i + 7;
            let end = start + bytes[start..].iter().take_while(|byte| identifier_byte(**byte)).count();
            edits.push((i, start, end));
        }
    });
    for (at, start, end) in edits {
        let name = &text[start..end];
        let Some(&target) = index_of.get(name) else {
            continue;
        };
        let Some(home) = groups[target].home else {
            continue;
        };
        if home == own {
            continue;
        }
        let replacement = if home == Home::Types {
            format!("{TYPES_CRATE}::")
        } else if groups[target].movable {
            "crate::tsrust_extern::".to_owned()
        } else {
            continue;
        };
        out.push_str(&text[last..at]);
        out.push_str(&replacement);
        last = start;
    }
    out.push_str(&text[last..]);
    out
}

fn widen_restricted(text: &str) -> String {
    text.replace("pub(crate) ", "pub ").replace("pub(super) ", "pub ")
}

/// Make a top-level item public, with its struct fields and inherent methods.
fn publish(item: &str) -> String {
    let mut offset = 0;
    for line in item.split_inclusive('\n') {
        if let Some((kind, _)) = keyword(line) {
            let attributes = attributes_length(line);
            let prefix = &item[..offset + attributes];
            let mut body = item[offset + attributes..].to_owned();
            for restricted in ["pub(crate) ", "pub(super) "] {
                if body.starts_with(restricted) {
                    body = format!("pub {}", &body[restricted.len()..]);
                }
            }
            if kind == "thread_local!" {
                return format!("{prefix}{}", publish_thread_local(&body));
            }
            if !(matches!(kind, "impl" | "use" | "macro_rules!") || body.starts_with("pub ") || body.starts_with("fn main(")) {
                body = format!("pub {body}");
            }
            let header = body.split('{').next().unwrap_or_default();
            let body = match kind {
                "struct" => publish_fields(&body),
                "impl" if !header.contains(" for ") => publish_methods(&body),
                "mod" => widen_restricted(&body),
                _ => body,
            };
            return format!("{prefix}{body}");
        }
        offset += line.len();
    }
    item.to_owned()
}

/// `static NAME` declarations inside `thread_local!` become `pub static NAME`.
fn publish_thread_local(body: &str) -> String {
    let mut out = String::with_capacity(body.len() + 8);
    let mut last = 0;
    for (at, _) in body.match_indices("static ") {
        let before = &body[..at];
        if before.ends_with('\'') || before.ends_with("pub ") || before.bytes().last().is_some_and(identifier_byte) {
            continue;
        }
        out.push_str(&body[last..at]);
        out.push_str("pub ");
        last = at;
    }
    out.push_str(&body[last..]);
    out
}

fn publish_fields(body: &str) -> String {
    // First `{`, `(` or `;` outside the generic parameter list.
    let bytes = body.as_bytes();
    let mut angle = 0;
    let mut opener = None;
    for (index, byte) in bytes.iter().enumerate() {
        match byte {
            b'<' => angle += 1,
            b'>' if index == 0 || bytes[index - 1] != b'-' => angle -= 1,
            b'{' | b'(' | b';' if angle == 0 => {
                opener = Some((index, *byte));
                break;
            }
            _ => {}
        }
    }
    match opener {
        Some((paren, b'(')) => {
            let mut depth = 0;
            let mut close = paren;
            for (index, byte) in bytes.iter().enumerate().skip(paren) {
                match byte {
                    b'(' | b'<' | b'[' => depth += 1,
                    b')' | b']' => depth -= 1,
                    b'>' if bytes[index - 1] != b'-' => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    close = index;
                    break;
                }
            }
            let fields = split_top_commas(&body[paren + 1..close])
                .into_iter()
                .map(|field| if field.starts_with("pub ") { field } else { format!("pub {field}") })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}{fields}{}", &body[..paren + 1], &body[close..])
        }
        Some((brace, b'{')) => {
            let mut depth = 0;
            for (index, byte) in bytes.iter().enumerate().skip(brace) {
                match byte {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return format!("{}{}", publish_braced(&body[..index + 1], brace), &body[index + 1..]);
                        }
                    }
                    _ => {}
                }
            }
            body.to_owned()
        }
        _ => body.to_owned(),
    }
}

fn publish_braced(body: &str, brace: usize) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len() + 64);
    out.push_str(&body[..brace]);
    let mut depth = 0;
    let mut field_start = true;
    let mut i = brace;
    while i < bytes.len() {
        let c = bytes[i];
        if depth == 1 && field_start && (c.is_ascii_alphabetic() || c == b'_') {
            if !body[i..].starts_with("pub") {
                out.push_str("pub ");
            }
            field_start = false;
        }
        match c {
            b'{' | b'(' | b'<' | b'[' => {
                depth += 1;
                if depth == 1 {
                    field_start = true;
                }
            }
            b'}' | b')' | b']' => depth -= 1,
            b'>' if bytes[i - 1] != b'-' => depth -= 1,
            b',' if depth == 1 => field_start = true,
            b'#' if depth == 1 && field_start => {
                let end = body[i..].find(']').map_or(bytes.len(), |end| i + end + 1);
                out.push_str(&body[i..end]);
                i = end;
                continue;
            }
            _ => {}
        }
        let width = body[i..].chars().next().map_or(1, char::len_utf8);
        out.push_str(&body[i..i + width]);
        i += width;
    }
    out
}

fn publish_methods(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut inserts = Vec::new();
    let mut depth = 0;
    code_bytes(body, |i| {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth != 1 || (i > 0 && identifier_byte(bytes[i - 1])) || !bytes[i].is_ascii_alphabetic() {
            return;
        }
        let rest = &body[i..];
        let mut probe = rest;
        while let Some(stripped) = ["const ", "unsafe ", "async "].iter().find_map(|q| probe.strip_prefix(q)) {
            probe = stripped;
        }
        let is_item = probe.starts_with("fn ")
            || probe.starts_with("type ")
            || (rest.starts_with("const ") && probe.as_bytes().first().is_some_and(u8::is_ascii_uppercase));
        if !is_item {
            return;
        }
        let before = body[..i].trim_end_matches(' ');
        let qualified = ["const", "unsafe", "async", "pub", "pub(crate)", "pub(super)"]
            .iter()
            .any(|word| before.ends_with(word) && before[..before.len() - word.len()].bytes().last().is_none_or(|byte| !identifier_byte(byte)));
        if !qualified {
            inserts.push(i);
        }
    });
    let mut out = String::with_capacity(body.len() + inserts.len() * 4);
    let mut last = 0;
    for at in inserts {
        out.push_str(&body[last..at]);
        out.push_str("pub ");
        last = at;
    }
    out.push_str(&body[last..]);
    widen_restricted(&out)
}

/// For a split closure body: impl generics, the struct type, and `this`'s type.
fn self_type<'a>(impl_head: &'a str, receiver: &str) -> Option<(&'a str, &'a str, String)> {
    let header = impl_head.split('{').next()?.trim();
    let (generics, rest) = generic_list(header.strip_prefix("impl")?);
    let target = rest.trim();
    let this = match receiver.trim() {
        "self" | "mut self" => target.to_owned(),
        "&self" => format!("&{target}"),
        "&mut self" => format!("&mut {target}"),
        _ => return None,
    };
    Some((generics, target, this))
}

fn detour_declaration(closure: &str, body: &Group) -> Option<String> {
    let signature = body.signature.as_ref()?;
    let (_, head) = body.body_of.as_ref()?;
    let (impl_generics, _, this) = self_type(head, signature.params.first()?)?;
    let generics = [impl_generics, signature.generics.as_str()].iter().filter(|g| !g.is_empty()).copied().collect::<Vec<_>>().join(", ");
    let generic = if generics.is_empty() { String::new() } else { format!("<{generics}>") };
    let rest = signature.params[1..].iter().map(|p| format!(", {}", declaration_param(p))).collect::<String>();
    let arrow = if signature.ret.is_empty() { String::new() } else { format!(" -> {}", signature.ret) };
    Some(format!("#[link_name = \"tsrust_closure_body_{closure}\"] pub safe fn closure_body_{closure}{generic}(this: {this}{rest}){arrow};"))
}

fn detour_shim(closure: &str, body: &Group) -> Option<String> {
    let signature = body.signature.as_ref()?;
    let (_, head) = body.body_of.as_ref()?;
    let receiver = signature.params.first()?.trim();
    let (impl_generics, target, _) = self_type(head, receiver)?;
    let generic = if impl_generics.is_empty() { String::new() } else { format!("<{impl_generics}>") };
    let method_generic = if signature.generics.is_empty() { String::new() } else { format!("<{}>", signature.generics) };
    let rest = signature.params[1..].iter().map(|p| format!(", {}", declaration_param(p))).collect::<String>();
    let args = signature.params[1..].iter().map(|p| format!(", {}", param_name(p))).collect::<String>();
    let arrow = if signature.ret.is_empty() { String::new() } else { format!(" -> {}", signature.ret) };
    Some(format!(
        "impl{generic} {target} {{ #[inline(always)] pub fn call{method_generic}({receiver}{rest}){arrow} {{ tsrust_extern::closure_body_{closure}(self{args}) }} }}\n"
    ))
}

fn detour_body(closure: &str, body: &Group) -> Option<String> {
    let signature = body.signature.as_ref()?;
    let (_, head) = body.body_of.as_ref()?;
    let receiver = signature.params.first()?.trim();
    let (impl_generics, target, this) = self_type(head, receiver)?;
    let trait_generic = if impl_generics.is_empty() { String::new() } else { format!("<{impl_generics}>") };
    let trait_args = trait_generic.clone();
    let method_generic = if signature.generics.is_empty() { String::new() } else { format!("<{}>", signature.generics) };
    let rest = signature.params[1..].iter().map(|p| format!(", {}", declaration_param(p))).collect::<String>();
    let args = signature.params[1..].iter().map(|p| format!(", {}", param_name(p))).collect::<String>();
    let arrow = if signature.ret.is_empty() { String::new() } else { format!(" -> {}", signature.ret) };
    let all = [impl_generics, signature.generics.as_str()].iter().filter(|g| !g.is_empty()).copied().collect::<Vec<_>>().join(", ");
    let generic = if all.is_empty() { String::new() } else { format!("<{all}>") };
    // The original method, renamed, implements a module-local trait.
    let text = body.texts.first()?;
    let mut lines = text.split('\n').map(str::to_owned).collect::<Vec<_>>();
    if lines.len() < 2 {
        return None;
    }
    lines[0] = format!("impl{trait_generic} TsrBody{closure}{trait_args} for {target} {{");
    lines[1] = lines[1].replacen("pub fn call", "fn tsrust_body", 1).replacen("fn call", "fn tsrust_body", 1);
    Some(format!(
        "pub trait TsrBody{closure}{trait_generic} {{ fn tsrust_body{method_generic}({receiver}{rest}){arrow}; }}\n{}\n#[unsafe(export_name = \"tsrust_closure_body_{closure}\")]\npub fn closure_body_{closure}{generic}(this: {this}{rest}){arrow} {{ TsrBody{closure}::tsrust_body(this{args}) }}\n",
        lines.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRAM: &str = "\
#![allow(dead_code)]
use std::fmt::Write;
#[derive(Clone)] struct Record1 { field0: String }
struct Record2(i64, Record1);
struct Closure0<'c0> {
    capture0: &'c0 Record1,
}
impl<'c0> Closure0<'c0> {
    fn call(&self, value0: i64) -> i64 {
        function1(value0) + self.capture0.field0.len() as i64
    }
}
impl<'c0> Callback<(i64,), i64> for Closure0<'c0> {
    fn invoke(&mut self, args: (i64,)) -> i64 { self.call(args.0) }
}
trait Callback<A, R> { fn invoke(&mut self, args: A) -> R; }
enum StoredCallable0<'c0> { V_0(Closure0<'c0>) }
fn function0(value0: Record1) -> i64 {
    let closure = Closure0 { capture0: &value0 };
    closure.call(2) + function1(3)
}
fn function1(mut value0: i64) -> i64 { value0 += 1; value0 }
fn function2(value0: &StoredCallable0) -> i64 { 0 }
fn main() { let _ = function0(Record1 { field0: String::new() }); }
";

    fn split_test() -> Vec<CrateSource> {
        let modules = vec!["/app/a.tsr".to_owned(), "/app/b.tsr".to_owned()];
        // function0 and Closure0 in a.tsr, function1 in b.tsr, function2 in b.tsr.
        let functions = |number: u64| Some(if number == 0 { 0 } else { 1 });
        let closures = |_: u64| Some(0);
        split(PROGRAM, &Provenance { modules: &modules, function_module: &functions, closure_module: &closures }).unwrap()
    }

    #[test]
    fn only_type_parameters_block_extern_declarations() {
        assert!(!super::type_parameters(""));
        assert!(!super::type_parameters("'c0, 'c1"));
        assert!(super::type_parameters("'c0, K1: ReadCallback<(), i64>"));
        assert!(super::type_parameters("F1: Callback<(i64,), ()>"));
    }
    #[test]
    fn items_used_by_several_crates_move_to_the_types_crate() {
        let crates = split_test();
        let names = crates.iter().map(|c| c.name.as_str()).collect::<Vec<_>>();
        let a = module_crate_name("/app/a.tsr");
        let b = module_crate_name("/app/b.tsr");
        assert_eq!(names, [TYPES_CRATE, a.as_str(), b.as_str(), "program"]);
        let types = &crates[0].source;
        // Closure0 is stored in a family that b.tsr's function2 uses, so its struct is shared.
        assert!(types.contains("pub struct Closure0<'c0> {\n    pub capture0: &'c0 Record1,"));
        assert!(types.contains("#[derive(Clone)] pub struct Record1 { pub field0: String }"));
        assert!(types.contains("pub struct Record2(pub i64, pub Record1);"));
        assert!(types.contains("pub trait Callback<A, R>"));
        // Its synchronous body stays in a.tsr's crate behind a link-time declaration.
        assert!(types.contains("pub safe fn closure_body_Closure0<'c0>(this: &Closure0<'c0>, value0: i64) -> i64;"));
        assert!(types.contains("tsrust_extern::closure_body_Closure0(self, value0)"));
        let module_a = &crates[1].source;
        assert!(module_a.contains("impl<'c0> TsrBodyClosure0<'c0> for Closure0<'c0> {\n    fn tsrust_body(&self, value0: i64) -> i64 {"));
        assert!(module_a.contains("#[link_name = \"tsrust_function1\"] pub safe fn function1(value0: i64) -> i64;"));
        let module_b = &crates[2].source;
        assert!(module_b.contains("#[unsafe(export_name = \"tsrust_function1\")]\npub fn function1(mut value0: i64)"));
        let program = &crates[3].source;
        assert!(program.contains(&format!("let _ = [{a}::tsrust_extern::function1, {b}::function1];")));
        assert!(program.contains("fn main()"));
        assert!(crates[3].program && crates[3].dependencies.len() == 3);
    }

    #[test]
    fn module_crate_names_are_valid_stable_and_distinct() {
        let name = module_crate_name("/app/src/components/Git Panel.tsrx");
        assert!(name.starts_with("tsrust_m_git_panel_") && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'));
        assert_eq!(name, module_crate_name("/app/src/components/Git Panel.tsrx"));
        assert_ne!(name, module_crate_name("/app/src/other/Git Panel.tsrx"));
    }

    #[test]
    fn crate_paths_follow_items_into_the_types_crate() {
        let program = "mod tsrust_source_segments { pub(crate) static SEGMENTS: &[usize] = &[]; }\nfn function0() -> usize { crate::tsrust_source_segments::SEGMENTS.len() + function1() }\nfn function1() -> usize { crate::function0() }\nfn main() { let _ = crate::tsrust_source_segments::SEGMENTS; function0(); }\n";
        let modules = vec!["/app/a.tsr".to_owned(), "/app/b.tsr".to_owned()];
        let functions = |number: u64| Some(number as usize);
        let closures = |_: u64| None;
        let crates = split(program, &Provenance { modules: &modules, function_module: &functions, closure_module: &closures }).unwrap();
        let a = crates.iter().find(|c| c.name == module_crate_name("/app/a.tsr")).unwrap();
        let b = crates.iter().find(|c| c.name == module_crate_name("/app/b.tsr")).unwrap();
        assert!(a.source.contains("tsrust_types::tsrust_source_segments::SEGMENTS"));
        assert!(b.source.contains("crate::tsrust_extern::function0()"));
        assert!(crates[0].source.contains("pub mod tsrust_source_segments { pub static SEGMENTS"));
    }

    #[test]
    fn inline_items_and_attribute_prefixes_are_separate_items() {
        let items = split_items("struct A { x: i64 }fn f() {}\n#[derive(Clone)] struct B;\nimpl<'a> T<'a> for &'a A { fn g(self) {} }\n").unwrap();
        assert_eq!(items, ["struct A { x: i64 }\n", "fn f() {}\n", "#[derive(Clone)] struct B;\n", "impl<'a> T<'a> for &'a A { fn g(self) {} }\n"]);
        match kind_name(&items[3]) {
            Kind::Impl { target, trait_name } => assert_eq!((target.as_deref(), trait_name.as_deref()), (Some("A"), Some("T"))),
            _ => panic!("impl"),
        }
    }

    #[test]
    fn publishing_skips_generic_defaults_trait_impls_and_existing_visibility() {
        assert_eq!(publish("struct M<K, P = (K, K)> { a: P, pub b: K }\n"), "pub struct M<K, P = (K, K)> { pub a: P, pub b: K }\n");
        assert_eq!(
            publish("impl A { async fn f(&self) {} pub(crate) fn g() {} const N: i64 = 1; }\n"),
            "impl A { pub async fn f(&self) {} pub fn g() {} pub const N: i64 = 1; }\n"
        );
        assert_eq!(publish("impl T for A { fn f(&self) {} }\n"), "impl T for A { fn f(&self) {} }\n");
        assert_eq!(publish("mod m { pub(super) fn f() {} }\n"), "pub mod m { pub fn f() {} }\n");
        assert_eq!(
            publish("thread_local! {\n    static RUNNING: Cell<bool> = const { Cell::new(false) };\n}\n"),
            "thread_local! {\n    pub static RUNNING: Cell<bool> = const { Cell::new(false) };\n}\n"
        );
        assert_eq!(
            publish("std::thread_local! {static SITE:Cell<Option<&'static Site>>=const{Cell::new(None)};}\n"),
            "std::thread_local! {pub static SITE:Cell<Option<&'static Site>>=const{Cell::new(None)};}\n"
        );
        match kind_name("std::thread_local! {static RUNNING: Cell<bool> = Cell::new(false);}\n") {
            Kind::Named(name) => assert_eq!(name, "RUNNING"),
            _ => panic!("thread_local"),
        }
    }
}
