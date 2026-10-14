//! AST-level module linking with explicit source coordinate segments.
use crate::project::{ProjectDiagnostic, ProjectGraph, SourceId, SourceLocation};
use crate::syntax::*;
use crate::{Analysis, Diagnostic, Span};
use std::collections::{BTreeMap, BTreeSet};

// Call type arguments are linked before local declarations are lifted. Include
// their lexical variables so they are not qualified as module-level type names.
fn extend_nested_call_types(functions: &[Function], span: Span, variables: &mut BTreeSet<String>) {
    let mut pending = functions.iter().filter(|function| function.scope_span.start <= span.start && span.end <= function.scope_span.end)
        .map(|function| (function.body.as_slice(), variables.clone())).collect::<Vec<_>>();
    while let Some((body, scope)) = pending.pop() {
        for statement in body {
            match &statement.kind {
                StmtKind::LocalFunction(function) if function.scope_span.start <= span.start && span.end <= function.scope_span.end => {
                    let mut nested = scope.clone();
                    nested.extend(function.type_parameters.iter().map(|parameter| parameter.text.clone()));
                    variables.extend(nested.iter().cloned());
                    pending.push((&function.body, nested));
                }
                StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } | StmtKind::Loop { body, .. } => pending.push((body, scope.clone())),
                StmtKind::If { then_body, else_body, .. } => {
                    pending.push((then_body, scope.clone()));
                    pending.push((else_body, scope.clone()));
                }
                StmtKind::Try(region) => {
                    pending.push((&region.body, scope.clone()));
                    if let Some(handler) = &region.catch { pending.push((&handler.body, scope.clone())); }
                    if let Some(finalizer) = &region.finally { pending.push((finalizer, scope.clone())); }
                }
                _ => {}
            }
        }
    }
}

/// Pattern names are locals for later parameter defaults, even though the
/// declarations remain in the leading body for checked source traversal.
#[inline(never)]
fn parameter_pattern_names(parameter: &Token, body: &[Stmt], locals: &mut BTreeSet<String>) {
    if !parameter.text.starts_with("pattern#") { return; }
    for stmt in body {
        if stmt.span.start >= parameter.span.start && stmt.span.end <= parameter.span.end {
            if let StmtKind::Let { name, .. } = &stmt.kind { locals.insert(name.text.clone()); }
        }
    }
}


pub struct ProjectAnalysis {
    pub graph: ProjectGraph,
    pub(crate) analysis: Analysis,
    segments: Vec<(SourceId, usize, usize)>,
}
impl ProjectAnalysis {
    pub fn emit_rust(&self) -> String {
        self.emit_rust_with_native_features().0
    }
    pub fn emit_rust_with_native_features(&self) -> (String, Vec<&'static str>) {
        crate::emit::emit_with_source_segments(
            &self.analysis.program,
            &crate::project_runtime::setup(&self.segments),
            &self.segments,
            Some(&self.graph),
        )
    }
    fn source_name(&self, name: &str) -> String {
        for (module, source) in self.graph.order.iter().enumerate() {
            let prefix = format!("__tsrust_module_{module}_");
            if let Some(name) = name.strip_prefix(&prefix) {
                return format!("{}::{name}", source.0);
            }
        }
        format!("{}::{name}", self.graph.entry.0)
    }
    pub fn contracts(&self) -> BTreeMap<String, crate::Mode> {
        self.analysis
            .contracts()
            .into_iter()
            .map(|(name, mode)| (self.source_name(&name), mode))
            .collect()
    }
    pub fn closures(&self) -> Vec<crate::ClosureInfo> {
        self.analysis
            .closures()
            .into_iter()
            .map(|mut closure| {
                closure.function = self.source_name(&closure.function);
                if let Some(location) = self.source_location(closure.span) {
                    closure.span = location.span;
                }
                for capture in &mut closure.captures {
                    if let Some(location) = self.source_location(capture.span) {
                        capture.span = location.span;
                    }
                }
                closure
            })
            .collect()
    }
    pub fn to_json(&self) -> String {
        let contracts = self
            .contracts()
            .iter()
            .map(|(name, mode)| {
                format!(
                    "{}:{}",
                    crate::json_string(name),
                    crate::json_string(mode.as_str())
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let closures = self.analysis.closures().iter().map(|closure| {
            let source=self.source_location(closure.span).expect("checked closure source");
            let captures=closure.captures.iter().map(|capture| {
                let location=self.source_location(capture.span).expect("checked capture source");
                format!("{{\"name\":{},\"storage\":{},\"access\":{},\"file\":{},\"start\":{},\"end\":{}}}",crate::json_string(&capture.name),crate::json_string(capture.storage.as_str()),crate::json_string(capture.access.as_str()),crate::json_string(&location.source.0),location.span.start,location.span.end)
            }).collect::<Vec<_>>().join(",");
            let parameters=closure.parameters.iter().map(|param|format!("{{\"name\":{},\"access\":{}}}",crate::json_string(&param.name),crate::json_string(param.access.as_str()))).collect::<Vec<_>>().join(",");
            format!("{{\"id\":{},\"template\":{},\"depth\":{},\"function\":{},\"file\":{},\"start\":{},\"end\":{},\"escapes\":{},\"invocation\":{},\"captures\":[{captures}],\"parameters\":[{parameters}]}}",closure.id,closure.template,closure.depth,crate::json_string(&self.source_name(&closure.function)),crate::json_string(&source.source.0),source.span.start,source.span.end,closure.escapes,crate::json_string(closure.invocation.as_str()))
        }).collect::<Vec<_>>().join(",");
        format!(
            "{{\"ok\":true,\"contracts\":{{{contracts}}},\"closures\":[{closures}],\"diagnostics\":[]}}"
        )
    }
    pub fn source_location(&self, span: Span) -> Option<SourceLocation> {
        locate(&self.segments, span)
    }
}
fn locate(segments: &[(SourceId, usize, usize)], span: Span) -> Option<SourceLocation> {
    segments
        .iter()
        .find(|(_, start, len)| span.start >= *start && span.end <= start + len)
        .map(|(source, start, _)| SourceLocation {
            source: source.clone(),
            span: Span::new(span.start - start, span.end - start),
        })
}
fn diagnostic(
    graph: &ProjectGraph,
    segments: &[(SourceId, usize, usize)],
    value: Diagnostic,
) -> ProjectDiagnostic {
    ProjectDiagnostic {
        fixes: value
            .fixes
            .into_iter()
            .filter_map(|fix| {
                locate(segments, fix.span).map(|location| crate::project::ProjectFix {
                    title: readable(&fix.title),
                    location,
                    replacement: fix.replacement,
                })
            })
            .collect(),
        code: value.code,
        message: readable(&value.message),
        location: locate(segments, value.span).unwrap_or(SourceLocation {
            source: graph.entry.clone(),
            span: Span::default(),
        }),
        related: value
            .related
            .into_iter()
            .filter_map(|(span, message)| {
                locate(segments, span).map(|location| (location, readable(&message)))
            })
            .collect(),
    }
}
fn symbol(module: usize, name: &str) -> String {
    format!("{}{module}_{name}", crate::type_aliases::LINKED_PREFIX)
}

/// Link independently parsed ASTs; internal checker coordinates are mapped to files explicitly.
pub fn check_graph(graph: ProjectGraph) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    let test_entry = crate::project::is_test_entry(&graph.entry.0);
    let entries = if test_entry { vec![graph.entry.clone()] } else { Vec::new() };
    check_graph_entry(graph, &entries, None, None)
}
pub(crate) fn check_graph_diagnostics(graph: ProjectGraph) -> Result<ProjectAnalysis, Vec<ProjectDiagnostic>> {
    let entries=if crate::project::is_test_entry(&graph.entry.0) {vec![graph.entry.clone()]} else {Vec::new()};
    let mut errors=Vec::new();
    match check_graph_entry(graph,&entries,Some(&mut errors),None) {
        Ok(analysis)=>Ok(analysis),
        Err(error)=>{if errors.is_empty(){errors.push(error);}Err(errors)}
    }
}
pub(crate) fn check_test_graph(graph: ProjectGraph) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    let entry = graph.entry.clone();
    check_graph_entry(graph, &[entry], None, None)
}
pub(crate) fn check_test_graph_entries(graph: ProjectGraph, entries: &[SourceId]) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    check_graph_entry(graph, entries, None, None)
}
pub(crate) fn check_graph_cached(graph:ProjectGraph,cache:&mut crate::project_session::ProjectSession)->Result<ProjectAnalysis,ProjectDiagnostic> {
    let entries=if crate::project::is_test_entry(&graph.entry.0){vec![graph.entry.clone()]}else{vec![]};
    check_graph_entry(graph,&entries,None,Some(cache))
}
fn check_graph_entry(graph: ProjectGraph, entries: &[SourceId], diagnostics: Option<&mut Vec<ProjectDiagnostic>>, mut cache:Option<&mut crate::project_session::ProjectSession>) -> Result<ProjectAnalysis, ProjectDiagnostic> {
    let mut programs = BTreeMap::new();
    let mut globals: BTreeMap<SourceId, BTreeMap<String, String>> = BTreeMap::new();
    let mut segments = vec![];
    let mut offset = 1;
    // Exported type aliases of each parsed module, with every type name
    // already in linked form. Modules are ordered dependencies first.
    let mut module_aliases: BTreeMap<SourceId, crate::type_aliases::Aliases> = BTreeMap::new();
    // Fields of each module's plain record types, in linked form, for
    // interfaces of importing modules that extend them.
    let mut module_records: BTreeMap<SourceId, crate::type_aliases::ImportedRecords> = BTreeMap::new();
    for (module, id) in graph.order.iter().enumerate() {
        let source = &graph.modules[id];
        segments.push((id.clone(), offset, source.source.text.len()));
        offset += source.source.text.len() + 1;
        let mut imported = crate::type_aliases::Aliases::new();
        for import in &source.imports {
            let Some(target) = import.target.as_ref().and_then(|target| module_aliases.get(target)) else { continue };
            for binding in &import.bindings {
                if let Some(alias) = target.get(&binding.exported) {
                    imported.insert(binding.local.clone(), alias.clone());
                }
            }
        }
        let mut imported_records = crate::type_aliases::ImportedRecords::new();
        for import in &source.imports {
            let Some(target) = import.target.as_ref().and_then(|target| module_records.get(target)) else { continue };
            for binding in &import.bindings {
                if let Some(fields) = target.get(&binding.exported) {
                    imported_records.insert(binding.local.clone(), fields.clone());
                }
            }
        }
        let parsed = if let Some(cache)=cache.as_deref_mut() {
            cache.parse(&graph,id,module,entries.contains(id),imported,imported_records)
        } else if let Some(data) = &source.data {
            crate::data_lowering::program(&data.value, &module.to_string())
        } else {crate::syntax::parse_module(&source.declaration_source, entries.contains(id), imported, imported_records)};
        let mut program = parsed.map_err(|value| {
            ProjectDiagnostic {
                code: value.code,
                message: readable(&value.message),
                location: SourceLocation {
                    source: id.clone(),
                    span: value.span,
                },
                related: vec![],
                fixes: Box::new([]),
            }
        })?;
        if let Some(base) = program.unresolved_bases.first() {
            return Err(ProjectDiagnostic {
                code: "type",
                message: format!("{:?} is not a record type that an interface can extend.", base.text),
                location: SourceLocation { source: id.clone(), span: base.span },
                related: vec![],
                fixes: Box::new([]),
            });
        }
        if entries.contains(id) {
            let registration = program.functions.iter_mut().find(|function| function.name.text == "main").expect("synthetic test registration");
            registration.name.text = "@testRegistration".into();
            registration.body.pop(); // Execution follows all files' registration, in one registry.
        }
        for function in &program.functions {
            if [
                "clone", "String", "Boolean", "Array", "Set", "Map", "Object", "console", "Number", "Math", "null",
                "typeof", "void",
            ]
            .contains(&function.name.text.as_str())
            {
                return Err(ProjectDiagnostic {
                    code: "duplicate",
                    message: format!("Function name {:?} is unavailable.", function.name.text),
                    location: SourceLocation {
                        source: id.clone(),
                        span: function.name.span,
                    },
                    related: vec![],
                    fixes: Box::new([]),
                });
            }
        }
        let names = program
            .records
            .iter()
            .map(|r| &r.name)
            .chain(program.functions.iter().map(|f| &f.name))
            .chain(program.module_bindings.iter().filter_map(|statement| match &statement.kind {
                StmtKind::Let { name, .. } => Some(name),
                _ => None,
            }))
            .map(|name| {
                let linked = if id == &graph.entry
                    && name.text == "main"
                    && program.functions.iter().any(|f| f.name.span == name.span)
                {
                    "main".into()
                } else {
                    symbol(module, &name.text)
                };
                (name.text.clone(), linked)
            })
            .collect::<BTreeMap<_, _>>();
        let mut names = names;
        // An alias has no linked declaration; its name only needs an entry so
        // an import of it resolves. Its uses are expanded during parsing.
        for alias in program.type_aliases.keys() {
            names.entry(alias.clone()).or_insert_with(|| symbol(module, alias));
        }
        let mut visible = names.clone();
        for import in &source.imports {
            if let Some(native) = import.native {
                for binding in &import.bindings {
                    if let Ok(crate::native_exports::Export::Type(item)) = crate::native_exports::lookup(native, &binding.exported) {
                        visible.insert(binding.local.clone(), crate::native_type_resolution::linked_name(item.id));
                    }
                }
            }
        }
        for import in &source.imports {
            let Some(target) = import.target.as_ref().and_then(|target| globals.get(target)) else { continue };
            for binding in &import.bindings {
                if let Some(linked) = target.get(graph.modules[import.target.as_ref().unwrap()].exported_local(&binding.exported)) {
                    visible.insert(binding.local.clone(), linked.clone());
                }
            }
        }
        let mut exported = std::mem::take(&mut program.type_aliases);
        for annotation in exported.values_mut() {
            crate::type_aliases::link_names(annotation, &|name| {
                visible.get(name).cloned().unwrap_or_else(|| symbol(module, name))
            });
        }
        module_aliases.insert(id.clone(), exported);
        let mut record_fields = crate::type_aliases::ImportedRecords::new();
        for record in &program.records {
            if record.type_parameters.is_empty()
                && record.enum_definition.is_none()
                && record.map_definition.is_none()
                && record.set_definition.is_none()
            {
                let mut fields = record.fields.clone();
                for field in &mut fields {
                    crate::type_aliases::link_names(&mut field.annotation, &|name| {
                        visible.get(name).cloned().unwrap_or_else(|| symbol(module, name))
                    });
                }
                record_fields.insert(record.name.text.clone(), fields);
            }
        }
        module_records.insert(id.clone(), record_fields);
        globals.insert(id.clone(), names);
        programs.insert(id.clone(), program);
    }
    let mut linked = Program {
        diagnostic_first: None,
        diagnostic_skips: Default::default(),
        diagnostic_contracts: Default::default(),
        unresolved_local_function: None,
        local_declarations: Vec::new(),
        local_capture_plans: Vec::new(),
        type_argument_checks: vec![],
        call_type_arguments: vec![],
        records: vec![],
        functions: vec![],
        module_bindings: vec![],
        module_owner_bindings: vec![],
        type_aliases: Default::default(),
        unresolved_bases: Vec::new(),
    };
    let mut closure_id = 0;
    for (module, id) in graph.order.iter().enumerate() {
        let mut names = globals[id].clone();
        for import in &graph.modules[id].imports {
            for binding in &import.bindings {
                names.insert(
                    binding.local.clone(),
                    if let Some(native) = import.native {
                        match crate::native_exports::lookup(native, &binding.exported).expect("validated native export") {
                            crate::native_exports::Export::Namespace(namespace) => namespace.symbol().into(),
                            crate::native_exports::Export::Function(function) => {
                                let span = Span::new(binding.span.start + segments[module].1, binding.span.end + segments[module].1);
                                crate::native_binding_types::prepare(&mut linked.records, function, span)
                                    .map_err(|value| diagnostic(&graph, &segments, value))?;
                                format!("@native:{native}#{}", binding.exported)
                            },
                            crate::native_exports::Export::Type(item) => {
                                let span = Span::new(binding.span.start + segments[module].1, binding.span.end + segments[module].1);
                                crate::native_type_resolution::intern(&mut linked.records, item.id, span)
                                    .map_err(|value| diagnostic(&graph, &segments, value))?;
                                crate::native_type_resolution::linked_name(item.id)
                            }
                        }
                    } else {
                        {
                            let target = import.target.as_ref().expect("loaded target");
                            let name = globals[target][graph.modules[target].exported_local(&binding.exported)].clone();
                            if graph.modules[target].data.is_some() { format!("{}{name}", crate::data_lowering::ACCESS) } else { name }
                        }
                    },
                );
            }
        }
        let mut program = programs.remove(id).unwrap();
        let lexical_types = crate::lexical_generic_scopes::inherited(&program);
        let mut visitor = Linker {
            names: &names,
            imported_bindings: graph.modules[id].imports.iter().flat_map(|import| import.bindings.iter().map(|binding| binding.local.clone())).collect(),
            assignment_error: None,
            testing_types: id.0 == "std:testing",
            module,
            offset: segments[module].1,
            closure_id: &mut closure_id,
            type_parameters: BTreeSet::new(),
        };
        for record in &mut program.records {
            visitor.type_parameters = record.type_parameters.iter().map(|parameter| parameter.text.clone()).collect();
            for parameter in &mut record.type_parameters { visitor.span(&mut parameter.span); }
            record.name.text = names[&record.name.text].clone();
            visitor.token_span(&mut record.name);
            for field in &mut record.fields {
                visitor.token_span(&mut field.name);
                visitor.value_annotation(&mut field.annotation);
            }
        }
        for (span, arguments) in &mut program.call_type_arguments {
            visitor.type_parameters = program.functions.iter().filter(|function| {
                function.scope_span.start <= span.start && span.end <= function.scope_span.end
            }).min_by_key(|function| function.scope_span.end - function.scope_span.start)
                .map(|function| function.type_parameters.iter()
                    .chain(lexical_types.get(&function.name.text).into_iter().flatten())
                    .map(|parameter| parameter.text.clone()).collect()).unwrap_or_default();
            extend_nested_call_types(&program.functions, *span, &mut visitor.type_parameters);
            visitor.span(span);
            for argument in arguments { visitor.value_annotation(argument); }
        }
        // Lifted bodies still refer to lexical captured bindings. Do not qualify
        // those names as module globals before checked environments are formed.
        let mut captured_names = BTreeMap::new();
        for plan in &program.local_capture_plans {
            for (node, direct) in plan.direct.iter().enumerate() {
                let captures: BTreeSet<String> = direct.iter().map(|capture| capture.name.clone()).collect();
                let declaration = &program.local_declarations[plan.node_start + node];
                captured_names.insert(declaration.emitted_name.clone(), captures);
            }
        }
        // Module declarations share canonical names, rather than introducing
        // function-local shadows. Visit each initializer before qualifying its owner.
        visitor.type_parameters.clear();
        for statement in &mut program.module_bindings {
            if let StmtKind::Let { name, annotation, value, .. } = &mut statement.kind {
                visitor.span(&mut statement.span);
                visitor.expr(value, &BTreeSet::new());
                if let Some(annotation) = annotation { visitor.value_annotation(annotation); }
                visitor.token_span(name);
                name.text = names[&name.text].clone();
            } else {
                visitor.body(std::slice::from_mut(statement), &mut BTreeSet::new());
            }
        }
        for function in &mut program.functions {
            visitor.span(&mut function.scope_span);
            visitor.type_parameters = function.type_parameters.iter()
                .chain(lexical_types.get(&function.name.text).into_iter().flatten())
                .map(|parameter| parameter.text.clone()).collect();
            for parameter in &mut function.type_parameters { visitor.span(&mut parameter.span); }
            let captures = captured_names.remove(&function.name.text).unwrap_or_default();
            function.name.text = names[&function.name.text].clone();
            visitor.token_span(&mut function.name);
            let mut locals = captures;
            for ((name, annotation, callback), initializer) in function.params.iter_mut().zip(&mut function.parameter_initializers) {
                visitor.initializer(initializer, &locals);
                parameter_pattern_names(name, &function.body, &mut locals);
                locals.insert(name.text.clone());
                visitor.token_span(name);
                if let Some(callback) = callback {
                    visitor.value_annotation(annotation);
                    visitor.callback(callback);
                } else {
                    visitor.value_annotation(annotation);
                }
            }
            if let Some(result) = &mut function.result {
                visitor.result(result);
            }
            visitor.body(&mut function.body, &mut locals);
        }
        if let Some(error) = visitor.assignment_error.take() {
            return Err(diagnostic(&graph, &segments, error));
        }
        let declaration_offset = linked.local_declarations.len();
        let scope_offset = linked.local_declarations.iter().map(|declaration| declaration.scope + 1).max().unwrap_or(0);
        let owner_offset = linked.functions.len();
        for plan in &mut program.local_capture_plans {
            for name in plan.owners.values_mut() { *name = names[name].clone(); }
            plan.rebase(declaration_offset, owner_offset, visitor.offset);
        }
        for declaration in &mut program.local_declarations {
            declaration.id += declaration_offset;
            declaration.function_owner += owner_offset;
            declaration.enclosing_owner += owner_offset;
            visitor.span(&mut declaration.statement_span);
            declaration.scope += scope_offset;
            visitor.token_span(&mut declaration.source_name);
            visitor.span(&mut declaration.function_scope);
            declaration.emitted_name = names[&declaration.emitted_name].clone();
        }
        if let Some(mut token) = program.unresolved_local_function.take() {
            visitor.token_span(&mut token);
            linked.unresolved_local_function = Some(token);
        }
        linked.local_declarations.extend(program.local_declarations);
        linked.local_capture_plans.extend(program.local_capture_plans);
        linked.call_type_arguments.extend(program.call_type_arguments);
        linked.module_bindings.extend(program.module_bindings);
        linked.records.extend(program.records);
        linked.functions.extend(program.functions);
    }
    if !entries.is_empty() {
        let mut entry = crate::syntax::parse_test_entry("").expect("empty synthetic entry").functions.pop().unwrap();
        let mut body = Vec::new();
        for id in entries {
            let segment = segments.iter().find(|(source, _, _)| source == id).expect("test source segment");
            let span = Span::new(segment.1, segment.1);
            body.push(Stmt { labels: Vec::new(), control_target: None, span, kind: StmtKind::Expr(Expr::new(ExprKind::Call(
                Box::new(Expr::new(ExprKind::Name("@native:std:testing-runtime#beginFile".into()), span)), vec![Expr::new(ExprKind::String(id.0.clone()), span)]
            ), span)) });
            body.push(Stmt { labels: Vec::new(), control_target: None, span, kind: StmtKind::Expr(Expr::new(ExprKind::Call(
                Box::new(Expr::new(ExprKind::Name(globals[id]["@testRegistration"].clone()), span)), Vec::new()
            ), span)) });
            body.push(Stmt { labels: Vec::new(), control_target: None, span, kind: StmtKind::Expr(Expr::new(ExprKind::Call(
                Box::new(Expr::new(ExprKind::Name("@native:std:testing-runtime#endSuite".into()), span)), Vec::new()
            ), span)) });
        }
        body.append(&mut entry.body);
        entry.body = body;
        linked.functions.push(entry);
    }
    crate::callables::module_state::lower(&mut linked).map_err(|value| diagnostic(&graph, &segments, value))?;
    crate::local_functions::resolve(&mut linked).map_err(|value| diagnostic(&graph, &segments, value))?;
    let checked = match crate::analysis::analyze(linked.clone()) {
        Err(value) if value.code == "recursion-inference" => {
            crate::analysis::analyze_recursive(linked.clone())
        }
        result => result,
    };
    let checked=match checked {
        Ok(checked)=>checked,
        Err(error)=>{
            if let Some(errors)=diagnostics {
                errors.extend(crate::semantic_diagnostics::collect(linked,error.clone()).into_iter().map(|error|diagnostic(&graph,&segments,error)));
            }
            return Err(diagnostic(&graph,&segments,error));
        }
    };
    Ok(ProjectAnalysis {
        graph,
        analysis: Analysis { program: checked },
        segments,
    })
}
struct Linker<'a> {
    imported_bindings: BTreeSet<String>,
    assignment_error: Option<Diagnostic>,
    testing_types: bool,
    names: &'a BTreeMap<String, String>,
    module: usize,
    offset: usize,
    closure_id: &'a mut usize,
    type_parameters: BTreeSet<String>,
}
impl Linker<'_> {
    fn span(&self, span: &mut Span) {
        span.start += self.offset;
        span.end += self.offset;
    }
    fn token_span(&self, token: &mut Token) {
        self.span(&mut token.span);
    }
    fn annotation(&self, token: &mut Token) {
        token.text = token
            .text
            .split('|')
            .map(|part| {
                let base = part.trim_end_matches("[]");
                let suffix = &part[base.len()..];
                let name = if self.type_parameters.contains(base) {
                    base.into()
                } else if base.starts_with(crate::type_aliases::LINKED_PREFIX) {
                    // Expanded from an imported alias; already a linked name.
                    base.into()
                } else { self.names.get(base).cloned().unwrap_or_else(|| {
                    if matches!(
                        base,
                        "int" | "number" | "boolean" | "string" | "void" | "null" | "undefined" | "Set" | "Map" | "Result"
                    ) {
                        base.into()
                    } else {
                        symbol(self.module, base)
                    }
                }) };
                format!("{name}{suffix}")
            })
            .collect::<Vec<_>>()
            .join("|");
        self.token_span(token);
    }
    fn value_annotation(&self, annotation: &mut crate::type_syntax::ValueAnnotation) {
        use crate::type_syntax::ValueAnnotationKind as Kind;
        self.span(&mut annotation.span);
        match &mut annotation.kind {
            Kind::Applied { name, .. } if self.testing_types && name.text == "ContainmentValue" && !self.type_parameters.contains(&name.text) && !self.names.contains_key(&name.text) => {
                name.text = "@testing:element".into();
                self.span(&mut name.span);
            }
            Kind::Named(name) | Kind::Applied { name, .. } => {
                let mut token = Token { text: std::mem::take(&mut name.text), span: name.span, kind: TokenKind::Name };
                self.annotation(&mut token);
                name.text = token.text;
                name.span = token.span;
            }
            _ => {}
        }
        match &mut annotation.kind {
            Kind::Literal(_) | Kind::Named(_) => {}
            Kind::Polymorphic { declaration, .. } => self.span(&mut declaration.span),
            Kind::Applied { arguments, .. } | Kind::Union(arguments) | Kind::Tuple(arguments) => {
                for argument in arguments { self.value_annotation(argument); }
            }
            Kind::Array(inner) | Kind::Group(inner) => self.value_annotation(inner),
            Kind::Callable { parameters, result, .. } => {
                for parameter in parameters {
                    self.span(&mut parameter.span);
                    self.span(&mut parameter.name.span);
                    if let Some(access) = &mut parameter.access { self.span(&mut access.span); }
                    self.value_annotation(&mut parameter.annotation);
                }
                self.value_annotation(result);
            }
        }
    }
    fn parameter(&self, param: &mut CallableParameter) {
        self.token_span(&mut param.name);
        if let Some(access) = &mut param.access {
            self.token_span(access);
        }
        match &mut param.ty {
            ParameterAnnotation::Named(token) => self.value_annotation(token),
            ParameterAnnotation::Callback(callback) => self.callback(callback),
            ParameterAnnotation::Inferred(token) => self.token_span(token),
        }
    }
    fn initializer(&mut self, initializer: &mut ParameterInitializer, locals: &BTreeSet<String>) {
        match initializer {
            ParameterInitializer::Required => {},
            ParameterInitializer::Optional(span) => self.span(span),
            ParameterInitializer::Default { value, span } => { self.span(span); self.expr(value, locals); }
        }
    }
    fn callback(&self, callback: &mut CallbackAnnotation) {
        self.span(&mut callback.span);
        for param in &mut callback.params {
            self.parameter(param);
        }
        self.result(&mut callback.result);
    }
    fn result(&self, result: &mut ResultAnnotation) {
        match result {
            ResultAnnotation::Inferred(_) => {},
            ResultAnnotation::Named(token) => self.value_annotation(token),
            ResultAnnotation::Callable {
                params,
                result,
                span,
                ..
            } => {
                self.span(span);
                for param in params {
                    self.parameter(param);
                }
                self.result(result);
            }
        }
    }
    fn assignment_target(&mut self, target: &Expr, locals: &BTreeSet<String>, offset_span: bool) {
        if let ExprKind::Name(name) = &target.kind {
            if !locals.contains(name) && self.imported_bindings.contains(name) && self.assignment_error.is_none() {
                let span = if offset_span { Span::new(target.span.start + self.offset, target.span.end + self.offset) } else { target.span };
                self.assignment_error = Some(Diagnostic::new("module-import-assignment", "Cannot reassign an imported binding; update it through its declaring module.", span));
            }
        }
    }

    fn body(&mut self, body: &mut [Stmt], locals: &mut BTreeSet<String>) {
        // Lexical declarations shadow module names throughout the block. Their
        // initialization readiness is checked after linking, including captures
        // in a function declared before the local variable it uses.
        for statement in body.iter() {
            match &statement.kind {
                StmtKind::LocalFunction(function) => { locals.insert(function.name.text.clone()); }
                StmtKind::Let { name, .. } => { locals.insert(name.text.clone()); }
                _ => {}
            }
        }
        for stmt in body {
            self.span(&mut stmt.span);
            for label in &mut stmt.labels {self.token_span(label);}
            match &mut stmt.kind {
                StmtKind::LocalFunction(function) => {
                    self.span(&mut function.scope_span);
                    self.token_span(&mut function.name);
                    let outer_types = self.type_parameters.clone();
                    for parameter in &mut function.type_parameters {
                        self.type_parameters.insert(parameter.text.clone());
                        self.span(&mut parameter.span);
                    }
                    let mut scope = locals.clone();
                    // A named function expression's written name is private to
                    // its own body; preserve it until lexical resolution.
                    scope.insert(crate::local_functions::written_name(&function.name.text).to_owned());
                    for ((name, annotation, callback), initializer) in function.params.iter_mut().zip(&mut function.parameter_initializers) {
                        self.initializer(initializer, &scope);
                        parameter_pattern_names(name, &function.body, &mut scope);
                        scope.insert(name.text.clone());
                        self.token_span(name);
                        self.value_annotation(annotation);
                        if let Some(callback) = callback { self.callback(callback); }
                    }
                    if let Some(result) = &mut function.result { self.result(result); }
                    self.body(&mut function.body, &mut scope);
                    self.type_parameters = outer_types;
                },
                StmtKind::Throw(value) => self.expr(value,locals),
                StmtKind::Try(region) => {
                    self.body(&mut region.body,&mut locals.clone());
                    if let Some(handler)=&mut region.catch {
                        let mut scope=locals.clone();
                        if let Some(name)=&mut handler.name {scope.insert(name.text.clone());self.token_span(name);}
                        self.body(&mut handler.body,&mut scope);
                    }
                    if let Some(body)=&mut region.finally {self.body(body,&mut locals.clone());}
                }

                StmtKind::Let {
                    name,
                    annotation,
                    value,
                    ..
                } => {
                    self.expr(value, locals);
                    if let Some(annotation) = annotation {
                        self.value_annotation(annotation);
                    }
                    locals.insert(name.text.clone());
                    self.token_span(name);
                }
                StmtKind::Block(body) => self.body(body, &mut locals.clone()),
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    self.expr(condition, locals);
                    self.body(then_body, &mut locals.clone());
                    self.body(else_body, &mut locals.clone());
                }
                StmtKind::While { condition, body } => {
                    self.expr(condition, locals);
                    self.body(body, &mut locals.clone());
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    if let Some(condition) = condition {
                        self.expr(condition, locals);
                    }
                    self.body(body, &mut locals.clone());
                    self.body(update, &mut locals.clone());
                }
                StmtKind::ForOf {
                    name,
                    iterable,
                    body,
                    ..
                } => {
                    self.expr(iterable, locals);
                    let mut scope = locals.clone();
                    scope.insert(name.text.clone());
                    self.token_span(name);
                    self.body(body, &mut scope);
                }
                StmtKind::Assign {
                    target,
                    value,
                    operator,
                } => {
                    self.assignment_target(target, locals, true);
                    self.expr(target, locals);
                    self.expr(value, locals);
                    if let Some((_, span)) = operator {
                        self.span(span);
                    }
                }
                StmtKind::Return(value) => {
                    if let Some(value) = value {
                        self.expr(value, locals);
                    }
                }
                StmtKind::Expr(value) => self.expr(value, locals),
                StmtKind::Break | StmtKind::Continue => {}
            }
        }
    }
    fn expr(&mut self, expr: &mut Expr, locals: &BTreeSet<String>) {
        self.span(&mut expr.span);
        match &mut expr.kind {
            ExprKind::SpreadCall(plan) => { for value in plan.expressions_mut() { self.expr(value, locals); } }
            ExprKind::SpreadValue(selected) => self.span(&mut selected.source.span),
            ExprKind::Name(name) => {
                if !locals.contains(name) {
                    if let Some(linked) = self.names.get(name) {
                        *name = linked.clone();
                    } else if crate::native_bindings::by_symbol(name).is_none()
                        && !matches!(name.as_str(), "@match:coverage" | "@match:variant" | "@match:payload")
                        && crate::syntax::global_number_member(name, false).is_none()
                        && crate::syntax::global_number_member(name, true).is_none()
                        && !matches!(
                        name.as_str(),
                        "Object#in" | "console" | "Math" | "Number" | "JSON" | "Array" | "Set" | "Map" | "Result" | "Ok" | "Err" | "Object" | "String" | "Boolean" | "clone" | "undefined"
                    ) {
                        *name = symbol(self.module, name);
                    }
                }
            }
            ExprKind::Optional { base, body, .. } => {
                self.expr(base, locals);
                self.expr(body, locals);
            }
            ExprKind::Selected { source, .. } => self.span(&mut source.span),
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                self.expr(predicate, locals);
                self.expr(consequent, locals);
                self.expr(alternate, locals);
            }
            ExprKind::CoalesceAssign {
                target,
                value,
                operator,
                ..
            }
            | ExprKind::Assign {
                target,
                value,
                operator,
            }
            | ExprKind::CompoundAssign {
                target,
                value,
                operator,
                ..
            }
            | ExprKind::LogicalAssign {
                target,
                value,
                operator,
                ..
            } => {
                self.span(operator);
                self.assignment_target(target, locals, true);
                self.expr(target, locals);
                self.expr(value, locals);
            }
            ExprKind::Update {
                target, operator, ..
            } => {
                self.span(operator);
                self.assignment_target(target, locals, true);
                self.expr(target, locals);
            }
            ExprKind::Unary {
                value, operator, ..
            } => {
                self.span(operator);
                self.expr(value, locals);
            }
            ExprKind::Binary {
                left,
                right,
                operator,
                ..
            } => {
                self.span(operator);
                self.expr(left, locals);
                self.expr(right, locals);
            }
            ExprKind::Closure {
                id,
                params,
                result,
                body,
            } => {
                *id = *self.closure_id;
                *self.closure_id += 1;
                let mut scope = locals.clone();
                for param in params {
                    self.initializer(&mut param.initializer, &scope);
                    parameter_pattern_names(&param.name, body, &mut scope);
                    scope.insert(param.name.text.clone());
                    self.parameter(param);
                }
                self.result(result);
                self.body(body, &mut scope);
            }
            ExprKind::TypeAssertion {value,annotation,..} => {
                self.value_annotation(annotation);
                self.expr(value,locals);
            }
            ExprKind::ArrayOf(values, element) | ExprKind::SetConstructor(values, element) | ExprKind::MapConstructor(values, element) | ExprKind::NativeConstructor(values, element) => {
                if let Some(element) = element { self.value_annotation(element); }
                for value in values { self.expr(value, locals); }
            }
            ExprKind::Sequence(values) | ExprKind::Template(values) | ExprKind::Array(values) => {
                for value in values {
                    self.expr(value, locals);
                }
            }
            ExprKind::OptionalChain(value)
            | ExprKind::Field(value, _)
            | ExprKind::Inject { value, .. }
            | ExprKind::Project { value, .. } => self.expr(value, locals),
            ExprKind::Record(fields) => {
                for (name, value) in fields {
                    self.token_span(name);
                    self.expr(value, locals);
                }
            }
            ExprKind::Index(left, right) => {
                self.expr(left, locals);
                self.expr(right, locals);
            }
            ExprKind::Call(value, args) => {
                self.expr(value, locals);
                for arg in args {
                    self.expr(arg, locals);
                }
            }
            ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::EnumMember { .. } | ExprKind::Number(_)
            | ExprKind::Boolean(_)
            | ExprKind::Null
            | ExprKind::Undefined => {}
        }
    }
}

fn readable(message: &str) -> String {
    let mut result = String::new();
    let mut remaining = message;
    while let Some(index) = remaining.find("__tsrust_module_") {
        result.push_str(&remaining[..index]);
        let tail = &remaining[index + "__tsrust_module_".len()..];
        let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && tail.as_bytes().get(digits) == Some(&b'_') {
            remaining = &tail[digits + 1..];
        } else {
            result.push_str("__tsrust_module_");
            remaining = tail;
        }
    }
    result.push_str(remaining);
    result
}

#[cfg(test)]
mod annotation_tests {
    use super::*;
    use crate::type_syntax::{ValueAnnotationKind as Kind, parse};

    #[test]
    fn generic_parameters_shadow_imports_only_inside_their_declaration() {
        let names = BTreeMap::from([
            ("T".into(), "__tsrust_module_0_ImportedT".into()),
            ("Box".into(), "__tsrust_module_0_Box".into()),
        ]);
        let mut closure_id = 0;
        let mut linker = Linker {
            names: &names, testing_types: false, module: 1, offset: 17,
            imported_bindings: BTreeSet::new(), assignment_error: None,
            closure_id: &mut closure_id,
            type_parameters: BTreeSet::from(["T".into()]),
        };
        let mut annotation = parse("Box<T>").unwrap();
        linker.value_annotation(&mut annotation);
        let Kind::Applied { name, arguments } = &annotation.kind else { panic!("application") };
        assert_eq!(name.text, "__tsrust_module_0_Box");
        let Kind::Named(parameter) = &arguments[0].kind else { panic!("parameter") };
        assert_eq!(parameter.text, "T");
        assert_eq!(parameter.span, Span::new(21, 22));
        linker.type_parameters.clear();
        let mut annotation = parse("T").unwrap();
        linker.value_annotation(&mut annotation);
        let Kind::Named(name) = annotation.into_kind() else { panic!("nominal") };
        assert_eq!(name.text, "__tsrust_module_0_ImportedT");
    }
}

#[cfg(test)]
#[test]
fn containment_projection_links_only_inside_the_bundled_testing_scope() {
    use crate::type_syntax::{parse,ValueAnnotationKind as Kind};
    let names=BTreeMap::new();let mut closure_id=0;
    for (testing_types,shadowed,wanted) in [(true,false,"@testing:element"),(false,false,"__tsrust_module_1_ContainmentValue"),(true,true,"ContainmentValue")] {
        let linker=Linker {testing_types,names:&names,module:1,offset:17,imported_bindings:BTreeSet::new(),assignment_error:None,closure_id:&mut closure_id,type_parameters:if shadowed {BTreeSet::from(["ContainmentValue".into(),"T".into()])}else{BTreeSet::from(["T".into()])}};
        let mut value=parse("ContainmentValue<T>").unwrap();linker.value_annotation(&mut value);
        let Kind::Applied{name,arguments}=&value.kind else {panic!("projection")};assert_eq!(name.text,wanted);
        assert_eq!(name.span,Span::new(17,33));
        let Kind::Named(argument)=&arguments[0].kind else {panic!("argument")};assert_eq!(argument.text,"T");
    }
}

#[cfg(test)]
#[test]
fn bundled_containment_projection_links_specializes_and_executes() {
    use crate::project::{ModuleSource,SourceFile,ModuleImport,ImportBinding};
    let standard=SourceId("std:testing".into());let entry=SourceId("main.tsr".into());
    let library="type Probe<T>={contains:Function<void,[expected:ReadOnly<ContainmentValue<T>>]>;};function make<T>(actual:T):Probe<T>{return {contains:(expected:int):void=>{console.log(expected);}};}function exercise(){const probe=make([1,2]);probe.contains(7);}";
    let main="function main(){exercise();}";
    let graph=ProjectGraph {entry:entry.clone(),order:vec![standard.clone(),entry.clone()],modules:BTreeMap::from([
        (standard.clone(),ModuleSource{data:None,source:SourceFile{id:standard.clone(),text:library.into()},declaration_source:library.into(),imports:vec![],exports:BTreeMap::from([("exercise".into(),Span::default())]),default_export:None}),
        (entry.clone(),ModuleSource{data:None,source:SourceFile{id:entry,text:main.into()},declaration_source:main.into(),imports:vec![ModuleImport{path:"std:testing".into(),span:Span::default(),names:Span::default(),bindings:vec![ImportBinding{exported:"exercise".into(),local:"exercise".into(),span:Span::default()}],target:Some(standard),native:None}],exports:BTreeMap::new(),default_export:None})
    ])};
    let checked=check_graph(graph).unwrap();
    crate::callable_dispatch::tests::execute(&checked.emit_rust(),"7\n");
}
