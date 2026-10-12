//! CLI adapter preserving the existing single-file output contract.
use crate::project::{
    FileSourceProvider, ProjectAnalysis, ProjectDiagnostic, SourceProvider, check_graph,
    load_project,
};
use crate::{Analysis, ClosureInfo, Diagnostic, Mode, Span, check, json_string, line_column};
use std::collections::BTreeMap;
use std::path::Path;

pub enum CheckedInput {
    Single(Analysis),
    Project(ProjectAnalysis),
}
pub enum InputDiagnostic {
    Single(Diagnostic),
    Project(ProjectDiagnostic),
}
pub fn check_input(path: &Path, source: &str) -> Result<CheckedInput, InputDiagnostic> {
    let module = uses_modules(source).map_err(InputDiagnostic::Single)?
        || path.to_str().is_some_and(crate::project::is_test_entry);
    if module {
        let path = path.to_str().ok_or_else(|| {
            InputDiagnostic::Single(Diagnostic::new(
                "module-path",
                "Source path must be valid UTF-8.",
                Span::default(),
            ))
        })?;
        check_graph(load_project(&FileSourceProvider, path).map_err(InputDiagnostic::Project)?)
            .map(CheckedInput::Project)
            .map_err(InputDiagnostic::Project)
    } else {
        check(source)
            .map(CheckedInput::Single)
            .map_err(InputDiagnostic::Single)
    }
}
/// Retain module parsing between dev checks while preserving single-file semantics.
pub fn check_input_cached(path:&Path,source:&str,session:&mut crate::project_session::ProjectSession)->Result<CheckedInput,InputDiagnostic> {
    check_input_cached_with_provider(path,source,session,&FileSourceProvider)
}
/// Check and resolve against one caller-owned source provider.
pub fn check_input_cached_with_provider(path:&Path,source:&str,session:&mut crate::project_session::ProjectSession,provider:&impl SourceProvider)->Result<CheckedInput,InputDiagnostic> {
    if uses_modules(source).map_err(InputDiagnostic::Single)? || path.to_str().is_some_and(crate::project::is_test_entry) {
        let path=path.to_str().ok_or_else(||InputDiagnostic::Single(Diagnostic::new("module-path","Source path must be valid UTF-8.",Span::default())))?;
        session.check(provider,path).map(CheckedInput::Project).map_err(InputDiagnostic::Project)
    } else {
        *session=crate::project_session::ProjectSession::default();
        check(source).map(CheckedInput::Single).map_err(InputDiagnostic::Single)
    }
}
/// Diagnostic checking shares strict successful analysis and collects parser errors.
pub fn check_input_diagnostics(path: &Path, source: &str) -> Result<CheckedInput, Vec<InputDiagnostic>> {
    let module = uses_modules(source).map_err(|error| vec![InputDiagnostic::Single(error)])?
        || path.to_str().is_some_and(crate::project::is_test_entry);
    if module {
        let path = path.to_str().ok_or_else(|| vec![InputDiagnostic::Single(Diagnostic::new(
            "module-path", "Source path must be valid UTF-8.", Span::default()))])?;
        crate::project::check_with_diagnostics(&FileSourceProvider, path)
            .map(CheckedInput::Project)
            .map_err(|errors| errors.into_iter().map(InputDiagnostic::Project).collect())
    } else {
        match check(source) {
            Ok(analysis) => Ok(CheckedInput::Single(analysis)),
            Err(error) => Err(crate::diagnostic_error(source,error).into_iter().map(InputDiagnostic::Single).collect()),
        }
    }
}

/// Check a test entry through the module graph without rewriting source coordinates.
pub fn check_test_input(path: &Path) -> Result<CheckedInput, InputDiagnostic> {
    let path = path.to_str().ok_or_else(|| InputDiagnostic::Single(Diagnostic::new("module-path", "Source path must be valid UTF-8.", Span::default())))?;
    crate::project_link::check_test_graph(crate::project::load_project_headers(&FileSourceProvider, path).map_err(InputDiagnostic::Project)?)
        .map(CheckedInput::Project).map_err(InputDiagnostic::Project)
}
/// Load selected test entries once and execute all registrations in one native registry.
pub fn check_test_inputs(paths: &[std::path::PathBuf]) -> Result<CheckedInput, InputDiagnostic> {
    let mut graph: Option<crate::project::ProjectGraph> = None;
    let mut entries = Vec::new();
    for path in paths {
        let path = path.to_str().ok_or_else(|| InputDiagnostic::Single(Diagnostic::new("module-path", "Source path must be valid UTF-8.", Span::default())))?;
        let loaded = crate::project::load_project_headers(&FileSourceProvider, path).map_err(InputDiagnostic::Project)?;
        if entries.contains(&loaded.entry) { continue; }
        entries.push(loaded.entry.clone());
        if let Some(graph) = &mut graph {
            for id in loaded.order {
                if !graph.modules.contains_key(&id) {
                    graph.modules.insert(id.clone(), loaded.modules[&id].clone());
                    graph.order.push(id);
                }
            }
        } else { graph = Some(loaded); }
    }
    let graph = graph.ok_or_else(|| InputDiagnostic::Single(Diagnostic::new("test-entry", "No test files selected.", Span::default())))?;
    crate::project_link::check_test_graph_entries(graph, &entries).map(CheckedInput::Project).map_err(InputDiagnostic::Project)
}

/// Deterministic file selection; discovery does not read or execute source files.
pub fn discover_test_inputs(paths: &[std::path::PathBuf]) -> Result<Vec<std::path::PathBuf>, String> {
    crate::stdlib::testing::discovery::select(paths)
}
impl CheckedInput {
    pub fn emit_rust(&self) -> String {
        match self {
            Self::Single(analysis) => analysis.emit_rust(),
            Self::Project(analysis) => analysis.emit_rust(),
        }
    }
    pub fn emit_rust_with_native_features(&self) -> (String, Vec<&'static str>) {
        match self {
            Self::Single(analysis) => analysis.emit_rust_with_native_features(),
            Self::Project(analysis) => analysis.emit_rust_with_native_features(),
        }
    }
    pub fn contracts(&self) -> BTreeMap<String, Mode> {
        match self {
            Self::Single(analysis) => analysis.contracts(),
            Self::Project(analysis) => analysis.contracts(),
        }
    }
    pub fn closures(&self) -> Vec<ClosureInfo> {
        match self {
            Self::Single(analysis) => analysis.closures(),
            Self::Project(analysis) => analysis.closures(),
        }
    }
    pub fn to_json(&self) -> String {
        match self {
            Self::Single(analysis) => analysis.to_json(),
            Self::Project(analysis) => analysis.to_json(),
        }
    }
}
impl InputDiagnostic {
    pub fn render(&self, path: &str, source: &str, color: bool) -> String {
        self.render_with_provider(path,source,color,&FileSourceProvider)
    }
    /// Render the same source versions that produced the diagnostic.
    pub fn render_with_provider(&self,path:&str,source:&str,color:bool,provider:&impl SourceProvider)->String {
        match self {
            Self::Single(diagnostic) => diagnostic.render_terminal(path, source, color),
            Self::Project(diagnostic) => {
                let text = provider
                    .read(&diagnostic.location.source)
                    .unwrap_or_default();
                let mut result = Diagnostic::new(
                    diagnostic.code,
                    &diagnostic.message,
                    diagnostic.location.span,
                )
                .render_terminal(&diagnostic.location.source.0, &text, color);
                for (location, message) in &diagnostic.related {
                    let text = provider.read(&location.source).unwrap_or_default();
                    let (line, column) = line_column(&text, location.span);
                    result.push_str(&format!(
                        "  {}:{line}:{column}: {message}\n",
                        location.source.0
                    ));
                }
                result
            }
        }
    }
    pub fn to_json(&self, source: &str) -> String {
        match self {
            Self::Single(diagnostic) => diagnostic.to_json(source),
            Self::Project(diagnostic) => {
                let related = diagnostic
                    .related
                    .iter()
                    .map(|(location, message)| {
                        format!(
                            "{{\"message\":{},\"location\":{}}}",
                            json_string(message),
                            location_json(location)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"code\":{},\"message\":{},\"location\":{},\"related\":[{related}]}}",
                    json_string(diagnostic.code),
                    json_string(&diagnostic.message),
                    location_json(&diagnostic.location)
                )
            }
        }
    }
}
fn location_json(location: &crate::project::SourceLocation) -> String {
    let text = FileSourceProvider
        .read(&location.source)
        .unwrap_or_default();
    let (line, column) = line_column(&text, location.span);
    format!(
        "{{\"file\":{},\"line\":{line},\"column\":{column},\"start\":{},\"end\":{}}}",
        json_string(&location.source.0),
        location.span.start,
        location.span.end
    )
}

/// Detect real top-level module declarations without interpreting comment or string contents.
pub fn uses_modules(source: &str) -> Result<bool, Diagnostic> {
    // An unfinished string or comment must not hide the module declarations around it.
    let tokens = crate::project::lex_tolerant(source)?;
    let mut depth = 0usize;
    Ok(tokens.iter().enumerate().any(|(index, token)| {
        if token.kind == crate::syntax::TokenKind::Symbol {
            if token.text == "{" {
                depth += 1;
            }
            if token.text == "}" {
                depth = depth.saturating_sub(1);
            }
        }
        if token.kind != crate::syntax::TokenKind::Name {
            return false;
        }
        let name = |offset: usize, texts: &[&str]| {
            tokens.get(index + offset).is_some_and(|next| {
                next.kind == crate::syntax::TokenKind::Name
                    && (texts.is_empty() || texts.contains(&next.text.as_str()))
            })
        };
        // A declaration that can only start at the top level also ends a body whose
        // closing brace is not written yet.
        match token.text.as_str() {
            "export" => depth == 0 || (name(1, &["function", "type", "enum", "interface"]) && name(2, &[])),
            "import" => {
                depth == 0
                    || tokens.get(index + 1).is_some_and(|next| {
                        next.kind == crate::syntax::TokenKind::Symbol && next.text == "{"
                    })
            }
            _ => false,
        }
    }))
}

/// Identify a top-level main declaration without treating member/string text as code.
pub fn declares_main(source: &str) -> Result<bool, Diagnostic> {
    // An unfinished string or comment must not hide the module declarations around it.
    let tokens = crate::project::lex_tolerant(source)?;
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if depth == 0
            && token.kind == crate::syntax::TokenKind::Name
            && token.text == "function"
            && tokens.get(index + 1).is_some_and(|name| {
                name.kind == crate::syntax::TokenKind::Name && name.text == "main"
            })
        {
            return Ok(true);
        }
        if token.kind == crate::syntax::TokenKind::Symbol {
            if token.text == "{" {
                depth += 1;
            }
            if token.text == "}" {
                depth = depth.saturating_sub(1);
            }
        }
    }
    Ok(false)
}
