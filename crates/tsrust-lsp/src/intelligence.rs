//! Tolerant source declarations for editing incomplete programs. This is not a
//! second type checker: unknown types stay unknown and diagnostics use `check`.
#[path="statement_extent.rs"]
mod statement_extent;
#[path="local_function_intelligence.rs"]
mod local_function_intelligence;
#[path = "project_generic_intelligence.rs"]
mod project_generic_intelligence;

#[path = "generic_intelligence.rs"]
mod generic_intelligence;
use crate::LineIndex;
use lsp_types::{Position, Range};
use serde_json::{Value, json};
use tsrust_compiler::Span;

#[derive(Clone, Debug)]
struct Token {
    text: String,
    span: Span,
    scope: usize,
    literal: bool,
    unterminated_string: bool,
}
#[derive(Clone, Debug)]
struct Scope {
    parent: Option<usize>,
    span: Span,
}
#[derive(Clone, Debug)]
struct Symbol {
    name: String,
    kind: u32,
    span: Span,
    extent: Span,
    scope: usize,
    ty: String,
    signature: String,
    docs: String,
    parameters: Vec<String>,
    owner: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckedGenericCall {
    span: Span,
    callee_span: Span,
    declaration_span: Span,
    parameters: Vec<tsrust_compiler::GenericParameterInfo>,
    result: Option<String>,
}
pub struct SourceIndex {
    /// `type Name = Type;` declarations: name to fully expanded type text.
    type_aliases: std::collections::BTreeMap<String, String>,
    generic_display_names: std::collections::BTreeMap<String, String>,
    nominal_definitions: std::collections::BTreeMap<String, Value>,
    generic_parameters: std::collections::BTreeMap<String, Vec<String>>,
    foreign_definitions: std::collections::BTreeMap<usize, Value>,
    native_parameter_docs: std::collections::BTreeMap<usize, Vec<String>>,
    native_origins: std::collections::BTreeMap<usize, usize>,
    entry_help: bool,
    source: String,
    lines: LineIndex,
    tokens: Vec<Token>,
    scopes: Vec<Scope>,
    pairs: Vec<Option<usize>>,
    symbols: Vec<Symbol>,
    parameter_symbols: std::collections::BTreeSet<usize>,
    parameter_available_from: std::collections::BTreeMap<usize, usize>,
    comments: Vec<(Span, String)>,
    templates: Vec<(usize, usize)>,
    end_scope: usize,
    pub(crate) checked_control_labels: Vec<(Span,Span)>,
    checked_views: Vec<(Span, String)>,
    checked_method_results: Vec<(Span, String)>,
    checked_optional_receivers: Vec<(Span, String)>,
    checked_generic_calls: Vec<CheckedGenericCall>,
    /// Commas that separate declarators in a const or let declaration, with
    /// the keyword of that declaration.
    declarator_separators: std::collections::HashMap<usize, &'static str>,
}
fn member_separator(text: &str) -> bool {
    matches!(text, "." | "?.")
}
fn name(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
impl SourceIndex {
    pub fn new(source: &str) -> Self {
        Self::with_entry_help(source, true)
    }
    pub(crate) fn with_entry_help(source: &str, entry_help: bool) -> Self {
        let mut index = Self {
            generic_display_names: Default::default(),
            nominal_definitions: Default::default(),
            generic_parameters: Default::default(),
            foreign_definitions: Default::default(),
            native_parameter_docs: Default::default(),
            native_origins: Default::default(),
            entry_help,
            source: source.into(),
            lines: LineIndex::new(source),
            tokens: vec![],
            scopes: vec![Scope {
                parent: None,
                span: Span::new(0, source.len()),
            }],
            pairs: vec![],
            symbols: vec![],
            parameter_symbols: Default::default(),
            parameter_available_from: Default::default(),
            comments: vec![],
            templates: vec![],
            end_scope: 0,
            checked_control_labels: vec![],
            type_aliases: Default::default(),
            checked_views: vec![],
            checked_method_results: vec![],
            checked_optional_receivers: vec![],
            checked_generic_calls: vec![],
            declarator_separators: std::collections::HashMap::new(),
        };
        let bytes = source.as_bytes();
        let mut offset = 0;
        let mut scope = 0;
        let mut stack: Vec<usize> = vec![];
        // Each frame keeps its first chunk and either chunk mode or expression brace depth.
        let mut templates: Vec<(usize, Option<usize>)> = vec![];
        while offset < bytes.len() {
            let start = offset;
            let ch = source[offset..].chars().next().unwrap();
            let chunk = templates.last().is_some_and(|(_, depth)| depth.is_none());
            if !chunk && ch.is_whitespace() {
                offset += ch.len_utf8();
                continue;
            }
            if !chunk && (source[offset..].starts_with("//") || source[offset..].starts_with("/*"))
            {
                let block = source[offset..].starts_with("/*");
                offset = if block {
                    source[offset + 2..]
                        .find("*/")
                        .map_or(source.len(), |n| offset + n + 4)
                } else {
                    source[offset..]
                        .find(['\n', '\r', '\u{2028}', '\u{2029}'])
                        .map_or(source.len(), |n| offset + n)
                };
                index
                    .comments
                    .push((Span::new(start, offset), source[start..offset].into()));
                continue;
            }
            let quoted = matches!(ch, '\'' | '"');
            let mut literal = quoted || ch.is_ascii_digit();
            let mut unterminated_string = false;
            let mut interpolation_end = false;
            let mut template_close = None;
            let mut template_chunk = false;
            if chunk && source[offset..].starts_with("${") {
                offset += 2;
                templates.last_mut().unwrap().1 = Some(0);
            } else if chunk || ch == '`' {
                literal = true;
                template_chunk = true;
                if !chunk {
                    templates.push((index.tokens.len(), None));
                    offset += 1;
                }
                unterminated_string = true;
                while offset < bytes.len() {
                    if bytes[offset] == b'\\' {
                        offset += 1;
                        if offset < bytes.len() {
                            let escaped = source[offset..].chars().next().unwrap();
                            offset += escaped.len_utf8();
                            if escaped == '\r' && bytes.get(offset) == Some(&b'\n') {
                                offset += 1;
                            }
                        }
                    } else if bytes[offset] == b'`' {
                        offset += 1;
                        template_close = Some(templates.pop().unwrap().0);
                        unterminated_string = false;
                        break;
                    } else if source[offset..].starts_with("${") {
                        unterminated_string = false;
                        break;
                    } else {
                        offset += source[offset..].chars().next().unwrap().len_utf8();
                    }
                }
            } else if ch == '}' && templates.last().is_some_and(|(_, depth)| *depth == Some(0)) {
                offset += 1;
                interpolation_end = true;
                templates.last_mut().unwrap().1 = None;
            } else if quoted {
                let delimiter = ch as u8;
                unterminated_string = true;
                offset += 1;
                while offset < bytes.len() {
                    if bytes[offset] == b'\\' {
                        offset += 1;
                        if offset < bytes.len() {
                            let escaped = source[offset..].chars().next().unwrap();
                            offset += escaped.len_utf8();
                            if escaped == '\r' && bytes.get(offset) == Some(&b'\n') {
                                offset += 1;
                            }
                        }
                    } else if bytes[offset] == delimiter {
                        offset += 1;
                        unterminated_string = false;
                        break;
                    } else {
                        offset += source[offset..].chars().next().unwrap().len_utf8();
                    }
                }
            } else if ch.is_ascii_alphabetic() || ch == '_' {
                offset += 1;
                while offset < bytes.len()
                    && (bytes[offset].is_ascii_alphanumeric() || bytes[offset] == b'_')
                {
                    offset += 1;
                }
            } else if let Some(span)=tsrust_compiler::numeric_literal_span(source,offset){
                offset=span.end;
            } else if ch.is_ascii_digit(){
                // Keep incomplete editor input tokenized even before it checks.
                offset+=1;
                while offset<bytes.len()&&(bytes[offset].is_ascii_alphanumeric()||bytes[offset]==b'_'){offset+=1;}
            } else if source[offset..].starts_with("...") {
                offset += 3;
            } else if (source[offset..].starts_with("?.")
                && !bytes.get(offset + 2).is_some_and(u8::is_ascii_digit))
                || source[offset..].starts_with("=>")
            {
                offset += 2;
            } else {
                offset += ch.len_utf8();
            }
            // Chunk contents must never masquerade as a name, brace, or comma token.
            let text = if template_chunk {
                "`"
            } else {
                &source[start..offset]
            };
            let id = index.tokens.len();
            index.tokens.push(Token {
                text: text.into(),
                span: Span::new(start, offset),
                scope,
                literal,
                unterminated_string,
            });
            index.pairs.push(None);
            if let Some(open) = template_close {
                index.templates.push((open, id));
            }
            if let Some((_, Some(depth))) = templates.last_mut() {
                if text == "{" {
                    *depth += 1;
                } else if text == "}" && !interpolation_end {
                    *depth = depth.saturating_sub(1);
                }
            }
            if matches!(text, "{" | "(" | "[" | "${") {
                stack.push(id);
                if text == "{" {
                    index.scopes.push(Scope {
                        parent: Some(scope),
                        span: Span::new(start, source.len()),
                    });
                    scope = index.scopes.len() - 1;
                }
            } else if matches!(text, "}" | ")" | "]") {
                let opening = match text {
                    "}" if interpolation_end => "${",
                    "}" => "{",
                    ")" => "(",
                    _ => "[",
                };
                if let Some(position) = stack.iter().rposition(|&n| index.tokens[n].text == opening)
                {
                    let open = stack[position];
                    stack.truncate(position);
                    index.pairs[open] = Some(id);
                    index.pairs[id] = Some(open);
                    if text == "}" && !interpolation_end && scope > 0 {
                        index.scopes[scope].span.end = offset;
                        scope = index.scopes[scope].parent.unwrap_or(0);
                    }
                }
            }
        }
        index.end_scope = scope;
        index.declarator_lists();
        index.declarations();
        index.materialize_generic_fields();
        index.cache_checked_views();
        index.refresh_destructured_names();
        index.refresh_checked_initializers();
        // Checked constructor results may reveal concrete generic owners.
        index.materialize_generic_fields();
        index
    }
    // Checked views are cached per document revision. Never derive flow facts here.
    fn cache_checked_views(&mut self) {
        let (analysis, inserted) = match tsrust_compiler::check(&self.source) {
            Ok(analysis) => (analysis, None),
            Err(_) => {
                let Some((probe, start, added, original_end)) = self.receiver_probe() else {
                    return;
                };
                let Ok(analysis) = tsrust_compiler::check(&probe) else {
                    return;
                };
                (analysis, Some((start, added, original_end)))
            }
        };
        let mut views = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        let mut methods = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        self.checked_control_labels=analysis.checked_control_labels();
        if let Some((start,added,original_end))=inserted {
            self.checked_control_labels.retain_mut(|(span,definition)|{
                for span in [span,definition] {if span.end<=start {}else if span.start>=start+added && span.end<=original_end+added {span.start-=added;span.end-=added;}else{return false;}}
                true
            });
        }
        for fact in analysis.checked_local_functions() {
            let mut span = fact.declaration_span;
            if let Some((start, added, original_end)) = inserted {
                if span.end <= start {} else if span.start >= start + added && span.end <= original_end + added {
                    span.start -= added; span.end -= added;
                } else { continue; }
            }
            self.apply_local_function_result(span, &fact.name, fact.result);
        }
        for (name, result) in analysis.checked_function_results() {
            if result != "void" {
                self.apply_function_result(&name, &result);
            }
        }
        let facts = analysis.checked_type_facts();
        for &(mut span, mut declaration) in &facts.generic_values {
            if facts.generic_values.iter().any(|(other_span, other_declaration)| *other_span == span && *other_declaration != declaration) { continue; }
            let remap = |span: &mut Span| {
                if let Some((start, added, original_end)) = inserted {
                    if span.end <= start { return true; }
                    if span.start >= start + added && span.end <= original_end + added {
                        span.start -= added; span.end -= added;
                    } else { return false; }
                }
                true
            };
            if !remap(&mut span) || !remap(&mut declaration) { continue; }
            let Some(source) = self.symbols.iter().find(|symbol| symbol.span == declaration && symbol.kind == 12).cloned() else { continue; };
            for symbol in self.symbols.iter_mut().filter(|symbol| symbol.span == span) {
                symbol.signature = source.signature.replacen(&format!("function {}", source.name), &format!("function {}", symbol.name), 1);
                symbol.kind = 12;
                symbol.parameters = source.parameters.clone();
                symbol.docs = source.docs.clone();
            }
        }
        for (owner,name,original_span,ty) in &facts.rest_record_fields {
            let mut span=*original_span;
            if let Some((start,added,original_end))=inserted {
                if span.end<=start {}else if span.start>=start+added && span.end<=original_end+added{span.start-=added;span.end-=added;}else{continue;}
            }
            self.symbols.push(Symbol{name:name.clone(),kind:8,span,extent:span,scope:0,ty:ty.clone(),signature:format!("{name}: {ty}"),docs:String::new(),parameters:Vec::new(),owner:Some(owner.clone())});
        }
        let mut optional = std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        let mut parameter_types =
            std::collections::BTreeMap::<(usize, usize), Option<String>>::new();
        let mut calls =
            std::collections::BTreeMap::<(usize, usize), Option<CheckedGenericCall>>::new();
        for mut fact in facts.generic_calls.into_iter().map(|fact| CheckedGenericCall {
                span: fact.span,
                callee_span: fact.callee_span,
                declaration_span: fact.declaration_span,
                parameters: fact.parameters,
                result: Some(fact.result),
            }).chain(facts.generic_parameter_calls.into_iter().map(|fact| CheckedGenericCall {
                span: fact.span,
                callee_span: fact.callee_span,
                declaration_span: fact.declaration_span,
                parameters: fact.parameters,
                result: None,
            })) {
            let remap = |span: &mut Span| {
                if let Some((start, added, original_end)) = inserted {
                    if span.end <= start {
                        return true;
                    }
                    if span.start >= start + added && span.end <= original_end + added {
                        span.start -= added;
                        span.end -= added;
                    } else {
                        return false;
                    }
                }
                true
            };
            if !remap(&mut fact.span)
                || !remap(&mut fact.callee_span)
                || !remap(&mut fact.declaration_span)
            {
                continue;
            }
            // Invocation result metadata describes the selected method. A
            // checked expression at the same span can include an optional
            // absence result and is authoritative for initializer inference.
            if let Some(result) = &fact.result
                && !facts.method_results.iter().any(|expression| expression.span == fact.span) {
                methods
                    .entry((fact.span.start, fact.span.end))
                    .and_modify(|existing| {
                        if existing.as_ref() != Some(result) {
                            *existing = None;
                        }
                    })
                    .or_insert(Some(result.clone()));
            }
            calls
                .entry((fact.callee_span.start, fact.callee_span.end))
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&fact) {
                        *existing = None;
                    }
                })
                .or_insert(Some(fact));
        }
        self.checked_generic_calls = calls.into_values().flatten().collect();
        for (mut span, ty, kind) in facts
            .projections
            .into_iter()
            .map(|fact| (fact.span, fact.ty, 0))
            .chain(
                facts
                    .method_results
                    .into_iter()
                    .map(|fact| (fact.span, fact.ty, 1)),
            )
            .chain(
                facts
                    .optional_receivers
                    .into_iter()
                    .map(|fact| (fact.span, fact.ty, 2)),
            )
            .chain(
                facts
                    .parameter_types
                    .into_iter()
                    .map(|fact| (fact.span, fact.ty, 3)),
            )
        {
            if let Some((start, added, original_end)) = inserted {
                if span.end <= start {
                    // The original prefix is unchanged.
                } else if span.start >= start + added && span.end <= original_end + added {
                    span.start -= added;
                    span.end -= added;
                } else {
                    continue;
                }
            }
            let destination = match kind {
                1 => &mut methods,
                2 => &mut optional,
                3 => &mut parameter_types,
                _ => &mut views,
            };
            destination
                .entry((span.start, span.end))
                .and_modify(|existing| {
                    if existing.as_ref() != Some(&ty) {
                        *existing = None;
                    }
                })
                .or_insert(Some(ty));
        }
        self.checked_views = views
            .into_iter()
            .filter_map(|((start, end), ty)| ty.map(|ty| (Span::new(start, end), ty)))
            .collect();
        self.checked_optional_receivers = optional
            .into_iter()
            .filter_map(|((start, end), ty)| ty.map(|ty| (Span::new(start, end), ty)))
            .collect();
        self.apply_checked_parameter_types(parameter_types);
        self.checked_method_results = methods
            .into_iter()
            .filter_map(|((start, end), ty)| ty.map(|ty| (Span::new(start, end), ty)))
            .collect();
    }
    fn apply_checked_parameter_types(
        &mut self,
        facts: std::collections::BTreeMap<(usize, usize), Option<String>>,
    ) {
        for id in &self.parameter_symbols {
            let symbol = &mut self.symbols[*id];
            let Some(Some(ty)) = facts.get(&(symbol.span.start, symbol.span.end)) else {
                continue;
            };
            symbol.ty = ty.clone();
            let label = symbol.signature.split(':').next().unwrap_or(&symbol.name);
            symbol.signature = format!("{label}: {ty}");
            if let Some(parameters) = callable_parameters(ty) {
                symbol.kind = 12;
                symbol.parameters = parameters;
            } else {
                symbol.kind = 13;
                symbol.parameters.clear();
            }
        }
    }
    fn display_type(&self, ty: &str) -> String {
        let mut display = ty.to_owned();
        let mut names = self.generic_display_names.iter().collect::<Vec<_>>();
        names.sort_by_key(|(identity, _)| std::cmp::Reverse(identity.len()));
        for (identity, label) in names {
            if identity.starts_with("__tsrust_private_") || identity.starts_with('@') {
                display = display.replace(identity, label);
            }
        }
        display
    }
    fn checked_generic_signature(&self, token: usize) -> Option<(String, Vec<String>)> {
        let span = self.tokens.get(token)?.span;
        let fact = self
            .checked_generic_calls
            .iter()
            .find(|fact| fact.callee_span == span)?;
        self.checked_generic_signature_for(fact, self.text(token))
    }
    fn checked_generic_signature_for(&self, fact: &CheckedGenericCall, callee: &str) -> Option<(String, Vec<String>)> {
        let parameters = fact
            .parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                parameter.ty.as_ref().map(|ty| {
                    let required_after = fact.parameters[index + 1..].iter().any(|parameter| {
                        !parameter.rest
                            && parameter.omission == tsrust_compiler::ParameterOmission::Required
                    });
                    let defaulted_slot = parameter.omission
                        == tsrust_compiler::ParameterOmission::Defaulted
                        && required_after;
                    let supplied = if defaulted_slot {
                        optional_parameter_type(ty)
                    } else {
                        ty.clone()
                    };
                    let access = if parameter.rest || callable_annotation(&supplied).is_some() {
                        tsrust_compiler::Mode::Copy
                    } else { parameter.access };
                    let supplied = tsrust_compiler::type_syntax::access_type_text(access, &self.display_type(&supplied));
                    format!(
                        "{}{}{}: {}",
                        if parameter.rest { "..." } else { "" },
                        parameter.name,
                        if !parameter.rest
                            && parameter.omission != tsrust_compiler::ParameterOmission::Required
                            && !required_after
                        {
                            "?"
                        } else {
                            ""
                        },
                        supplied
                    )
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some((
            format!(
                "function {}({}){}",
                callee,
                parameters.join(", "),
                fact.result.as_ref().map_or(String::new(), |result| format!(": {}", self.display_type(result)))
            ),
            parameters,
        ))
    }
    fn checked_view(&self, token: usize) -> Option<&str> {
        let token = self.tokens.get(token)?;
        self.checked_views
            .iter()
            .filter(|(span, _)| span.start <= token.span.start && span.end == token.span.end)
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(_, ty)| ty.as_str())
    }
    // Only ?. uses this payload query; it does not refine ordinary member access.
    fn checked_optional_receiver(&self, token: usize) -> Option<&str> {
        let token = self.tokens.get(token)?;
        self.checked_optional_receivers
            .iter()
            .filter(|(span, _)| span.start <= token.span.start && span.end == token.span.end)
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(_, ty)| ty.as_str())
    }
    fn checked_method_result(&self, token: usize) -> Option<&str> {
        let token = self.tokens.get(token)?;
        self.checked_method_results
            .iter()
            .filter(|(span, _)| span.start <= token.span.start && span.end == token.span.end)
            .min_by_key(|(span, _)| span.end - span.start)
            .map(|(_, ty)| ty.as_str())
    }
    // Use an exact checked expression span, never an inferred join of its operands.
    fn checked_expression_type(&self, token: usize) -> Option<(String, usize)> {
        let start = self.tokens.get(token)?.span.start;
        self.checked_method_results
            .iter()
            .filter(|(span, _)| span.start == start || (span.start>start && self.text(token)=="(" && self.tokens[token..].iter().take_while(|value|value.span.start<span.start).all(|value|&self.source[value.span.start..value.span.end]=="(") && self.pairs[token].is_some_and(|close|self.tokens[close].span.end<span.end)))
            .filter_map(|(span, ty)| {
                let end = self.tokens[token..]
                    .iter()
                    .position(|token| token.span.end == span.end)?
                    + token
                    + 1;
                let mut end=end;
                // Checked grouped arms omit their source-only closing group.
                // Include groups opened inside this expression, never its caller.
                while self.text(end)==")" && self.pairs.get(end).copied().flatten().is_some_and(|open|self.tokens[open].span.start>=start) {end+=1;}
                Some((ty.clone(), end))
            })
            .max_by_key(|(_, end)| *end)
    }
    // An incomplete EOF member can be checked through a readonly projection.
    // typeof preserves the physical tag; Project metadata supplies only proven views.
    fn receiver_probe(&self) -> Option<(String, usize, usize, usize)> {
        let last = self.tokens.len().checked_sub(1)?;
        if !self.source[self.tokens[last].span.end..].trim().is_empty() {
            return None;
        }
        let dot = if member_separator(self.text(last)) {
            last
        } else if name(self.text(last)) && last > 0 && member_separator(self.text(last - 1)) {
            last - 1
        } else {
            return None;
        };
        let end = dot.checked_sub(1)?;
        let mut begin = end;
        if self.text(begin) == ")" {
            begin = self.pairs[begin]?;
        }
        while begin >= 2 && member_separator(self.text(begin - 1)) && name(self.text(begin - 2)) {
            begin -= 2;
        }
        if begin > 0 && matches!(self.text(begin - 1), "." | "?." | "]") {
            return None;
        }
        let mut first = begin;
        let mut stop = end + 1;
        while self.text(first) == "(" && self.pairs[first] == Some(stop - 1) {
            first += 1;
            stop -= 1;
        }
        if !name(self.text(first)) {
            return None;
        }
        let mut position = first + 1;
        while position < stop {
            if !member_separator(self.text(position)) || !name(self.text(position + 1)) {
                return None;
            }
            position += 2;
        }
        let start = self.tokens[begin].span.start;
        let original_end = self.tokens[end].span.end;
        let receiver = &self.source[start..original_end];
        let inserted = "typeof ";
        let mut probe = format!("{}{inserted}{receiver};", &self.source[..start]);
        let mut blocks = 0usize;
        for token in &self.tokens[..dot] {
            if token.text == "{" {
                blocks += 1;
            } else if token.text == "}" {
                blocks = blocks.checked_sub(1)?;
            }
        }
        probe.extend(std::iter::repeat_n('}', blocks));
        Some((probe, start, inserted.len(), original_end))
    }
    // A destructured name has no initializer of its own; its type is the
    // checked type of its position.
    fn refresh_destructured_names(&mut self) {
        for symbol in 0..self.symbols.len() {
            if self.symbols[symbol].kind != 13 {
                continue;
            }
            let span = self.symbols[symbol].span;
            let mut found = self.checked_method_results.iter().filter(|(fact, _)| *fact == span);
            let (Some((_, ty)), None) = (found.next(), found.next()) else { continue };
            let ty = ty.clone();
            let Some(keyword) = self.symbols[symbol].signature.split(' ').next().filter(|word| matches!(*word, "const" | "let")) else {
                continue;
            };
            self.symbols[symbol].signature = format!("{keyword} {}: {ty}", self.symbols[symbol].name);
            self.symbols[symbol].ty = ty;
        }
    }
    fn refresh_checked_initializers(&mut self) {
        for id in 0..self.tokens.len() {
            if self.declaration_keyword(id).is_some()
                && self.text(id + 2) == "="
                && let Some(symbol) = self
                    .symbols
                    .iter()
                    .position(|s| s.span.start == self.tokens[id + 1].span.start && s.kind != 12)
            {
                if let Some((origin, end)) = self.native_initializer_origin(id + 3, 0)
                    && matches!(self.text(end), ";" | "," | "}" | "")
                    && let Some(parameter_docs) = self.native_parameter_docs.get(&origin).cloned()
                {
                    let native = self.symbols[origin].clone();
                    let result = if let Some((_, end)) = callable_annotation(&native.ty) {
                        native.ty[end + 1..]
                            .trim_start()
                            .strip_prefix("=>")
                            .unwrap_or("")
                            .trim()
                            .to_owned()
                    } else {
                        native.ty.clone()
                    };
                    let ty = format!("({}) => {result}", native.parameters.join(", "));
                    let signature = format!("{} {}: {ty}", self.declaration_keyword(id).unwrap_or("const"), self.text(id + 1));
                    let target = &mut self.symbols[symbol];
                    target.kind = 12;
                    target.ty = ty.clone();
                    target.signature = signature;
                    target.docs = native.docs;
                    target.parameters = native.parameters;
                    self.native_parameter_docs.insert(symbol, parameter_docs);
                    self.native_origins
                        .insert(symbol, self.native_origins[&origin]);
                    continue;
                }
                let ty = self.initializer_type(id + 3, self.tokens[id].span.start);
                if !ty.is_empty() {
                    if let Some(parameters) = callable_parameters(&ty) {
                        self.symbols[symbol].kind = 12;
                        self.symbols[symbol].parameters = parameters;
                    }
                    self.symbols[symbol].ty = ty.clone();
                    self.symbols[symbol].signature =
                        format!("{} {}: {ty}", self.declaration_keyword(id).unwrap_or("const"), self.text(id + 1));
                }
            }
        }
    }
    /// Only propagate a native identity when every selected branch names that identity.
    fn native_initializer_origin(&self, start: usize, depth: usize) -> Option<(usize, usize)> {
        if depth > 8 {
            return None;
        }
        if self.text(start) == "("
            && let Some(close) = self.pairs.get(start).copied().flatten()
            && let Some((origin, end)) = self.native_initializer_origin(start + 1, depth + 1)
            && end == close
        {
            return Some((origin, close + 1));
        }
        if let Some(origin) =
            self.resolve_name(self.text(start), self.tokens.get(start)?.span.start)
            && self.native_origins.contains_key(&origin)
        {
            return Some((origin, start + 1));
        }
        let mut question = start;
        while !matches!(self.text(question), "?" | ";" | "," | "}" | ":" | "") {
            question = if matches!(self.text(question), "(" | "[") {
                self.pairs.get(question).copied().flatten()? + 1
            } else {
                question + 1
            };
        }
        if self.text(question) != "?" {
            return None;
        }
        let (left, colon) = self.native_initializer_origin(question + 1, depth + 1)?;
        if self.text(colon) != ":" {
            return None;
        }
        let (right, end) = self.native_initializer_origin(colon + 1, depth + 1)?;
        (self.native_origins[&left] == self.native_origins[&right]).then_some((left, end))
    }
    fn text(&self, id: usize) -> &str {
        self.tokens.get(id).map_or("", |t| t.text.as_str())
    }
    /// Canonical form of a written type, with type aliases expanded.
    fn written_type(&self, text: &str) -> String {
        canonical_data_type(&expand_type_aliases(text, &self.type_aliases))
    }
    /// Collects top-level `type Name = Type;` declarations that are not
    /// record bodies and expands the aliases they mention.
    fn collect_type_aliases(&mut self) {
        let mut written = std::collections::BTreeMap::new();
        for id in 0..self.tokens.len() {
            if self.text(id) == "type"
                && self.tokens[id].scope == 0
                && name(self.text(id + 1))
                && self.text(id + 2) == "="
                && self.text(id + 3) != "{"
                && (id == 0 || matches!(self.text(id - 1), ";" | "}" | "export"))
            {
                let end = self.end_type(id + 3);
                written.insert(self.text(id + 1).to_owned(), self.snippet(id + 3, end));
            }
        }
        // Bounded rounds also end a self-referential declaration, which the
        // compiler reports.
        for _ in 0..16 {
            let mut changed = false;
            let snapshot = written.clone();
            for text in written.values_mut() {
                let expanded = expand_type_aliases(text, &snapshot);
                changed |= expanded != *text;
                *text = expanded;
            }
            if !changed {
                break;
            }
        }
        self.type_aliases = written;
    }
    fn snippet(&self, start: usize, end: usize) -> String {
        if start >= end || end > self.tokens.len() {
            return String::new();
        }
        self.source[self.tokens[start].span.start..self.tokens[end - 1].span.end]
            .trim()
            .into()
    }
    fn end_type(&self, start: usize) -> usize {
        let mut n = start;
        let mut angles = 0usize;
        while n < self.tokens.len()
            && !(angles == 0 && matches!(self.text(n), "=" | ";" | "," | "}" | "{" | ")" | "=>"))
        {
            if self.text(n) == "<" {
                angles += 1;
            } else if self.text(n) == ">" {
                angles = angles.saturating_sub(1);
            }
            if matches!(self.text(n), "(" | "[")
                && let Some(end) = self.pairs[n]
            {
                let callable = self.text(n) == "(";
                n = end + 1;
                if callable && self.text(n) == "=>" {
                    n += 1;
                }
            } else {
                n += 1;
            }
        }
        n
    }
    fn docs_before(&self, start: usize) -> String {
        let mut cursor = start;
        let mut parts = vec![];
        for (span, text) in self.comments.iter().rev() {
            if span.end > cursor {
                continue;
            }
            let gap = &self.source[span.end..cursor];
            if !(gap.trim().is_empty() || gap.trim() == "export")
                || gap.chars().filter(|&c| c == '\n').count() > 1
            {
                break;
            }
            parts.push(
                text.trim_start_matches('/')
                    .trim_start_matches('*')
                    .trim_end_matches("*/")
                    .lines()
                    .map(|line| line.trim().trim_start_matches('*').trim())
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
            cursor = span.start;
        }
        parts.reverse();
        parts.join("\n")
    }
    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        id: usize,
        kind: u32,
        ty: String,
        signature: String,
        extent: Span,
        docs_start: usize,
        scope: usize,
    ) -> usize {
        let token = &self.tokens[id];
        let symbol = Symbol {
            name: token.text.clone(),
            kind,
            span: token.span,
            extent,
            scope,
            ty,
            signature,
            docs: self.docs_before(docs_start),
            parameters: vec![],
            owner: None,
        };
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }
    /// `const` or `let` for a declaration keyword, or for a comma that starts
    /// a later declarator of the same declaration.
    fn declaration_keyword(&self, id: usize) -> Option<&'static str> {
        match self.text(id) {
            "const" => Some("const"),
            "let" => Some("let"),
            "," => self.declarator_separators.get(&id).copied(),
            _ => None,
        }
    }
    /// Find the commas that separate declarators: `let a = 1, [b, c] = pair;`.
    /// A comma inside brackets belongs to an initializer or pattern.
    fn declarator_lists(&mut self) {
        for id in 0..self.tokens.len() {
            let keyword = match self.text(id) {
                "const" => "const",
                "let" => "let",
                _ => continue,
            };
            let mut n = id + 1;
            while n < self.tokens.len() {
                match self.text(n) {
                    "(" | "[" | "{" => match self.pairs[n] {
                        Some(close) => n = close,
                        None => break,
                    },
                    ")" | "]" | "}" | ";" | "of" | "" => break,
                    "," => {
                        // A declarator is a name or pattern followed by `=` or
                        // `:`; a comma in a generic type argument list is not one.
                        let after = match self.text(n + 1) {
                            "[" | "{" => self.pairs[n + 1].map(|close| close + 1),
                            _ => Some(n + 2),
                        };
                        if after.is_some_and(|after| matches!(self.text(after), "=" | ":")) {
                            self.declarator_separators.insert(n, keyword);
                        }
                    }
                    _ => {}
                }
                n += 1;
            }
        }
    }
    fn declarations(&mut self) {
        // Loop-header declarations belong to the complete loop, not its parent.
        for id in 0..self.tokens.len() {
            if self.text(id) != "for" || self.text(id + 1) != "(" {
                continue;
            }
            let Some(close) = self.pairs[id + 1] else {
                continue;
            };
            let body = close + 1;
            let last = self.source_statement_end(body, 0);
            let end = self.tokens.get(last).map_or(self.source.len(), |token| token.span.end);
            let parent = self.tokens[id].scope;
            let new_scope = self.scopes.len();
            self.scopes.push(Scope { parent: Some(parent), span: Span::new(self.tokens[id].span.start, end) });
            let start = self.tokens[id].span.start;
            for scope in &mut self.scopes[..new_scope] {
                if scope.parent == Some(parent) && start <= scope.span.start && scope.span.end <= end {
                    scope.parent = Some(new_scope);
                }
            }
            for token in &mut self.tokens[id..=last] {
                if token.scope == parent { token.scope = new_scope; }
            }
        }
        self.collect_type_aliases();
        let mut interface_bases: Vec<(String, Vec<String>)> = Vec::new();
        for id in 0..self.tokens.len() {
            if self.text(id) == "type"
                && self.text(id + 2) == "="
                && self.type_aliases.contains_key(self.text(id + 1))
                && self.text(id + 3) != "{"
            {
                let end = self.end_type(id + 3);
                let alias = self.text(id + 1).to_owned();
                let signature = format!("type {alias} = {}", self.snippet(id + 3, end));
                let expanded = canonical_data_type(&self.type_aliases[&alias]);
                self.add(
                    id + 1,
                    26,
                    expanded,
                    signature,
                    Span::new(self.tokens[id].span.start, self.tokens[end.saturating_sub(1)].span.end),
                    self.tokens[id].span.start,
                    self.tokens[id].scope,
                );
                continue;
            }
            if matches!(self.text(id), "function" | "type" | "enum" | "interface") && name(self.text(id + 1)) {
                let name_id = id + 1;
                let mut header = id + 2;
                if self.text(header) == "<" {
                    header += 1;
                    while name(self.text(header)) {
                        header += 1;
                        if self.text(header) == "," {
                            header += 1;
                        } else {
                            break;
                        }
                    }
                    if self.text(header) != ">" {
                        continue;
                    }
                    header += 1;
                }
                if self.text(id) == "function" && self.text(header) == "(" {
                    let Some(close) = self.pairs[header] else {
                        continue;
                    };
                    let result_end = if self.text(close + 1) == ":" {
                        self.end_type(close + 2)
                    } else {
                        close + 1
                    };
                    let ty = if self.text(close + 1) == ":" {
                        self.written_type(&self.snippet(close + 2, result_end))
                    } else {
                        String::new()
                    };
                    let body = (self.text(result_end) == "{").then_some(result_end);
                    let extent_end = body
                        .and_then(|b| self.pairs[b])
                        .map_or(self.tokens[close].span.end, |e| self.tokens[e].span.end);
                    let body_scope = body
                        .and_then(|b| {
                            self.scopes
                                .iter()
                                .position(|s| s.span.start == self.tokens[b].span.start)
                        })
                        .unwrap_or(self.tokens[id].scope);
                    // The name of a function expression is visible only inside
                    // that function; a declaration belongs to its block.
                    let expression = id > 0 && !matches!(self.text(id - 1), ";" | "{" | "}" | "export");
                    let declaration = self.add(
                        name_id,
                        12,
                        ty,
                        self.snippet(id, result_end),
                        Span::new(self.tokens[id].span.start, extent_end),
                        self.tokens[id].span.start,
                        if expression { body_scope } else { self.tokens[id].scope },
                    );
                    let parameters = self.parameters(header + 1, close, body_scope);
                    self.symbols[declaration].parameters = parameters;
                    if self.entry_help
                        && self.symbols[declaration].name == "main"
                        && self.symbols[declaration].scope == 0
                    {
                        let docs = &mut self.symbols[declaration].docs;
                        if !docs.is_empty() {
                            docs.push_str("\n\n");
                        }
                        docs.push_str("Program entry point. Tsrust automatically calls main() when you run this file with the VS Code play button or tsrust run. No main(); call is needed. Declare it with no parameters and no return value. Other functions run only when called.\n\n```tsrust\nfunction main() {\n    console.log(\"Hello\");\n}\n// Running the file prints Hello.\n```");
                    }
                } else if self.text(id) == "enum" && self.text(header) == "{" {
                    let end = self.pairs[header].unwrap_or(self.tokens.len() - 1);
                    let owner = self.text(name_id).to_owned();
                    self.add(name_id, 10, owner.clone(), format!("enum {owner}"),
                        Span::new(self.tokens[id].span.start, self.tokens[end].span.end),
                        self.tokens[id].span.start, self.tokens[id].scope);
                    let mut n = header + 1;
                    while n < end {
                        if name(self.text(n)) {
                            let member = self.add(n, 22, owner.clone(),
                                format!("{}.{}: {owner}", owner, self.text(n)),
                                self.tokens[n].span, self.tokens[id].span.start, self.tokens[id].scope);
                            self.symbols[member].owner = Some(owner.clone());
                        }
                        n += 1;
                        while n < end && self.text(n) != "," {
                            if let Some(close) = self.pairs[n].filter(|&close| close > n) { n = close + 1; }
                            else { n += 1; }
                        }
                        n += 1;
                    }
                } else if let Some(open) = if self.text(id) == "type" && self.text(header) == "=" && self.text(header + 1) == "{" {
                    Some(header + 1)
                } else if self.text(id) == "interface" {
                    // `interface Name extends Base, Other {`
                    (header..self.tokens.len()).take_while(|&n| !matches!(self.text(n), ";" | "")).find(|&n| self.text(n) == "{")
                } else {
                    None
                } {
                    let end = self.pairs[open].unwrap_or(self.tokens.len() - 1);
                    let owner = self.text(name_id).to_string();
                    if self.text(id) == "interface" {
                        let bases = (header..open)
                            .filter(|&n| name(self.text(n)) && self.text(n) != "extends")
                            .map(|n| self.text(n).to_owned())
                            .collect::<Vec<_>>();
                        if !bases.is_empty() {
                            interface_bases.push((owner.clone(), bases));
                        }
                    }
                    if header > id + 2 {
                        let parameters = (id + 3..header - 1)
                            .filter(|&n| name(self.text(n)))
                            .map(|n| self.text(n).to_owned())
                            .collect();
                        self.generic_parameters.insert(owner.clone(), parameters);
                    }
                    self.add(
                        name_id,
                        23,
                        owner.clone(),
                        self.snippet(id, end + 1),
                        Span::new(self.tokens[id].span.start, self.tokens[end].span.end),
                        self.tokens[id].span.start,
                        self.tokens[id].scope,
                    );
                    let mut n = open + 1;
                    while n < end {
                        // `name?: Type` is a field that a literal may omit.
                        let optional = name(self.text(n)) && self.text(n + 1) == "?" && self.text(n + 2) == ":";
                        if name(self.text(n)) && (self.text(n + 1) == ":" || optional) {
                            let colon = n + 1 + usize::from(optional);
                            let end_type = self.end_type(colon + 1).min(end);
                            let written = self.written_type(&self.snippet(colon + 1, end_type));
                            let ty = if optional && !written.split('|').any(|member| member.trim() == "undefined") {
                                canonical_data_type(&format!("{written} | undefined"))
                            } else {
                                written.clone()
                            };
                            let field = self.add(
                                n,
                                8,
                                ty.clone(),
                                format!("{}{}: {ty}", self.text(n), if optional { "?" } else { "" }),
                                self.tokens[n].span,
                                self.tokens[n].span.start,
                                self.tokens[n].scope,
                            );
                            self.symbols[field].owner = Some(owner.clone());
                            n = end_type + 1;
                        } else {
                            n += 1;
                        }
                    }
                }
            }
            if self.declaration_keyword(id).is_some()
                && matches!(self.text(id + 1), "[" | "{")
                && let Some(close) = self.pairs[id + 1]
            {
                // Destructuring: a name before `,`, `]` or `}` is declared; a
                // field name before `:` only selects. Types come from checking.
                // Without checked facts (incomplete source), first-level names
                // take their type from the written type of the whole value.
                let start = self.tokens[id].span.start;
                let whole = match self.text(close + 1) {
                    "=" => self.initializer_type(close + 2, start),
                    "of" => {
                        let iterable = self.infer(close + 2, start, 0);
                        iterable_element_type(&iterable).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                let binding_names=self.pattern_binding_names(id+1,close);
                let array_pattern = self.text(id + 1) == "[";
                let mut nesting = 0usize;
                let mut position = 0usize;
                for n in id + 2..close {
                    match self.text(n) {
                        "[" | "{" => nesting += 1,
                        "]" | "}" => nesting = nesting.saturating_sub(1),
                        "," if nesting == 0 => position += 1,
                        _ => {}
                    }
                    if binding_names.contains(&n) {
                        let written = if nesting != 0 || whole.is_empty() {
                            String::new()
                        } else if array_pattern {
                            match tsrust_compiler::type_syntax::parse(&whole).map(|annotation| annotation.kind) {
                                Ok(tsrust_compiler::type_syntax::ValueAnnotationKind::Tuple(positions)) => positions
                                    .get(position)
                                    .map(|field| whole[field.span.start..field.span.end].to_owned())
                                    .unwrap_or_default(),
                                _ => array_element_type(&whole).unwrap_or("").to_owned(),
                            }
                        } else {
                            let field = if self.text(n - 1) == ":" { self.text(n - 2) } else { self.text(n) };
                            self.symbols
                                .iter()
                                .find(|symbol| symbol.owner.as_deref() == Some(whole.as_str()) && symbol.name == field)
                                .map(|symbol| symbol.ty.clone())
                                .unwrap_or_default()
                        };
                        let ty = self.checked_method_result(n).map(str::to_owned).unwrap_or(written);
                        let signature = format!(
                            "{} {}{}",
                            self.declaration_keyword(id).unwrap_or("const"),
                            self.text(n),
                            if ty.is_empty() { String::new() } else { format!(": {ty}") }
                        );
                        self.add(n, 13, ty, signature, self.tokens[n].span, self.tokens[id].span.start, self.tokens[id].scope);
                    }
                }
            }
            if self.declaration_keyword(id).is_some() && name(self.text(id + 1)) {
                let n = id + 1;
                let end = if self.text(n + 1) == ":" {
                    self.end_type(n + 2)
                } else {
                    n + 1
                };
                let mut ty = if self.text(n + 1) == ":" {
                    self.written_type(&self.snippet(n + 2, end))
                } else {
                    String::new()
                };
                if ty.is_empty() && self.text(end) == "in" { ty = "string".into(); }
                if ty.is_empty() && self.text(end) == "of" {
                    let iterable = self.infer(end + 1, self.tokens[id].span.start, 0);
                    ty = iterable_element_type(&iterable).unwrap_or_default();
                }
                if ty.is_empty() && self.text(end) == "=" {
                    ty = self.initializer_type(end + 1, self.tokens[id].span.start);
                }
                let signature = format!(
                    "{} {}{}",
                    self.declaration_keyword(id).unwrap_or("const"),
                    self.text(n),
                    if ty.is_empty() {
                        String::new()
                    } else {
                        format!(": {ty}")
                    }
                );
                self.add(
                    n,
                    13,
                    ty,
                    signature,
                    self.tokens[n].span,
                    self.tokens[id].span.start,
                    self.tokens[id].scope,
                );
                // Arrow bindings are callable even while their body is incomplete.
                // A function expression is the same with `function` before its
                // parameter list and a block where the arrow would be.
                let expression = self.text(end) == "=" && self.text(end + 1) == "function";
                let named = expression && name(self.text(end + 2)) && self.text(end + 3) == "(";
                let list = end + 1 + usize::from(expression) + usize::from(named);
                if self.text(end) == "="
                    && self.text(list) == "("
                    && let Some(close) = self.pairs[list]
                {
                    let tail = if self.text(close + 1) == ":" {
                        self.end_type(close + 2)
                    } else {
                        close + 1
                    };
                    if expression && self.text(tail) == "{" {
                        let symbol = self.symbols.len() - 1;
                        let body_scope = self
                            .scopes
                            .iter()
                            .position(|s| s.span.start == self.tokens[tail].span.start)
                            .unwrap_or(self.tokens[id].scope);
                        let params = self.parameters(list + 1, close, body_scope);
                        if let Some(declared) = callable_parameters(&self.symbols[symbol].ty) {
                            self.symbols[symbol].kind = 12;
                            self.symbols[symbol].parameters = declared;
                        } else if self.text(n + 1) != ":" {
                            self.symbols[symbol].kind = 12;
                            self.symbols[symbol].parameters = params;
                            self.symbols[symbol].ty = if self.text(close + 1) == ":" {
                                self.written_type(&self.snippet(close + 2, tail))
                            } else {
                                String::new()
                            };
                            self.symbols[symbol].signature = format!(
                                "const {} = {}",
                                self.text(n),
                                self.snippet(end + 1, tail)
                            );
                        }
                    } else if !expression && self.text(tail) == "=>" {
                        let symbol = self.symbols.len() - 1;
                        let body = tail + 1;
                        let body_scope = self
                            .scopes
                            .iter()
                            .position(|s| {
                                s.span.start
                                    == self.tokens.get(body).map_or(usize::MAX, |t| t.span.start)
                            })
                            .unwrap_or(self.tokens[id].scope);
                        let params = if self.text(body) == "{" {
                            self.parameters(end + 2, close, body_scope)
                        } else {
                            callable_parameters(&format!(
                                "{} => unknown",
                                self.snippet(end + 1, close + 1)
                            ))
                            .unwrap_or_default()
                        };
                        if let Some(declared) = callable_parameters(&self.symbols[symbol].ty) {
                            self.symbols[symbol].kind = 12;
                            // Keep the declared ownership/access contract for help;
                            // parameters above still index the arrow body's names.
                            self.symbols[symbol].parameters = declared;
                        } else if self.text(n + 1) != ":" {
                            self.symbols[symbol].kind = 12;
                            self.symbols[symbol].parameters = params;
                            self.symbols[symbol].ty = if self.text(close + 1) == ":" {
                                self.written_type(&self.snippet(close + 2, tail))
                            } else {
                                String::new()
                            };
                            self.symbols[symbol].signature = format!(
                                "const {} = {}",
                                self.text(n),
                                self.snippet(end + 1, tail + 1)
                            );
                        }
                    }
                }
            }
        }
        // An interface has the fields of the types it extends. Bases may be
        // declared later, so repeat until no interface gains a field.
        for _ in 0..16 {
            let mut added = false;
            for (owner, bases) in &interface_bases {
                for base in bases {
                    let inherited = self
                        .symbols
                        .iter()
                        .filter(|symbol| symbol.kind == 8 && symbol.owner.as_deref() == Some(base.as_str()))
                        .cloned()
                        .collect::<Vec<_>>();
                    for mut field in inherited {
                        if self.symbols.iter().any(|symbol| {
                            symbol.kind == 8 && symbol.owner.as_deref() == Some(owner.as_str()) && symbol.name == field.name
                        }) {
                            continue;
                        }
                        field.owner = Some(owner.clone());
                        self.symbols.push(field);
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        // Every function expression declares its parameters in its body scope.
        for keyword in 0..self.tokens.len() {
            if self.text(keyword) != "function" || self.text(keyword + 1) != "(" {
                continue;
            }
            let Some(close) = self.pairs[keyword + 1] else { continue };
            let body = if self.text(close + 1) == ":" { self.end_type(close + 2) } else { close + 1 };
            if self.text(body) != "{" {
                continue;
            }
            let Some(scope) = self
                .scopes
                .iter()
                .position(|scope| scope.span.start == self.tokens[body].span.start)
            else {
                continue;
            };
            let (start, end) = (keyword + 2, close);
            if self.symbols.iter().enumerate().any(|(id, symbol)| {
                self.parameter_symbols.contains(&id)
                    && symbol.span.start >= self.tokens[start].span.start
                    && symbol.span.start < self.tokens[end].span.start
                    && self.contains_scope(scope, symbol.scope)
            }) {
                continue;
            }
            self.parameters(start, end, scope);
        }
        // A catch clause declares its error name in the handler body.
        for keyword in 0..self.tokens.len() {
            if self.text(keyword) != "catch"
                || self.text(keyword + 1) != "("
                || !name(self.text(keyword + 2))
                || self.text(keyword + 3) != ")"
                || self.text(keyword + 4) != "{"
            {
                continue;
            }
            let binding = keyword + 2;
            let body = keyword + 4;
            let Some(scope) = self
                .scopes
                .iter()
                .position(|scope| scope.span.start == self.tokens[body].span.start)
            else {
                continue;
            };
            // The caught type comes from checking: the first checked use of
            // the name in the handler carries it.
            let close = self.pairs[body].unwrap_or(self.tokens.len());
            let ty = (body + 1..close)
                .filter(|&use_site| self.text(use_site) == self.text(binding))
                .find_map(|use_site| self.checked_view(use_site).map(str::to_owned))
                .unwrap_or_default();
            let signature = format!(
                "(catch) {}{}",
                self.text(binding),
                if ty.is_empty() { String::new() } else { format!(": {ty}") }
            );
            self.add(binding, 13, ty, signature, self.tokens[binding].span, self.tokens[binding].span.start, scope);
        }
        // Every arrow has its own parameter scope, including concise bodies.
        for arrow in 0..self.tokens.len() {
            if self.text(arrow) != "=>" {
                continue;
            }
            let mut header = arrow;
            // A result annotation may intervene between the parameter list and arrow.
            while header > 0
                && self.text(header - 1) != ")"
                && self.text(header - 1) != ";"
                && self.text(header - 1) != "="
            {
                header -= 1;
            }
            if arrow == 0 {
                continue;
            }
            let single = name(self.text(arrow - 1))
                && (header == 0
                    || (self.text(arrow.saturating_sub(2)) != ":"
                        && self.text(arrow.saturating_sub(2)) != ")"));
            let (start, end) = if single {
                (arrow - 1, arrow)
            } else {
                let close = if self.text(arrow - 1) == ")" {
                    arrow - 1
                } else {
                    header.saturating_sub(1)
                };
                let Some(open) = self.pairs.get(close).copied().flatten() else {
                    continue;
                };
                if self.text(open) != "(" {
                    continue;
                }
                (open + 1, close)
            };
            let scope = if self.text(arrow + 1) == "{" {
                let Some(scope) = self
                    .scopes
                    .iter()
                    .position(|scope| scope.span.start == self.tokens[arrow + 1].span.start)
                else {
                    continue;
                };
                scope
            } else {
                let begin = arrow + 1;
                if begin >= self.tokens.len() {
                    continue;
                }
                let mut end = begin;
                while end < self.tokens.len()
                    && !matches!(self.text(end), "," | ";" | ")" | "]" | "}")
                {
                    if matches!(self.text(end), "(" | "[" | "{")
                        && let Some(close) = self.pairs[end]
                    {
                        end = close + 1;
                    } else {
                        end += 1;
                    }
                }
                if end == begin {
                    continue;
                }
                let parent = self.tokens[begin].scope;
                let scope = self.scopes.len();
                self.scopes.push(Scope {
                    parent: Some(parent),
                    span: Span::new(
                        self.tokens[begin].span.start,
                        self.tokens[end.saturating_sub(1)].span.end,
                    ),
                });
                for token in &mut self.tokens[begin..end] {
                    if token.scope == parent {
                        token.scope = scope;
                    }
                }
                scope
            };
            if self.symbols.iter().enumerate().any(|(id, symbol)| {
                self.parameter_symbols.contains(&id)
                    && symbol.span.start >= self.tokens[start].span.start
                    && symbol.span.start < self.tokens[end].span.start
                    && self.contains_scope(scope, symbol.scope)
            }) {
                continue;
            }
            self.parameters(start, end, scope);
        }
        // Resolve straightforward initializers after all forward declarations exist.
        for id in 0..self.tokens.len() {
            if self.declaration_keyword(id).is_some()
                && self.text(id + 2) == "="
                && let Some(symbol) = self.symbols.iter().position(|s| {
                    s.span.start == self.tokens[id + 1].span.start
                        && s.ty.is_empty()
                        && s.kind != 12
                })
            {
                let ty = self.initializer_type(id + 3, self.tokens[id].span.start);
                if !ty.is_empty() {
                    self.symbols[symbol].ty = ty.clone();
                    self.symbols[symbol].signature =
                        format!("{} {}: {ty}", self.declaration_keyword(id).unwrap_or("const"), self.text(id + 1));
                }
            }
        }
        for symbol in &mut self.symbols {
            if symbol.kind == 13
                && let Some(parameters) = callable_parameters(&symbol.ty)
            {
                symbol.kind = 12;
                symbol.parameters = parameters;
            }
        }
    }
    fn initialized_parameter_scope(&mut self, start: usize, end: usize, body: usize) -> usize {
        if !(start..end).any(|token| self.text(token) == "=") {
            return body;
        }
        let header_start = self.tokens[start].span.start;
        let header_end = self.tokens[end - 1].span.end;
        let body_scope = (self.scopes[body].span.start > header_end).then_some(body);
        let parent = body_scope.map_or(body, |scope| self.scopes[scope].parent.unwrap_or(0));
        let extent = body_scope.map_or(header_end, |scope| self.scopes[scope].span.end);
        let scope = self.scopes.len();
        self.scopes.push(Scope {
            parent: Some(parent),
            span: Span::new(header_start, extent),
        });
        for child in &mut self.scopes[..scope] {
            if child.parent == Some(parent)
                && header_start <= child.span.start
                && child.span.start < extent
            {
                child.parent = Some(scope);
            }
        }
        scope
    }
    fn parameter_end(&self, mut token: usize, end: usize) -> usize {
        while token < end && self.text(token) != "," {
            if matches!(self.text(token), "(" | "[" | "{" | "${")
                && let Some(close) = self.pairs[token]
            {
                token = close + 1;
            } else if self.text(token) == "<"
                && let Some(open) = token
                    .checked_sub(1)
                    .and_then(|callee| self.member_call_open(callee))
            {
                token = open;
            } else {
                token += 1;
            }
        }
        token.min(end)
    }
    fn pattern_binding_names(&self,open:usize,close:usize)->Vec<usize>{
        fn collect(index:&SourceIndex,open:usize,close:usize,names:&mut Vec<usize>,depth:usize){
            if depth>32{return;}
            let record=index.text(open)=="{";
            let mut token=open+1;
            while token<close {
                if index.text(token)==","{token+=1;continue;}
                if record && name(index.text(token)) && index.text(token+1)==":" {token+=2;}
                if matches!(index.text(token),"["|"{") && let Some(end)=index.pairs[token] {
                    collect(index,token,end,names,depth+1);token=end+1;
                }else if name(index.text(token)){names.push(token);token+=1;}else{token+=1;}
                if index.text(token)=="="{token=index.parameter_end(token+1,close);}
                if index.text(token)==","{token+=1;}
            }
        }
        let mut names=Vec::new();collect(self,open,close,&mut names,0);names
    }

    fn parameters(&mut self, start: usize, end: usize, scope: usize) -> Vec<String> {
        let scope = self.initialized_parameter_scope(start, end, scope);
        let mut n = start;
        let mut parameters = vec![];
        while n < end {
            let begin = n;
            let rest = self.text(n) == "...";
            if rest {
                n += 1;
            }
            // A destructured parameter declares the names of its pattern;
            // their types come from the checked positions.
            if matches!(self.text(n), "[" | "{")
                && let Some(close) = self.pairs[n]
                && close < end
            {
                for inner in self.pattern_binding_names(n,close) {
                    {
                        self.add(
                            inner,
                            13,
                            String::new(),
                            format!("const {}", self.text(inner)),
                            self.tokens[inner].span,
                            self.tokens[inner].span.start,
                            scope,
                        );
                    }
                }
                let colon = close + 1;
                let type_end = if self.text(colon) == ":" { self.end_type(colon + 1).min(end) } else { colon };
                let parameter_end = if self.text(type_end) == "=" { self.parameter_end(type_end + 1, end) } else { type_end };
                parameters.push(self.snippet(begin, parameter_end));
                n = parameter_end + 1;
                continue;
            }
            let access_begin = n;
            let optional = self.text(n + 1) == "?";
            let colon = n + if optional { 2 } else { 1 };
            let annotated = self.text(colon) == ":";
            let type_end = if annotated {
                self.end_type(colon + 1).min(end)
            } else {
                n + 1
            };
            let parameter_end = if self.text(type_end) == "=" {
                self.parameter_end(type_end + 1, end)
            } else {
                type_end
            };
            if name(self.text(n)) {
                let declared = if annotated {
                    self.written_type(&self.snippet(colon + 1, type_end))
                } else {
                    String::new()
                };
                let ty = if optional {
                    optional_parameter_type(&declared)
                } else {
                    declared
                };
                let signature = self.snippet(begin, parameter_end);
                let access = self.snippet(access_begin, n);
                let body_signature = format!(
                    "{}{}{}",
                    if access.is_empty() {
                        String::new()
                    } else {
                        format!("{access} ")
                    },
                    self.text(n),
                    if ty.is_empty() {
                        String::new()
                    } else {
                        format!(": {ty}")
                    }
                );
                let parameter = self.add(
                    n,
                    13,
                    ty,
                    body_signature,
                    self.tokens[n].span,
                    self.tokens[n].span.start,
                    scope,
                );
                self.parameter_symbols.insert(parameter);
                self.parameter_available_from.insert(
                    parameter,
                    self.tokens[parameter_end.saturating_sub(1)].span.end,
                );
                parameters.push(signature);
            }
            n = parameter_end + 1;
        }
        parameters
    }
    // Locate generic member argument delimiters only; checked facts supply types.
    fn member_call_open(&self, method: usize) -> Option<usize> {
        let mut next = method.checked_add(1)?;
        if self.text(next) == "?." {
            next += 1;
        }
        if self.text(next) == "(" {
            return Some(next);
        }
        let checked_generic = self.tokens.get(method).is_some_and(|token| {
            self.checked_generic_calls
                .iter()
                .any(|fact| fact.callee_span == token.span)
        });
        let array_constructor = self.text(method) == "of"
            && method >= 2
            && member_separator(self.text(method - 1))
            && self.text(method - 2) == "Array";
        let set_constructor = matches!(self.text(method), "Set" | "Map") && method > 0 && self.text(method - 1) == "new";
        if !(checked_generic || array_constructor || set_constructor) || self.text(next) != "<" {
            return None;
        }
        let mut depth = 0usize;
        for token in next..self.tokens.len() {
            match self.text(token) {
                "<" => depth += 1,
                ">" => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return (self.text(token + 1) == "(").then_some(token + 1);
                    }
                }
                ";" | "=" => return None,
                _ => {}
            }
        }
        None
    }
    fn call_callee(&self, open: usize) -> Option<usize> {
        let mut previous = open.checked_sub(1)?;
        if self.text(previous) == "?." {
            previous = previous.checked_sub(1)?;
        }
        while self.text(previous) == "]" {
            let mut depth = 1usize;
            while depth != 0 {
                previous = previous.checked_sub(1)?;
                match self.text(previous) {
                    "]" => depth += 1,
                    "[" => depth -= 1,
                    _ => (),
                }
            }
            previous = previous.checked_sub(1)?;
        }
        if name(self.text(previous)) {
            return Some(previous);
        }
        if self.text(previous) != ">" {
            return None;
        }
        let mut depth = 0usize;
        for token in (0..open).rev() {
            match self.text(token) {
                ">" => depth += 1,
                "<" => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        let mut method = token.checked_sub(1)?;
                        if self.text(method) == "?." {
                            method = method.checked_sub(1)?;
                        }
                        return (self.member_call_open(method) == Some(open)).then_some(method);
                    }
                }
                ";" | "=" => return None,
                _ => {}
            }
        }
        None
    }
    fn primitive_method_type(method: &str, receiver: String) -> String {
        match (method, receiver.as_str()) {
            ("valueOf", "string" | "boolean" | "number" | "int") => receiver,
            ("toString", "string" | "boolean" | "int" | "number") => "string".into(),
            ("toLocaleString", "int" | "number") => "string".into(),
            ("isWellFormed", "string") => "boolean".into(),
            ("toWellFormed", "string") => "string".into(),
            ("normalize", "string") => "string".into(),
            ("toFixed" | "toExponential" | "toPrecision", "number") => "string".into(),
            ("join" | "toString" | "toLocaleString", ty) if array_element_type(ty).is_some() => "string".into(),
            ("unshift", ty) if array_element_type(ty).is_some() => "int".into(),
            ("entries", ty) if array_element_type(ty).is_some() => format!("[int,{}][]",array_element_type(ty).unwrap()),
            ("keys", ty) if array_element_type(ty).is_some() => "int[]".into(),
            ("values", ty) if array_element_type(ty).is_some() => receiver,
            ("splice" | "toSorted", ty) if array_element_type(ty).is_some() => receiver,
            ("sort", ty) if array_element_type(ty).is_some() => "void".into(),
            ("at", "string") => "string | undefined".into(),
            // These keep the receiver's element type, whatever it is.
            ("slice" | "concat" | "filter" | "with" | "toSpliced" | "toReversed", ty) if array_element_type(ty).is_some() => receiver,
            ("includes" | "some" | "every", ty) if array_element_type(ty).is_some() => "boolean".into(),
            ("indexOf" | "lastIndexOf" | "findIndex" | "findLastIndex", ty) if array_element_type(ty).is_some() => "int".into(),
            ("pop" | "shift" | "at" | "find" | "findLast", ty) => {
                array_removal_result(ty).unwrap_or_default()
            }
            _ if receiver == "string" => builtins_for(&receiver)
                .into_iter()
                .find(|entry| entry["name"] == method)
                .and_then(|entry| {
                    entry["signature"]
                        .as_str()
                        .and_then(|s| s.rsplit_once("): ").map(|(_, result)| result.to_owned()))
                })
                .unwrap_or_default(),
            _ => String::new(),
        }
    }
    fn primitive_method_accepts_arguments(method: &str, receiver: &str) -> bool {
        if matches!(
            method,
            "join"
                | "at"
                | "find"
                | "findLast"
                | "unshift"
                | "splice"
                | "sort"
                | "toSorted"
                | "flat"
                | "flatMap"
                | "slice"
                | "concat"
                | "filter"
                | "with"
                | "toSpliced"
                | "includes"
                | "some"
                | "every"
                | "indexOf"
                | "lastIndexOf"
                | "findIndex"
                | "findLastIndex"
        ) && array_element_type(receiver).is_some()
        {
            return true;
        }
        if receiver == "string" {
            return builtins_for(receiver)
                .into_iter()
                .find(|entry| entry["name"] == method)
                .and_then(|entry| {
                    entry["parameters"]
                        .as_array()
                        .map(|parameters| !parameters.is_empty())
                })
                .unwrap_or(false);
        }
        matches!(
            (method, receiver),
            ("toString", "int" | "number") | ("toFixed" | "toExponential" | "toPrecision", "number")
        )
    }
    // Recognize only complete simple expressions; do not infer binary expressions
    // from their first operand. Primitive calls preserve this rule.
    fn set_constructor_type(&self, id: usize, offset: usize, depth: usize) -> Option<(String, usize)> {
        if depth > 8 || self.text(id) != "new" || !matches!(self.text(id + 1), "Set" | "Map") { return None; }
        let open = self.member_call_open(id + 1)?;
        let close = self.pairs.get(open).copied().flatten()?;
        let key = if self.text(id + 2) == "<" {
            self.snippet(id + 3, open - 1)
        } else {
            let source = self.infer(open + 1, offset, depth + 1);
            if self.text(id + 1) == "Map" {
                if let Some(types) = source.strip_prefix("Map<").and_then(|s| s.strip_suffix('>')) { types.into() }
                else { array_element_type(&source)?.strip_prefix('[')?.strip_suffix(']')?.into() }
            } else if source == "string" { source } else if let Some(key) = array_element_type(&source) { key.into() }
            else { source.strip_prefix("Set<")?.strip_suffix('>')?.into() }
        };
        Some((format!("{}<{key}>", self.text(id + 1)), close + 1))
    }
    fn simple_expression_type(
        &self,
        id: usize,
        offset: usize,
        depth: usize,
    ) -> Option<(String, usize)> {
        if depth > 8 {
            return None;
        }
        let generic = self.tokens.get(id).and_then(|token| {
            self.checked_generic_calls
                .iter()
                .find(|fact| fact.callee_span == token.span && fact.result.is_some())
        });
        let (mut ty, mut end) = if let Some(result) = self.checked_expression_type(id) {
            result
        } else if let Some(fact) = generic {
            let close = self
                .tokens
                .iter()
                .position(|token| token.span.end == fact.span.end)?;
            (fact.result.clone()?, close + 1)
        } else if let Some(result) = self.set_constructor_type(id, offset, depth) {
            result
        } else if self.text(id).starts_with('`') {
            let end = self.templates.iter().find(|(open, _)| *open == id)?.1;
            ("string".into(), end + 1)
        } else if matches!(self.text(id), "typeof" | "void") {
            let (_, end) = self.simple_expression_type(id + 1, offset, depth + 1)?;
            (
                if self.text(id) == "typeof" {
                    "string"
                } else {
                    "undefined"
                }
                .into(),
                end,
            )
        } else if matches!(self.text(id), "-" | "+") {
            // An expression starts at this token, so the sign is a prefix sign.
            let (operand, end) = self.simple_expression_type(id + 1, offset, depth + 1)?;
            if !matches!(operand.as_str(), "int" | "number") {
                return None;
            }
            (operand, end)
        } else if self.text(id) == "Array"
            && self.text(id + 1) == "."
            && self.text(id + 2) == "from"
            && self.text(id + 3) == "("
        {
            // One argument copies an array or splits a string into characters;
            // a mapping function's result comes from the checked call.
            let close = self.pairs.get(id + 3).copied().flatten()?;
            let (source, end) = self.simple_expression_type(id + 4, offset, depth + 1)?;
            let result = if let Some(result)=self.checked_method_result(close) {
                result.to_owned()
            } else if end == close {
                if source == "string" {
                    "string[]".to_owned()
                } else if source.ends_with("[]") {
                    source
                } else if let Some(element)=iterable_element_type(&source) {
                    let grouped=tsrust_compiler::type_syntax::parse(&element).is_ok_and(|annotation|matches!(annotation.kind,tsrust_compiler::type_syntax::ValueAnnotationKind::Union(_)));
                    if grouped{format!("({element})[]")}else{format!("{element}[]")}
                } else {
                    return None;
                }
            } else {
                self.checked_method_result(close)?.to_owned()
            };
            (result, close + 1)
        } else if self.text(id) == "Array" && self.text(id + 1) == "." && self.text(id + 2) == "of"
        {
            let open = self.member_call_open(id + 2)?;
            let close = self.pairs.get(open).copied().flatten()?;
            (self.checked_method_result(close)?.to_owned(), close + 1)
        } else if self.text(id) == "Object" && self.text(id + 1) == "." && matches!(self.text(id + 2), "values" | "entries") && self.text(id + 3) == "(" {
            // The element type depends on the record's fields; use the checked result.
            let close = self.pairs.get(id + 3).copied().flatten()?;
            (self.checked_method_result(close)?.to_owned(), close + 1)
        } else if self.text(id) == "Object" && self.text(id + 1) == "." && self.text(id + 2) == "keys" && self.text(id + 3) == "(" {
            let close = self.pairs.get(id + 3).copied().flatten()?;
            ("string[]".into(), close + 1)
        } else if ((self.text(id) == "Array" && self.text(id + 2) == "isArray")
            || (self.text(id) == "Object" && matches!(self.text(id + 2), "is" | "hasOwn")))
            && self.text(id + 1) == "."
            && self.text(id + 3) == "("
        {
            let close = self.pairs.get(id + 3).copied().flatten()?;
            ("boolean".into(), close + 1)
        } else if self.text(id) == "Number"
            && self.text(id + 1) == "."
            && matches!(self.text(id + 2), "parseFloat" | "parseInt")
            && self.text(id + 3) == "("
        {
            let close = self.pairs.get(id + 3).copied().flatten()?;
            ("number".into(), close + 1)
        } else if self.text(id) == "String"
            && self.text(id + 1) == "."
            && matches!(self.text(id + 2), "fromCodePoint" | "fromCharCode" | "raw")
            && self.text(id + 3) == "("
        {
            let close = self.pairs.get(id + 3).copied().flatten()?;
            ("string".into(), close + 1)
        } else if self.text(id) == "Math" && self.text(id + 1) == "." {
            // Every Math function and constant is a number.
            if self.text(id + 3) == "(" {
                let close = self.pairs.get(id + 3).copied().flatten()?;
                ("number".into(), close + 1)
            } else {
                ("number".into(), id + 3)
            }
        } else if self.text(id) == "(" {
            let close = self.pairs.get(id).copied().flatten()?;
            let (ty, end) = self.simple_expression_type(id + 1, offset, depth + 1)?;
            if end != close {
                return None;
            }
            (ty, close + 1)
        } else {
            let end = if self.text(id) == "[" {
                self.pairs.get(id).copied().flatten()? + 1
            } else if self.text(id + 1) == "(" {
                self.pairs.get(id + 1).copied().flatten()? + 1
            } else {
                id + 1
            };
            (self.infer(id, offset, depth + 1), end)
        };
        loop {
            if self.text(end) == "!" {
                ty = self.checked_method_result(end).map(str::to_owned).unwrap_or_else(|| nonnullable_assertion_type(&ty));
                if ty.is_empty() { return None; }
                end += 1;
                continue;
            }
            if self.text(end) == "(" && end > id && self.text(end - 1) == "]" {
                let close = self.pairs.get(end).copied().flatten()?;
                ty = callable_result_type(&ty)?;
                end = close + 1;
                continue;
            }
            if self.text(end) == "[" && ty == "string" {
                // A string position is a one-character string.
                let close = self.pairs.get(end).copied().flatten()?;
                end = close + 1;
                continue;
            }
            if self.text(end) == "[" {
                let close = self.pairs.get(end).copied().flatten()?;
                ty = if let Ok(annotation) = tsrust_compiler::type_syntax::parse(&ty) {
                    if let tsrust_compiler::type_syntax::ValueAnnotationKind::Tuple(positions) = annotation.kind {
                        let position = self.text(end + 1).parse::<usize>().ok()?;
                        if close != end + 2 { return None; }
                        let field = positions.get(position)?;
                        ty[field.span.start..field.span.end].to_owned()
                    } else { array_element_type(&ty)?.to_owned() }
                } else { array_element_type(&ty)?.to_owned() };
                end = close + 1;
                continue;
            }
            if self.text(end) == "." && name(self.text(end + 1)) && self.text(end + 2) != "(" {
                let field = end + 1;
                ty = self.checked_view(field).map(str::to_owned).or_else(|| {
                    self.symbols
                        .iter()
                        .find(|symbol| {
                            symbol.owner.as_deref() == Some(ty.as_str())
                                && symbol.name == self.text(field)
                        })
                        .map(|symbol| symbol.ty.clone())
                })?;
                end += 2;
                continue;
            }
            if self.text(end) != "." || self.text(end + 2) != "(" {
                break;
            }
            let close = self.pairs.get(end + 2).copied().flatten()?;
            let callable = self.field_callable_result(&ty, self.text(end + 1), end + 1);
            if callable.is_none()
                && close != end + 3
                && !Self::primitive_method_accepts_arguments(self.text(end + 1), &ty)
            {
                return None;
            }
            ty = self
                .checked_method_result(close)
                .map(str::to_owned)
                .or(callable)
                .unwrap_or_else(|| Self::primitive_method_type(self.text(end + 1), ty));
            if ty.is_empty() {
                return None;
            }
            end = close + 1;
        }
        Some((ty, end))
    }
    /// Token after a comparison operator that starts at `id`. The scanner
    /// splits operators into single symbols, so adjacent spans are required.
    fn after_comparison(&self, id: usize) -> Option<usize> {
        let joined = |count: usize| {
            (0..count).all(|step| self.tokens.get(id + step + 1).is_some())
                && (1..count).all(|step| self.tokens[id + step - 1].span.end == self.tokens[id + step].span.start)
        };
        for operator in [["=", "=", "="].as_slice(), &["!", "=", "="], &["=", "="], &["!", "="], &["<", "="], &[">", "="], &["<"], &[">"]] {
            if operator.iter().enumerate().all(|(step, text)| self.text(id + step) == *text) && joined(operator.len()) {
                return Some(id + operator.len());
            }
        }
        None
    }
    fn initializer_type(&self, id: usize, offset: usize) -> String {
        let Some((ty, end)) = self.simple_expression_type(id, offset, 0) else {
            return String::new();
        };
        if matches!(self.text(end), ";" | "," | "}" | "") {
            ty
        } else if let Some(right) = self.after_comparison(end)
            && self
                .simple_expression_type(right, offset, 0)
                .is_some_and(|(_, after)| matches!(self.text(after), ";" | "," | "}" | ""))
        {
            // One comparison of two typed operands.
            "boolean".into()
        } else {
            String::new()
        }
    }
    fn infer(&self, id: usize, offset: usize, depth: usize) -> String {
        if depth > 8 {
            return String::new();
        }
        if let Some(ty) = self.checked_view(id) {
            if self.text(id + 1) == "(" {
                return callable_result_type(ty).unwrap_or_else(|| ty.into());
            }
            return ty.into();
        }
        if let Some((ty, _)) = self.set_constructor_type(id, offset, depth) { return ty; }
        let text = self.text(id);
        if matches!(text.as_bytes().first(), Some(b'\'' | b'"' | b'`')) {
            return "string".into();
        }
        if text == "null" {
            return "null".into();
        }
        if text == "undefined" {
            return self
                .resolve_name(text, offset)
                .map_or_else(|| "undefined".into(), |s| self.symbols[s].ty.clone());
        }
        if matches!(text, "true" | "false") {
            return "boolean".into();
        }
        if text.bytes().next().is_some_and(|c| c.is_ascii_digit()) {
            return if text.contains(['.', 'e', 'E']) {
                "number"
            } else {
                "int"
            }
            .into();
        }
        if text == "[" {
            if self.text(id + 1) == "]" {
                return "unknown[]".into();
            }
            let ty = self.infer(id + 1, offset, depth + 1);
            return if ty.is_empty() { ty } else { format!("{ty}[]") };
        }
        if text == "clone" && self.text(id + 1) == "(" {
            return self.infer(id + 2, offset, depth + 1);
        }
        // Global number names, unless the program declares the name itself.
        if matches!(text, "parseInt" | "parseFloat" | "isNaN" | "isFinite" | "NaN" | "Infinity")
            && !(id > 0 && member_separator(self.text(id - 1)))
            && self.resolve_token(id).is_none()
        {
            let call = self.text(id + 1) == "(";
            match text {
                "parseInt" | "parseFloat" if call => return "number".into(),
                "isNaN" | "isFinite" if call => return "boolean".into(),
                "NaN" | "Infinity" => return "number".into(),
                _ => {}
            }
        }
        if matches!(text, "String" | "Boolean") && self.text(id + 1) == "(" {
            return if text == "String" {
                "string"
            } else {
                "boolean"
            }
            .into();
        }
        if text == "Number" && self.text(id + 1) == "(" {
            return "number".into();
        }
        if name(text)
            && let Some(s) = self.resolve_name(text, offset)
        {
            let symbol = &self.symbols[s];
            if self.text(id + 1) == "("
                && !symbol.signature.starts_with("function ")
                && !symbol.signature.contains(" = ")
                && let Some(result) = callable_result_type(&symbol.ty)
            {
                return result;
            }
            return symbol.ty.clone();
        }
        String::new()
    }
    fn scope_at(&self, offset: usize) -> usize {
        if offset == self.source.len() {
            return self.end_scope;
        }
        self.scopes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.span.start <= offset && offset < s.span.end)
            .max_by_key(|(_, s)| s.span.start)
            .map_or(0, |(i, _)| i)
    }
    fn contains_scope(&self, mut child: usize, parent: usize) -> bool {
        loop {
            if child == parent {
                return true;
            }
            if let Some(next) = self.scopes[child].parent {
                child = next;
            } else {
                return false;
            }
        }
    }
    fn scope_depth(&self, mut scope: usize) -> usize {
        let mut depth = 0;
        while let Some(parent) = self.scopes[scope].parent {
            depth += 1;
            scope = parent;
        }
        depth
    }
    fn resolve_name(&self, text: &str, offset: usize) -> Option<usize> {
        let scope = self.scope_at(offset);
        self.symbols
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                s.name == text
                    && s.owner.is_none()
                    && self.contains_scope(scope, s.scope)

            })
            .max_by_key(|(_, s)| (self.scope_depth(s.scope), s.span.start))
            .and_then(|(id, _)| self.parameter_available(id, offset).then_some(id))
    }
    fn parameter_available(&self, symbol: usize, offset: usize) -> bool {
        self.parameter_available_from
            .get(&symbol)
            .is_none_or(|&available| available <= offset)
    }
    fn offset(&self, position: Position) -> Option<usize> {
        let mut start = 0;
        for _ in 0..position.line {
            start += self.source.get(start..)?.find('\n')? + 1;
        }
        let line = self
            .source
            .get(start..)?
            .split('\n')
            .next()?
            .trim_end_matches('\r');
        let mut units = 0;
        for (n, ch) in line.char_indices() {
            if units == position.character {
                return Some(start + n);
            }
            units += ch.len_utf16() as u32;
        }
        (units == position.character).then_some(start + line.len())
    }
    fn token_at(&self, offset: usize) -> Option<usize> {
        self.tokens
            .iter()
            .position(|t| t.span.start <= offset && offset < t.span.end)
            .or_else(|| {
                self.tokens
                    .iter()
                    .rposition(|t| t.span.end == offset && name(&t.text))
            })
    }
    fn excluded(&self, offset: usize) -> bool {
        self.comments
            .iter()
            .any(|(s, _)| s.start <= offset && offset < s.end)
            || self
                .token_at(offset)
                .is_some_and(|id| self.tokens[id].literal)
            || self
                .tokens
                .last()
                .is_some_and(|token| token.unterminated_string && token.span.end == offset)
    }
    fn receiver_type(&self, dot: usize) -> String {
        if dot == 0 {
            return String::new();
        }
        if self.text(dot) == "?." {
            return self.checked_optional_receiver(dot - 1).unwrap_or("").into();
        }
        // A successful contiguous hop has its own checked payload view.
        if let Some(ty) = self.checked_view(dot - 1) {
            return ty.into();
        }
        self.receiver_type_end(dot - 1, 0)
    }
    fn field_callable_result(&self, receiver: &str, field: &str, token: usize) -> Option<String> {
        let symbol = self
            .symbols
            .iter()
            .find(|symbol| symbol.owner.as_deref() == Some(receiver) && symbol.name == field)?;
        callable_result_type(self.checked_view(token).unwrap_or(&symbol.ty))
    }
    fn receiver_type_end(&self, n: usize, depth: usize) -> String {
        if let Some(ty) = self.checked_method_result(n) {
            return ty.into();
        }
        if let Some(ty) = self.checked_view(n) {
            return ty.into();
        }
        if depth > 8 {
            return String::new();
        }
        if self.text(n) == "!" && n > 0 {
            return nonnullable_assertion_type(&self.receiver_type_end(n - 1, depth + 1));
        }
        if self.templates.iter().any(|(_, end)| *end == n) {
            return "string".into();
        }
        if self.text(n) == "]"
            && let Some(open) = self.pairs[n]
        {
            if open > 0 {
                let receiver = self.receiver_type_end(open - 1, depth + 1);
                if let Some(element) = array_element_type(&receiver) {
                    return element.into();
                }
                // A string position is a one-character string.
                if receiver == "string" {
                    return receiver;
                }
                if !receiver.is_empty() {
                    return String::new();
                }
            }
            return self.infer(open, self.tokens[n].span.start, depth + 1);
        }
        if self.text(n) == ")"
            && let Some(open) = self.pairs[n]
        {
            if open > 0 && self.text(open - 1) == "]" {
                let callee = self.receiver_type_end(open - 1, depth + 1);
                if let Some(result) = callable_result_type(&callee) {
                    return result;
                }
            }
            if open >= 3
                && self.text(open - 3) == "Object"
                && self.text(open - 2) == "."
                && matches!(self.text(open - 1), "values" | "entries")
                && let Some(close) = self.pairs.get(open).copied().flatten()
                && let Some(result) = self.checked_method_result(close)
            {
                return result.to_owned();
            }
            if open >= 3 && self.text(open - 3) == "Object" && self.text(open - 2) == "." && self.text(open - 1) == "keys" {
                return "string[]".into();
            }
            if open >= 3
                && self.text(open - 2) == "."
                && ((self.text(open - 3) == "Array" && self.text(open - 1) == "isArray")
                    || (self.text(open - 3) == "Object" && matches!(self.text(open - 1), "is" | "hasOwn")))
            {
                return "boolean".into();
            }
            if open >= 3
                && self.text(open - 3) == "Number"
                && self.text(open - 2) == "."
                && matches!(self.text(open - 1), "parseFloat" | "parseInt")
            {
                return "number".into();
            }
            if open >= 3
                && self.text(open - 3) == "String"
                && self.text(open - 2) == "."
                && matches!(self.text(open - 1), "fromCodePoint" | "fromCharCode" | "raw")
            {
                return "string".into();
            }
            if open >= 3 && self.text(open - 3) == "Math" && self.text(open - 2) == "." {
                return "number".into();
            }
            if open >= 3 && self.text(open - 2) == "." {
                let receiver = self.receiver_type_end(open - 3, depth + 1);
                if let Some(result) =
                    self.field_callable_result(&receiver, self.text(open - 1), open - 1)
                {
                    return result;
                }
                if open + 1 != n
                    && !Self::primitive_method_accepts_arguments(self.text(open - 1), &receiver)
                {
                    return String::new();
                }
                return Self::primitive_method_type(self.text(open - 1), receiver);
            }
            if let Some((ty, end)) =
                self.simple_expression_type(open + 1, self.tokens[n].span.start, depth + 1)
                && end == n
            {
                return ty;
            }
            return self.infer(open.saturating_sub(1), self.tokens[n].span.start, depth + 1);
        }
        if n > 1
            && self.text(n - 1) == "."
            && let Some(s) = self.resolve_token(n)
        {
            return self.symbols[s].ty.clone();
        }
        self.infer(n, self.tokens[n].span.start, depth + 1)
    }
    fn resolve_token(&self, id: usize) -> Option<usize> {
        if self.checked_control_labels.iter().any(|(span,_)|*span==self.tokens[id].span) {return None;}
        if let Some(s) = self
            .symbols
            .iter()
            .position(|s| s.span.start == self.tokens[id].span.start)
        {
            return Some(s);
        }
        if id > 0 && member_separator(self.text(id - 1)) {
            let ty = self.receiver_type(id - 1);
            return self
                .symbols
                .iter()
                .position(|s| s.owner.as_deref() == Some(ty.as_str()) && s.name == self.text(id));
        }
        if self.text(id + 1) == ":" {
            return None;
        } // Object literal keys are not local variables.
        self.resolve_name(self.text(id), self.tokens[id].span.start)
    }
    fn range(&self, span: Span) -> Range {
        self.lines
            .range(&self.source, span)
            .expect("source index spans originate at UTF-8 boundaries")
    }
    fn documentation(&self, s: &Symbol) -> String {
        format!(
            "```tsrust\n{}\n```{}",
            s.signature,
            if s.docs.is_empty() {
                String::new()
            } else {
                format!("\n\n{}", s.docs)
            }
        )
    }
    pub(crate) fn import_native(&mut self, module: &str, exported: &str, local: &str, span: Span) {
        let Ok(binding) = tsrust_compiler::native_bindings::lookup(module, exported) else {
            return;
        };
        let Some(catalog) = builtins_for(module)
            .into_iter()
            .find(|entry| entry["name"] == exported)
        else {
            return;
        };
        let type_name = |ty| match ty {
            tsrust_compiler::native_bindings::NativeType::String => "string",
            tsrust_compiler::native_bindings::NativeType::Boolean => "boolean",
            tsrust_compiler::native_bindings::NativeType::Int => "int",
            tsrust_compiler::native_bindings::NativeType::Number => "number",
            tsrust_compiler::native_bindings::NativeType::Void => "void",
            tsrust_compiler::native_bindings::NativeType::OwnedVoidCallback => "Owned<Function<void>>",
            tsrust_compiler::native_bindings::NativeType::SharedCallback => "Function",
            tsrust_compiler::native_bindings::NativeType::ObservedValue => "unknown",
        };
        let parameters = binding
            .parameters
            .iter()
            .map(|parameter| {
                format!(
                    "{}{}{}: {}{}",
                    if parameter.rest { "..." } else { "" },
                    parameter.name,
                    if parameter.optional { "?" } else { "" },
                    type_name(parameter.ty),
                    if parameter.rest { "[]" } else { "" }
                )
            })
            .collect::<Vec<_>>();
        let docs = catalog["parameters"]
            .as_array()
            .map(|parameters| {
                parameters
                    .iter()
                    .map(|parameter| parameter["documentation"].as_str().unwrap_or("").to_owned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.native_parameter_docs.insert(self.symbols.len(), docs);
        if let Some(id) = tsrust_compiler::native_bindings::BINDINGS
            .iter()
            .position(|entry| entry.module == module && entry.name == exported)
        {
            self.native_origins.insert(self.symbols.len(), id);
        }
        self.symbols.push(Symbol {
            name: local.into(),
            kind: 12,
            span,
            extent: span,
            scope: 0,
            ty: type_name(binding.result).into(),
            signature: format!(
                "function {local}({}): {}",
                parameters.join(", "),
                type_name(binding.result)
            ),
            docs: catalog["documentation"].as_str().unwrap_or("").into(),
            parameters,
            owner: None,
        });
    }
    pub(crate) fn is_record_declaration(&self, name: &str) -> bool {
        self.symbols
            .iter()
            .any(|s| s.name == name && matches!(s.kind, 10 | 23) && s.scope == 0 && s.owner.is_none())
    }
    pub(crate) fn import_declaration(
        &mut self,
        dependency: &SourceIndex,
        exported: &str,
        local: &str,
        span: Span,
        uri: &str,
        types: &std::collections::BTreeMap<String, String>,
    ) -> bool {
        let Some(declaration) = dependency.symbols.iter().find(|s| {
            s.name == exported && s.scope == 0 && s.owner.is_none() && matches!(s.kind, 10 | 12 | 23)
        }) else {
            return false;
        };
        let mut symbol = declaration.clone();
        if let Some(module) = uri.strip_prefix("tsrust-stdlib:///")
            && let Some(entry) = builtins_for(module).into_iter().find(|entry| entry["name"] == exported)
            && let Some(documentation) = entry["documentation"].as_str()
        {
            symbol.docs = documentation.to_owned();
        }
        if symbol.kind == 12 {
            symbol.ty = remap_imported_type(&symbol.ty, types).unwrap_or_default();
            // Rest labels have no initializer; remap their declared array through
            // the same imported nominal identities as ordinary checked types.
            for parameter in &mut symbol.parameters {
                if parameter.trim_start().starts_with("...")
                    && let Some((name, ty)) = parameter.split_once(':')
                    && let Some(ty) = remap_imported_type(ty, types)
                {
                    let mapped = format!("{name}:{ty}");
                    symbol.signature = symbol.signature.replacen(parameter.as_str(), &mapped, 1);
                    *parameter = mapped;
                }
            }
        }

        symbol.name = local.into();
        if symbol.kind == 10 { symbol.ty = local.into(); }
        symbol.scope = 0;
        symbol.span = span;
        symbol.extent = span;
        symbol.signature =
            symbol
                .signature
                .replacen(&format!(" {exported}"), &format!(" {local}"), 1);
        let id = self.symbols.len();
        self.foreign_definitions.insert(
            id,
            json!({"uri":uri,"range":dependency.range(declaration.span)}),
        );
        self.symbols.push(symbol);
        if matches!(declaration.kind, 10 | 23) {
            self.import_record_fields(dependency, exported, local, span, uri, types);
        }
        true
    }
    fn import_record_fields(
        &mut self,
        dependency: &SourceIndex,
        exported: &str,
        local: &str,
        span: Span,
        uri: &str,
        types: &std::collections::BTreeMap<String, String>,
    ) {
        let mut pending = vec![(exported.to_owned(), local.to_owned())];
        let mut visited = std::collections::BTreeSet::new();
        while let Some((record, owner)) = pending.pop() {
            if !visited.insert((record.clone(), owner.clone())) {
                continue;
            }
            let origin = dependency
                .nominal_definitions
                .get(&record)
                .cloned()
                .or_else(|| {
                    dependency
                        .symbols
                        .iter()
                        .find(|symbol| matches!(symbol.kind, 10 | 23) && symbol.name == record)
                        .map(|symbol| json!({"uri":uri,"range":dependency.range(symbol.span)}))
                });
            if let Some(origin) = origin {
                self.nominal_definitions.insert(owner.clone(), origin);
            }
            let generic = dependency.generic_parameters.get(&record);
            if let Some(parameters) = generic {
                self.generic_parameters
                    .insert(owner.clone(), parameters.clone());
            }
            for (field_id, field) in dependency
                .symbols
                .iter()
                .enumerate()
                .filter(|(_, s)| s.owner.as_deref() == Some(record.as_str()))
            {
                if self
                    .symbols
                    .iter()
                    .any(|s| s.owner.as_deref() == Some(owner.as_str()) && s.name == field.name)
                {
                    continue;
                }
                let ty = if generic.is_some()
                    || owner.starts_with("__tsrust_private_")
                    || (callable_storage_type(&field.ty) || contains_callable_annotation(&field.ty))
                {
                    let mut mappings = types.clone();
                    for symbol in &dependency.symbols {
                        let candidate = if matches!(symbol.kind, 10 | 23) {
                            Some(symbol.name.as_str())
                        } else {
                            symbol.owner.as_deref()
                        };
                        let Some(candidate) = candidate.filter(|candidate| name(candidate)) else {
                            continue;
                        };
                        let identity = mappings
                            .entry(candidate.to_owned())
                            .or_insert_with(|| {
                                if candidate.starts_with("__tsrust_private_") {
                                    candidate.to_owned()
                                } else {
                                    let encoded = uri
                                        .as_bytes()
                                        .iter()
                                        .map(|byte| format!("{byte:02x}"))
                                        .collect::<String>();
                                    format!("__tsrust_private_{encoded}_{candidate}")
                                }
                            })
                            .clone();
                        self.generic_display_names.insert(
                            identity.clone(),
                            dependency
                                .generic_display_names
                                .get(candidate)
                                .cloned()
                                .unwrap_or_else(|| candidate.to_owned()),
                        );
                        if field
                            .ty
                            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                            .any(|token| token == candidate)
                        {
                            pending.push((candidate.to_owned(), identity));
                        }
                    }
                    mappings.insert(record.clone(), owner.clone());
                    if let Some(parameters) = generic {
                        for parameter in parameters {
                            mappings.insert(parameter.clone(), parameter.clone());
                        }
                    }
                    remap_imported_type(&field.ty, &mappings).unwrap_or_default()
                } else {
                    field
                        .ty
                        .split('|')
                        .map(|part| {
                            let base = part.trim_end_matches("[]");
                            let suffix = &part[base.len()..];
                            if matches!(
                                base,
                                "int"
                                    | "number"
                                    | "boolean"
                                    | "string"
                                    | "void"
                                    | "null"
                                    | "undefined"
                            ) {
                                return Some(part.to_owned());
                            }
                            if !dependency.is_record_declaration(base)
                                && !dependency
                                    .symbols
                                    .iter()
                                    .any(|symbol| symbol.owner.as_deref() == Some(base))
                            {
                                return None;
                            }
                            // Private names stay out of lexical completion and cannot capture a local type.
                            let identity = types.get(base).cloned().unwrap_or_else(|| {
                                if base.starts_with('@') {
                                    base.to_owned()
                                } else {
                                    format!("@{uri}#{base}")
                                }
                            });
                            pending.push((base.to_owned(), identity.clone()));
                            Some(format!("{identity}{suffix}"))
                        })
                        .collect::<Option<Vec<_>>>()
                        .map(|parts| parts.join("|"))
                        .unwrap_or_default()
                };
                let mut imported = field.clone();
                if uri.starts_with("tsrust-stdlib:///")
                    && let Some(entry) = builtins_for(&record).into_iter().find(|entry| entry["name"] == field.name)
                    && let Some(documentation) = entry["documentation"].as_str()
                {
                    imported.docs = documentation.to_owned();
                }
                imported.ty = ty;
                if imported.kind == 22 { imported.signature = format!("{owner}.{}: {}", imported.name, self.display_type(&imported.ty)); }
                if contains_callable_annotation(&imported.ty) {
                    imported.signature =
                        format!("{}: {}", imported.name, self.display_type(&imported.ty));
                }
                imported.owner = Some(owner.clone());
                imported.scope = 0;
                imported.span = span;
                imported.extent = span;
                self.foreign_definitions.insert(
                    self.symbols.len(),
                    dependency
                        .foreign_definitions
                        .get(&field_id)
                        .cloned()
                        .unwrap_or_else(|| json!({"uri":uri,"range":dependency.range(field.span)})),
                );
                self.symbols.push(imported);
            }
        }
    }
    pub(crate) fn finish_imports(&mut self) {
        self.materialize_generic_fields();
        self.refresh_checked_initializers();
    }
    /// Resolved token locations and their declaration identity, for project references.
    pub(crate) fn reference_locations(&self, uri: &str) -> Vec<(Value, Value, bool)> {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| !token.literal && name(&token.text))
            .filter_map(|(id, token)| {
                if let Some((span,definition))=self.checked_control_labels.iter().find(|(span,_)|*span==token.span){
                    return Some((json!({"uri":uri,"range":self.range(*definition)}),json!({"uri":uri,"range":self.range(*span)}),*span==*definition));
                }
                let symbol = self.resolve_token(id)?;
                let declaration = &self.symbols[symbol];
                let foreign = self.foreign_definitions.get(&symbol);
                let definition = foreign
                    .cloned()
                    .unwrap_or_else(|| json!({"uri":uri,"range":self.range(declaration.span)}));
                Some((
                    definition,
                    json!({"uri":uri,"range":self.range(token.span)}),
                    foreign.is_none() && token.span == declaration.span,
                ))
            })
            .collect()
    }
    pub fn request(&self, method: &str, params: &Value, uri: &str) -> Result<Value, String> {
        if method == "textDocument/documentSymbol" {
            return Ok(self.document_outline());
        }
        if method == "textDocument/foldingRange" {
            let mut ranges = vec![];
            for (id, token) in self.tokens.iter().enumerate() {
                if token.text == "{"
                    && let Some(end) = self.pairs[id]
                {
                    let r = self.range(Span::new(token.span.start, self.tokens[end].span.end));
                    if r.end.line > r.start.line {
                        ranges.push(
                            json!({"startLine":r.start.line,"endLine":r.end.line,"kind":"region"}),
                        );
                    }
                }
            }
            for (span, _) in &self.comments {
                let r = self.range(*span);
                if r.end.line > r.start.line {
                    ranges.push(
                        json!({"startLine":r.start.line,"endLine":r.end.line,"kind":"comment"}),
                    );
                }
            }
            return Ok(json!(ranges));
        }
        let position: Position =
            serde_json::from_value(params.get("position").cloned().unwrap_or(Value::Null))
                .map_err(|_| "Missing or invalid position")?;
        let offset = self
            .offset(position)
            .ok_or("Position is outside the document or splits a surrogate pair")?;
        if method != "textDocument/signatureHelp" && self.excluded(offset) {
            return Ok(if method == "textDocument/completion" {
                json!([])
            } else {
                Value::Null
            });
        }
        if method == "textDocument/completion" {
            return Ok(self.completion(offset));
        }
        if method == "textDocument/signatureHelp" {
            return Ok(self.signature_help(offset));
        }
        let Some(token) = self.token_at(offset) else {
            return Ok(Value::Null);
        };
        if !name(self.text(token)) {
            return Ok(Value::Null);
        }
        if let Some((_,definition))=self.checked_control_labels.iter().find(|(span,_)|*span==self.tokens[token].span){
            let declaration=*definition;
            return match method {
                "textDocument/hover"=>Ok(json!({"contents":{"kind":"markdown","value":format!("```tsrust\nlabel {}\n```\n\nTarget of labeled break or continue. Labels occupy a separate namespace from values.",self.text(token))},"range":self.range(self.tokens[token].span)})),
                "textDocument/definition"=>Ok(json!({"uri":uri,"range":self.range(declaration)})),
                "textDocument/references"|"textDocument/documentHighlight"=>{
                    let include=params.pointer("/context/includeDeclaration").and_then(Value::as_bool).unwrap_or(true);
                    let result=self.checked_control_labels.iter().filter(|(span,target)|*target==declaration && (include||*span!=declaration)).map(|(span,_)|if method=="textDocument/references" {json!({"uri":uri,"range":self.range(*span)})}else{json!({"range":self.range(*span),"kind":1})}).collect::<Vec<_>>();Ok(json!(result))
                }
                _=>Err("Method is not supported".into()),
            };
        }
        if method == "textDocument/hover"
            && let Some(builtin) = self
                .builtin_at(token)
                .or_else(|| self.global_builtin_at(token))
        {
            return Ok(
                json!({"contents":{"kind":"markdown","value":format!("```tsrust\n{}\n```\n\n{}",builtin["signature"].as_str().unwrap_or(""),builtin["documentation"].as_str().unwrap_or(""))},"range":self.range(self.tokens[token].span)}),
            );
        }
        let Some(symbol) = self.resolve_token(token) else {
            return Ok(Value::Null);
        };
        let s = &self.symbols[symbol];
        match method {
            "textDocument/hover" => Ok(
                json!({"contents":{"kind":"markdown","value":format!("{}{}",self.checked_generic_signature(token).map_or_else(||self.documentation(s),|(signature,_)|format!("```tsrust\n{signature}\n```{}",if s.docs.is_empty(){String::new()}else{format!("\n\n{}",s.docs)})),self.checked_view(token).filter(|ty|*ty!=s.ty).map_or(String::new(),|ty|format!("\n\nNarrowed here to `{ty}`.")))},"range":self.range(self.tokens[token].span)}),
            ),
            "textDocument/definition" => Ok(self
                .foreign_definitions
                .get(&symbol)
                .cloned()
                .unwrap_or_else(|| json!({"uri":uri,"range":self.range(s.span)}))),
            "textDocument/references" | "textDocument/documentHighlight" => {
                let include = params
                    .pointer("/context/includeDeclaration")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let refs = self
                    .tokens
                    .iter()
                    .enumerate()
                    .filter(|(id, t)| {
                        t.text == s.name
                            && (include || t.span.start != s.span.start)
                            && self.resolve_token(*id) == Some(symbol)
                    })
                    .map(|(_, t)| {
                        if method == "textDocument/references" {
                            json!({"uri":uri,"range":self.range(t.span)})
                        } else {
                            json!({"range":self.range(t.span),"kind":1})
                        }
                    })
                    .collect::<Vec<_>>();
                Ok(json!(refs))
            }
            _ => Err("Method is not supported".into()),
        }
    }
    fn global_builtin_at(&self, token: usize) -> Option<Value> {
        if (token > 0 && member_separator(self.text(token - 1)))
            || self.resolve_token(token).is_some()
        {
            return None;
        }
        if !(self.text(token) == "void" && token > 0 && self.text(token - 1) == ":")
            && let Some(entry) = builtins_for("globals")
                .into_iter()
                .find(|entry| entry["name"] == self.text(token))
        {
            // A global that is also a namespace lists what `Name.` offers.
            let mut entry = entry;
            if matches!(self.text(token), "String" | "Object" | "Array" | "Map" | "Number") {
                let members = builtins_for(self.text(token))
                    .into_iter()
                    .filter_map(|member| member["name"].as_str().map(str::to_owned))
                    .collect::<Vec<_>>();
                if !members.is_empty() {
                    let documentation = format!(
                        "{}\n\nStatic members: {}.",
                        entry["documentation"].as_str().unwrap_or(""),
                        members.join(", ")
                    );
                    entry["documentation"] = json!(documentation);
                }
            }
            return Some(entry);
        }
        let (signature, docs, parameters) = match self.text(token) {
            "console" => (
                "namespace console",
                "Provides terminal output through log, info and debug (stdout), warn and error (stderr), plus table for structured data. Logging accepts zero or more primitives (including null/undefined), arrays or records. Arguments evaluate left to right and are borrowed; arrays and records have bounded inspection. Nullable values print their present payload, null or undefined; nested nullable fields are inspected recursively. Logging supports format placeholders, quoted inspection of arrays and records, and box-drawn tables. Also provides assertions, groups, counts, timers, dir, clear and native traces; inspector methods are no-ops.\n\n```tsrust\nconsole.log(\"user:\", \"Pat\", 7); // prints user: Pat 7\n```",
                vec![],
            ),
            "Math" => (
                "namespace Math",
                "Provides mathematical number functions and read-only constants. Angles use radians; arguments and results use native f64 number values. Convert int variables explicitly with Number(value). Transcendental results may vary slightly by platform.\n\n```tsrust\nconsole.log(Math.sqrt(9.0)); // prints 3\n```",
                vec![],
            ),
            "clone" => (
                "clone(value: T): T",
                "Creates an independent copy of a string, array, record or nullable union with an owned payload, preserving its type and active variant. Null/undefined variants stay null/undefined; an active string, array or record payload and its owned descendants are copied. Copies owned descendants of arrays/records; static string text may remain safely shared. Borrows the source so it stays reusable. Primitive scalars, Copy-only nullable unions and callable values are unsupported.\n\n@param value - Required string, array, record or nullable union with an owned payload to copy without consuming it.\n\n```tsrust\nconst source:int[]=[1,2];\nlet copy=clone(source);\ncopy[0]=9;\nconsole.log(source[0]); // prints 1\nconst nullable:int[]|null=[3];\nconsole.log(String(clone(nullable)),String(nullable)); // prints 3 3\n```",
                vec!["value: T"],
            ),
            "int" => (
                "type int",
                "Signed 64-bit integer, ranging from -9223372036854775808 through 9223372036854775807. Arithmetic uses native checked integer contracts; invalid arithmetic fails instead of silently converting to floating point. Copy value; use Number(value) for explicit conversion.\n\n```tsrust\nconst count:int=3;\nconsole.log(count + 1); // prints 4\n```",
                vec![],
            ),
            "number" => (
                "type number",
                "IEEE 754 double-precision floating-point value, including NaN, infinities and signed zero. Distinct from int; integer-valued literals can use an expected number type. Primitive copy with no owned storage.\n\n```tsrust\nconst ratio:number=1.5;\nconsole.log(ratio * 2.0); // prints 3\n```",
                vec![],
            ),
            "string" => (
                "type string",
                "Valid UTF-8 text. length and slicing/search coordinates use bytes; charAt uses Unicode scalar indices. Lone surrogates are unsupported. Owned strings follow borrow/move rules; clone preserves a reusable owner.\n\n```tsrust\nconst text:string=\"é\";\nconsole.log(text.length); // prints 2\n```",
                vec![],
            ),
            "boolean" => (
                "type boolean",
                "The values true and false. Conditions require boolean rather than JavaScript truthiness coercion. Primitive copy with no owned storage.\n\n```tsrust\nconst ready:boolean=true;\nconsole.log(ready); // prints true\n```",
                vec![],
            ),
            "void" => (
                "type void",
                "A function result with no usable value, such as console.log or array.push. It cannot be used as a numeric, string or array result.\n\n```tsrust\nfunction greet():void { console.log(\"Hello\"); }\n```",
                vec![],
            ),
            _ => return None,
        };
        let mut documentation = docs.to_string();
        if matches!(self.text(token), "console" | "Math" | "Number" | "Array" | "Object" | "String" | "Map") {
            let members = builtins_for(self.text(token))
                .into_iter()
                .filter_map(|e| e["name"].as_str().map(str::to_owned))
                .collect::<Vec<_>>();
            documentation.push_str(&format!("\n\nSupported members: {}.", members.join(", ")));
        }
        Some(json!({"signature":signature,"documentation":documentation,"parameters":parameters}))
    }
    fn builtin_at(&self, token: usize) -> Option<Value> {
        if token < 2 || !member_separator(self.text(token - 1)) {
            return None;
        }
        let receiver = if matches!(
            self.text(token - 2),
            "Math" | "Number" | "console" | "Array" | "Object" | "String" | "Map"
        ) {
            self.text(token - 2).into()
        } else {
            self.receiver_type(token - 1)
        };
        let mut entry = builtins_for(&receiver)
            .into_iter()
            .find(|b| b["name"] == self.text(token))?;
        if (matches!(self.text(token), "flat" | "flatMap")
            || (receiver == "Array" && matches!(self.text(token),"of"|"from"))
            || (receiver == "Map" && self.text(token)=="groupBy"))
            && let Some(open) = self.member_call_open(token)
            && let Some(close) = self.pairs.get(open).copied().flatten()
            && let Some(result) = self.checked_method_result(close)
            && let Some((parameters, _)) = entry["signature"]
                .as_str()
                .and_then(|s| s.rsplit_once("): "))
        {
            if receiver == "Array"
                && self.text(token) == "of"
                && let Some(element) = result.strip_suffix("[]")
            {
                entry["signature"] = json!(format!("of<{element}>(...items: {result}): {result}"));
                entry["parameters"] = json!([format!("...items: {result}")]);
            } else {
                entry["signature"] = json!(format!("{parameters}): {result}"));
            }
        }
        Some(entry)
    }
    fn completion(&self, offset: usize) -> Value {
        let token = self.token_at(offset);
        let prefix_start = token
            .filter(|&id| name(self.text(id)))
            .map_or(offset, |id| self.tokens[id].span.start);
        let prefix = &self.source[prefix_start..offset];
        let before = self.tokens.iter().rposition(|t| t.span.end <= prefix_start);
        let replace = self.range(Span::new(prefix_start, offset));
        if before.is_some_and(|id| member_separator(self.text(id))) {
            let dot = before.unwrap();
            let ty = if dot > 0
                && matches!(self.text(dot - 1), "Math" | "Number" | "console" | "Array" | "Object" | "String" | "Map")
            {
                self.text(dot - 1).into()
            } else {
                self.receiver_type(dot)
            };
            let mut items=builtins_for(&ty).into_iter().filter(|b|b["name"].as_str().unwrap_or("").starts_with(prefix)).map(|b|json!({"label":b["name"],"kind":if b["parameters"].is_null(){10}else{2},"detail":b["signature"],"documentation":{"kind":"markdown","value":b["documentation"]},"textEdit":{"range":replace,"newText":b["name"]}})).collect::<Vec<_>>();
            items.extend(self.symbols.iter().filter(|s|s.owner.as_deref()==Some(ty.as_str()) && s.name.starts_with(prefix)).map(|s|json!({"label":s.name,"kind":5,"detail":s.signature,"documentation":{"kind":"markdown","value":s.docs},"textEdit":{"range":replace,"newText":s.name}})));
            return json!(items);
        }
        let mut items = vec![];
        let scope = self.scope_at(offset);
        let mut visible = std::collections::BTreeMap::new();
        for (id, s) in self.symbols.iter().enumerate() {
            if s.owner.is_none()
                && self.contains_scope(scope, s.scope)

            {
                let key = (self.scope_depth(s.scope), s.span.start);
                if visible
                    .get(s.name.as_str())
                    .is_none_or(|&(previous, _)| key > previous)
                {
                    visible.insert(s.name.as_str(), (key, id));
                }
            }
        }
        for (_, (_, id)) in visible {
            if !self.parameter_available(id, offset) {
                continue;
            }
            let s = &self.symbols[id];
            // A later local binding masks an outer function without becoming
            // an available completion before its initialization point.
            if s.span.start > offset && !self.parameter_symbols.contains(&id)
                && !matches!(s.kind, 10 | 23) && !s.signature.starts_with("function ") {
                continue;
            }
            if s.name.starts_with(prefix) {
                items.push(json!({"label":s.name,"kind":match s.kind {12=>3,10=>13,23=>7,_=>6},"detail":s.signature,"documentation":{"kind":"markdown","value":s.docs},"textEdit":{"range":replace,"newText":s.name}}));
            }
        }
        for word in [
            "function",
            "const",
            "let",
            "type",
            "enum",
            "if",
            "else",
            "while",
            "do",
            "for",
            "of",
            "in",
            "return",
            "break",
            "continue",
            "Function",
            "Mutable",
            "ReadOnly",
            "Once",
            "Owned",
            "int",
            "number",
            "string",
            "boolean",
            "void",
            "null",
            "undefined",
            "typeof",
            "true",
            "false",
            "clone",
            "Number",
            "Math",
            "console",
            "String",
            "Boolean",
            "Array",
            "Object",
            "parseInt",
            "parseFloat",
            "isNaN",
            "isFinite",
            "NaN",
            "Infinity",
        ] {
            if word.starts_with(prefix) {
                if let Some(entry) = builtins_for("globals")
                    .into_iter()
                    .find(|entry| entry["name"] == word)
                {
                    items.push(json!({"label":word,"kind":14,"detail":entry["signature"],"documentation":{"kind":"markdown","value":entry["documentation"]},"textEdit":{"range":replace,"newText":word}}));
                    continue;
                }
                items.push(
                    json!({"label":word,"kind":14,"textEdit":{"range":replace,"newText":word}}),
                );
            }
        }
        json!(items)
    }
    // Count only complete ordinary array literals; a nested spread is not a
    // known-width element. This is source syntax, never an inferred array type.
    fn literal_spread_length(&self, mut begin: usize, mut end: usize) -> Option<usize> {
        while self.text(begin) == "("
            && self.pairs.get(begin).copied().flatten() == end.checked_sub(1)
        {
            begin += 1;
            end -= 1;
        }
        if self.text(begin) != "[" || self.pairs.get(begin).copied().flatten() != end.checked_sub(1)
        {
            return None;
        }
        let stop = end - 1;
        let mut token = begin + 1;
        let mut count = 0usize;
        while token < stop {
            if matches!(self.text(token), "," | "...") {
                return None;
            }
            let next = self.call_argument_end(token, stop);
            count = count.checked_add(1)?;
            token = if next < stop { next + 1 } else { stop };
        }
        Some(count)
    }
    // Skip commas belonging to nested expressions and explicit call type args.
    fn call_argument_end(&self, mut token: usize, stop: usize) -> usize {
        while token < stop {
            if self.text(token) == "," {
                return token;
            }
            if matches!(self.text(token), "(" | "[" | "{" | "${") {
                let Some(close) = self
                    .pairs
                    .get(token)
                    .copied()
                    .flatten()
                    .filter(|close| *close < stop)
                else {
                    return stop;
                };
                token = close + 1;
                continue;
            }
            if self.text(token) == "<" && token > 0 && name(self.text(token - 1)) {
                let mut depth = 0usize;
                let mut call = None;
                for next in token..stop {
                    match self.text(next) {
                        "<" => depth += 1,
                        ">" => {
                            depth = depth.saturating_sub(1);
                            if depth == 0 {
                                call = (self.text(next + 1) == "(").then_some(next + 1);
                                break;
                            }
                        }
                        ";" | "=" => break,
                        _ => {}
                    }
                }
                if let Some(open) = call {
                    token = open;
                    continue;
                }
            }
            token += 1;
        }
        stop
    }
    fn call_argument_parameter(
        &self,
        open: usize,
        offset: usize,
        parameters: &[String],
    ) -> Option<usize> {
        let last = parameters.len().saturating_sub(1);
        let stop = self
            .tokens
            .partition_point(|token| token.span.start < offset);
        let mut begin = open + 1;
        let mut minimum = 0usize;
        let mut exact = true;
        let mut spread = false;
        loop {
            let end = self.call_argument_end(begin, stop);
            let width = if self.text(begin) == "..." {
                spread = true;
                self.literal_spread_length(begin + 1, end)
            } else {
                Some(1)
            };
            if end == stop {
                if !spread {
                    return Some(minimum.min(last));
                }
                if parameters.is_empty() || width == Some(0) {
                    return None;
                }
                let low = minimum.min(last);
                let high = if exact {
                    width
                        .and_then(|width| minimum.checked_add(width - 1))
                        .map(|position| position.min(last))
                } else {
                    None
                };
                return (high == Some(low) || low == last).then_some(low);
            }
            if let Some(width) = width {
                minimum = minimum.checked_add(width)?;
            } else {
                exact = false;
            }
            begin = end + 1;
        }
    }
    // Computed callees use checked value types. Only enclosing grouping
    // parentheses may be skipped; never reach inside a factory's arguments.
    fn expression_callee_signature(&self, open:usize)->Option<(String,Vec<String>)> {
        let call_start = self.tokens.get(open)?.span.start;
        if let Some(fact) = self.checked_generic_calls.iter().filter(|fact| {
            if !(fact.span.start <= call_start && call_start < fact.span.end && fact.callee_span.end <= call_start) { return false; }
            let mut angles = 0usize;
            for token in self.tokens.iter().take(open).filter(|token| token.span.start >= fact.callee_span.end) {
                if angles > 0 {
                    match token.text.as_str() {
                        "<" => angles += 1,
                        ">" | ">>" | ">>>" => angles = angles.saturating_sub(token.text.len()),
                        _ => (),
                    }
                } else {
                    match token.text.as_str() { ")" | "?." => (), "<" => angles = 1, _ => return false }
                }
            }
            angles == 0
        }).min_by_key(|fact| fact.span.end - fact.span.start) {
            return self.checked_generic_signature_for(fact, self.source.get(fact.callee_span.start..fact.callee_span.end)?);
        }
        if self.text(open.checked_sub(1)?)!=")" {return None;}
        let mut end=open;
        loop {
            let stop=self.tokens.get(end.checked_sub(1)?)?.span.end;
            if let Some((span,ty))=self.checked_method_results.iter()
                .filter(|(span,ty)|span.end==stop && callable_parameters(ty).is_some())
                .max_by_key(|(span,_)|span.start) {
                let parameters=callable_parameters(ty)?.iter().map(|parameter|self.display_type(parameter)).collect();
                return Some((format!("{}: {}",&self.source[span.start..self.tokens[open].span.start],self.display_type(ty)),parameters));
            }
            let close=end-1;
            if self.text(close)!=")" {return None;}
            let group=self.pairs[close]?;
            if self.call_callee(group).is_some() {return None;}
            end=close;
        }
    }
    fn signature_help(&self, offset: usize) -> Value {
        // The nearest unmatched call parenthesis owns commas; skip nested calls,
        // literals and indexing so they do not advance the outer parameter.
        let mut stack: Vec<(usize, usize)> = vec![];
        for (id, t) in self
            .tokens
            .iter()
            .enumerate()
            .take_while(|(_, t)| t.span.start < offset)
        {
            match t.text.as_str() {
                "(" | "[" | "{" | "${" => stack.push((id, 0)),
                ")" | "]" | "}" => {
                    stack.pop();
                }
                "," => {
                    if let Some((_, count)) = stack.last_mut() {
                        *count += 1;
                    }
                }
                _ => {}
            }
        }
        for (open, _) in stack.into_iter().rev() {
            if self.text(open) != "(" {
                continue;
            }
            if let Some((label,parameters))=self.expression_callee_signature(open) {
                let active=self.call_argument_parameter(open,offset,&parameters);
                let parameters=parameters.iter().map(|parameter|json!({"label":parameter})).collect::<Vec<_>>();
                let mut help=json!({"signatures":[{"label":label,"parameters":parameters}],"activeSignature":0});
                if let Some(active)=active {help["activeParameter"]=json!(active);}
                return help;
            }
            let Some(callee) = self.call_callee(open) else {
                continue;
            };
            if callee > 0 && self.text(callee - 1) == "function" {
                return Value::Null;
            }
            let (label, docs, parameters) = if let Some(b) = self.builtin_at(callee).or_else(|| {
                matches!(
                    self.text(callee),
                    "Number" | "clone" | "String" | "Boolean" | "Set" | "Map" | "parseInt" | "parseFloat" | "isNaN" | "isFinite"
                )
                    .then(|| self.global_builtin_at(callee))
                    .flatten()
            }) {
                let mut b = b;
                if matches!(self.text(callee), "Set" | "Map") && self.text(callee + 1) == "<" && self.text(open - 1) == ">" {
                    let key = self.snippet(callee + 2, open - 1);
                    if self.text(callee) == "Map" && let Some((key,value)) = key.split_once(',') {
                        if let Some(signature) = b["signature"].as_str() { b["signature"] = json!(signature.replace("Key",key.trim()).replace("Value",value.trim())); }
                        if let Some(parameters) = b["parameters"].as_array_mut() { for parameter in parameters { if let Some(text) = parameter.as_str() { *parameter = json!(text.replace("Key",key.trim()).replace("Value",value.trim())); } } }
                    }
                    if let Some(signature) = b["signature"].as_str() { b["signature"] = json!(signature.replace("Element", &key)); }
                    if let Some(parameters) = b["parameters"].as_array_mut() { for parameter in parameters { if let Some(text) = parameter.as_str() { *parameter = json!(text.replace("Element", &key)); } } }
                }
                (
                    b["signature"].as_str().unwrap_or("").to_string(),
                    b["documentation"].as_str().unwrap_or("").to_string(),
                    b["parameters"].as_array().map_or(vec![], |p| {
                        p.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    }),
                )
            } else if let Some(id) = self.resolve_token(callee) {
                let s = &self.symbols[id];
                let mut callable_type = s.ty.as_str();
                let mut index_depth = 0usize;
                let mut indices = 0usize;
                for token in callee + 1..open {
                    match self.text(token) {
                        "[" => {
                            if index_depth == 0 {
                                indices += 1;
                            }
                            index_depth += 1;
                        }
                        "]" => index_depth = index_depth.saturating_sub(1),
                        _ => (),
                    }
                }
                let checked_callable = self
                    .checked_view(if indices == 0 { callee } else { open - 1 })
                    .filter(|ty| callable_annotation(ty).is_some());
                if let Some(ty) = checked_callable {
                    callable_type = ty;
                } else {
                    for _ in 0..indices {
                        callable_type = ungroup_callable_type(callable_type)
                            .strip_suffix("[]")
                            .unwrap_or("")
                            .trim();
                    }
                }
                callable_type = ungroup_callable_type(callable_type);
                let callable = if s.signature.starts_with("function ") {
                    None
                } else {
                    callable_parameters(callable_type)
                };
                if callable.is_none()
                    && self.checked_generic_signature(callee).is_none()
                    && !s.signature.starts_with("function ")
                    && (s.kind != 12 || !s.signature.contains(" = "))
                {
                    continue;
                }
                let (signature, parameters) =
                    self.checked_generic_signature(callee).unwrap_or_else(|| {
                        let parameters = callable
                            .unwrap_or_else(|| s.parameters.clone())
                            .iter()
                            .map(|parameter| self.display_type(parameter))
                            .collect();
                        let signature = if indices != 0 {
                            format!(
                                "{}: {}",
                                &self.source
                                    [self.tokens[callee].span.start..self.tokens[open].span.start],
                                self.display_type(callable_type)
                            )
                        } else {
                            checked_callable.map_or_else(
                                || self.display_type(&s.signature),
                                |ty| format!("{}: {}", s.name, self.display_type(ty)),
                            )
                        };
                        (signature, parameters)
                    });
                (signature, s.docs.clone(), parameters)
            } else {
                continue;
            };
            let native_docs = self
                .resolve_token(callee)
                .and_then(|id| self.native_parameter_docs.get(&id));
            let active = self.call_argument_parameter(open, offset, &parameters);
            let parameters=parameters.iter().enumerate().map(|(index,p)|json!({"label":p,"documentation":{"kind":"markdown","value":native_docs.and_then(|docs|docs.get(index)).cloned().unwrap_or_else(||parameter_doc(&docs,p))}})).collect::<Vec<_>>();
            let mut help = json!({"signatures":[{"label":label,"documentation":{"kind":"markdown","value":docs},"parameters":parameters}],"activeSignature":0});
            if let Some(active) = active {
                help["activeParameter"] = json!(active);
            }
            return help;
        }
        Value::Null
    }
}
fn remap_imported_type(
    ty: &str,
    types: &std::collections::BTreeMap<String, String>,
) -> Option<String> {
    if let Some(mapped) = types.get(ty) {
        return Some(mapped.clone());
    }
    let mut output = String::new();
    let mut offset = 0;
    while offset < ty.len() {
        let ch = ty[offset..].chars().next()?;
        if ch.is_ascii_alphabetic() || ch == '_' {
            let end = offset
                + ty[offset..]
                    .bytes()
                    .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    .count();
            let token = &ty[offset..end];
            let parameter_name = ty[end..]
                .trim_start()
                .trim_start_matches('?')
                .trim_start()
                .starts_with(':');
            if parameter_name
                || matches!(
                    token,
                    "int"
                        | "number"
                        | "boolean"
                        | "string"
                        | "void"
                        | "null"
                        | "undefined"
                        | "Function"
                        | "Mutable"
                        | "ReadOnly"
                        | "Once"
                        | "Owned"
                        | "Set"
                        | "Map"
                )
            {
                output.push_str(token);
            } else {
                output.push_str(types.get(token)?);
            }
            offset = end;
        } else {
            output.push(ch);
            offset += ch.len_utf8();
        }
    }
    Some(output)
}

fn ungroup_callable_type(mut ty: &str) -> &str {
    loop {
        ty = ty.trim();
        if !ty.starts_with('(') {
            return ty;
        }
        let mut depth = 0usize;
        let mut closing = None;
        for (offset, character) in ty.char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        closing = Some(offset);
                        break;
                    }
                }
                _ => (),
            }
        }
        if closing != Some(ty.len() - 1) {
            return ty;
        }
        ty = &ty[1..ty.len() - 1];
    }
}

fn callable_storage_type(mut ty: &str) -> bool {
    loop {
        ty = ungroup_callable_type(ty);
        if callable_annotation(ty).is_some() {
            return true;
        }
        let Some(element) = ty.strip_suffix("[]") else {
            return false;
        };
        ty = element;
    }
}

fn contains_callable_annotation(ty: &str) -> bool {
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) else {
        return callable_annotation(ty).is_some();
    };
    let mut pending = vec![&annotation];
    while let Some(value) = pending.pop() {
        match &value.kind {
            Kind::Callable { .. } => return true,
            Kind::Group(inner) => pending.push(inner),
            Kind::Union(members) => pending.extend(members),
            _ => {}
        }
    }
    false
}

fn callable_annotation(ty: &str) -> Option<(usize, usize)> {
    // A nullable callable is not directly invocable. Only a checked payload view
    // may supply its contract; a callable's own nullable result stays callable.
    let begin = if let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) {
        let mut value = &annotation;
        while let tsrust_compiler::type_syntax::ValueAnnotationKind::Group(inner) = &value.kind {
            value = inner;
        }
        if !matches!(
            value.kind,
            tsrust_compiler::type_syntax::ValueAnnotationKind::Callable { .. }
        ) {
            return None;
        }
        if named_function_type(&ty[value.span.start..value.span.end]) {
            // Function<...> has no parenthesized list; its parts come from
            // the structured annotation.
            return Some((value.span.start, value.span.start));
        }
        value.span.start
    } else {
        // Preserve tolerant help for incomplete and native optional signatures.
        0
    };
    let start = begin + ty[begin..].find('(')?;
    if !matches!(
        ty[begin..start]
            .split_whitespace()
            .collect::<Vec<_>>()
            .as_slice(),
        []
    ) {
        return None;
    }
    let mut depth = 0;
    let mut end = None;
    for (offset, ch) in ty[start..].char_indices() {
        if ch == '(' {
            depth += 1;
        }
        if ch == ')' {
            depth -= 1;
            if depth == 0 {
                end = Some(start + offset);
                break;
            }
        }
    }
    let end = end?;
    if !ty[end + 1..].trim_start().starts_with("=>") {
        return None;
    }
    Some((start, end))
}

fn callable_result_type(ty: &str) -> Option<String> {
    if let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) {
        let mut value = &annotation;
        while let tsrust_compiler::type_syntax::ValueAnnotationKind::Group(inner) = &value.kind {
            value = inner;
        }
        let tsrust_compiler::type_syntax::ValueAnnotationKind::Callable { result, .. } =
            &value.kind
        else {
            return None;
        };
        return Some(canonical_data_type(&ty[result.span.start..result.span.end]));
    }
    let (_, end) = callable_annotation(ty)?;
    Some(canonical_data_type(
        ty[end + 1..].trim_start().strip_prefix("=>")?.trim(),
    ))
}

/// `Function<...>`, or a ReadOnly/Mutable/Once/Owned wrapper around a type.
fn named_function_type(ty: &str) -> bool {
    ["Function", "Mutable", "ReadOnly", "Once", "Owned"].iter().any(|name| {
        ty.trim_start()
            .strip_prefix(name)
            .is_some_and(|rest| rest.trim_start().starts_with('<'))
    })
}

fn callable_parameters(ty: &str) -> Option<Vec<String>> {
    if let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) {
        let mut value = &annotation;
        while let tsrust_compiler::type_syntax::ValueAnnotationKind::Group(inner) = &value.kind {
            value = inner;
        }
        if let tsrust_compiler::type_syntax::ValueAnnotationKind::Callable { parameters, .. } = &value.kind
            && named_function_type(&ty[value.span.start..value.span.end])
        {
            return Some(
                parameters
                    .iter()
                    .map(|parameter| ty[parameter.span.start..parameter.span.end].trim().to_string())
                    .collect(),
            );
        }
    }
    let (start, end) = callable_annotation(ty)?;
    let mut parameters = vec![];
    let mut begin = start + 1;
    let mut nesting = 0;
    for (offset, ch) in ty[start + 1..end].char_indices() {
        match ch {
            '(' | '[' | '{' | '<' => nesting += 1,
            ')' | ']' | '}' => nesting -= 1,
            '>' if nesting > 0 && ty.as_bytes().get(start + offset) != Some(&b'=') => nesting -= 1,
            ',' if nesting == 0 => {
                parameters.push(ty[begin..start + 1 + offset].trim().to_string());
                begin = start + 2 + offset;
            }
            _ => {}
        }
    }
    if !ty[begin..end].trim().is_empty() {
        parameters.push(ty[begin..end].trim().to_string());
    }
    Some(parameters)
}

fn optional_parameter_type(ty: &str) -> String {
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    fn has_undefined(value: &tsrust_compiler::type_syntax::ValueAnnotation) -> bool {
        match &value.kind {
            Kind::Group(inner) => has_undefined(inner),
            Kind::Union(members) => members.iter().any(has_undefined),
            Kind::Named(name) => name.text == "undefined",
            _ => false,
        }
    }
    if tsrust_compiler::type_syntax::parse(ty)
        .ok()
        .is_some_and(|value| has_undefined(&value))
    {
        return canonical_data_type(ty);
    }
    let callable = tsrust_compiler::type_syntax::parse(ty)
        .ok()
        .is_some_and(|annotation| {
            let mut value = &annotation;
            while let Kind::Group(inner) = &value.kind {
                value = inner;
            }
            matches!(value.kind, Kind::Callable { .. })
        });
    canonical_data_type(&format!(
        "{} | undefined",
        if callable {
            format!("({ty})")
        } else {
            ty.into()
        }
    ))
}

fn parameter_doc(docs: &str, label: &str) -> String {
    let name = label
        .split([':', '='])
        .next()
        .unwrap_or("")
        .split_whitespace()
        .last()
        .unwrap_or("")
        .trim_start_matches("...")
        .trim_end_matches('?');
    let authored: String = docs
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("@param ")
                .and_then(|tail| tail.strip_prefix(name))
                .filter(|tail| tail.starts_with(char::is_whitespace))
                .map(|tail| tail.trim().trim_start_matches('-').trim().into())
        })
        .unwrap_or_default();
    if label.trim_start().starts_with("...") {
        let behavior = "Collects only remaining supplied arguments into a fresh owning array: Copy elements copy and owned elements transfer. Fixed parameters retain their access contracts; omitted/default padding adds no tail elements.";
        if authored.is_empty() {
            behavior.into()
        } else {
            format!("{authored}\n\n{behavior}")
        }
    } else {
        authored
    }
}
// Source fallback for unfinished assertions; a callable's nullable result is
// inside its callable node and must not be stripped as a top-level alternative.
fn nonnullable_assertion_type(ty: &str) -> String {
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) else { return String::new(); };
    let mut pending = vec![&annotation];
    let mut present = vec![];
    while let Some(value) = pending.pop() {
        match &value.kind {
            Kind::Group(inner) => pending.push(inner),
            Kind::Union(members) => pending.extend(members.iter().rev()),
            Kind::Named(name) if matches!(name.text.as_str(), "null" | "undefined" | "void") => {},
            _ => {
                let member = ty[value.span.start..value.span.end].trim();
                if !present.contains(&member) { present.push(member); }
            }
        }
    }
    if present.len() == 1 { present[0].to_owned() } else { String::new() }
}

// Normalize only nullable data annotations. Callable parameter/result syntax stays intact.
/// Replaces alias names in a written type with their expansions. A union
/// or function expansion is parenthesized so a following `[]` applies to all
/// of it.
fn expand_type_aliases(ty: &str, aliases: &std::collections::BTreeMap<String, String>) -> String {
    if aliases.is_empty() {
        return ty.into();
    }
    let mut output = String::with_capacity(ty.len());
    let mut rest = ty;
    while !rest.is_empty() {
        let word = rest
            .char_indices()
            .find(|(_, ch)| !(ch.is_alphanumeric() || *ch == '_' || *ch == '$'))
            .map_or(rest.len(), |(index, _)| index);
        if word == 0 {
            let ch = rest.chars().next().unwrap();
            output.push(ch);
            rest = &rest[ch.len_utf8()..];
            continue;
        }
        match aliases.get(&rest[..word]) {
            Some(expansion) if expansion.contains('|') || expansion.contains("=>") => {
                output.push('(');
                output.push_str(expansion);
                output.push(')');
            }
            Some(expansion) => output.push_str(expansion),
            None => output.push_str(&rest[..word]),
        }
        rest = &rest[word..];
    }
    output
}

fn canonical_data_type(ty: &str) -> String {
    // A function type keeps its written spelling in either form.
    let callable = ty.contains("=>") || contains_callable_annotation(ty);
    if ty.contains('<') && !callable {
        return generic_intelligence::canonical(ty)
            .unwrap_or_else(|| ty.chars().filter(|ch| !ch.is_whitespace()).collect());
    }
    if callable {
        if let Ok(annotation) = tsrust_compiler::type_syntax::parse(ty) {
            let mut inner = &annotation;
            while let tsrust_compiler::type_syntax::ValueAnnotationKind::Group(value) = &inner.kind
            {
                inner = value;
            }
            return ty[inner.span.start..inner.span.end].trim().into();
        }
        return ty.trim().into();
    }
    fn members(ty: &str, output: &mut Vec<String>, level: usize) {
        let ty = ty.trim();
        if level > 128 {
            output.push(ty.into());
            return;
        }
        let mut depth = 0i32;
        let mut begin = 0;
        let mut split = false;
        let mut outer = ty.starts_with('(') && ty.ends_with(')');
        for (offset, ch) in ty.char_indices() {
            match ch {
                '(' | '[' | '<' => depth += 1,
                ')' | ']' | '>' => depth -= 1,
                '|' if depth == 0 => {
                    members(&ty[begin..offset], output, level + 1);
                    begin = offset + 1;
                    split = true;
                }
                _ => {}
            }
            if depth == 0 && offset + ch.len_utf8() < ty.len() {
                outer = false;
            }
        }
        if split {
            members(&ty[begin..], output, level + 1);
        } else if outer {
            members(&ty[1..ty.len() - 1], output, level + 1);
        } else if !output.iter().any(|member| member == ty) {
            output.push(ty.into());
        }
    }
    let mut alternatives = vec![];
    members(ty, &mut alternatives, 0);
    if alternatives
        .iter()
        .filter(|ty| !matches!(ty.as_str(), "null" | "undefined"))
        .count()
        > 1
    {
        return ty.trim().into();
    }
    alternatives.sort_by_key(|ty| match ty.as_str() {
        "null" => 1,
        "undefined" => 2,
        _ => 0,
    });
    alternatives.join(" | ")
}

// Whole nullable arrays require a checked view. A union inside a tuple position
// or callable signature does not make its surrounding array nullable.
fn array_element_type(ty: &str) -> Option<&str> {
    if !ty.contains('|') { return ty.strip_suffix("[]"); }
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let annotation=tsrust_compiler::type_syntax::parse(ty).ok()?;
    let Kind::Array(inner)=&annotation.kind else {return None};
    let mut leaf=inner.as_ref();
    while let Kind::Array(value)|Kind::Group(value)=&leaf.kind {leaf=value;}
    // A union leaf is an ordinary element; its text keeps the grouping.
    let _ = leaf;
    ty.get(inner.span.start..inner.span.end)
}

fn iterable_element_type(ty:&str)->Option<String>{
    if ty=="string"{return Some("string".to_owned());}
    if let Some(element)=array_element_type(ty){return Some(element.to_owned());}
    use tsrust_compiler::type_syntax::ValueAnnotationKind as Kind;
    let annotation=tsrust_compiler::type_syntax::parse(ty).ok()?;
    let Kind::Applied{name,arguments}=&annotation.kind else{return None};
    let part=|value:&tsrust_compiler::type_syntax::ValueAnnotation|ty[value.span.start..value.span.end].trim();
    match(name.text.as_str(),arguments.as_slice()){
        ("Set",[key])=>Some(part(key).to_owned()),
        ("Map",[key,value])=>Some(format!("[{}, {}]",part(key),part(value))),
        _=>None,
    }
}

// A removal result is a value union; member access still needs a checked guard.
fn array_removal_result(ty: &str) -> Option<String> {
    let element = array_element_type(ty)?;
    let mut leaf = element;
    while let Some(inner) = leaf.strip_suffix("[]") {
        leaf = inner;
    }
    if leaf == "unknown" {
        return None;
    }
    // A grouped union element joins the result union without its grouping.
    let element = if leaf == element && element.starts_with('(') && element.ends_with(')') {
        &element[1..element.len() - 1]
    } else {
        element
    };
    Some(canonical_data_type(&format!("{element} | undefined")))
}

// Substitute type identifiers once. Injected names must never be substituted again.
fn builtin_type_substitution(text:&str,types:&[(&str,&str)])->String {
    let mut output=String::new();let mut offset=0;
    while offset<text.len(){
        let byte=text.as_bytes()[offset];
        if byte.is_ascii_alphabetic() || byte==b'_' {
            let start=offset;offset+=1;
            while offset<text.len() && (text.as_bytes()[offset].is_ascii_alphanumeric() || text.as_bytes()[offset]==b'_'){offset+=1;}
            let token=&text[start..offset];
            if let Some((_,ty))=types.iter().find(|(name,_)|*name==token){
                if ty.contains('|') && text[offset..].trim_start().starts_with("[]") {
                    output.push('(');output.push_str(ty);output.push(')');
                }else{output.push_str(ty);}
            }else{output.push_str(token);}
        }else{
            let ch=text[offset..].chars().next().unwrap();output.push(ch);offset+=ch.len_utf8();
        }
    }
    output
}

fn builtins_for(ty: &str) -> Vec<Value> {
    let set_key = ty.strip_prefix("Set<").and_then(|text| text.strip_suffix('>'));
    let map_types = ty.strip_prefix("Map<").and_then(|text| text.strip_suffix('>')).and_then(|text| text.split_once(',')).map(|(key,value)|(key.trim(),value.trim()));
    let family = if map_types.is_some() { "Map" } else if set_key.is_some() { "Set" } else if array_element_type(ty).is_some() {
        "Array"
    } else if tsrust_compiler::type_syntax::parse(ty).is_ok_and(|value|matches!(value.kind,tsrust_compiler::type_syntax::ValueAnnotationKind::Tuple(_))) {
        "Tuple"
    } else if ty == "Array" {
        "ArrayConstructor"
    } else if ty == "Map" {
        "MapConstructor"
    } else if ty == "String" {
        "StringConstructor"
    } else {
        ty
    };
    static DATA: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    let data = DATA.get_or_init(|| {
        serde_json::from_str(include_str!("builtins.json")).expect("checked built-in catalog")
    });
    let mut entries = data[family].as_array().cloned().unwrap_or_default();
    if let Some((key,value)) = map_types {
        for entry in &mut entries {
            if let Some(signature) = entry["signature"].as_str() { entry["signature"] = json!(builtin_type_substitution(signature,&[("Key",key),("Value",value)])); }
            if let Some(parameters) = entry["parameters"].as_array_mut() { for parameter in parameters { if let Some(text) = parameter.as_str() { *parameter = json!(builtin_type_substitution(text,&[("Key",key),("Value",value)])); } } }
        }
    }
    if let Some(key) = set_key {
        for entry in &mut entries {
            if let Some(signature) = entry["signature"].as_str() { entry["signature"] = json!(builtin_type_substitution(signature,&[("Element",key)])); }
            if let Some(parameters) = entry["parameters"].as_array_mut() { for parameter in parameters { if let Some(text) = parameter.as_str() { *parameter = json!(builtin_type_substitution(text,&[("Element",key)])); } } }
        }
    }
    if let Some(element) = array_element_type(ty) {
        // Search accepts one alternative of a grouped primitive union element.
        let searched = element
            .strip_prefix('(')
            .and_then(|inner| inner.strip_suffix(')'))
            .filter(|inner| !inner.contains(['(', '[', '<', '{']))
            .unwrap_or(element);
        entries.retain(|entry| {
            if matches!(
                entry["name"].as_str(),
                Some("pop" | "shift" | "at" | "find" | "findLast")
            ) && array_removal_result(ty).is_none()
            {
                return false;
            }
            !matches!(
                entry["name"].as_str(),
                Some("includes" | "indexOf" | "lastIndexOf")
            ) || searched
                .split('|')
                .all(|member| matches!(member.trim(), "int" | "number" | "boolean" | "string" | "null" | "undefined"))
        });
        for entry in &mut entries {
            if matches!(entry["name"].as_str(), Some("sort" | "toSorted"))
                && !matches!(
                    element,
                    "int" | "number" | "boolean" | "string" | "null" | "undefined"
                )
            {
                if let Some(signature) = entry["signature"].as_str() {
                    entry["signature"] = json!(signature.replace("compare?:", "compare:"));
                }
                if let Some(parameters) = entry["parameters"].as_array_mut() {
                    for parameter in parameters {
                        if let Some(label) = parameter.as_str() {
                            *parameter = json!(label.replace("compare?:", "compare:"));
                        }
                    }
                }
            }
            if let Some(signature) = entry["signature"].as_str() {
                entry["signature"] = json!(
                    signature
                        .replace("Element", element)
                        .replace("Primitive", searched)
                );
            }
            if matches!(
                entry["name"].as_str(),
                Some("pop" | "shift" | "at" | "find" | "findLast")
            ) && let Some(result) = array_removal_result(ty)
            {
                entry["signature"] = json!(format!(
                    "{}({}): {result}",
                    entry["name"].as_str().unwrap(),
                    if entry["name"] == "at" {
                        "index: int".to_owned()
                    } else if matches!(entry["name"].as_str(), Some("find" | "findLast")) {
                        format!(
                            "callback: (value: {element}, index: int, source: {element}[]) => boolean"
                        )
                    } else {
                        String::new()
                    }
                ));
            }
            if let Some(parameters) = entry["parameters"].as_array_mut() {
                for p in parameters {
                    if let Some(text) = p.as_str() {
                        *p = json!(
                            text.replace("Element", element)
                                .replace("Primitive", searched)
                        );
                    }
                }
            }
        }
    }
    entries
}
