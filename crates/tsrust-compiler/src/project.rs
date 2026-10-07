//! Source-indexed module graph loading. This does not type-check or emit a project.
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

pub struct FileSourceProvider;
impl SourceProvider for FileSourceProvider {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let path = match importer {
            Some(id) => Path::new(&id.0)
                .parent()
                .unwrap_or(Path::new("."))
                .join(path),
            None => PathBuf::from(path),
        };
        let canonical =
            std::fs::canonicalize(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let identity = canonical
            .to_str()
            .ok_or("Source path must be valid UTF-8.")?;
        Ok(SourceId(identity.to_owned()))
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
    pub bindings: Vec<ImportBinding>,
    pub target: Option<SourceId>,
    /// Registry identity for a native standard-library import; never a disk source.
    pub native: Option<&'static str>,
}
#[derive(Clone, Debug)]
pub struct ModuleSource {
    pub source: SourceFile,
    pub imports: Vec<ModuleImport>,
    pub exports: BTreeMap<String, Span>,
    /// Import/export markers are replaced with spaces; byte coordinates are unchanged.
    pub declaration_source: String,
}
#[derive(Clone, Debug)]
pub struct ProjectGraph {
    pub entry: SourceId,
    pub modules: BTreeMap<SourceId, ModuleSource>,
    /// Dependencies precede importers, in source import order.
    pub order: Vec<SourceId>,
}

/// Conventional test entry names enable top-level registration in compiler/editor checks.
pub fn is_test_entry(path: &str) -> bool {
    path.ends_with(".test.tsr") || path.ends_with(".spec.tsr")
}

fn parse_module(source: SourceFile, declarations: bool) -> Result<ModuleSource, ProjectDiagnostic> {
    let id = &source.id;
    let tokens = syntax::lex(&source.text).map_err(|e| source_error(id, e))?;
    let mut bytes = source.text.as_bytes().to_vec();
    let mut imports = vec![];
    let mut exports = BTreeMap::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < tokens.len() && tokens[i].kind != TokenKind::End {
        let token = &tokens[i];
        if depth == 0 && token.kind == TokenKind::Name && token.text == "import" {
            let start = token.span.start;
            i += 1;
            let fail = |index: usize, message: &str| {
                error(
                    id,
                    "module-syntax",
                    message,
                    tokens[index.min(tokens.len() - 1)].span,
                )
            };
            if tokens[i].kind != TokenKind::Symbol || tokens[i].text != "{" {
                return Err(fail(
                    i,
                    "Expected named import: import { name } from './file.tsr';",
                ));
            }
            i += 1;
            let mut bindings = vec![];
            loop {
                if tokens[i].kind != TokenKind::Name {
                    return Err(fail(i, "Expected an imported name."));
                }
                let exported = tokens[i].text.clone();
                let span = tokens[i].span;
                let mut local = exported.clone();
                i += 1;
                if tokens[i].kind == TokenKind::Name && tokens[i].text == "as" {
                    i += 1;
                    if tokens[i].kind != TokenKind::Name {
                        return Err(fail(i, "Expected a local import alias."));
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
                    break;
                }
                if tokens[i].kind != TokenKind::Symbol || tokens[i].text != "," {
                    return Err(fail(i, "Expected ',' or '}' in named import."));
                }
                i += 1;
                if tokens[i].kind == TokenKind::Symbol && tokens[i].text == "}" {
                    i += 1;
                    break;
                }
            }
            if tokens[i].kind != TokenKind::Name || tokens[i].text != "from" {
                return Err(fail(i, "Expected 'from' after import bindings."));
            }
            i += 1;
            if tokens[i].kind != TokenKind::String {
                return Err(fail(i, "Expected a quoted module path."));
            }
            let path = tokens[i].text.clone();
            let span = tokens[i].span;
            if crate::stdlib::source(&path).is_none() && !path.starts_with("std:")
                && (!(path.starts_with("./") || path.starts_with("../"))
                    || !path.ends_with(".tsr")
                    || path.contains(['\\', '\0', '\n', '\r']))
            {
                return Err(error(
                    id,
                    "module-path",
                    "Imports require an explicit relative .tsr path or a registered std: module.",
                    span,
                ));
            }
            i += 1;
            if tokens[i].kind != TokenKind::Symbol || tokens[i].text != ";" {
                return Err(fail(i, "Expected ';' after import."));
            }
            for byte in &mut bytes[start..tokens[i].span.end] {
                if !matches!(*byte, b'\n' | b'\r') {
                    *byte = b' ';
                }
            }
            imports.push(ModuleImport {
                path,
                span,
                bindings,
                target: None,
                native: None,
            });
        } else if depth == 0 && token.kind == TokenKind::Name && token.text == "export" {
            if tokens[i + 1].kind != TokenKind::Name
                || !matches!(tokens[i + 1].text.as_str(), "function" | "type" | "enum" | "interface")
                || tokens[i + 2].kind != TokenKind::Name
            {
                return Err(error(
                    id,
                    "module-export",
                    "Only export function, export type, export interface, and export enum declarations are supported.",
                    token.span,
                ));
            }
            let name = &tokens[i + 2];
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
        let parsed = if is_test_entry(&id.0) { syntax::parse_test_entry(&declaration_source) } else { syntax::parse(&declaration_source) };
        let program = parsed.map_err(|e| source_error(id, e))?;
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
            .collect::<Vec<_>>()
    } else {
        // Only top-level declaration names are inspected; bodies can be incomplete.
        let mut depth = 0usize;
        let mut declared = vec![];
        for pair in tokens.windows(2) {
            if depth == 0
                && pair[0].kind == TokenKind::Name
                && matches!(pair[0].text.as_str(), "function" | "type" | "enum" | "interface")
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
        source,
        imports,
        exports,
        declaration_source,
    })
}

/// Load and validate a module graph without type checking, native builds or execution.
pub fn load_project(
    provider: &impl SourceProvider,
    entry: &str,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    load_graph(provider, entry, true)
}

/// Validate module headers and dependencies for tolerant editor assistance.
/// Bodies need not parse. This is not evidence that a project compiles; use load_project/check_graph for compilation.
pub fn load_project_headers(
    provider: &impl SourceProvider,
    entry: &str,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    load_graph(provider, entry, false)
}
fn load_graph(
    provider: &impl SourceProvider,
    entry: &str,
    declarations: bool,
) -> Result<ProjectGraph, ProjectDiagnostic> {
    let entry = provider.resolve(None, entry).map_err(|message| {
        error(
            &SourceId(entry.into()),
            "module-load",
            message,
            Span::default(),
        )
    })?;
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
    )?;
    Ok(graph)
}
fn visit(
    provider: &impl SourceProvider,
    id: SourceId,
    graph: &mut ProjectGraph,
    active: &mut BTreeSet<SourceId>,
    chain: &mut Vec<(SourceId, Span)>,
    declarations: bool,
) -> Result<(), ProjectDiagnostic> {
    active.insert(id.clone());
    let text = if let Some(source) = crate::stdlib::source(&id.0) {
        source.to_owned()
    } else {
        provider.read(&id)
            .map_err(|message| error(&id, "module-load", message, Span::default()))?
    };
    let mut module = parse_module(
        SourceFile {
            id: id.clone(),
            text,
        },
        declarations,
    )?;
    for import in &mut module.imports {
        if import.path == "std:testing-runtime" && id.0 != "std:testing" {
            return Err(error(&id, "module-native", "Testing runtime bindings are private; import from std:testing.", import.span));
        }
        if crate::stdlib::source(&import.path).is_none() && import.path.starts_with("std:") {
            for binding in &import.bindings {
                let native = crate::native_bindings::lookup(&import.path, &binding.exported)
                    .map_err(|message| error(&id, "module-native", message, binding.span))?;
                import.native = Some(native.module);
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
            visit(provider, target.clone(), graph, active, chain, declarations)?;
            chain.pop();
        }
        let dependency = &graph.modules[&target];
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
