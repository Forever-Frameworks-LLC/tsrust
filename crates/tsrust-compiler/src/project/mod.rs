//! Source-indexed module graph loading. This does not type-check or emit a project.
pub mod rust;
pub use crate::project_link::{ProjectAnalysis, check_graph};
pub use crate::project_facts::{ProjectLocalFunctionInfo, ProjectCheckedTypeFacts, ProjectExpressionTypeInfo, ProjectGenericCallInfo, ProjectGenericParameterCallInfo, ProjectParameterInfo, ProjectType};
use crate::syntax::{self, TokenKind};
use crate::{Diagnostic, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceId(pub String);

/// Read the immutable source of a bundled standard-library module.
/// Editors use this for module documentation and diagnostic source ranges.
pub fn bundled_source(id: &SourceId) -> Option<&'static str> {
    crate::stdlib::source(&id.0)
}

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub id: SourceId,
    pub text: String,
}

/// Providers resolve identity before loading. Editors can implement this with snapshots.
pub trait SourceProvider {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String>;
    fn read(&self, id: &SourceId) -> Result<String, String>;
}

/// An unmodified compiler-development facade uses the same module context as
/// its bundled std import. Never infer trust from a basename or directory layout:
/// require this tool's canonical implementation path and exact bundled bytes.
/// Call only for an entry; imported user files retain normal private-binding checks.
pub fn bundled_entry(provider:&impl SourceProvider,id:&SourceId)->Option<SourceId> {
    static UI:std::sync::LazyLock<PathBuf>=std::sync::LazyLock::new(||{
        let path=Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib/ui/api.tsr");
        std::fs::canonicalize(&path).unwrap_or(path)
    });
    if Path::new(&id.0)!=UI.as_path(){return None;}
    let logical=SourceId("std:ui".into());
    (provider.read(id).ok()?.as_str()==bundled_source(&logical)?).then_some(logical)
}

pub struct FileSourceProvider;
impl SourceProvider for FileSourceProvider {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let candidates = if let Some(id)=importer {
            let aliases=crate::project_resolution::discover(Path::new(&id.0),|path|match std::fs::read_to_string(path) {
                Ok(text)=>Ok(Some(text)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(error)=>Err(format!("{}: {error}",path.display()))
            })?;
            crate::project_resolution::candidates(Path::new(&id.0),path,aliases.as_ref())?
        } else {vec![PathBuf::from(path)]};
        let mut canonical=None;
        for candidate in &candidates {
            match std::fs::metadata(candidate) {
                Ok(metadata) if metadata.is_file()=>{canonical=Some(std::fs::canonicalize(candidate).map_err(|error|format!("{}: {error}",candidate.display()))?);break;},
                Ok(_)=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
                Err(error)=>return Err(format!("{}: {error}",candidate.display())),
            }
        }
        let canonical=canonical.ok_or_else(||format!("Cannot resolve {path}; tried {}",candidates.iter().map(|path|path.display().to_string()).collect::<Vec<_>>().join(", ")))?;
        if importer.is_none(){crate::project_resolution::discover(&canonical,|path|match std::fs::read_to_string(path){Ok(text)=>Ok(Some(text)),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(e)=>Err(e.to_string())})?;}
        let identity = canonical
            .to_str()
            .ok_or("Source path must be valid UTF-8.")?;
        let id=SourceId(identity.to_owned());
        Ok(if importer.is_none(){bundled_entry(self,&id).unwrap_or(id)}else{id})
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        if let Some(source) = crate::stdlib::source(&id.0) {
            return Ok(source.to_owned());
        }
        std::fs::read_to_string(&id.0).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub source: SourceId,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ProjectDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub location: SourceLocation,
    pub related: Vec<(SourceLocation, String)>,
    pub fixes: Box<[ProjectFix]>,
}

#[derive(Clone, Debug)]
pub struct ProjectFix {
    pub title: String,
    pub location: SourceLocation,
    pub replacement: String,
}
fn error(
    id: &SourceId,
    code: &'static str,
    message: impl Into<String>,
    span: Span,
) -> ProjectDiagnostic {
    ProjectDiagnostic {
        code,
        message: message.into(),
        location: SourceLocation {
            source: id.clone(),
            span,
        },
        related: vec![],
        fixes: Box::new([]),
    }
}
fn source_error(id: &SourceId, diagnostic: Diagnostic) -> ProjectDiagnostic {
    ProjectDiagnostic {
        fixes: diagnostic
            .fixes
            .into_iter()
            .map(|fix| ProjectFix {
                title: fix.title,
                location: SourceLocation {
                    source: id.clone(),
                    span: fix.span,
                },
                replacement: fix.replacement,
            })
            .collect(),
        code: diagnostic.code,
        message: diagnostic.message,
        location: SourceLocation {
            source: id.clone(),
            span: diagnostic.span,
        },
        related: diagnostic
            .related
            .into_iter()
            .map(|(span, message)| {
                (
                    SourceLocation {
                        source: id.clone(),
                        span,
                    },
                    message,
                )
            })
            .collect(),
    }
}
#[derive(Clone, Debug)]
pub struct ImportBinding {
    pub exported: String,
    pub local: String,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ModuleImport {
    pub path: String,
    pub span: Span,
    /// The import binding range, including default and named bindings.
    pub names: Span,
    pub bindings: Vec<ImportBinding>,
    pub target: Option<SourceId>,
    /// Registry identity for a native standard-library import; never a disk source.
    pub native: Option<&'static str>,
}
/// Validated data is distinct from executable source declarations.
#[derive(Clone, Debug)]
pub struct DataModule {
    pub(crate) value: crate::data_module::Value,
}
#[derive(Clone, Debug)]
pub struct DataSchema {
    pub root: crate::type_syntax::ValueAnnotation,
    pub records: Vec<DataRecord>,
}
#[derive(Clone, Debug)]
pub struct DataRecord {
    pub name: String,
    pub span: Span,
    pub fields: Vec<DataField>,
}
#[derive(Clone, Debug)]
pub struct DataField {
    pub name: String,
    pub span: Span,
    pub annotation: crate::type_syntax::ValueAnnotation,
    pub optional: bool,
}
impl DataModule {
    /// Compiler-derived schema for editor assistance; no second JSON inference.
    pub fn schema(&self) -> DataSchema {
        let mut lowered = crate::data_lowering::lower(&self.value, "editor").expect("validated data module");
        let names: BTreeMap<_, _> = lowered.records.iter().map(|record| (record.name.text.clone(), format!("JsonData{}", record.name.span.start))).collect();
        let rename = |name: &str| names.get(name).cloned().unwrap_or_else(|| name.into());
        crate::type_aliases::link_names(&mut lowered.annotation, &rename);
        let records = lowered.records.into_iter().map(|record| DataRecord {
            name: names[&record.name.text].clone(), span: record.name.span,
            fields: record.fields.into_iter().map(|mut field| {
                crate::type_aliases::link_names(&mut field.annotation, &rename);
                DataField { name: field.name.text, span: field.name.span, annotation: field.annotation, optional: field.optional }
            }).collect(),
        }).collect();
        DataSchema { root: lowered.annotation, records }
    }
}
#[derive(Clone, Debug)]
pub struct ModuleSource {
    pub rust: Option<rust::Module>,
    pub data: Option<DataModule>,
    pub source: SourceFile,
    pub imports: Vec<ModuleImport>,
    pub exports: BTreeMap<String, Span>,
    pub default_export: Option<syntax::Token>,
    /// Import/export markers are replaced with spaces; byte coordinates are unchanged.
    pub declaration_source: String,
}
impl ModuleSource {
    /// Checked Rust adapter spans refer to generated declarations, not Rust bytes.
    pub fn checked_source(&self)->&str {if self.rust.is_some(){&self.declaration_source}else{&self.source.text}}
    pub fn exported_local<'a>(&'a self, name: &'a str) -> &'a str {
        if name == "default" { self.default_export.as_ref().map(|token|token.text.as_str()).unwrap_or(name) } else { name }
    }
}
#[derive(Clone, Debug)]
pub struct ProjectGraph {
    pub entry: SourceId,
    pub modules: BTreeMap<SourceId, ModuleSource>,
    /// Dependencies precede importers, in source import order.
    pub order: Vec<SourceId>,
}

/// Strict, unresolved module headers only. Resolution and export validation are
/// repeated against current inputs, so cached headers cannot preserve stale edges.
#[derive(Default)]
pub(crate) struct HeaderCache {
    entries:BTreeMap<SourceId,ModuleSource>,
    pub(crate) parsed:usize,
    pub(crate) reused:usize,
}
impl HeaderCache {
    pub(crate) fn begin(&mut self){self.parsed=0;self.reused=0;}
    pub(crate) fn clear(&mut self){self.entries.clear();}
    pub(crate) fn retain(&mut self,graph:&ProjectGraph)->usize {
        self.entries.retain(|id,_|graph.modules.contains_key(id));self.entries.len()
    }
    fn parse(&mut self,source:SourceFile)->Result<ModuleSource,ProjectDiagnostic> {
        if let Some(module)=self.entries.get(&source.id).filter(|module|module.source.text==source.text) {
            self.reused+=1;return Ok(module.clone());
        }
        self.parsed+=1;self.entries.remove(&source.id);
        let module=parse_module(source,true)?;
        self.entries.insert(module.source.id.clone(),module.clone());Ok(module)
    }
}

/// Conventional test entry names enable top-level registration in compiler/editor checks.
pub fn is_test_entry(path: &str) -> bool {
    path.ends_with(".test.tsr") || path.ends_with(".spec.tsr")
}

/// Tokens of a source with an unfinished string, template or comment. The rest of the
/// line that fails to lex is treated as blank, so later lines keep their declarations and
/// every token keeps its position in the original text.
pub(crate) fn lex_tolerant(text: &str) -> Result<Vec<syntax::Token>, Diagnostic> {
    let mut bytes = text.as_bytes().to_vec();
    loop {
        let current = std::str::from_utf8(&bytes).expect("blanking whole characters preserves UTF-8");
        let error = match syntax::lex(current) {
            Ok(tokens) => return Ok(tokens),
            Err(error) => error,
        };
        let mut start = error.span.start.min(bytes.len());
        while !current.is_char_boundary(start) {
            start -= 1;
        }
        let end = bytes[start..]
            .iter()
            .position(|byte| matches!(byte, b'\n' | b'\r'))
            .map_or(bytes.len(), |offset| start + offset);
        if bytes[start..end].iter().all(|byte| *byte == b' ') {
            return Err(error);
        }
        bytes[start..end].fill(b' ');
    }
}
pub(crate) fn parse_module(source: SourceFile, declarations: bool) -> Result<ModuleSource, ProjectDiagnostic> {
    if Path::new(&source.id.0).extension().is_some_and(|value| value == "rs") { return rust::module(source); }
    if let Some(extension) = Path::new(&source.id.0).extension().and_then(|value| value.to_str())
        && matches!(extension, "json" | "jsonc" | "txt")
    {
        let value = if extension=="txt" {crate::data_module::Value{span:Span::new(0,source.text.len()),kind:crate::data_module::Kind::String(source.text.clone())}}else{crate::data_module::parse(&source.text, extension == "jsonc").map_err(|failure| {
            let mut start = failure.offset.min(source.text.len());
            while !source.text.is_char_boundary(start) { start -= 1; }
            let end = start + source.text[start..].chars().next().map_or(0, char::len_utf8);
            error(&source.id, "data-json", format!("{} ({})", failure.message, failure.code), Span::new(start, end))
        })?};
        crate::data_shape::infer(&value).map_err(|failure| error(&source.id, "data-type", failure.message, failure.span))?;
        return Ok(ModuleSource {
            rust: None,
            exports: BTreeMap::from([("default".into(), value.span)]),
            default_export: Some(syntax::Token { text: crate::data_lowering::OWNER.into(), span: value.span, kind: TokenKind::Symbol }),
            data: Some(DataModule { value }), source, imports: vec![],
            declaration_source: String::new(),
        });
    }
    let id = &source.id;
    let tokens = if declarations {
        syntax::lex(&source.text).map_err(|e| source_error(id, e))?
    } else {
        lex_tolerant(&source.text).map_err(|e| source_error(id, e))?
    };
    let mut bytes = source.text.as_bytes().to_vec();
    let mut imports = vec![];
    let mut exports = BTreeMap::new();
    let mut default_export = None;
    let mut depth = 0usize;
    let mut i = 0;
    'scan: while i < tokens.len() && tokens[i].kind != TokenKind::End {
        let token = &tokens[i];
        // An export or import can only start at the top level. Header loading uses that to
        // leave a body whose closing brace is not written yet.
        let unfinished_body = !declarations
            && depth > 0
            && token.kind == TokenKind::Name
            && match token.text.as_str() {
                "export" => tokens.get(i + 1).is_some_and(|next| next.kind == TokenKind::Name && matches!(next.text.as_str(), "function" | "type" | "enum" | "interface" | "const" | "let"))
                    && tokens.get(i + 2).is_some_and(|name| name.kind == TokenKind::Name),
                "import" => tokens.get(i + 1).is_some_and(|next| next.kind == TokenKind::Symbol && next.text == "{"),
                _ => false,
            };
        if (depth == 0 || unfinished_body) && token.kind == TokenKind::Name && token.text == "import" {
            let start = token.span.start;
            depth = 0;
            // Header loading keeps the complete imports of a file while one is being typed.
            macro_rules! incomplete {
                ($error:expr) => {{
                    if declarations {
                        return Err($error);
                    }
                    continue 'scan;
                }};
            }
            i += 1;
            let fail = |index: usize, message: &str| {
                error(
                    id,
                    "module-syntax",
                    message,
                    tokens[index.min(tokens.len() - 1)].span,
                )
            };
            let mut bindings = vec![];
            let mut names_start = tokens[i].span.start;
            let has_default = tokens[i].kind == TokenKind::Name;
            if has_default {
                bindings.push(ImportBinding { exported: "default".into(), local: tokens[i].text.clone(), span: tokens[i].span });
                i += 1;
                if tokens[i].text == "," { i += 1; if tokens[i].text != "{" { incomplete!(fail(i,"Expected named bindings after the default import comma.")); } }
            }
            let names_end = if tokens[i].kind == TokenKind::Symbol && tokens[i].text == "{" {
            if !has_default {names_start=tokens[i].span.end;}
            i += 1;
            loop {

                if !declarations && tokens[i].kind == TokenKind::Symbol && tokens[i].text == "}" {
                    // Header loading accepts braces whose names are not written yet.
                    i += 1;
                    break tokens[i - 1].span.start;
                }
                if tokens[i].kind != TokenKind::Name {
                    incomplete!(fail(i, "Expected an imported name."));
                }
                let exported = tokens[i].text.clone();
                let span = tokens[i].span;
                let mut local = exported.clone();
                i += 1;
                if tokens[i].kind == TokenKind::Name && tokens[i].text == "as" {
                    i += 1;
                    if tokens[i].kind != TokenKind::Name {
                        incomplete!(fail(i, "Expected a local import alias."));
                    }
                    local = tokens[i].text.clone();
                    i += 1;
                }
                bindings.push(ImportBinding {
                    exported,
                    local,
                    span,
                });
                if tokens[i].kind == TokenKind::Symbol && tokens[i].text == "}" {
                    i += 1;
                    break tokens[i - 1].span.start;
                }
                if tokens[i].kind != TokenKind::Symbol || tokens[i].text != "," {
                    incomplete!(fail(i, "Expected ',' or '}' in named import."));
                }
                i += 1;
                if tokens[i].kind == TokenKind::Symbol && tokens[i].text == "}" {
                    i += 1;
                    break tokens[i - 1].span.start;
                }
            }
            } else if has_default { tokens[i-1].span.end } else {
                incomplete!(fail(i,"Expected a default or named import."));
            };
            if tokens[i].kind != TokenKind::Name || tokens[i].text != "from" {
                incomplete!(fail(i, "Expected 'from' after import bindings."));
            }
            i += 1;
            if tokens[i].kind != TokenKind::String {
                incomplete!(fail(i, "Expected a quoted module path."));
            }
            let path = tokens[i].text.clone();
            let span = tokens[i].span;
            if crate::stdlib::source(&path).is_none() && !path.starts_with("std:")
                && (path.is_empty() || path.starts_with('/')
                    || Path::new(&path).extension().is_some_and(|extension|!matches!(extension.to_str(), Some("tsr" | "tsrx" | "rs" | "json" | "jsonc" | "txt")))
                    || path.contains(['\\', '\0', '\n', '\r']))
            {
                return Err(error(
                    id,
                    "module-path",
                    "Imports require a relative source path, configured alias, or registered std: module.",
                    span,
                ));
            }
            i += 1;
            if tokens[i].kind != TokenKind::Symbol || tokens[i].text != ";" {
                incomplete!(fail(i, "Expected ';' after import."));
            }
            for byte in &mut bytes[start..tokens[i].span.end] {
                if !matches!(*byte, b'\n' | b'\r') {
                    *byte = b' ';
                }
            }
            imports.push(ModuleImport {
                path,
                span,
                names: Span::new(names_start, names_end),
                bindings,
                target: None,
                native: None,
            });
        } else if (depth == 0 || unfinished_body) && token.kind == TokenKind::Name && token.text == "export" {
            depth = 0;
            if tokens.get(i+1).is_some_and(|next|next.text=="default") {
                let marker=tokens[i+1].span;
                if exports.contains_key("default") {return Err(error(id,"module-duplicate","A module can have only one default export.",marker));}
                let asynchronous=tokens[i+2].text=="async";
                let declaration=i+2+usize::from(asynchronous);
                let name_index=declaration+1;
                let next=&tokens[declaration];
                let name;
                if next.text=="function" {
                    if tokens[name_index].kind==TokenKind::Name {
                        name=tokens[name_index].clone();
                        for byte in &mut bytes[token.span.start..if asynchronous {tokens[i+2].span.start}else{next.span.start}] {if !matches!(*byte,b'\n'|b'\r'){*byte=b' ';}}
                    } else if matches!(tokens[name_index].text.as_str(),"("|"<") {
                        let mut serial=0;
                        let local=loop {let candidate=format!("__d{serial}");if !tokens.iter().any(|token|token.text==candidate){break candidate;}serial+=1;};
                        let replacement=format!("{}function {local}",if asynchronous{"async "}else{""});
                        let start=token.span.start;let end=next.span.end;
                        // Reuse the removed export prefix without shifting parameter/body spans.
                        bytes[start..end].fill(b' ');bytes[start..start+replacement.len()].copy_from_slice(replacement.as_bytes());
                        name=syntax::Token {kind:TokenKind::Name,text:local,span:Span::new(start+if asynchronous{15}else{9},start+replacement.len())};
                    } else {return Err(error(id,"module-export","Expected a default function declaration.",next.span));}
                    i=declaration;
                } else if next.kind==TokenKind::Name && tokens.get(i+3).is_some_and(|end|end.text==";") {
                    name=next.clone();
                    for byte in &mut bytes[token.span.start..tokens[name_index].span.end] {if !matches!(*byte,b'\n'|b'\r'){*byte=b' ';}}
                    i+=3;
                } else {return Err(error(id,"module-export","Default exports require a function declaration or a local declaration name.",next.span));}
                exports.insert("default".into(),name.span);default_export=Some(name);
                i+=1;continue;
            }
            let declaration = if tokens.get(i+1).is_some_and(|token|token.text=="async") {
                if tokens.get(i+2).is_none_or(|token|token.text!="function") {return Err(error(id,"module-export","export async requires a function declaration.",token.span));}
                i+2
            } else {i+1};
            if tokens[declaration].kind != TokenKind::Name
                || !matches!(tokens[declaration].text.as_str(), "function" | "type" | "enum" | "interface" | "const" | "let")
                || tokens.get(declaration+1).is_none_or(|token|token.kind!=TokenKind::Name)
            {
                return Err(error(
                    id,
                    "module-export",
                    "Supported exports are function, type, interface, enum, const, let declarations and default functions.",
                    token.span,
                ));
            }
            let name = &tokens[declaration+1];
            if exports.insert(name.text.clone(), name.span).is_some() {
                return Err(error(
                    id,
                    "module-duplicate",
                    "Duplicate exported name.",
                    name.span,
                ));
            }
            bytes[token.span.start..token.span.end].fill(b' ');
        } else if token.kind == TokenKind::Symbol {
            if token.text == "{" {
                depth += 1;
            }
            if token.text == "}" {
                depth = depth.saturating_sub(1);
            }
        }
        i += 1;
    }
    let declaration_source =
        String::from_utf8(bytes).expect("whitespace replacement preserves UTF-8");
    let mut names = BTreeMap::new();
    let declared = if declarations {
        let parsed = syntax::parse_module(&declaration_source, is_test_entry(&id.0), Default::default(), Default::default());
        let program = parsed.map_err(|e| source_error(id, e))?;
        if let Some(default)=&default_export {
            if !program.functions.iter().any(|function|function.name.text==default.text)
                && !program.module_bindings.iter().any(|statement| matches!(&statement.kind, syntax::StmtKind::Let { name, .. } if name.text == default.text)) {
                return Err(error(id,"module-export","Default export must refer to a local function or module binding.",default.span));
            }
        }
        program
            .records
            .iter()
            .map(|record| record.name.clone())
            .chain(
                program
                    .functions
                    .iter()
                    .map(|function| function.name.clone()),
            )
            .chain(program.module_bindings.iter().filter_map(|statement| match &statement.kind {
                syntax::StmtKind::Let { name, .. } => Some(name.clone()),
                _ => None,
            }))
            .collect::<Vec<_>>()
    } else {
        // Only top-level declaration names are inspected; bodies can be incomplete.
        let mut depth = 0usize;
        let mut declared = vec![];
        for pair in tokens.windows(2) {
            if depth == 0
                && pair[0].kind == TokenKind::Name
                && matches!(pair[0].text.as_str(), "function" | "type" | "enum" | "interface" | "const" | "let")
                && pair[1].kind == TokenKind::Name
            {
                declared.push(pair[1].clone());
            }
            if pair[0].kind == TokenKind::Symbol {
                match pair[0].text.as_str() {
                    "{" => depth += 1,
                    "}" => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        declared
    };
    for name in declared {
        if let Some(previous) = names.insert(name.text.clone(), name.span) {
            let mut diagnostic = error(
                id,
                "module-duplicate",
                format!("Duplicate declaration {:?}.", name.text),
                name.span,
            );
            diagnostic.related.push((
                SourceLocation {
                    source: id.clone(),
                    span: previous,
                },
                "Previous declaration here.".into(),
            ));
            return Err(diagnostic);
        }
    }
    for import in &imports {
        for binding in &import.bindings {
            if names.insert(binding.local.clone(), binding.span).is_some() {
                return Err(error(
                    id,
                    "module-duplicate",
                    format!(
                        "Import name {:?} conflicts with another binding.",
                        binding.local
                    ),
                    binding.span,
                ));
            }
        }
    }
    Ok(ModuleSource {
        rust: None,
        data: None,
        source,
        imports,
        exports,
        default_export,
        declaration_source,
    })
}

/// Load and validate a module graph without type checking, native builds or execution.
pub fn load_project(
    provider: &impl SourceProvider,
    entry: &str,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    load_graph(provider, entry, true, None)
}

/// Check normally, then collect independent semantic errors or module syntax errors.
/// Header graphs and recovered syntax never produce executable analysis.
pub fn check_with_diagnostics(
    provider: &impl SourceProvider,
    entry: &str,
) -> Result<ProjectAnalysis, Vec<ProjectDiagnostic>> {
    let mut errors = match load_project(provider, entry) {
        Ok(graph)=>match crate::project_link::check_graph_diagnostics(graph) {
            Ok(analysis)=>return Ok(analysis),
            Err(errors)=>errors,
        },
        Err(error)=>vec![error],
    };
    if !errors.iter().any(|error|error.code=="syntax") {return Err(errors);}
    if let Ok(graph) = load_project_headers(provider, entry) {
        for (id, module) in &graph.modules {
            errors.extend(syntax::module_diagnostics(&module.declaration_source, is_test_entry(&id.0))
                .into_iter().map(|error| source_error(id, error)));
        }
    }
    errors.sort_by(|a,b| (&a.location.source, a.location.span.start, a.location.span.end, a.code)
        .cmp(&(&b.location.source, b.location.span.start, b.location.span.end, b.code)));
    errors.dedup_by(|a,b| a.location == b.location && a.code == b.code && a.message == b.message);
    Err(errors)
}

/// Validate module headers and dependencies for tolerant editor assistance.
/// Bodies need not parse. This is not evidence that a project compiles; use load_project/check_graph for compilation.
pub fn load_project_headers(
    provider: &impl SourceProvider,
    entry: &str,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    load_graph(provider, entry, false, None)
}
pub(crate) fn load_project_cached(provider:&impl SourceProvider,entry:&str,cache:&mut HeaderCache)->Result<ProjectGraph,ProjectDiagnostic> {
    load_graph(provider,entry,true,Some(cache))
}
fn load_graph(
    provider: &impl SourceProvider,
    entry: &str,
    declarations: bool,
    mut cache:Option<&mut HeaderCache>,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    let entry = provider.resolve(None, entry).map_err(|message| {
        // Keep entry coordinates available when configuration fails before traversal.
        let _=provider.read(&SourceId(entry.into()));
        error(
            &SourceId(entry.into()),
            "module-load",
            message,
            Span::default(),
        )
    })?;
    if declarations && !entry.0.starts_with("std:") {
        for root in Path::new(&entry.0).parent().unwrap_or(Path::new(".")).ancestors() {
            let config=SourceId(root.join("tsrust.config.json").to_string_lossy().into_owned());
            if let Ok(text)=provider.read(&config) {
                let value:serde_json::Value=serde_json::from_str(&text).map_err(|e|error(&config,"ui-config",e.to_string(),Span::default()))?;
                if let Some(ui)=value.get("ui") {crate::ui_project::Config::parse(ui).map_err(|e|error(&config,"ui-theme",e,Span::new(0,text.len().min(1))))?;}
                break;
            }
        }
    }
    let mut graph = ProjectGraph {
        entry: entry.clone(),
        modules: BTreeMap::new(),
        order: vec![],
    };
    let mut active = BTreeSet::new();
    let mut chain = vec![];
    visit(
        provider,
        entry,
        &mut graph,
        &mut active,
        &mut chain,
        declarations,
        &mut cache,
    )?;
    let mut package:Option<&rust::dependencies::Package>=None;
    for (id,module) in &graph.modules {
        if let Some(current)=module.rust.as_ref().and_then(|rust|rust.package.as_ref()) {
            if package.is_some_and(|package|package.manifest!=current.manifest || package.lock!=current.lock){return Err(error(id,"rust-bindings","Binding modules in one program must share dependency versions, features and lockfile.",Span::default()));}
            package=Some(current);
        }
    }
    Ok(graph)
}
fn visit(
    provider: &impl SourceProvider,
    id: SourceId,
    graph: &mut ProjectGraph,
    active: &mut BTreeSet<SourceId>,
    chain: &mut Vec<(SourceId, Span)>,
    declarations: bool,
    cache:&mut Option<&mut HeaderCache>,
) -> Result<(), ProjectDiagnostic> {
    active.insert(id.clone());
    let text = if let Some(source) = crate::stdlib::source(&id.0) {
        source.to_owned()
    } else {
        provider.read(&id)
            .map_err(|message| error(&id, "module-load", message, Span::default()))?
    };
    if declarations && id.0.ends_with(".json") && crate::ui_project::configured_theme(Path::new(&id.0),|path|{
        let source=SourceId(path.to_string_lossy().into_owned());
        match provider.read(&source) {Ok(value)=>Ok(Some(value)),Err(_)=>Ok(None)}
    }).map_err(|message|error(&id,"ui-config",message,Span::default()))? {
        crate::ui_project::theme::Theme::parse(&crate::ui_project::theme_document(Path::new(&id.0),&text).map(|(value,_)|value.to_string()).unwrap_or_else(||text.clone())).map_err(|message|error(&id,"ui-theme",message,Span::new(0,text.len().min(1))))?;
    }
    let source=SourceFile{id:id.clone(),text};
    let mut module=if id.0.ends_with(".bindings.json") {
        let (mut module,inputs)=rust::bindings(provider,source)?;
        for input in inputs {
            if graph.modules.contains_key(&input.id){return Err(error(&id,"rust-bindings","Binding inputs cannot also be imported directly.",Span::default()));}
            let target=input.id.clone();
            graph.order.push(target.clone());
            graph.modules.insert(target.clone(),ModuleSource{source:input,rust:None,data:None,imports:vec![],exports:BTreeMap::new(),default_export:None,declaration_source:String::new()});
            module.imports.push(ModuleImport{path:target.0.clone(),span:Span::default(),names:Span::default(),bindings:vec![],target:Some(target),native:None});
        }
        module
    }else if let Some(cache)=cache.as_deref_mut(){cache.parse(source)?}else{parse_module(source,declarations)?};
    for import in &mut module.imports {
        if import.target.is_some(){continue;}
        if import.path == "std:ui-runtime" && id.0 != "std:ui" {
            return Err(error(&id, "module-native",
                "UI runtime bindings are private; import from std:ui.",
                import.span,
            ));
        }
        if import.path == "std:testing-runtime" && id.0 != "std:testing" {
            return Err(error(&id, "module-native", "Testing runtime bindings are private; import from std:testing.", import.span));
        }
        if crate::stdlib::source(&import.path).is_none() && import.path.starts_with("std:") {
            if !declarations
                && let Some(module) = crate::native_exports::known_module(&import.path)
            {
                // A native name that is still being typed is left out, as for a
                // source module; the known module still offers its exports.
                import.bindings.retain(|binding| crate::native_exports::lookup(module, &binding.exported).is_ok());
                import.native = Some(module);
            }
            for binding in &import.bindings {
                let native = crate::native_exports::lookup(&import.path, &binding.exported)
                    .map_err(|message| error(&id, "module-native", message, binding.span))?;
                import.native = Some(native.module());
            }
            continue;
        }
        let target = if crate::stdlib::source(&import.path).is_some() {
            SourceId(import.path.clone())
        } else {
            provider.resolve(Some(&id), &import.path)
                .map_err(|message| error(&id, "module-load", message, import.span))?
        };
        if active.contains(&target) {
            let mut diagnostic = error(
                &id,
                "module-cycle",
                format!("Import cycle reaches {}.", target.0),
                import.span,
            );
            diagnostic.related = chain
                .iter()
                .map(|(source, span)| {
                    (
                        SourceLocation {
                            source: source.clone(),
                            span: *span,
                        },
                        "Import path here.".into(),
                    )
                })
                .collect();
            return Err(diagnostic);
        }
        if !graph.modules.contains_key(&target) {
            if chain.len() >= 128 {
                return Err(error(
                    &id,
                    "module-depth",
                    "Module import depth exceeds 128.",
                    import.span,
                ));
            }
            chain.push((id.clone(), import.span));
            visit(provider, target.clone(), graph, active, chain, declarations, cache)?;
            chain.pop();
        }
        let dependency = &graph.modules[&target];
        if !declarations {
            // A name that is still being typed is not an export yet. Header loading keeps
            // the other bindings; strict loading reports the name below.
            import.bindings.retain(|binding| dependency.exports.contains_key(&binding.exported));
        }
        for binding in &import.bindings {
            if !dependency.exports.contains_key(&binding.exported) {
                return Err(error(
                    &id,
                    "module-private",
                    format!("{} does not export {:?}.", target.0, binding.exported),
                    binding.span,
                ));
            }
        }
        import.target = Some(target);
    }
    active.remove(&id);
    graph.order.push(id.clone());
    graph.modules.insert(id, module);
    Ok(())
}

#[cfg(test)]
mod data_project_tests;

pub mod cli;
pub(crate) mod facts;
pub(crate) mod link;
pub mod resolution;
pub(crate) mod runtime;
pub mod session;
pub mod source_discovery;

pub(crate) mod data;
