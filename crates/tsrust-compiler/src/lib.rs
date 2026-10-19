//! In-process source analysis and Rust generation for the ownership prototype.
//!
//! The library does not invoke subprocesses or read source files. CLI and future
//! editor/LSP clients can share this analysis entry point and source diagnostics.
mod types;
mod generics;
mod records;
mod callables;
mod native;
mod analysis;
mod parameters;
mod parameter_arity;
pub use parameters::ParameterOmission;
pub(crate) use callables::contracts as callable_contracts;
pub(crate) use callables::storage as callable_storage;
pub(crate) use callables::native_view_callable_storage as native_view_callable_storage;
pub(crate) use callables::native_view_family_registry as native_view_family_registry;
pub(crate) use callables::native_view_origin_remap as native_view_origin_remap;
pub(crate) use callables::capture_loan_owners as capture_loan_owners;
pub(crate) use callables::capture_loan_identity as capture_loan_identity;
pub(crate) use callables::layout as callable_layout;
pub(crate) use callables::polymorphic_callable_storage as polymorphic_callable_storage;
pub(crate) use callables::polymorphic_invocation_plans as polymorphic_invocation_plans;
pub(crate) use callables::size as callable_size;
pub(crate) use callables::dispatch as callable_dispatch;
pub(crate) use callables::borrowed_callable_dispatch as borrowed_callable_dispatch;
pub(crate) use callables::borrowed_callable_contracts as borrowed_callable_contracts;
pub(crate) use callables::annotations as callable_annotations;
pub(crate) use callables::resolution as callable_resolution;
pub(crate) use callables::compatibility as callable_compatibility;
pub(crate) use callables::environment_demand as callable_environment_demand;
mod cloneability;
mod definite_assignment;
mod completion;
mod match_coverage;
mod error_effects;
mod diagnostic;
mod semantic_diagnostics;
mod emit;
mod fixed_storage;
mod flow;
mod formatting;
pub(crate) use generics::records as generic_records;
pub(crate) use generics::functions as generic_functions;
pub(crate) use generics::lexical_generic_scopes as lexical_generic_scopes;
pub(crate) use generics::inference as generic_inference;
pub(crate) use generics::validation as generic_validation;
pub(crate) use generics::call_validation as generic_call_validation;
pub(crate) use generics::defaults as generic_defaults;
mod source_walk;
pub use native::bindings as native_bindings;
pub use native::return_borrows as native_return_borrows;
pub use native::signatures as native_signatures;
pub use native::types as native_types;
pub(crate) use native::binding_types as native_binding_types;
pub use native::type_resolution as native_type_resolution;
pub use native::exports as native_exports;
pub use native::cache as native_cache;
pub use native::runtime as native_runtime;
pub(crate) use native::emit as native_emit;
pub use stdlib::path::runtime as native_path;
pub use stdlib::path::relative as native_path_relative;
pub use formatting::format;
pub use syntax::jsx::attribute_context as jsx_attribute_context;
mod projections;
pub use projections::{CheckedTypeFacts, LocalFunctionInfo, GenericCallInfo, GenericParameterCallInfo, GenericParameterInfo, MethodResultInfo, ProjectionInfo};
pub mod project;
pub use project::resolution as project_resolution;
pub(crate) use project::data::module as data_module;
pub(crate) use project::data::shape as data_shape;
pub(crate) use project::data::lowering as data_lowering;
pub use project::session as project_session;
pub use project::cli as project_cli;
pub use project::source_discovery as source_discovery;
pub(crate) use project::link as project_link;
pub(crate) use project::facts as project_facts;
mod result_identity;
pub(crate) use project::runtime as project_runtime;
mod stdlib;
pub(crate) use records::layout as record_layout;
pub(crate) use records::scoped_record_fields as scoped_record_fields;
pub(crate) use records::scoped_record_layout as scoped_record_layout;
pub(crate) use records::scoped_record_places as scoped_record_places;
pub(crate) use records::argument_access_loans as argument_access_loans;
pub(crate) use records::scoped_record_bridge as scoped_record_bridge;
pub(crate) use callables::aggregate_callable_facts as aggregate_callable_facts;
pub(crate) use callables::environment_lifetimes as environment_lifetimes;
pub(crate) use records::recursive_view_shapes as recursive_view_shapes;
pub(crate) use records::recursive_links as recursive_links;
pub(crate) use records::runtime as record_runtime;
mod runtime;
mod storage;
pub mod documentation;
mod syntax;
pub(crate) use callables::local_function_values as local_function_values;
pub(crate) use callables::local_functions as local_functions;
pub(crate) use callables::local_capture_plan as local_capture_plan;
pub(crate) use callables::local_readiness as local_readiness;
pub use types::syntax as type_syntax;
pub use types::literals as literal_types;
pub use syntax::match_syntax;
pub(crate) use callables::named_capture_lowering as named_capture_lowering;
pub(crate) use types::aliases as type_aliases;
pub(crate) use types::substitution as type_substitution;

pub use analysis::Mode;
pub use syntax::numeric_literal_span;
pub use syntax::{TypeAliasCatalog, TypeAliasInfo};
pub use syntax::diagnostic_directives::DiagnosticSuppressions;
pub use diagnostic::{Diagnostic, Span, SuggestedFix, json_string, line_column};

/// Exact Rust backend release validated for this version of the compiler.
/// Keep synchronized with the workspace rust-toolchain.toml (checked by tests).
pub const RUST_BACKEND_VERSION: &str = "1.96.1";
use std::collections::BTreeMap;

/// A successfully checked syntax tree. Constructed only through `check`.
pub struct Analysis {
    program: analysis::CheckedProgram,
}

/// One capture's storage cost and the closure body's access requirement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureInfo {
    pub name: String,
    pub storage: Mode,
    pub access: Mode,
    pub span: Span,
}

/// One closure argument's inferred access, distinct from capture access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureParameterInfo {
    pub name: String,
    pub access: Mode,
}

/// Inspectable closure contract. IDs are deterministic within one analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureInfo {
    /// True for a checked owned-callback template; only specializations are emitted.
    pub template: bool,
    pub depth: usize,
    pub id: usize,
    pub function: String,
    pub span: Span,
    pub escapes: bool,
    pub invocation: Mode,
    pub captures: Vec<CaptureInfo>,
    pub parameters: Vec<ClosureParameterInfo>,
}

/// Structured source provenance for one materialized nominal generic record.
/// Concrete argument annotations remain structured; clients must not parse display names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenericRecordInfo {
    pub name: String,
    pub declaration: String,
    pub declaration_span: Span,
    pub arguments: Vec<type_syntax::ValueAnnotation>,
}

impl Analysis {
    pub fn warnings(&self) -> Vec<Diagnostic> { self.program.task_warnings() }
    /// Materialized record applications in deterministic native record order.
    /// This is declaration provenance, not inferred expression or flow metadata.
    /// Nested applications are resolved nominal names, not reparsable source text;
    /// their own entries supply structured arguments. Spans retain source provenance.
    pub fn generic_records(&self) -> Vec<GenericRecordInfo> {
        self.program.records.iter().filter_map(|record| {
            record.application.as_ref().map(|application| GenericRecordInfo {
                name: record.name.text.clone(),
                declaration: application.declaration.text.clone(),
                declaration_span: application.declaration.span,
                arguments: application.arguments.clone(),
            })
        }).collect()
    }

    pub fn contracts(&self) -> BTreeMap<String, Mode> {
        self.program.contracts()
    }
    pub fn emit_rust(&self) -> String {
        emit::emit(&self.program)
    }
    /// Generated Rust and deterministic features selected by emitted native bindings.
    pub fn emit_rust_with_native_features(&self) -> (String, Vec<&'static str>) {
        emit::emit_with_native_features(&self.program, "")
    }

    pub fn closures(&self) -> Vec<ClosureInfo> {
        self.program
            .closures
            .iter()
            .enumerate()
            .map(|(id, closure)| ClosureInfo {
                id,
                template: closure.function.template,
                depth: closure.depth,
                function: closure.function.name.clone(),
                span: closure.span,
                escapes: closure.escapes,
                invocation: closure.invocation,
                parameters: closure
                    .parameters
                    .iter()
                    .map(|&id| {
                        let binding = &closure.function.bindings[id];
                        ClosureParameterInfo {
                            name: binding.name.clone(),
                            access: binding.mode,
                        }
                    })
                    .collect(),
                captures: closure
                    .captures
                    .iter()
                    .map(|capture| CaptureInfo {
                        name: closure.function.bindings[capture.inner].name.clone(),
                        storage: capture.storage,
                        access: capture.access,
                        span: capture.span,
                    })
                    .collect(),
            })
            .collect()
    }
    pub fn to_json(&self) -> String {
        let contracts = self
            .contracts()
            .iter()
            .map(|(name, mode)| format!("{}:{}", json_string(name), json_string(mode.as_str())))
            .collect::<Vec<_>>()
            .join(",");
        let closures = self.closures().iter().map(|closure| {
            let captures = closure.captures.iter().map(|capture| format!(
                "{{\"name\":{},\"storage\":{},\"access\":{},\"start\":{},\"end\":{}}}",
                json_string(&capture.name), json_string(capture.storage.as_str()),
                json_string(capture.access.as_str()), capture.span.start, capture.span.end,
            )).collect::<Vec<_>>().join(",");
            let parameters = closure.parameters.iter().map(|param| format!("{{\"name\":{},\"access\":{}}}", json_string(&param.name), json_string(param.access.as_str()))).collect::<Vec<_>>().join(",");
            format!("{{\"id\":{},\"depth\":{},\"template\":{},\"function\":{},\"start\":{},\"end\":{},\"escapes\":{},\"invocation\":{},\"parameters\":[{parameters}],\"captures\":[{captures}]}}",
                closure.id, closure.depth, closure.template, json_string(&closure.function), closure.span.start, closure.span.end,
                closure.escapes, json_string(closure.invocation.as_str()))
        }).collect::<Vec<_>>().join(",");
        format!(
            "{{\"ok\":true,\"contracts\":{{{contracts}}},\"closures\":[{closures}],\"diagnostics\":[]}}"
        )
    }
}

/// Parse, resolve, type-check, and infer ownership entirely in process.
/// Source ranges use half-open UTF-8 byte offsets.
pub fn check(source: &str) -> Result<Analysis, Diagnostic> {
    let program = match analysis::analyze(syntax::parse(source)?) {
        Err(error) if error.code == "recursion-inference" => {
            analysis::analyze_recursive(syntax::parse(source)?)?
        }
        result => result?,
    };
    Ok(Analysis { program })
}

/// Collect diagnostics without exposing a recovered tree to code generation.
/// Syntax recovery currently resumes at statement boundaries inside blocks.
/// Semantic replay discards invalid statements without exposing an executable tree.
pub fn diagnostics(source: &str) -> Vec<Diagnostic> {
    match check(source) {
        Ok(analysis) => analysis.warnings(),
        Err(error) => diagnostic_error(source,error),
    }
}

pub(crate) fn diagnostic_error(source: &str, error: Diagnostic) -> Vec<Diagnostic> {
    if error.code=="syntax" {
        let errors=syntax::diagnostics(source);
        return if errors.is_empty() {vec![error]} else {errors};
    }
    match syntax::parse(source) {
        Ok(program)=>semantic_diagnostics::collect(program,error),
        Err(_)=>vec![error],
    }
}

pub(crate) use types::unions as unions;

#[cfg(test)]
mod nesting_probe;

pub mod ui_project;
