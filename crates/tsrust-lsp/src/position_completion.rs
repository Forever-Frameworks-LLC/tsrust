//! Completion for positions that are not a member access or a scope name:
//! import paths, record literal fields and match arms.
use super::*;

impl SourceIndex {
    /// Module paths inside the string after `from`. Standard modules come
    /// from the compiler's binding and type tables; project modules are the
    /// source files beside the document.
    pub(super) fn import_path_completion(&self, offset: usize, uri: &str) -> Option<Value> {
        let token = self.tokens.iter().position(|token| token.literal && token.span.start < offset && (offset < token.span.end || (offset == token.span.end && token.unterminated_string)))
            .or_else(|| self.tokens.iter().position(|token| token.literal && token.span.start < offset && offset == token.span.end - 1))?;
        if token == 0 || self.text(token - 1) != "from" {
            return None;
        }
        let quote = self.source[self.tokens[token].span.start..].chars().next().filter(|quote| matches!(quote, '\'' | '"'))?;
        let start = self.tokens[token].span.start + quote.len_utf8();
        let typed = self.source.get(start..offset)?;
        let replace = self.range(Span::new(start, offset));
        let mut modules = std::collections::BTreeSet::new();
        for binding in tsrust_compiler::native_bindings::BINDINGS {
            modules.insert(binding.module.to_owned());
        }
        for entry in builtins_catalog_modules() {
            modules.insert(entry);
        }
        modules.retain(|module| module.starts_with("std:") && !module.ends_with("-runtime"));
        let mut items = modules.into_iter().filter(|module| module.starts_with(typed)).map(|module| {
            json!({"label": module, "kind": 9, "detail": "standard module", "textEdit": {"range": replace, "newText": module}})
        }).collect::<Vec<_>>();
        for path in sibling_modules(uri) {
            if path.starts_with(typed) {
                items.push(json!({"label": path, "kind": 17, "detail": "project module", "textEdit": {"range": replace, "newText": path}}));
            }
        }
        Some(json!(items))
    }

    /// The innermost `{` that is still open at `offset`.
    fn open_brace(&self, offset: usize) -> Option<usize> {
        let mut open = Vec::new();
        for (id, token) in self.tokens.iter().enumerate() {
            if token.span.start >= offset {
                break;
            }
            match token.text.as_str() {
                "{" | "(" | "[" | "${" => open.push(id),
                "}" | ")" | "]" => {
                    open.pop();
                }
                _ => {}
            }
        }
        open.last().copied().filter(|&id| self.text(id) == "{")
    }

    /// The cursor starts a new entry directly inside the brace `open`.
    fn entry_start(&self, open: usize, prefix_start: usize) -> bool {
        self.tokens.iter().rposition(|token| token.span.end <= prefix_start).is_some_and(|before| {
            before == open || (matches!(self.text(before), "," | "}") && self.open_brace(self.tokens[before].span.start + 1) == Some(open)) || (self.text(before) == "}" && self.pairs[before].is_some_and(|inner| inner > open))
        })
    }

    /// Fields of the record type that a literal is written for.
    pub(super) fn record_literal_completion(&self, offset: usize, prefix_start: usize) -> Option<Value> {
        let open = self.open_brace(prefix_start)?;
        if open == 0 || !self.entry_start(open, prefix_start) {
            return None;
        }
        // `const name: Type = {`: the annotation names the record.
        let ty = if self.text(open - 1) == "=" {
            let mut colon = open - 1;
            while colon > 0 && !matches!(self.text(colon), ":" | ";" | "{" | "}" | "const" | "let") {
                colon -= 1;
            }
            (self.text(colon) == ":").then(|| self.snippet(colon + 1, open - 1))?
        } else {
            return None;
        };
        let ty = self.written_type(ty.trim());
        let mut fields = self.symbols.iter().filter(|symbol| symbol.owner.as_deref() == Some(ty.as_str())).map(|symbol| (symbol.name.clone(), symbol.ty.clone())).collect::<Vec<_>>();
        if fields.is_empty() {
            fields = self.common_record_fields(&ty, true).into_iter().collect();
        }
        if fields.is_empty() {
            return None;
        }
        let end = self.pairs[open].unwrap_or(self.tokens.len());
        let written = (open + 1..end).filter(|&id| self.text(id + 1) == ":" && self.open_brace(self.tokens[id].span.start) == Some(open)).map(|id| self.text(id)).collect::<Vec<_>>();
        let prefix = &self.source[prefix_start..offset];
        let replace = self.range(Span::new(prefix_start, offset));
        Some(json!(fields.into_iter().filter(|(name, _)| name.starts_with(prefix) && !written.contains(&name.as_str())).map(|(name, ty)| {
            json!({"label": name, "kind": 5, "detail": format!("{name}: {}", self.display_type(&ty)), "textEdit": {"range": replace, "newText": format!("{name}: ")}})
        }).collect::<Vec<_>>()))
    }

    /// Alternatives of a `match` that are not written yet.
    pub(super) fn match_arm_completion(&self, offset: usize, prefix_start: usize) -> Option<Value> {
        let open = self.open_brace(prefix_start)?;
        if open < 3 || self.text(open - 1) != ")" || !self.entry_start(open, prefix_start) {
            return None;
        }
        let call = self.pairs[open - 1]?;
        if call == 0 || self.text(call - 1) != "match" {
            return None;
        }
        let end = self.pairs[open].unwrap_or(self.tokens.len());
        let written = (open + 1..end).filter(|&id| self.open_brace(self.tokens[id].span.start) == Some(open)
            && (id == open + 1 || self.entry_start(open, self.tokens[id].span.start)))
            .filter(|&id| self.text(id + 1) == "=>" || self.text(id + 1) == "(" && self.pairs[id + 1].is_some_and(|close| self.text(close + 1) == "=>"))
            .map(|id| self.text(id)).collect::<Vec<_>>();
        if written.contains(&"_") { return Some(json!([])); }
        let receiver = self.expression_type(open - 2);
        let variants = if let Some((ok, error)) = self.result_alternatives(&receiver) {
            vec![("Ok".to_string(), ok, "value"), ("Err".to_string(), error, "error")]
        } else {
            let expanded = self.type_aliases.get(&receiver).map_or(receiver.as_str(), String::as_str);
            let parts = expanded.split('|').map(str::trim).collect::<Vec<_>>();
            if parts.iter().all(|part| self.nominal_definitions.contains_key(*part) || self.symbols.iter().any(|symbol| symbol.name == *part && symbol.kind == 23)) {
                parts.into_iter().map(|name| (name.to_string(), name.to_string(), "value")).collect()
            } else if expanded == "boolean" {
                return Some(json!(["true", "false", "_"].into_iter().filter(|name| name.starts_with(&self.source[prefix_start..offset]) && !written.contains(name)).map(|name| json!({"label":name,"kind":20,"textEdit":{"range":self.range(Span::new(prefix_start,offset)),"newText":format!("{name} => $0")},"insertTextFormat":2})).collect::<Vec<_>>()));
            } else { return None; }
        };
        let prefix = &self.source[prefix_start..offset];
        let replace = self.range(Span::new(prefix_start, offset));
        let mut items = Vec::new();
        for (variant, ty, binding) in variants {
            if !variant.starts_with(prefix) || written.contains(&variant.as_str()) {
                continue;
            }
            let ty = self.display_type(&ty);
            let (pattern, detail) = if ungroup_callable_type(&ty) == "void" { (format!("{variant}(_)"), format!("{variant}()")) } else { (format!("{variant}(${{1:{binding}}})"), format!("{variant}({binding}: {ty})")) };
            items.push(json!({"label": variant, "kind": 20, "detail": detail, "insertTextFormat": 2,
                "documentation": {"kind": "markdown", "value": format!("Select the `{variant}` alternative and bind its value.")},
                "textEdit": {"range": replace, "newText": format!("{pattern} => {{\n  $0\n}}")}}));
        }
        Some(json!(items))
    }
}

/// Documented modules that the compiler accepts as an import path. The
/// compiler has no public list of standard modules, so each documented name
/// is confirmed once by loading the headers of a module that imports it.
fn builtins_catalog_modules() -> Vec<String> {
    static MODULES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    MODULES
        .get_or_init(|| {
            let accepted = |module: &str, name: &str| {
                let Ok(entry) = crate::project_diagnostics::source_id("file:///tsrust-module-probe/main.tsr") else {
                    return false;
                };
                let mut workspace = crate::workspace::Workspace::default();
                workspace.open(entry.clone(), 1, format!("import {{ {name} }} from '{module}';\nfunction main() {{}}\n"));
                tsrust_compiler::project::load_project_headers(&workspace.snapshot(), &entry.0).is_ok()
            };
            builtin_catalog()
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(module, _)| module.starts_with("std:"))
                .filter(|(module, entries)| {
                    tsrust_compiler::native_exports::known_module(module).is_some()
                        || entries.as_array().into_iter().flatten().filter_map(|entry| entry["name"].as_str()).any(|name| accepted(module, name))
                })
                .map(|(module, _)| module.clone())
                .collect()
        })
        .clone()
}

/// Relative import paths of the source files beside the document.
fn sibling_modules(uri: &str) -> Vec<String> {
    let Some(path) = uri.strip_prefix("file://") else {
        return Vec::new();
    };
    let path = std::path::Path::new(path);
    let Some(directory) = path.parent() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut pending = vec![(directory.to_path_buf(), String::from("./"), 0usize)];
    while let Some((directory, prefix, depth)) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let entry_path = entry.path();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            if entry_path.is_dir() {
                if depth < 3 {
                    pending.push((entry_path, format!("{prefix}{name}/"), depth + 1));
                }
            } else if (name.ends_with(".tsr") || name.ends_with(".rs") || name.ends_with(".bindings.json")) && entry_path != path {
                found.push(format!("{prefix}{name}"));
            }
        }
    }
    found.sort();
    found
}
