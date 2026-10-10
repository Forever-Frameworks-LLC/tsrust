//! In-process source analysis and Rust generation for the ownership prototype.
//!
//! The library does not invoke subprocesses or read source files. CLI and future
//! editor/LSP clients can share this analysis entry point and source diagnostics.
mod analysis;
mod parameters;
mod parameter_arity;
pub use parameters::ParameterOmission;
mod callable_contracts;
mod callable_storage;
mod capture_loan_owners;
mod capture_loan_identity;
mod callable_layout;
mod polymorphic_callable_storage;
mod polymorphic_invocation_plans;
mod callable_size;
mod callable_dispatch;
mod borrowed_callable_dispatch;
mod borrowed_callable_contracts;
mod callable_annotations;
mod callable_resolution;
mod callable_compatibility;
mod callable_environment_demand;
mod cloneability;
mod definite_assignment;
mod completion;
mod match_coverage;
mod error_effects;
mod diagnostic;
mod emit;
mod fixed_storage;
mod flow;
mod formatting;
mod generic_records;
mod generic_functions;
mod lexical_generic_scopes;
mod generic_inference;
mod generic_validation;
mod generic_call_validation;
mod generic_defaults;
mod source_walk;
pub mod native_bindings;
pub mod native_signatures;
pub mod native_cache;
mod native_emit;
pub use stdlib::path::runtime as native_path;
pub use stdlib::path::relative as native_path_relative;
pub use formatting::format;
mod projections;
pub use projections::{CheckedTypeFacts, LocalFunctionInfo, GenericCallInfo, GenericParameterCallInfo, GenericParameterInfo, MethodResultInfo, ProjectionInfo};
pub mod project;
pub mod project_cli;
mod project_link;
mod project_facts;
mod result_identity;
mod project_runtime;
mod stdlib;
mod record_layout;
mod scoped_record_fields;
mod scoped_record_layout;
mod scoped_record_places;
mod argument_access_loans;
mod scoped_record_bridge;
mod aggregate_callable_facts;
mod environment_lifetimes;
mod record_runtime;
mod runtime;
mod storage;
mod syntax;
mod local_function_values;
mod local_functions;
mod local_capture_plan;
mod local_readiness;
pub mod type_syntax;
pub use syntax::match_syntax;
mod named_capture_lowering;
mod type_aliases;
mod type_substitution;

pub use analysis::Mode;
pub use syntax::numeric_literal_span;
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

mod unions;
