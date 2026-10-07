#[path = "owned_array_patterns.rs"]
mod owned_array_patterns;
#[path = "inferred_callable_arrays.rs"]
mod inferred_callable_arrays;
#[path = "return_type_inference.rs"]
mod return_type_inference;
#[path = "named_capture_layout.rs"]
mod named_capture_layout;
#[path = "named_capture_access.rs"]
mod named_capture_access;
#[path = "named_capture_context.rs"]
mod named_capture_context;
#[path = "capture_context.rs"]
mod capture_context;
#[path = "callable_expression.rs"]
mod callable_expression;
#[path = "exception_state.rs"]
mod exception_state;
#[path = "labeled_control.rs"]
mod labeled_control;
#[path = "object_spread.rs"]
mod object_spread;
#[path = "destructuring_rest.rs"]
mod destructuring_rest;
#[path = "object_keys.rs"]
mod object_keys;
#[path = "object_values.rs"]
mod object_values;
#[path = "array_entries.rs"]
mod array_entries;
#[path = "logical_values.rs"]
mod logical_values;
#[path = "generic_function_values.rs"]
pub(crate) mod generic_function_values;
#[path = "closure_methods.rs"]
mod closure_methods;
#[path = "generic_value_adapters.rs"]
mod generic_value_adapters;
#[path = "checked_value_annotations.rs"]
mod checked_value_annotations;
#[path = "generic_expression_adapters.rs"]
mod generic_expression_adapters;
#[path = "generic_value_metadata.rs"]
mod generic_value_metadata;
#[path = "deferred_generic_values.rs"]
mod deferred_generic_values;
#[path = "generic_calls.rs"]
mod generic_calls;
#[path = "optional_chain.rs"]
mod optional_chain;
#[path = "rest_calls.rs"]
mod rest_calls;
#[path = "spread_calls.rs"]
mod spread_calls;
pub(crate) use spread_calls::spread_position_range;
#[path = "spread_builtins.rs"]
mod spread_builtins;

use crate::diagnostic::{Diagnostic, Span};
use crate::syntax::*;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    Copy,
    Read,
    Mutate,
    Consume,
}
impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Read => "read",
            Self::Mutate => "mutate",
            Self::Consume => "consume",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub name: String,
    pub ty: Type,
    pub parameter: bool,
    pub input_ty: Option<Type>,
    pub omission: crate::ParameterOmission,
    pub rest: bool,
    pub declaration_span: Option<Span>,
    pub initializer: Option<Box<Expr>>,
    pub rebindable: bool,
    pub capture_reference: bool,
    /// Shared element storage exists only for the enclosing for-of iteration.
    pub borrowed_element: Option<Span>,
    /// Scoped readonly result borrowed from assignment storage.
    pub borrowed_place: Option<Span>,
    /// Receiver roots kept alive by a scoped optional-element view.
    pub view_roots: Vec<usize>,
    pub callback_invocation: Option<Mode>,
    pub callback_reference: bool,
    /// Source callback ownership remains exact after concrete specialization.
    pub callback_kind: Option<CallbackKind>,
    pub mode: Mode,
    pub mutable: bool,
    pub transferred: Option<Span>,
    pub moved_fields: BTreeMap<Vec<String>, Span>,
    pub used: Option<Span>,
}

#[derive(Debug)]
pub struct CheckedFunction {
    pub template: bool,
    pub specialization: bool,
    pub name: String,
    pub bindings: Vec<Binding>,
    pub parameter_count: usize,
    pub result: Type,
    pub body: Vec<Stmt>,
}

pub struct Capture {
    pub outer: usize,
    pub inner: usize,
    pub storage: Mode,
    pub access: Mode,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub enum ClosureForward {
    Once(usize),
    Abstract(usize),
    /// A declared function is a capture-free concrete callable origin.
    Function(usize),
    /// Capture-free registry origin; direct calls retain optional arity metadata.
    Native(usize),
    /// Fixed facade preserving source tail cardinality for omission-sensitive slots.
    RestAdapter { fixed: usize, counted: bool },
}

pub struct CheckedClosureMethod {
    /// Receiver access belongs to this invocation body, not its environment.
    pub invocation: Mode,
    /// Method binding slots paired with the environment capture fields.
    pub captures: Vec<usize>,
    pub parameters: Vec<usize>,
    pub function: CheckedFunction,
}

#[derive(Clone)]
struct ClosureMethodSource {
    context: Context,
    parameters: Vec<usize>,
    result: Type,
    body: Vec<Stmt>,
}

pub struct CheckedClosure {
    /// Stable environment; only invocation bodies materialize per witness vector.
    pub polymorphic: bool,
    pub methods: Vec<CheckedClosureMethod>,
    /// Forwarding descriptors share contracts without copying checked bodies.
    pub forward: Option<ClosureForward>,
    pub borrowed: bool,
    pub lifetime_count: usize,
    pub depth: usize,
    pub parameters: Vec<usize>,
    pub function: CheckedFunction,
    pub captures: Vec<Capture>,
    pub span: Span,
    pub escapes: bool,
    pub invocation: Mode,
}

#[derive(Clone, Copy, Debug)]
pub struct ArgumentContract {
    /// Type accepted in the supplied argument slot, including undefined.
    pub ty: Type,
    pub mode: Mode,
    pub omission: crate::ParameterOmission,
    pub rest: bool,
}

impl From<&Binding> for ArgumentContract {
    fn from(binding: &Binding) -> Self {
        Self {
            ty: binding.input_ty.unwrap_or(binding.ty),
            mode: binding.mode,
            omission: binding.omission,
            rest: binding.rest,
        }
    }
}

pub struct CallbackContract {
    pub(crate) structural: crate::callable_contracts::ContractId,
    pub parameter_names: Vec<String>,
    pub params: Vec<ArgumentContract>,
    pub result: Type,
    pub kind: CallbackKind,
}

impl CallbackContract {
    pub(crate) fn invocation(&self, signatures: &crate::callable_contracts::Contracts) -> Mode {
        if let Some(storage) =
            crate::callable_storage::StorageContract::checked(signatures, self.structural)
        {
            storage.invocation(signatures)
        } else if self.kind.is_once() {
            Mode::Consume
        } else {
            self.kind.invocation()
        }
    }
}


pub struct CheckedProgram {
    pub(crate) named_capture_layouts: Vec<named_capture_layout::Layout>,
    pub(crate) named_capture_access: HashMap<usize, Vec<named_capture_access::Access>>,
    pub(crate) local_capture_plans: Vec<crate::local_capture_plan::Plan>,
    pub(crate) local_declarations: Vec<crate::local_functions::Declaration>,
    pub(crate) error_effects: crate::error_effects::ErrorEffects,
    pub(crate) stored_contracts: crate::callable_storage::Registry,
    pub(crate) callable_shapes: crate::callable_contracts::Contracts,
    pub(crate) function_specializations: HashMap<usize,usize>,
    pub(crate) deferred_generic_values: HashSet<usize>,
    pub(crate) deferred_generic_sources: HashMap<usize, Token>,
    pub(crate) deferred_generic_nominals: Vec<(Span, Span)>,
    pub generic_declarations: HashMap<usize, Token>,
    pub generic_parameter_types: HashMap<usize, Vec<Type>>,
    pub records: Vec<Record>,
    pub unions: Vec<crate::unions::UnionType>,
    pub functions: Vec<CheckedFunction>,
    pub closures: Vec<CheckedClosure>,
    pub callback_contracts: Vec<CallbackContract>,
    pub(crate) absent_callback_payloads: HashSet<usize>,
}

impl CheckedProgram {
    pub fn contracts(&self) -> BTreeMap<String, Mode> {
        self.functions
            .iter()
            .filter(|fun| !fun.specialization)
            .flat_map(|fun| {
                fun.bindings
                    .iter()
                    .take(fun.parameter_count)
                    .map(|b| (format!("{}.{}", fun.name, b.name), b.mode))
            })
            .collect()
    }
}

#[derive(Clone)]
struct Signature {
    params: Vec<Type>,
    body_params: Vec<Type>,
    result: Type,
    inferred_result: bool,
}
/// State carried between the phases of one conditional expression check.
struct ConditionalPlan {
    else_flow: crate::flow::Facts,
    hint: Option<Type>,
    assignment_alias: bool,
    borrowed: bool,
    infer_nullable_closure: bool,
    before: Vec<(usize, Availability)>,
    first_loan: usize,
    left_context: Option<Type>,
    right_context: Option<Type>,
    after_left: Vec<Availability>,
    after_left_flow: crate::flow::Facts,
    left_loans: Vec<(usize, Mode, Span)>,
}

#[derive(Clone, Default)]
struct Availability {
    transferred: Option<Span>,
    fields: BTreeMap<Vec<String>, Span>,
}
impl Availability {
    fn of(binding: &Binding) -> Self {
        Self {
            transferred: binding.transferred,
            fields: binding.moved_fields.clone(),
        }
    }
    fn restore(self, binding: &mut Binding) {
        binding.transferred = self.transferred;
        binding.moved_fields = self.fields;
    }
    fn join(mut self, other: Self) -> Self {
        self.transferred = self.transferred.or(other.transferred);
        for (path, span) in other.fields {
            self.fields.entry(path).or_insert(span);
        }
        self
    }
}

#[derive(Clone)]
struct Projection {
    root: String,
    path: Vec<String>,
    replacing: bool,
    span: Span,
}

#[derive(Clone)]
struct LoopFrame {
    labels: Vec<Token>,
    target: ControlTarget,
    flow_entry: crate::flow::Facts,
    flow_required: crate::flow::Facts,
    break_flow: Option<crate::flow::Facts>,
    continue_flow: Option<crate::flow::Facts>,
    entry: Vec<(usize, Availability)>,
    breaks: Vec<Availability>,
    break_seen: bool,
    // New loops execute a tail after continue, so ownership checks happen
    // after joining their fallthrough and continue states, not at continue.
    defer_continue: bool,
    continues: Option<Vec<Availability>>,
    span: Span,
}

type CallbackMetadata = (Option<Mode>, bool, Option<CallbackKind>);

#[derive(Clone)]
struct Context {
    region_loop_depth:Option<usize>,
    region_entry_loans:Option<usize>,
    exception_environment:Option<std::rc::Rc<exception_state::CheckEnvironment>>,
    completion_frame:Option<Box<exception_state::CompletionFrame>>,
    error_frame:Option<Box<exception_state::ErrorFrame>>,
    selected_access: HashMap<usize, Mode>,
    flow: crate::flow::Facts,
    constant_absence: HashMap<usize,Type>,
    keep_flow: Vec<crate::flow::Place>,
    mutation_place: Option<crate::flow::Place>,
    suppressed_flow: HashSet<crate::flow::Place>,
    observer_places: HashSet<crate::flow::Place>,
    condition_flows: HashMap<(usize, usize), crate::flow::Outcomes>,
    has_field_moves: bool,
    projection: Option<Projection>,
    replacement: Option<Span>,
    pending_names: HashSet<String>,
    lexical_scopes: Vec<HashSet<String>>,
    callable_scope_depth: usize,
    effects_only: bool,
    loops: Vec<LoopFrame>,
    template: bool,
    bindings: Vec<Binding>,
    /// Source declaration starts resolve independently of current lexical spelling.
    source_bindings: HashMap<usize, usize>,
    names: HashMap<String, usize>,
    capture_loans: Vec<(usize, Mode, Span)>,
    iteration_loans: Vec<(usize, Span)>,
    capture_boundary: usize,
    named_capture_range: (usize, usize),
    closure_depth: usize,
    inferred_body_result: bool,
    returning_closure: bool,
    returned_callable: Option<Type>,
    function_name: String,
    /// Parameters declared ReadOnly. A const taken from one of their
    /// positions borrows it for reading.
    readonly_parameters: HashSet<usize>,
}
impl Context {
    #[inline]
    fn is_capture(&self, slot: usize) -> bool {
        slot < self.capture_boundary || (slot >= self.named_capture_range.0 && slot < self.named_capture_range.1)
    }

    fn new(function_name: String) -> Self {
        Self {
            region_loop_depth:None,
            region_entry_loans:None,
            exception_environment:None,
            completion_frame:None,
            error_frame:None,
            selected_access: HashMap::new(),
            flow: HashMap::new(),
            constant_absence: HashMap::new(),
            keep_flow: Vec::new(),
            mutation_place: None,
            suppressed_flow: HashSet::new(),
            observer_places: HashSet::new(),
            condition_flows: HashMap::new(),
            has_field_moves: false,
            projection: None,
            replacement: None,
            pending_names: HashSet::new(),
            lexical_scopes: Vec::new(),
            callable_scope_depth: 0,
            effects_only: false,
            loops: Vec::new(),
            template: false,
            bindings: Vec::new(),
            source_bindings: HashMap::new(),
            names: HashMap::new(),
            capture_loans: Vec::new(),
            iteration_loans: Vec::new(),
            capture_boundary: 0,
            named_capture_range: (0, 0),
            closure_depth: 0,
            inferred_body_result: false,
            returning_closure: false,
            returned_callable: None,
            function_name,
            readonly_parameters: HashSet::new(),
        }
    }
}
type Loans = HashMap<usize, Vec<(Mode, Span)>>;

/// Inspect statement bodies without adding recursive analyzer frames.
pub(crate) fn has_exception_syntax(body:&[Stmt])->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    while let Some(statement)=pending.pop() {
        match &statement.kind {
            StmtKind::Try(_)|StmtKind::Throw(_)=>return true,
            StmtKind::Block(body)|StmtKind::While{body,..}|StmtKind::ForOf{body,..}=>pending.extend(body),
            StmtKind::Loop{body,update,..}=>{pending.extend(body);pending.extend(update);},
            StmtKind::If{then_body,else_body,..}=>{pending.extend(then_body);pending.extend(else_body);},
            _=>{},
        }
    }
    false
}

pub fn analyze(program: Program) -> Result<CheckedProgram, Diagnostic> {
    finish_analysis(Analyzer::new(program)?.run()?)
}

// Incomplete capture layouts stay private, even after typed flow checking.
fn finish_analysis(mut checked: CheckedProgram) -> Result<CheckedProgram, Diagnostic> {
    definitely_assigned(&mut checked)?;
    checked.named_capture_layouts = named_capture_layout::build(&checked)?;
    crate::named_capture_lowering::prepare(&mut checked)?;
    Ok(checked)
}

/// Deferred `let` declarations are assigned before every use, in function
/// bodies and in the checked bodies of function values.
#[inline(never)]
fn definitely_assigned(checked: &mut CheckedProgram) -> Result<(), Diagnostic> {
    let exceptions=crate::emit::needs_exception_transport(checked);
    checked.error_effects = if exceptions {crate::error_effects::annotate_inferred_callbacks(checked)}else{crate::error_effects::annotate(checked)};
    if exceptions {crate::error_effects::materialize_runtime_record(checked);}
    crate::error_effects::materialize_catch_types(checked)?;
    let captures = checked
        .closures
        .iter()
        .map(|closure| {
            (closure.span, closure.captures.iter().map(|capture| capture.outer).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    for function in &mut checked.functions {
        crate::definite_assignment::check(&mut function.body, &captures)?;
    }
    for closure in &mut checked.closures {
        crate::definite_assignment::check(&mut closure.function.body, &captures)?;
        for method in &mut closure.methods {
            crate::definite_assignment::check(&mut method.function.body, &captures)?;
        }
    }
    crate::local_readiness::check_checked(checked)?;
    if exceptions && !checked.local_capture_plans.iter().any(|plan| plan.pending_diagnostic().is_some()) {
        crate::error_effects::defer_unreferenced_error_definitions(checked);
        crate::emit::validate_exception_program(checked)?;
    }
    Ok(())
}

/// Resolve finite access lattices before a fresh definitive ownership analysis.
/// Speculative checked trees are discarded and are never exposed or emitted.
pub fn analyze_recursive(program: Program) -> Result<CheckedProgram, Diagnostic> {
    let initial = Analyzer::new(program.clone())?;
    let mut modes = initial
        .signatures
        .iter()
        .map(|signature| {
            signature
                .params
                .iter()
                .map(|ty| if ty.is_copy() { Mode::Copy } else { Mode::Read })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let parameter_count = modes.iter().map(Vec::len).sum::<usize>();
    let mut capture_access = named_capture_access::Lattice::default();
    // Every non-copy coordinate can rise from read to mutate to consume only.
    let mut remaining = parameter_count.saturating_mul(2) + 1;
    while remaining > 0 {
        remaining -= 1;
        let mut analyzer = Analyzer::new(program.clone())?;
        analyzer.assumed_modes = Some(modes.clone());
        analyzer.assumed_capture_access = capture_access.clone();
        analyzer.effects_only = true;
        analyzer.speculative_access_run = true;
        let inferred = analyzer.run()?;
        for function in inferred.functions.iter().skip(modes.len()) {
            let added = function
                .bindings
                .iter()
                .take(function.parameter_count)
                .map(|binding| {
                    if binding.ty.is_copy() {
                        Mode::Copy
                    } else {
                        Mode::Read
                    }
                })
                .collect::<Vec<_>>();
            remaining += added.len().saturating_mul(2) + 1;
            modes.push(added);
        }
        let (mut changed, mut discovered) = capture_access.observe(&inferred.named_capture_access);
        let (generic_changed, generic_discovered) = capture_access.observe_declarations(&inferred.named_capture_access, &inferred.generic_declarations);
        changed |= generic_changed;
        discovered += generic_discovered;
        remaining += discovered.saturating_mul(2) + usize::from(discovered != 0);
        for (current, function) in modes.iter_mut().zip(&inferred.functions) {
            for (mode, binding) in current.iter_mut().zip(&function.bindings) {
                let next = (*mode).max(binding.mode);
                changed |= next != *mode;
                *mode = next;
            }
        }
        if !changed {
            let mut analyzer = Analyzer::new(program)?;
            analyzer.assumed_modes = Some(modes);
            analyzer.assumed_capture_access = capture_access;
            return finish_analysis(analyzer.run()?);
        }
    }
    unreachable!("finite recursive access lattice must converge")
}

#[cfg(test)]
mod exception_escape_tests {
    use super::*;
    #[test]
    fn thrown_and_returned_callable_origins_cross_regions_without_catch_shadow_leaks() {
        for (source,expected) in [
            ("function main(){throw callback;}",true),
            ("function main(){try{throw callback;}catch{}}",true),
            ("function main(){try{}catch(callback){throw callback;}}",false),
            ("function main(){try{}finally{return callback;}}",true),
        ] {
            let program=crate::syntax::parse(source).unwrap();
            let names=HashMap::from([("callback".to_owned(),91)]);
            let mut escaping=HashSet::new();
            Analyzer::escape_origins(&program.functions[0].body,&names,&mut escaping);
            assert_eq!(escaping.contains(&91),expected,"{source}");
        }
    }
}

struct Analyzer {
    named_captures: named_capture_context::Contexts,
    assumed_capture_access: named_capture_access::Lattice,
    program: Program,
    // Retained source specialization state for the checked-inference handoff.
    generic_functions: crate::generic_functions::Cache,
    generic_records: crate::generic_records::Cache,
    unions: crate::unions::UnionArena,
    templates: HashMap<usize, Function>,
    specializations: HashMap<(usize, Vec<Type>), usize>,
    original_count: usize,
    record_names: HashMap<String, usize>,
    function_names: HashMap<String, usize>,
    fields: Vec<HashMap<String, Type>>,
    signatures: Vec<Signature>,
    checked: Vec<Option<CheckedFunction>>,
    visiting: Vec<bool>,
    function_stack: Vec<usize>,
    assumed_modes: Option<Vec<Vec<Mode>>>,
    effects_only: bool,
    /// The whole analyzer is a discarded access-inference pass. Stored
    /// callable origins are noted for error-effect lookup without layout
    /// validation. A temporary effects-only scope in a definitive run never
    /// sets this.
    speculative_access_run: bool,
    builtin_literals: bool,
    observing_values: bool,
    recursive_callbacks: HashSet<usize>,
    depth: usize,
    closures: Vec<CheckedClosure>,
    closure_types: HashMap<ClosureKey, usize>,
    once_returns: HashMap<usize, usize>,
    rest_adapters: HashMap<(Type,Type,Mode),usize>,
    abstract_returns: HashMap<usize, usize>,
    function_values: HashMap<usize, usize>,
    deferred_generic_values: HashMap<usize, deferred_generic_values::Source>,
    closure_method_sources: HashMap<usize, ClosureMethodSource>,
    closure_specializations: HashMap<(usize, Vec<Type>, bool, bool), usize>,
    callables: crate::callable_resolution::Tables,
    absent_callback_payloads: HashMap<usize, usize>,
    record_cloneability: Option<(usize, crate::cloneability::Cloneability)>,
    escaping: HashSet<usize>,
    discovering_results: bool,
    known_results: HashMap<usize, Type>,
    result_shapes: HashMap<usize, usize>,
    result_waits: HashMap<usize, Span>,
    pending_shapes: HashSet<usize>,
    validated_closures: HashSet<usize>,
}

#[derive(PartialEq, Eq, Hash)]
struct CaptureKey {
    name: String,
    ty: Type,
    storage: Mode,
    access: Mode,
    callback_invocation: Option<Mode>,
    callback_reference: bool,
    callback_kind: Option<CallbackKind>,
}

/// A source arrow has the same concrete type when its capture layout and
/// callable contracts match. Unused outer specialization parameters do not
/// create new native environment types; captured callable types still do.
#[derive(PartialEq, Eq, Hash)]
struct ClosureKey {
    origin: usize,
    template: bool,
    escapes: bool,
    depth: usize,
    result: Type,
    invocation: Mode,
    captures: Vec<CaptureKey>,
    parameters: Vec<(Type, Mode)>,
}

fn number_literal_context(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Integer(_) | ExprKind::EnumMember { .. } | ExprKind::Number(_) => true,
        ExprKind::Unary {
            op: UnaryOp::Positive | UnaryOp::Negate,
            value,
            ..
        } => number_literal_context(value),
        _ => false,
    }
}

pub(crate) fn number_predicate_target(name: &str) -> Option<CallTarget> {
    Some(match name {
        "isFinite" => CallTarget::NumberIsFinite,
        "isNaN" => CallTarget::NumberIsNaN,
        "isInteger" => CallTarget::NumberIsInteger,
        "isSafeInteger" => CallTarget::NumberIsSafeInteger,
        _ => return None,
    })
}

/// Internal method name for the `codePointAt` slot array. `#` cannot appear in
/// a source member name, so programs cannot call it.
pub(crate) const STRING_CODE_POINT_SLOT: &str = "codePointAt#slot";

pub(crate) fn math_target(name: &str) -> Option<CallTarget> {
    Some(match name {
        "abs" => CallTarget::MathAbs,
        "floor" => CallTarget::MathFloor,
        "ceil" => CallTarget::MathCeil,
        "trunc" => CallTarget::MathTrunc,
        "sqrt" => CallTarget::MathSqrt,
        "round" => CallTarget::MathRound,
        "sign" => CallTarget::MathSign,
        "fround" => CallTarget::MathFround,
        "f16round" => CallTarget::MathF16round,
        "random" => CallTarget::MathRandom,
        "min" => CallTarget::MathMin,
        "max" => CallTarget::MathMax,
        "clz32" => CallTarget::MathClz32,
        "imul" => CallTarget::MathImul,
        "acos" => CallTarget::MathAcos,
        "acosh" => CallTarget::MathAcosh,
        "asin" => CallTarget::MathAsin,
        "asinh" => CallTarget::MathAsinh,
        "atan" => CallTarget::MathAtan,
        "atanh" => CallTarget::MathAtanh,
        "cbrt" => CallTarget::MathCbrt,
        "cos" => CallTarget::MathCos,
        "cosh" => CallTarget::MathCosh,
        "exp" => CallTarget::MathExp,
        "expm1" => CallTarget::MathExpm1,
        "log" => CallTarget::MathLog,
        "log10" => CallTarget::MathLog10,
        "log1p" => CallTarget::MathLog1p,
        "log2" => CallTarget::MathLog2,
        "sin" => CallTarget::MathSin,
        "sinh" => CallTarget::MathSinh,
        "tan" => CallTarget::MathTan,
        "tanh" => CallTarget::MathTanh,
        "atan2" => CallTarget::MathAtan2,
        "pow" => CallTarget::MathPow,
        "hypot" => CallTarget::MathHypot,
        _ => return None,
    })
}

pub(crate) fn resolve_value_type(
    annotation: &crate::type_syntax::ValueAnnotation,
    allow_void: bool,
    record_names: &HashMap<String, usize>,
    unions: &crate::unions::UnionArena,
) -> Result<Type, Diagnostic> {
    use crate::type_syntax::ValueAnnotationKind as Kind;
    match &annotation.kind {
        Kind::Group(inner) => resolve_value_type(inner, allow_void, record_names, unions),
        Kind::Union(members) => {
            let members = members
                .iter()
                .map(|member| resolve_value_type(member, true, record_names, unions))
                .collect::<Result<Vec<_>, _>>()?;
            unions.intern(members, annotation.span)
        }
        Kind::Array(_) => resolve_array_annotation(annotation, record_names, unions),
        Kind::Tuple(_) => Err(Diagnostic::new("type", "Tuple annotation has not been materialized.", annotation.span)),
        Kind::Applied { .. } => Err(Diagnostic::new(
            "unsupported",
            "Generic type applications are not yet supported by the type checker.",
            annotation.span,
        )),
        Kind::Callable { .. } => Err(Diagnostic::new(
            "unsupported",
            "Callable value annotations are not yet supported in this position.",
            annotation.span,
        )),
        Kind::Named(name) => match name.text.as_str() {
            "int" => Ok(Type::Int),
            "number" => Ok(Type::Number),
            "boolean" => Ok(Type::Boolean),
            "string" => Ok(Type::String),
            "null" => Ok(Type::Null),
            "undefined" => Ok(Type::Undefined),
            "void" if allow_void => Ok(Type::Void),
            _ => record_names
                .get(&name.text)
                .copied()
                .map(|id| unions.named_type(id))
                .ok_or_else(|| {
                    Diagnostic::new(
                        "type",
                        match name.text.as_str() {
                            "Function" => "Function needs a result type, such as Function<int> or Function<int, [string]>.".to_owned(),
                            "Mutable" | "ReadOnly" | "Once" | "Owned" => format!("{} wraps one type, such as {}<Function<int>>.", name.text, name.text),
                            _ => format!("Unknown or unsupported type {:?}.", name.text),
                        },
                        name.span,
                    )
                }),
        },
    }
}

fn resolve_array_annotation(
    annotation: &crate::type_syntax::ValueAnnotation,
    record_names: &HashMap<String, usize>,
    unions: &crate::unions::UnionArena,
) -> Result<Type, Diagnostic> {
    use crate::type_syntax::ValueAnnotationKind as Kind;
    let mut inner = annotation;
    let mut dimensions = 0;
    loop {
        match &inner.kind {
            Kind::Group(value) => inner = value,
            Kind::Array(value) => {
                dimensions += 1;
                inner = value;
            }
            _ => break,
        }
    }
    if dimensions > 128 {
        return Err(Diagnostic::new(
            "unsupported",
            "Array type nesting exceeds the prototype limit of 128.",
            annotation.span,
        ));
    }
    let depth = (dimensions - 1) as u8;
    let element = match &inner.kind {
        Kind::Named(name) => match name.text.as_str() {
            "int" => Some(ArrayElement::Int(depth)),
            "number" => Some(ArrayElement::Number(depth)),
            "boolean" => Some(ArrayElement::Boolean(depth)),
            "string" => Some(ArrayElement::String(depth)),
            "null" => Some(ArrayElement::Null(depth)),
            "undefined" => Some(ArrayElement::Undefined(depth)),
            _ => record_names
                .get(&name.text)
                .copied()
                .and_then(|id| ArrayElement::from_value_type(unions.named_type(id))).map(|leaf| leaf.with_depth(depth)),
        },
        Kind::Union(_) => match resolve_value_type(inner, false, record_names, unions)? {
            Type::Union(id) => Some(ArrayElement::Union(id, depth)),
            // Duplicate alternatives collapse to one ordinary leaf.
            single => ArrayElement::from_value_type(single).map(|leaf| leaf.with_depth(depth)),
        },
        Kind::Applied { .. } => {
            return Err(Diagnostic::new(
                "unsupported",
                "Generic type applications are not yet supported by the type checker.",
                inner.span,
            ));
        }
        _ => None,
    };
    element.map(Type::Array).ok_or_else(|| {
        Diagnostic::new(
            "array-element",
            "Array leaves must be int, number, boolean, string, or a named record.",
            annotation.span,
        )
    })
}

pub(crate) use crate::record_layout::append_checked_record_fields;

#[derive(Clone, Copy)]
enum CallbackOrigin {
    Parameter(usize),
    Closure(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum NativeCallbackTarget {
    Function(usize),
    Method(usize, usize),
}

struct CallbackCall {
    target: NativeCallbackTarget,
    arguments: Vec<CallbackOrigin>,
    span: Span,
    /// A context-dependent method is counted per caller witness vector, but
    /// its own outgoing calls have no standalone typed body to traverse.
    leaf: bool,
}

/// Collect typed native calls without recursively walking the syntax tree.
/// Closure bodies are separate native roots; their syntax has been moved out
/// of the creating expression by checking.
fn callback_calls(
    function: &CheckedFunction,
    functions: &[&CheckedFunction],
    closures: &[CheckedClosure],
    parameter_ids: &[usize],
) -> Result<Vec<CallbackCall>, Diagnostic> {
    enum Node<'a> {
        Statement(&'a Stmt),
        Expression(&'a Expr),
    }
    let parameters = parameter_ids
        .iter()
        .map(|&id| &function.bindings[id])
        .filter_map(|binding| match binding.ty {
            Type::Callback(id) => Some(id),
            _ => None,
        })
        .enumerate()
        .map(|(position, id)| (id, position))
        .collect::<HashMap<_, _>>();
    let mut nodes = function
        .body
        .iter()
        .rev()
        .map(Node::Statement)
        .collect::<Vec<_>>();
    // Defaults are checked callee expressions, not part of the source body.
    // Forwarding closures are excluded as native roots by the caller below.
    nodes.extend(function.bindings.iter().rev().filter_map(|binding| {
        binding.initializer.as_deref().map(Node::Expression)
    }));
    let mut calls = Vec::new();
    while let Some(node) = nodes.pop() {
        match node {
            Node::Statement(stmt) => match &stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Let { value, .. }
                | StmtKind::Expr(value)
                | StmtKind::Return(Some(value)) | StmtKind::Throw(value) => nodes.push(Node::Expression(value)),
                StmtKind::Assign { target, value, .. } => {
                    nodes.push(Node::Expression(value));
                    nodes.push(Node::Expression(target));
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    nodes.extend(else_body.iter().rev().map(Node::Statement));
                    nodes.extend(then_body.iter().rev().map(Node::Statement));
                    nodes.push(Node::Expression(condition));
                }
                StmtKind::While { condition, body } => {
                    nodes.extend(body.iter().rev().map(Node::Statement));
                    nodes.push(Node::Expression(condition));
                }
                StmtKind::Try(region) => nodes.extend(region.statements().rev().map(Node::Statement)),
                StmtKind::Block(body) => nodes.extend(body.iter().rev().map(Node::Statement)),
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    nodes.extend(update.iter().rev().map(Node::Statement));
                    nodes.extend(body.iter().rev().map(Node::Statement));
                    if let Some(condition) = condition {
                        nodes.push(Node::Expression(condition));
                    }
                }
                StmtKind::ForOf { iterable, body, .. } => {
                    nodes.extend(body.iter().rev().map(Node::Statement));
                    nodes.push(Node::Expression(iterable));
                }
                StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => (),
            },
            Node::Expression(expr) => match &expr.kind {
                ExprKind::SpreadCall(plan) => nodes.extend(plan.expressions().map(Node::Expression)),
                ExprKind::SpreadValue(_) => (),
                ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => nodes.push(Node::Expression(value)),
                ExprKind::Optional { base, body, .. } => {
                    nodes.push(Node::Expression(base));
                    nodes.push(Node::Expression(body));
                }
                ExprKind::Selected { .. } => (),
                ExprKind::Call(callee, args) => {
                    let target = match expr.target {
                        Some(
                            CallTarget::Function(target) | CallTarget::ForwardFunction(target),
                        ) => {
                            let checked = functions[target];
                            (!checked.template).then(|| {
                                (
                                    NativeCallbackTarget::Function(target),
                                    checked,
                                    (0..checked.parameter_count).collect::<Vec<_>>(),
                                )
                            })
                        }
                        Some(CallTarget::ClosureMethod(method)) => {
                            let Type::Closure(mut closure) =
                                callee.ty.expect("checked method receiver")
                            else {
                                unreachable!("checked method environment")
                            };
                            if let Some(ClosureForward::Once(inner)) = closures[closure].forward {
                                closure = inner;
                            }
                            let checked = &closures[closure].methods[method];
                            Some((
                                NativeCallbackTarget::Method(closure, method),
                                &checked.function,
                                checked.parameters.clone(),
                            ))
                        }
                        _ => None,
                    };
                    if let Some((target, checked, parameter_ids)) = target {
                        let mut arguments = Vec::new();
                        for (&parameter, arg) in parameter_ids.iter().zip(args) {
                            if matches!(checked.bindings[parameter].ty, Type::Callback(_)) {
                                let origin = match arg.ty.expect("checked callback argument") {
                                    Type::Closure(id) => CallbackOrigin::Closure(id),
                                    Type::Callback(id) => CallbackOrigin::Parameter(*parameters.get(&id).ok_or_else(|| Diagnostic::new(
                                        "callback-environment", "Cannot resolve this callback's native parameter path.", arg.span))?),
                                    _ => unreachable!("checked callback type"),
                                };
                                arguments.push(origin);
                            }
                        }
                        calls.push(CallbackCall {
                            target,
                            arguments,
                            span: expr.span,
                            leaf: checked.template,
                        });
                    }
                    nodes.extend(args.iter().rev().map(Node::Expression));
                    nodes.push(Node::Expression(callee));
                }
                ExprKind::Project { value, .. }
                | ExprKind::Inject { value, .. }
                | ExprKind::Unary { value, .. }
                | ExprKind::Update { target: value, .. }
                | ExprKind::Field(value, _) => nodes.push(Node::Expression(value)),
                ExprKind::Conditional {
                    predicate,
                    consequent,
                    alternate,
                } => {
                    nodes.push(Node::Expression(alternate));
                    nodes.push(Node::Expression(consequent));
                    nodes.push(Node::Expression(predicate));
                }
                ExprKind::Binary { left, right, .. }
                | ExprKind::CompoundAssign {
                    target: left,
                    value: right,
                    ..
                }
                | ExprKind::Assign {
                    target: left,
                    value: right,
                    ..
                }
                | ExprKind::CoalesceAssign {
                    target: left,
                    value: right,
                    ..
                }
                | ExprKind::LogicalAssign {
                    target: left,
                    value: right,
                    ..
                }
                | ExprKind::Index(left, right) => {
                    nodes.push(Node::Expression(right));
                    nodes.push(Node::Expression(left));
                }
                ExprKind::Array(values)
                | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _)
                | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                    nodes.extend(values.iter().rev().map(Node::Expression))
                }
                ExprKind::Record(fields) => nodes.extend(
                    fields
                        .iter()
                        .rev()
                        .map(|(_, value)| Node::Expression(value)),
                ),
                ExprKind::Closure { .. }
                | ExprKind::Name(_)
                | ExprKind::String(_)
                | ExprKind::Integer(_)
                | ExprKind::EnumMember { .. } | ExprKind::Number(_)
                | ExprKind::Boolean(_)
                | ExprKind::Null
                | ExprKind::Undefined => (),
            },
        }
    }
    Ok(calls)
}

/// Callback borrows cannot be captured, so reusable/once generic parameters
/// forward flat concrete environment types rather than growing wrapper types.
/// Track actual type vectors, including repeated types and fresh closed arrows.
fn validate_callback_instances(
    functions: &[&CheckedFunction],
    closures: &[CheckedClosure],
    recursive: &HashSet<usize>,
    definitions: &[Function],
) -> Result<(), Diagnostic> {
    let mut calls = HashMap::new();
    let mut visited = HashSet::new();
    let mut pending = Vec::new();
    let mut counts = HashMap::<NativeCallbackTarget, usize>::new();
    for (id, function) in functions.iter().enumerate() {
        if function.template {
            continue;
        }
        let node = NativeCallbackTarget::Function(id);
        let parameters = (0..function.parameter_count).collect::<Vec<_>>();
        calls.insert(
            node,
            callback_calls(function, functions, closures, &parameters)?,
        );
        if !parameters
            .iter()
            .any(|&p| matches!(function.bindings[p].ty, Type::Callback(_)))
        {
            visited.insert((node, Vec::new()));
            pending.push((node, Vec::new()));
        }
    }
    let mut roots = Vec::new();
    for (origin, closure) in closures.iter().enumerate() {
        if !closure.function.template
            && !closure.polymorphic
            && !matches!(closure.forward, Some(ClosureForward::Function(_)))
        {
            roots.extend(callback_calls(
                &closure.function,
                functions,
                closures,
                &closure.parameters,
            )?);
        }
        for (method, checked) in closure.methods.iter().enumerate() {
            if checked.function.template {
                continue;
            }
            let node = NativeCallbackTarget::Method(origin, method);
            calls.insert(
                node,
                callback_calls(&checked.function, functions, closures, &checked.parameters)?,
            );
            // Every method is reached through a typed caller edge. Do not seed
            // generic methods with a fabricated empty witness vector.
        }
    }
    let mut roots_pending = true;
    loop {
        let (arguments, outgoing) = if roots_pending {
            roots_pending = false;
            (Vec::new(), roots.as_slice())
        } else if let Some((node, arguments)) = pending.pop() {
            (
                arguments,
                calls
                    .get(&node)
                    .expect("typed native callback target")
                    .as_slice(),
            )
        } else {
            break;
        };
        for call in outgoing {
            let next = call
                .arguments
                .iter()
                .map(|origin| match *origin {
                    CallbackOrigin::Parameter(position) => arguments[position],
                    CallbackOrigin::Closure(id) => id,
                })
                .collect::<Vec<_>>();
            if visited.insert((call.target, next.clone())) {
                let count = counts.entry(call.target).or_default();
                *count += 1;
                let guarded = match call.target {
                    NativeCallbackTarget::Function(id) => recursive.contains(&id),
                    NativeCallbackTarget::Method(_, _) => true,
                };
                if guarded && *count > 128 {
                    let related = match call.target {
                        NativeCallbackTarget::Function(id) => definitions[id].name.span,
                        NativeCallbackTarget::Method(origin, _) => closures[origin].span,
                    };
                    return Err(Diagnostic::new("unsupported", "Recursive callback instantiations exceed the prototype limit of 128 per native target.", call.span)
                        .related(related, "This native function or method requires too many concrete callback path vectors."));
                }
                if !call.leaf {
                    pending.push((call.target, next));
                }
            }
        }
    }
    Ok(())
}

fn remap_closure_type(ty: &mut Type, ids: &[Option<usize>]) -> Result<(), Diagnostic> {
    if let Type::Closure(id) = ty {
        *id = ids[*id].ok_or_else(|| {
            Diagnostic::new(
                "unsupported",
                "Callable result retains an unresolved environment type.",
                Span::default(),
            )
        })?;
    }
    Ok(())
}

fn remap_function_closures(
    function: &mut CheckedFunction,
    ids: &[Option<usize>],
) -> Result<(), Diagnostic> {
    remap_closure_type(&mut function.result, ids)?;
    for binding in &mut function.bindings {
        remap_closure_type(&mut binding.ty, ids)?;
        if let Some(ty) = &mut binding.input_ty {
            remap_closure_type(ty, ids)?;
        }
    }
    enum Node<'a> {
        Statement(&'a mut Stmt),
        Expression(&'a mut Expr),
    }
    let mut nodes = function
        .body
        .iter_mut()
        .map(Node::Statement)
        .collect::<Vec<_>>();
    nodes.extend(function.bindings.iter_mut().filter_map(|binding| {
        binding.initializer.as_deref_mut().map(Node::Expression)
    }));
    while let Some(node) = nodes.pop() {
        match node {
            Node::Statement(stmt) => match &mut stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Let { value, .. }
                | StmtKind::Expr(value)
                | StmtKind::Return(Some(value)) | StmtKind::Throw(value) => nodes.push(Node::Expression(value)),
                StmtKind::Assign { target, value, .. } => {
                    nodes.push(Node::Expression(target));
                    nodes.push(Node::Expression(value));
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    nodes.push(Node::Expression(condition));
                    nodes.extend(then_body.iter_mut().map(Node::Statement));
                    nodes.extend(else_body.iter_mut().map(Node::Statement));
                }
                StmtKind::While { condition, body } => {
                    nodes.push(Node::Expression(condition));
                    nodes.extend(body.iter_mut().map(Node::Statement));
                }
                StmtKind::Try(region) => nodes.extend(region.statements_mut().map(Node::Statement)),
                StmtKind::Block(body) => nodes.extend(body.iter_mut().map(Node::Statement)),
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    if let Some(condition) = condition {
                        nodes.push(Node::Expression(condition));
                    }
                    nodes.extend(body.iter_mut().map(Node::Statement));
                    nodes.extend(update.iter_mut().map(Node::Statement));
                }
                StmtKind::ForOf { iterable, body, .. } => {
                    nodes.push(Node::Expression(iterable));
                    nodes.extend(body.iter_mut().map(Node::Statement));
                }
                StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => (),
            },
            Node::Expression(expr) => {
                if let Some(ty) = &mut expr.ty {
                    remap_closure_type(ty, ids)?;
                }
                // The saved call origin names the same environment arena. An
                // origin that was not retained leaves the call without one, so
                // its error contract stays unresolved rather than misdirected.
                if let Some(CallableOrigin::Closure(origin)) = expr.callable_origin {
                    expr.callable_origin = ids.get(origin).copied().flatten().map(CallableOrigin::Closure);
                }
                match &mut expr.kind {
                    ExprKind::SpreadCall(plan) => {
                        for segment in &mut plan.segments { remap_closure_type(&mut segment.element, ids)?; }
                        for parameter in &mut plan.parameters { remap_closure_type(&mut parameter.ty, ids)?; }
                        for ty in &mut plan.selected_types { remap_closure_type(ty, ids)?; }
                        nodes.extend(plan.expressions_mut().map(Node::Expression));
                    }
                    ExprKind::SpreadValue(_) => (),
                    ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => nodes.push(Node::Expression(value)),
                    ExprKind::Optional { base, body, .. } => {
                        nodes.push(Node::Expression(base));
                        nodes.push(Node::Expression(body));
                    }
                    ExprKind::Selected { .. } => (),
                    ExprKind::Project {
                        value, narrowed, ..
                    } => {
                        remap_closure_type(narrowed, ids)?;
                        nodes.push(Node::Expression(value));
                    }
                    ExprKind::Inject { value, .. }
                    | ExprKind::Unary { value, .. }
                    | ExprKind::Update { target: value, .. }
                    | ExprKind::Field(value, _) => nodes.push(Node::Expression(value)),
                    ExprKind::Conditional {
                        predicate,
                        consequent,
                        alternate,
                    } => {
                        nodes.push(Node::Expression(predicate));
                        nodes.push(Node::Expression(consequent));
                        nodes.push(Node::Expression(alternate));
                    }
                    ExprKind::Binary { left, right, .. }
                    | ExprKind::CompoundAssign {
                        target: left,
                        value: right,
                        ..
                    }
                    | ExprKind::Assign {
                        target: left,
                        value: right,
                        ..
                    }
                    | ExprKind::CoalesceAssign {
                        target: left,
                        value: right,
                        ..
                    }
                    | ExprKind::LogicalAssign {
                        target: left,
                        value: right,
                        ..
                    }
                    | ExprKind::Index(left, right) => {
                        nodes.push(Node::Expression(left));
                        nodes.push(Node::Expression(right));
                    }
                    ExprKind::Array(values)
                    | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _)
                    | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                        nodes.extend(values.iter_mut().map(Node::Expression))
                    }
                    ExprKind::Record(fields) => {
                        nodes.extend(fields.iter_mut().map(|(_, value)| Node::Expression(value)))
                    }
                    ExprKind::Call(callee, args) => {
                        nodes.push(Node::Expression(callee));
                        nodes.extend(args.iter_mut().map(Node::Expression));
                    }
                    ExprKind::Closure { .. }
                    | ExprKind::Name(_)
                    | ExprKind::String(_)
                    | ExprKind::Integer(_)
                    | ExprKind::EnumMember { .. } | ExprKind::Number(_)
                    | ExprKind::Boolean(_)
                    | ExprKind::Null
                    | ExprKind::Undefined => (),
                }
            }
        }
    }
    Ok(())
}

impl Analyzer {
    fn new(mut program: Program) -> Result<Self, Diagnostic> {
        crate::local_readiness::check(&mut program)?;
        if let Some(base) = program.unresolved_bases.first() {
            return Err(Diagnostic::new(
                "type",
                format!("{:?} is not a record type that an interface can extend.", base.text),
                base.span,
            ));
        }
        let mut generic_functions = crate::generic_functions::specialize(&mut program)?;
        if let Some((span, arguments)) = program.call_type_arguments.first() {
            return Err(Diagnostic::new(
                "type",
                "Explicit type arguments require a declared generic function.",
                arguments.first().map_or(*span, |argument| argument.span),
            ));
        }
        let unions = crate::unions::UnionArena::default();
        let mut generic_records = crate::generic_records::specialize_in(&mut program, &unions)?;
        let mut record_names = HashMap::new();
        let mut function_names = HashMap::new();
        let mut callables = crate::callable_resolution::Tables::default();
        let mut fields = Vec::new();
        for (index, record) in program.records.iter().enumerate() {
            if [
                "string",
                "int",
                "number",
                "boolean",
                "null",
                "undefined",
                "void",
            ]
            .contains(&record.name.text.as_str())
                || record_names
                    .insert(record.name.text.clone(), index)
                    .is_some()
            {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Type {:?} is already defined.", record.name.text),
                    record.name.span,
                ));
            }
        }
        for argument in &mut program.type_argument_checks {
            generic_records.materialize(
                argument,
                &mut program.records,
                &mut record_names,
                &unions,
            )?;
            resolve_value_type(argument, false, &record_names, &unions)?;
        }
        append_checked_record_fields(
            &mut program.records,
            &record_names,
            &unions,
            &mut fields,
            &mut callables,
        )?;
        generic_functions.seed_checked(&program, &record_names, &unions)?;
        for (index, fun) in program.functions.iter().enumerate() {
            if [
                "clone", "String", "Boolean", "Array", "console", "Number", "Math", "null",
                "typeof", "void",
            ]
            .contains(&fun.name.text.as_str())
                || function_names
                    .insert(fun.name.text.clone(), index)
                    .is_some()
            {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Function name {:?} is unavailable.", fun.name.text),
                    fun.name.span,
                ));
            }
        }
        let mut escaping = HashSet::new();
        for fun in &program.functions {
            let mut names = HashMap::new();
            for ((name,_,_),initializer) in fun.params.iter().zip(&fun.parameter_initializers) {
                Self::parameter_escape_origins(&name.text,initializer,&mut names,&mut escaping);
            }
            Self::escape_origins(&fun.body, &names, &mut escaping);
        }
        let count = program.functions.len();
        let templates = program
            .functions
            .iter()
            .enumerate()
            .filter(|(_, fun)| {
                fun.params.iter().any(|(_, _, c)| {
                    c.as_ref().is_some_and(|c| {
                        c.kind.is_owned()
                            || c.params.iter().any(|p| match &p.ty {
                                ParameterAnnotation::Callback(_) => true,
                                // A nullable callback argument needs the same
                                // caller-specialized contract as a present one.
                                ParameterAnnotation::Named(annotation) => crate::callable_annotations::nullable_callable(annotation),
                                ParameterAnnotation::Inferred(_) => false,
                            })
                            || matches!(*c.result, ResultAnnotation::Callable { .. })
                    })
                })
            })
            .map(|(id, fun)| (id, fun.clone()))
            .collect();
        let named_captures = named_capture_context::Contexts::new(&program, &function_names);
        let mut analyzer = Self {
            named_captures,
            assumed_capture_access: Default::default(),
            generic_functions,
            generic_records,
            unions,
            templates,
            specializations: HashMap::new(),
            original_count: count,
            program,
            record_names,
            function_names,
            fields,
            signatures: Vec::new(),
            checked: (0..count).map(|_| None).collect(),
            visiting: vec![false; count],
            function_stack: Vec::new(),
            assumed_modes: None,
            effects_only: false,
            speculative_access_run: false,
            builtin_literals: false,
            observing_values: false,
            recursive_callbacks: HashSet::new(),
            depth: 0,
            closures: Vec::new(),
            closure_types: HashMap::new(),
            once_returns: HashMap::new(),
            rest_adapters: HashMap::new(),
            abstract_returns: HashMap::new(),
            function_values: HashMap::new(),
            deferred_generic_values: HashMap::new(),
            closure_method_sources: HashMap::new(),
            closure_specializations: HashMap::new(),
            callables,
            absent_callback_payloads: HashMap::new(),
            record_cloneability: None,
            escaping,
            discovering_results: false,
            known_results: HashMap::new(),
            result_shapes: HashMap::new(),
            result_waits: HashMap::new(),
            pending_shapes: HashSet::new(),
            validated_closures: HashSet::new(),
        };
        for index in 0..count {
            let source_params = analyzer.program.functions[index].params.clone();
            let mut params = Vec::new();
            for (_, ty, callback) in &source_params {
                params.push(if let Some(annotation) = callback {
                    analyzer.resolve_callback(annotation)?
                } else {
                    analyzer
                        .callable_resolver()
                        .resolve_value_annotation(ty, false)?
                });
            }
            if params.iter().copied().any(|ty| ty.union().is_some() && analyzer.unions.payload(ty).is_some_and(|payload|matches!(payload,Type::Callback(id) if analyzer.callback_needs_context(id)))) {
                analyzer.templates.insert(index,analyzer.program.functions[index].clone());
            }
            let annotation = analyzer.program.functions[index].result.take();
            let result = annotation
                .as_ref()
                .map(|value| analyzer.resolve_result(value))
                .transpose()?
                .unwrap_or(Type::Void);
            if let Some(application) = &analyzer.program.functions[index].application {
                let mut bindings = application
                    .variables
                    .iter()
                    .zip(&application.arguments)
                    .map(|(variable, argument)| {
                        Ok((
                            variable.text.clone(),
                            resolve_value_type(
                                argument,
                                false,
                                &analyzer.record_names,
                                &analyzer.unions,
                            )?,
                        ))
                    })
                    .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
                let callable_shapes = params
                    .iter()
                    .copied()
                    .filter_map(|ty| {
                        let payload = analyzer.unions.payload(ty).unwrap_or(ty);
                        analyzer
                            .checked_generic_callable_shape(payload)
                            .map(|shape| (payload, shape))
                    })
                    .collect();
                let types = crate::generic_inference::Types { callable_annotations: None,
                    inline_shapes: None,
                    callable_shapes: Some(&callable_shapes),
                    records: &analyzer.program.records,
                    names: &analyzer.record_names,
                    unions: &analyzer.unions,
                };
                let hints = params.iter().copied().map(Some).collect::<Vec<_>>();
                types.solve(
                    crate::generic_inference::Constraints {
                        variables: &application.variables,
                        parameters: &application.patterns,
                        arguments: &hints,
                        result: None,
                        expected: None,
                        span: analyzer.program.functions[index].name.span,
                    },
                    &mut bindings,
                )?;
            }
            let body_params = params.clone();
            for (ty, initializer) in params.iter_mut().zip(&analyzer.program.functions[index].parameter_initializers) {
                if initializer.omission().is_omittable() { *ty = analyzer.unions.intern([*ty, Type::Undefined], analyzer.program.functions[index].name.span)?; }
            }
            analyzer.signatures.push(Signature { params, body_params, result, inferred_result: annotation.is_none() && analyzer.program.functions[index].name.text != "main" && return_type_inference::has_value_return(&analyzer.program.functions[index].body) });
        }
        Ok(analyzer)
    }

    fn escape_value_origin(expr: &Expr, names: &HashMap<String, usize>) -> Option<usize> {
        match &expr.kind {
            ExprKind::TypeAssertion { value, .. } => Self::escape_value_origin(value, names),
            ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                value,
                ..
            } => Self::escape_value_origin(value, names),
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                left,
                right,
                ..
            } => Self::escape_value_origin(left, names)
                .or_else(|| Self::escape_value_origin(right, names)),
            ExprKind::Closure { id, .. } => Some(*id),
            ExprKind::Name(name) => names.get(name).copied(),
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                let origin = Self::escape_value_origin(consequent, names)?;
                (Self::escape_value_origin(alternate, names) == Some(origin)).then_some(origin)
            }
            _ => None,
        }
    }
    fn mark_escaping_value(
        expr: &Expr,
        names: &HashMap<String, usize>,
        escaping: &mut HashSet<usize>,
    ) {
        if let Some(origin) = Self::escape_value_origin(expr, names) {
            escaping.insert(origin);
        }
        if let ExprKind::Binary {
            op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
            left,
            right,
            ..
        } = &expr.kind
        {
            Self::mark_escaping_value(left, names, escaping);
            Self::mark_escaping_value(right, names, escaping);
        }
        if let ExprKind::Unary {
            op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
            value,
            ..
        } = &expr.kind
        {
            Self::mark_escaping_value(value, names, escaping);
        }
        if let ExprKind::Conditional {
            consequent,
            alternate,
            ..
        } = &expr.kind
        {
            Self::mark_escaping_value(consequent, names, escaping);
            Self::mark_escaping_value(alternate, names, escaping);
        }
    }

    // Resolve only direct local origins here; actual value/name validation still
    // happens during checking. Each block gets its own lexical name map.
    #[inline(never)]
    fn escape_origins(
        body: &[Stmt],
        outer: &HashMap<String, usize>,
        escaping: &mut HashSet<usize>,
    ) {
        let mut names = outer.clone();
        for stmt in body {
            match &stmt.kind {
                StmtKind::Let { value, .. }
                | StmtKind::Expr(value)
                | StmtKind::Throw(value)
                | StmtKind::Return(Some(value)) => {
                    Self::expression_escapes(value, &names, escaping)
                }
                StmtKind::Assign { target, value, .. } => {
                    Self::expression_escapes(target, &names, escaping);
                    Self::expression_escapes(value, &names, escaping);
                }
                StmtKind::If { condition, .. } | StmtKind::While { condition, .. } => {
                    Self::expression_escapes(condition, &names, escaping)
                }
                StmtKind::ForOf { iterable, .. } => {
                    Self::expression_escapes(iterable, &names, escaping)
                }
                StmtKind::Loop {
                    condition: Some(condition),
                    ..
                } => Self::expression_escapes(condition, &names, escaping),
                _ => (),
            }
            match &stmt.kind {
                StmtKind::Let {
                    name,
                    annotation,
                    value,
                    ..
                } => {
                    if annotation
                        .as_ref()
                        .and_then(crate::callable_annotations::ownership)
                        .is_some_and(CallbackKind::is_owned)
                    {
                        Self::mark_escaping_value(value, &names, escaping);
                    }
                    // Declaration moves retain the source environment's origin so
                    // a later return chooses owned capture storage at creation.
                    let origin = Self::escape_value_origin(value, &names);
                    names.remove(&name.text);
                    if let Some(id) = origin {
                        names.insert(name.text.clone(), id);
                    }
                }
                StmtKind::Return(Some(value)) | StmtKind::Throw(value) => Self::mark_escaping_value(value, &names, escaping),
                StmtKind::Try(region) => {
                    Self::escape_origins(&region.body,&names,escaping);
                    if let Some(handler)=&region.catch {
                        let mut catch_names=names.clone();
                        if let Some(name)=&handler.name {catch_names.remove(&name.text);}
                        Self::escape_origins(&handler.body,&catch_names,escaping);
                    }
                    if let Some(body)=&region.finally {Self::escape_origins(body,&names,escaping);}
                }
                StmtKind::While { body, .. } | StmtKind::Block(body) => {
                    Self::escape_origins(body, &names, escaping)
                }
                StmtKind::Loop { body, update, .. } => {
                    Self::escape_origins(body, &names, escaping);
                    Self::escape_origins(update, &names, escaping);
                }
                StmtKind::ForOf { name, body, .. } => {
                    let mut loop_names = names.clone();
                    loop_names.remove(&name.text);
                    Self::escape_origins(body, &loop_names, escaping);
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    Self::escape_origins(then_body, &names, escaping);
                    Self::escape_origins(else_body, &names, escaping);
                }
                _ => (),
            }
        }
    }

    fn callable_resolver(&mut self) -> crate::callable_resolution::Resolver<'_> {
        self.callables.resolver(&self.record_names, &self.unions)
    }

    fn resolve_callback(&mut self, annotation: &CallbackAnnotation) -> Result<Type, Diagnostic> {
        self.callable_resolver().resolve_callback(annotation)
    }

    fn resolve_result(&mut self, annotation: &ResultAnnotation) -> Result<Type, Diagnostic> {
        self.callable_resolver().resolve_result(annotation)
    }

    /// Stated access is the parameter's contract: the body may use the value
    /// less, never more, and callers lend or give exactly what is stated.
    fn stated_parameter_access(name: &str, binding: &mut Binding, declared: Mode, span: Span) -> Result<(), Diagnostic> {
        if binding.ty.is_copy() {
            return Err(Diagnostic::new("callback-access", "ReadOnly, Mutable and Owned apply only to data-owning parameters; scalars copy.", span));
        }
        if binding.mode > declared {
            let wrapper = |mode: Mode| match mode {
                Mode::Read => "ReadOnly",
                Mode::Mutate => "Mutable",
                Mode::Consume => "Owned",
                Mode::Copy => "no wrapper",
            };
            return Err(Diagnostic::new(
                "parameter-access",
                format!(
                    "Parameter {name:?} is declared {}, but the function {} it; declare it {}.",
                    wrapper(declared),
                    if binding.mode == Mode::Consume { "takes ownership of" } else { "changes" },
                    wrapper(binding.mode)
                ),
                span,
            ));
        }
        binding.mode = declared;
        Ok(())
    }

    fn declared_parameter_access(&self, index: usize, context: &mut Context) -> Result<(), Diagnostic> {
        let function = &self.program.functions[index];
        for (position, access) in function.parameter_access.iter().enumerate() {
            if let Some((declared, span)) = *access {
                Self::stated_parameter_access(&function.params[position].0.text, &mut context.bindings[position], declared, span)?;
            }
        }
        Ok(())
    }

    /// An annotated arrow parameter cannot be void, and only a data-owning
    /// parameter can state access.
    fn arrow_parameter_contract(&self, param: &CallableParameter, ty: Type) -> Result<(), Diagnostic> {
        if matches!(param.ty, ParameterAnnotation::Inferred(_)) {
            return Ok(());
        }
        if ty == Type::Void {
            return Err(Diagnostic::new("unsupported", "Closure arguments cannot have type void.", param.ty.span()));
        }
        if let Some(access) = param.access.as_ref().filter(|_| !param.rest) {
            if matches!(self.unions.payload(ty).unwrap_or(ty), Type::Callback(_)) {
                return Err(Diagnostic::new("callback-access", "A Function argument states its ownership with ReadOnly<Function<...>>, Mutable<Function<...>> or Once<Function<...>>, not a data access wrapper.", access.span));
            }
            if ty.is_copy() {
                return Err(Diagnostic::new("callback-access", "ReadOnly, Mutable and Owned apply only to data-owning arguments; scalars copy.", access.span));
            }
        }
        Ok(())
    }

    fn callback_needs_context(&self, id: usize) -> bool {
        let contract = &self.callables.callbacks[id];
        contract.kind.is_owned() || matches!(contract.result, Type::Callable(_))
            || contract.params.iter().any(|param| {
                matches!(
                    self.unions.payload(param.ty).unwrap_or(param.ty),
                    Type::Callback(_)
                )
            })
    }

    fn parameter_escape_origins(name: &str, initializer: &ParameterInitializer, names: &mut HashMap<String,usize>, escaping: &mut HashSet<usize>) {
        let origin = if let ParameterInitializer::Default{value,..}=initializer {
            Self::expression_escapes(value,names,escaping);
            Self::escape_value_origin(value,names)
        } else {None};
        names.remove(name);
        if let Some(origin)=origin {names.insert(name.to_owned(),origin);}
    }

    fn expression_escapes(
        expr: &Expr,
        names: &HashMap<String, usize>,
        escaping: &mut HashSet<usize>,
    ) {
        match &expr.kind {
            ExprKind::Closure { params, body, .. } => {
                let mut names=names.clone();
                for parameter in params {Self::parameter_escape_origins(&parameter.name.text,&parameter.initializer,&mut names,escaping);}
                Self::escape_origins(body,&names,escaping);
            },
            ExprKind::Call(callee, args) => {
                Self::expression_escapes(callee, names, escaping);
                for arg in args {
                    Self::expression_escapes(arg, names, escaping);
                }
            }
            ExprKind::Project { value, .. }
            | ExprKind::Inject { value, .. }
            | ExprKind::Unary { value, .. }
            | ExprKind::Update { target: value, .. } => {
                Self::expression_escapes(value, names, escaping)
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                Self::expression_escapes(predicate, names, escaping);
                Self::expression_escapes(consequent, names, escaping);
                Self::expression_escapes(alternate, names, escaping);
            }
            ExprKind::Binary { left, right, .. }
            | ExprKind::CompoundAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::Assign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::CoalesceAssign {
                target: left,
                value: right,
                ..
            }
            | ExprKind::LogicalAssign {
                target: left,
                value: right,
                ..
            } => {
                Self::expression_escapes(left, names, escaping);
                Self::expression_escapes(right, names, escaping);
            }
            ExprKind::Field(base, _) => Self::expression_escapes(base, names, escaping),
            ExprKind::Index(base, index) => {
                Self::expression_escapes(base, names, escaping);
                Self::expression_escapes(index, names, escaping);
            }
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                for value in values {
                    Self::expression_escapes(value, names, escaping);
                }
            }
            ExprKind::Record(fields) => {
                for (_, expr) in fields {
                    Self::expression_escapes(expr, names, escaping);
                }
            }
            _ => (),
        }
    }

    fn require_parameter_source(
        &self,
        actual: crate::callable_compatibility::Parameters<'_>,
        expected: &[ArgumentContract],
        span: Span,
    ) -> Result<(), Diagnostic> {
        Self::arity(actual.len(), expected.len(), span)?;
        let actual_parameters = actual.iter().collect::<Vec<_>>();
        let actual_rest=actual_parameters.last().is_some_and(|parameter|parameter.rest);
        let expected_rest=expected.last().is_some_and(|parameter|parameter.rest);
        if actual_rest!=expected_rest || crate::parameter_arity::minimum_arguments(&actual_parameters[..actual_parameters.len()-usize::from(actual_rest)], |parameter|parameter.omission.is_omittable()) != crate::parameter_arity::minimum_arguments(&expected[..expected.len()-usize::from(expected_rest)], |parameter|parameter.omission.is_omittable()) {
            return Err(Diagnostic::new("callback-contract", "Callable accepted argument positions and rest contracts must match.", span));
        }
        for (actual, expected) in actual_parameters.into_iter().zip(expected) {
            self.require(actual.ty, Some(expected.ty), span)?;
            let callback_reborrow = matches!(expected.ty, Type::Callback(id)
                if self.callables.callbacks[id].kind == CallbackKind::Reusable)
                && actual.mode == Mode::Read
                && expected.mode == Mode::Mutate;
            if !self.effects_only && actual.mode != expected.mode && !callback_reborrow {
                return Err(Diagnostic::new(
                    "callback-access",
                    format!(
                        "Expected {} argument access, found {}. Callable access contracts must match exactly.",
                        expected.mode.as_str(),
                        actual.mode.as_str()
                    ),
                    span,
                ));
            }
        }
        Ok(())
    }


    fn closure(
        &mut self,
        outer: &mut Context,
        syntax: (usize, &[CallableParameter], &ResultAnnotation),
        contextual: Option<Type>,
        body: &mut Vec<Stmt>,
        span: Span,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let (origin, params, result) = syntax;
        let generic_wrapper = if let [statement] = body.as_slice()
            && let StmtKind::Return(Some(call)) = &statement.kind
            && let ExprKind::Call(callee, _) = &call.kind
            && call.span == span && callee.span == span
            && let ExprKind::Name(name) = &callee.kind
            && self.generic_functions.template(name).is_some() { Some(name.as_str()) } else { None };
        if let Some(name) = generic_wrapper && contextual.is_none()
            && (params.iter().any(|parameter| matches!(parameter.ty, ParameterAnnotation::Inferred(_)))
                || (params.is_empty() && matches!(result, ResultAnnotation::Inferred(_)))) {
            return self.deferred_captured_generic_value(outer, name, origin, span, loans);
        }
        let contextual_result = generic_wrapper.and_then(|_| contextual)
            .and_then(|ty| self.checked_generic_callable_shape(self.unions.payload(ty).unwrap_or(ty)))
            .map(|(_, result)| result);
        let inferred_result = matches!(result, ResultAnnotation::Inferred(_)) && contextual_result.is_none();
        let mut result = if let Some(result) = contextual_result { result }
            else if inferred_result { Type::Void } else { self.resolve_result(result)? };
        let escapes = self.escaping.contains(&origin);
        let mut visible = outer.names.values().copied().collect::<Vec<_>>();
        // A nested function called in the body can need a variable that has no
        // visible name here: a shadowed one, or one that only a called function uses.
        let hidden_captures = !self.program.local_capture_plans.is_empty();
        if hidden_captures {
            visible.extend(outer.source_bindings.values().copied());
        }
        visible.sort_unstable();
        visible.dedup();
        let mut context = capture_context::project(outer, &visible);
        // Preserve lexical aliases as well as binding declaration names. Adapter
        // wrappers introduce a private alias to avoid parameter-name shadowing.
        context.names = outer.names.iter().map(|(name, slot)| (name.clone(), visible.binary_search(slot).expect("visible name"))).collect();
        context.inferred_body_result = inferred_result;
        if self.discovering_results {
            context.pending_names = outer.pending_names.clone();
        }
        let contextual = contextual.and_then(|ty| match self.unions.payload(ty).unwrap_or(ty) {
            Type::Callback(id) => Some(self.callables.callbacks[id].params.clone()),
            Type::Callable(id) => Some(self.callables.results[id].params.clone()),
            Type::StoredCallable(id) => self.callables.storage.declaration(id).map(|id|self.callables.callbacks[id].params.clone()),
            _ => None,
        });
        let mut parameters = Vec::new();
        let mut declared = Vec::new();
        let inherited_names = context.names.clone();
        // Parameter names shadow captures, but must remain unique among themselves.
        context.names.clear();
        for (position, param) in params.iter().enumerate() {
            let ty = match &param.ty {
                ParameterAnnotation::Named(annotation) => self.callable_resolver().resolve_value_annotation(annotation,true)?,
                ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation)?,
                ParameterAnnotation::Inferred(_) => contextual.as_ref().and_then(|params|params.get(position)).map_or(Type::Void, |parameter| {
                    // The compiler's accepted input type supplies context; its
                    // undefined admission is not a body type inference rule.
                    if matches!(param.initializer,ParameterInitializer::Default{..}) {
                        parameter.ty.union().and_then(|id|self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),param.name.span).ok()).unwrap_or(parameter.ty)
                    } else {parameter.ty}
                }),
            };
            self.arrow_parameter_contract(param, ty)?;
            let binding = Self::bind(&mut context,&param.name,ty,true)?;
            parameters.push(binding);
            declared.push(ty);
        }
        context.names = inherited_names;
        for (position, (param,&binding)) in params.iter().zip(&parameters).enumerate() {
            let mut body_ty=declared[position];
            let default_mode=if body_ty.is_copy(){Mode::Copy}else{match param.access.as_ref().map(|access|access.text.as_str()){Some("read")=>Mode::Read,Some("mutate")=>Mode::Mutate,_=>Mode::Consume}};
            let checked_initializer = if body_ty == Type::Void && matches!(param.ty,ParameterAnnotation::Inferred(_)) {
                let ParameterInitializer::Default{value,..}=&param.initializer else {return Err(Diagnostic::new("parameter-inference",format!("Cannot infer the type of arrow parameter `{}`. Add a type annotation, such as `({}: string) => ...`, or pass this arrow where a callback parameter type is known.", param.name.text, param.name.text),param.name.span));};
                let mut value=value.clone();
                let expected=self.expression_type_hint(&context,&value);
                body_ty=self.expression(&mut context,&mut value,Mode::Consume,&Loans::new(),expected)?;
                Some(value)
            } else {self.check_parameter_default(&mut context,&param.initializer,body_ty,default_mode)?};
            let input_ty=if param.initializer.omission().is_omittable() {self.unions.intern([body_ty,Type::Undefined],param.name.span)?} else {body_ty};
            if matches!(param.initializer,ParameterInitializer::Optional(_)) {body_ty=input_ty;}
            context.bindings[binding].ty=body_ty;
            context.bindings[binding].input_ty=Some(input_ty);
            context.bindings[binding].omission=param.initializer.omission();
            context.bindings[binding].rest=param.rest;
            if param.rest {
                if !matches!(body_ty,Type::Array(_)) {return Err(Diagnostic::new("rest-type","A rest parameter requires a supported homogeneous array type.",param.name.span));}
                if param.access.as_ref().is_some_and(|access|access.text!="consume") {return Err(Diagnostic::new("rest-access","A rest parameter owns its pack; only explicit consume access is supported.",param.name.span));}
                context.bindings[binding].mode=Mode::Consume;
            }
            context.bindings[binding].declaration_span=Some(param.name.span);
            context.bindings[binding].initializer=checked_initializer;
            if matches!(default_mode,Mode::Read|Mode::Mutate)
                && let Some(value)=context.bindings[binding].initializer.as_deref()
                && Self::root_binding(value).is_some() {
                context.bindings[binding].view_roots=Self::view_sources(&context,value);
            }
            context.names.insert(param.name.text.clone(),binding);
            self.initialized_parameter_fact(&mut context,binding);
            if body_ty.is_copy() {context.bindings[binding].mode=Mode::Copy;}
            if let Type::Callback(id)=self.unions.payload(body_ty).unwrap_or(body_ty) {
                let kind=self.callables.callbacks[id].kind;
                context.bindings[binding].callback_kind=Some(kind);
                context.bindings[binding].callback_reference=matches!(kind,CallbackKind::Reusable|CallbackKind::Read);
                context.bindings[binding].callback_invocation=Some(kind.invocation());
                if kind.is_owned()||kind.is_once() {context.bindings[binding].mode=Mode::Consume;}
            }
            if param.access.as_ref().is_some_and(|access|access.text=="read") && !body_ty.is_copy() {
                context.readonly_parameters.insert(binding);
            }
            self.initialize_parameter_pattern(&mut context,&param.name,body,param.access.as_ref().is_some_and(|access|access.text=="consume"))?;
        }
        let method_template = parameters.iter().any(|&parameter| matches!(self.unions.payload(context.bindings[parameter].ty).unwrap_or(context.bindings[parameter].ty),Type::Callback(_)));
        let source_body=method_template.then(||body.clone());
        let declared_result=result;
        let method_context = method_template.then(|| context.clone());
        if method_template {
            context.template = true;
        }
        for (param, &binding) in params.iter().zip(&parameters) {
            if param.access.as_ref().is_some_and(|access| access.text == "read") && !param.rest && !context.bindings[binding].ty.is_copy() {
                context.readonly_parameters.insert(binding);
            }
        }
        let returned = self
            .check_source_callable_body(&mut context, Self::uninitialized_body(body), result)
            .map_err(|error| self.filter_ownership_fixes(&context, error))?;
        for (param, &binding) in params.iter().zip(&parameters) {
            if let Some(access) = param.access.as_ref().filter(|_| !param.rest) {
                let declared = match access.text.as_str() {
                    "read" => Mode::Read,
                    "mutate" => Mode::Mutate,
                    _ => Mode::Consume,
                };
                Self::stated_parameter_access(&param.name.text, &mut context.bindings[binding], declared, access.span)?;
            }
        }
        if inferred_result { result = self.inferred_body_result(body)?; }
        if result != Type::Void && !returned {
            return Err(Diagnostic::new(
                "type",
                "Closure must return a value on every path.",
                span,
            ));
        }
        let result = if self.callable_result_type(result) {
            if self.discovering_results && context.returned_callable.is_none() {
                return Err(Self::pending_result(span));
            }
            context
                .returned_callable
                .expect("checked callable closure return")
        } else {
            result
        };
        let result = if method_template && !inferred_result {
            declared_result
        } else {
            result
        };
        self.finalize_nullable_locals(&mut context.bindings, body)?;
        let mut captures = Vec::new();
        let mut invocation = Mode::Read;
        for (inner, &outer_id) in visible.iter().enumerate() {
            let binding = &context.bindings[inner];
            if let Some(usage) = binding.used {
                if self.discovering_results && self.has_pending_result(binding.ty) {
                    return Err(Self::pending_result(span));
                }
                let access = binding.mode;
                invocation = invocation.max(access);
                let storage = if binding.ty.is_copy() {
                    if access == Mode::Mutate && !escapes {
                        Mode::Mutate
                    } else {
                        Mode::Copy
                    }
                } else if escapes {
                    Mode::Consume
                } else {
                    access
                };
                if !self.effects_only
                    && escapes
                    && let Type::Closure(id) = binding.ty
                    && self.closures[id].borrowed
                {
                    return Err(Diagnostic::new(
                        "closure-escape",
                        "An escaping closure cannot retain a borrowed callable environment.",
                        span,
                    )
                    .related(self.closures[id].span, "Borrowed callable is created here."));
                }
                Self::access(outer, outer_id, storage, span, loans)?;
                if matches!(storage, Mode::Read | Mode::Mutate) {
                    outer.capture_loans.push((outer_id, storage, span));
                }
                captures.push(Capture {
                    outer: outer_id,
                    inner,
                    storage,
                    access,
                    span: usage,
                });
            }
        }
        // Captured scalar writes borrow local storage, or mutate a private copied
        // field in an escaping environment. Record the actual representation so
        // scalar reads and whole-binding stores lower through references correctly.
        for capture in &captures {
            let binding = &mut context.bindings[capture.inner];
            binding.capture_reference = match capture.storage {
                Mode::Read if matches!(binding.ty, Type::BorrowedUnion(_) | Type::Union(_)) => {
                    false
                }
                Mode::Read | Mode::Mutate => true,
                Mode::Consume => invocation != Mode::Consume,
                Mode::Copy => capture.access == Mode::Mutate && invocation == Mode::Mutate,
            };
            binding.mode = if capture.storage == Mode::Consume
                || (capture.storage == Mode::Copy && binding.capture_reference)
            {
                invocation
            } else {
                capture.storage
            };
        }
        // Provisional recursive summaries can overstate borrowed lifetimes.
        // Cap the count just above the supported limit; inference never emits
        // this layout, and final checking still enforces the actual guard.
        let lifetime_count = captures.iter().fold(0usize, |total, capture| {
            let nested = match self
                .unions
                .payload(context.bindings[capture.inner].ty)
                .unwrap_or(context.bindings[capture.inner].ty)
            {
                Type::Closure(id) => self.closures[id].lifetime_count,
                _ => 0,
            };
            (total + usize::from(matches!(capture.storage, Mode::Read | Mode::Mutate)) + nested)
                .min(129)
        });
        if !self.effects_only && lifetime_count > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Closure environment lifetime complexity exceeds the prototype limit of 128.",
                span,
            ));
        }
        let borrowed = lifetime_count != 0;
        let key = ClosureKey {
            origin,
            template: outer.template,
            escapes,
            depth: context.closure_depth,
            result,
            invocation,
            captures: captures
                .iter()
                .map(|capture| {
                    let binding = &context.bindings[capture.inner];
                    CaptureKey {
                        name: binding.name.clone(),
                        ty: binding.ty,
                        storage: capture.storage,
                        access: capture.access,
                        callback_invocation: binding.callback_invocation,
                        callback_reference: binding.callback_reference,
                        callback_kind: binding.callback_kind,
                    }
                })
                .collect(),
            parameters: parameters
                .iter()
                .map(|&id| {
                    let binding = &context.bindings[id];
                    (binding.ty, binding.mode)
                })
                .collect(),
        };
        if let Some(&id) = self.closure_types.get(&key) {
            // Creation effects and loans above still apply to this invocation.
            // Emission reuses the first fully checked body and capture layout.
            if !self.discovering_results {
                self.closures[id].function.bindings = context.bindings;
                self.closures[id].function.body = if method_template {
                    body.clear();
                    Vec::new()
                } else {
                    std::mem::take(body)
                };
                self.closures[id].captures = captures;
                self.closures[id].parameters = parameters;
                self.validated_closures.insert(id);
            } else {
                body.clear();
            }
            if let (Some(context), Some(body)) = (method_context, source_body) {
                self.closure_method_sources.insert(
                    id,
                    ClosureMethodSource {
                        context,
                        parameters: self.closures[id].parameters.clone(),
                        result: declared_result,
                        body,
                    },
                );
            }
            return Ok(Type::Closure(id));
        }
        let id = self.closures.len();
        self.closure_types.insert(key, id);
        if let (Some(context), Some(body)) = (method_context, source_body) {
            self.closure_method_sources.insert(
                id,
                ClosureMethodSource {
                    context,
                    parameters: parameters.clone(),
                    result: declared_result,
                    body,
                },
            );
        }
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        self.closures.push(CheckedClosure {
            polymorphic: method_template,
            methods: Vec::new(),
            forward: None,
            borrowed,
            lifetime_count,
            depth: context.closure_depth,
            parameters,
            function: CheckedFunction {
                template: outer.template,
                specialization: false,
                name: outer.function_name.clone(),
                bindings: context.bindings,
                parameter_count: 0,
                result,
                body: if method_template {
                    body.clear();
                    Vec::new()
                } else {
                    std::mem::take(body)
                },
            },
            captures,
            span,
            escapes,
            invocation,
        });
        Ok(Type::Closure(id))
    }

    // Resolve declared values in an independent function context. Forwarding
    // environments have no owners, while argument modes remain the target's
    // exact inferred contracts. Keep this work off recursive expression frames.
    fn native_function_value(
        &mut self,
        id: usize,
        span: Span,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let binding = &crate::native_bindings::BINDINGS[id];
        let count = match expected {
            Some(Type::Callback(contract)) => self.callables.callbacks[contract].params.len(),
            Some(Type::Callable(contract)) => self.callables.results[contract].params.len(),
            Some(Type::Closure(origin)) => self.closures[origin].parameters.len(),
            _ => binding.parameters.len(),
        };
        self.native_function_shape(id, span, count)
    }

    fn native_function_shape(
        &mut self,
        id: usize,
        span: Span,
        count: usize,
    ) -> Result<Type, Diagnostic> {
        let binding = &crate::native_bindings::BINDINGS[id];
        if !binding.accepts_arity(count) {
            return Err(Diagnostic::new(
                "arity",
                "Native function does not accept the callback's argument count.",
                span,
            ));
        }
        let existing = self.closures.iter().position(|closure| {
            matches!(closure.forward, Some(ClosureForward::Native(origin)) if origin == id)
                && closure.parameters.len() == count
        });
        let parameter_types = binding.parameters[..count].iter()
            .map(|parameter| self.native_parameter_type(parameter.ty))
            .collect::<Result<Vec<_>, _>>()?;
        let origin = existing.unwrap_or_else(|| {
            let bindings = binding.parameters[..count]
                .iter()
                .zip(parameter_types)
                .map(|(parameter, ty)| Binding { rest: false,
                    name: parameter.name.into(),
                    ty,
                    parameter: true,
                    input_ty: None,
                    declaration_span: None,
                    initializer: None,
                    rebindable: false,
                    capture_reference: false,
                    borrowed_element: None,
                    borrowed_place: None,
                    callback_invocation: None,
                    callback_reference: false,
                    callback_kind: None,
                    mode: parameter.access,
                    mutable: false,
                    transferred: None,
                    moved_fields: BTreeMap::new(),
                    used: None,
                    view_roots: Vec::new(),
                 omission: crate::ParameterOmission::Required, })
                .collect();
            let origin = self.closures.len();
            self.closures.push(CheckedClosure {
                polymorphic: false,
                methods: Vec::new(),
                forward: Some(ClosureForward::Native(id)),
                borrowed: false,
                lifetime_count: 0,
                depth: 0,
                parameters: (0..count).collect(),
                function: CheckedFunction {
                    template: false,
                    specialization: false,
                    name: binding.name.into(),
                    bindings,
                    parameter_count: 0,
                    result: native_type(binding.result),
                    body: Vec::new(),
                },
                captures: Vec::new(),
                span,
                escapes: true,
                invocation: Mode::Read,
            });
            origin
        });
        self.validated_closures.insert(origin);
        Ok(Type::Closure(origin))
    }

    /// A capturing nested function is checked when a call supplies its typed
    /// defining context. Its declaring function is checked first, because a
    /// linked program can order the lifted function before it. A function that
    /// no call reaches waits for its completed lexical types; the deferred
    /// placeholder is replaced when that body can be checked.
    #[inline(never)]
    fn capturing_function_is_unused(&mut self, index: usize) -> Result<bool, Diagnostic> {
        if !self.named_captures.needs_context(index) || self.named_captures.ready(index) {
            return Ok(false);
        }
        if let Some(owner) = self.named_captures.declaring_function(index, &self.program, &self.function_names)
            && owner != index
            && self.checked[owner].is_none()
            && !self.visiting[owner]
        {
            if !self.capturing_function_is_unused(owner)? {
                self.function(owner)?;
            }
        }
        if self.named_captures.ready(index) || self.checked[index].is_some() {
            return Ok(false);
        }
        self.checked[index] = Some(CheckedFunction {
            template: true,
            specialization: false,
            name: self.program.functions[index].name.text.clone(),
            bindings: Vec::new(),
            parameter_count: 0,
            result: self.signatures[index].result,
            body: Vec::new(),
        });
        Ok(true)
    }

    /// Checks the hidden capture arguments of a direct nested function
    /// call. Written arguments that keep a borrow into the call stay active,
    /// so passing a captured binding with a conflicting access is rejected;
    /// an argument that only copied a value out of it is not.
    #[inline(never)]
    fn named_capture_arguments(
        &mut self,
        context: &mut Context,
        captures: &[(usize, Mode)],
        args: &[Expr],
        params: &[ArgumentContract],
        span: Span,
        loans: &Loans,
    ) -> Result<(), Diagnostic> {
        let mut active = loans.clone();
        for (arg, contract) in args.iter().zip(params) {
            if !arg.ty.is_some_and(Type::is_copy) {
                for binding in Self::loan_roots(arg, contract.mode) {
                    active.entry(binding).or_default().push((contract.mode, arg.span));
                }
            }
        }
        for &(slot, mode) in captures {
            Self::access(context, slot, mode, span, &active)?;
            if mode == Mode::Mutate {
                context.bindings[slot].mutable = true;
            }
            if !context.bindings[slot].ty.is_copy() || mode == Mode::Mutate {
                active.entry(slot).or_default().push((mode, span));
            }
        }
        Ok(())
    }

    fn function_value(&mut self, name: &str, span: Span) -> Result<Type, Diagnostic> {
        let index = self.function_names[name];
        if name == "main" {
            return Err(Diagnostic::new(
                "unsupported",
                "The entry point cannot be used as a callable value.",
                span,
            ));
        }
        if self.named_captures.needs_context(index) {
            return Err(Diagnostic::new(
                "unsupported",
                "A nested function with a generic signature that uses variables of its enclosing function cannot be used as a value yet. Call it from an arrow function, for example `(value) => name(value)`.",
                span,
            ));
        }
        let higher = self.signatures[index]
            .params
            .iter()
            .any(|ty| matches!(ty, Type::Callback(_)));
        // Discovery may know a factory's concrete result before checking its
        // body. An unresolved shape must not become a forwarding environment.
        if !higher
            && self.discovering_results
            && self.callable_result_type(self.signatures[index].result)
        {
            if !self.known_results.contains_key(&index) {
                self.result_waits.insert(index, span);
                return Err(Self::pending_result(span));
            }
        } else {
            self.function(index)?;
        }
        let result = if higher {
            self.signatures[index].result
        } else {
            self.checked[index]
                .as_ref()
                .map(|function| function.result)
                .or_else(|| self.known_results.get(&index).copied())
                .unwrap_or(self.signatures[index].result)
        };
        if !higher && (matches!(result, Type::Callable(_)) || self.has_pending_result(result)) {
            return Err(Self::pending_result(span));
        }
        let bindings = if let Some(function) = &self.checked[index] {
            function.bindings[..function.parameter_count].iter().cloned().map(|mut binding| {
                binding.ty = binding.input_ty.unwrap_or(binding.ty);
                binding.input_ty = None;
                binding.initializer = None;
                binding.declaration_span = None;
                binding
            }).collect()
        } else {
            let modes = &self
                .assumed_modes
                .as_ref()
                .expect("recursive access summaries")[index];
            self.program.functions[index]
                .params
                .iter()
                .zip(&self.signatures[index].params)
                .zip(modes)
                .map(|(((name, _, _), &ty), &mode)| Binding { rest: false,
                    name: name.text.clone(),
                    ty,
                    parameter: true,
                    input_ty: None,
                    declaration_span: None,
                    initializer: None,
                    rebindable: false,
                    capture_reference: false,
                    borrowed_element: None,
                    borrowed_place: None,
                    view_roots: Vec::new(),
                    callback_invocation: None,
                    callback_reference: false,
                    callback_kind: None,
                    mode,
                    mutable: false,
                    transferred: None,
                    moved_fields: BTreeMap::new(),
                    used: None,
                 omission: crate::ParameterOmission::Required, })
                .collect()
        };
        let id = if let Some(&id) = self.function_values.get(&index) {
            // A provisional recursive/discovery reference uses the same native
            // origin once the target's checked bindings and result are ready.
            self.closures[id].function.bindings = bindings;
            self.closures[id].function.result = result;
            id
        } else {
            let id = self.closures.len();
            self.closures.push(CheckedClosure {
                polymorphic: false,
                methods: Vec::new(),
                forward: Some(ClosureForward::Function(index)),
                borrowed: false,
                lifetime_count: 0,
                depth: 0,
                parameters: (0..bindings.len()).collect(),
                function: CheckedFunction {
                    template: false,
                    specialization: false,
                    name: name.into(),
                    bindings,
                    parameter_count: 0,
                    result,
                    body: Vec::new(),
                },
                captures: Vec::new(),
                span,
                escapes: true,
                invocation: Mode::Read,
            });
            self.function_values.insert(index, id);
            id
        };
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        Ok(Type::Closure(id))
    }

    fn callback_value_hint(
        &mut self,
        context: &Context,
        expression: &Expr,
    ) -> Result<Option<Type>, Diagnostic> {
        if let ExprKind::Name(name) = &expression.kind
            && !context.names.contains_key(name)
            && let Some((id, _)) = crate::native_bindings::by_symbol(name)
        {
            return self
                .native_function_value(id, expression.span, None)
                .map(Some);
        }
        if let ExprKind::Name(name) = &expression.kind
            && !context.names.contains_key(name)
            && !context.pending_names.contains(name)
            && self.function_names.contains_key(name)
        {
            return self.function_value(name, expression.span).map(Some);
        }
        if matches!(expression.kind, ExprKind::Unary { op: UnaryOp::NonNull, .. }) {
            return Ok(self.expression_type_hint(context, expression));
        }
        if let ExprKind::Unary {
            op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected,
            value,
            ..
        } = &expression.kind
        {
            return self.callback_value_hint(context, value);
        }
        if let ExprKind::Binary {
            op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
            left,
            right,
            ..
        } = &expression.kind
        {
            let ty = self.expression_type_hint(context, left);
            return self.callback_value_hint(
                context,
                if ty.is_some_and(|ty| matches!(ty, Type::Null | Type::Undefined)) {
                    right
                } else {
                    left
                },
            );
        }
        if let ExprKind::CoalesceAssign { target, value, .. } = &expression.kind {
            let fallback = self.callback_value_hint(context, value)?;
            if fallback.is_some_and(|ty| {
                ty.union().is_none() && !matches!(ty, Type::Null | Type::Undefined)
            }) {
                return Ok(self
                    .expression_type_hint(context, target)
                    .and_then(|ty| self.unions.payload(ty).or(Some(ty))));
            }
        }
        if let ExprKind::Conditional {
            consequent,
            alternate,
            ..
        } = &expression.kind
        {
            let left = self.callback_value_hint(context, consequent)?;
            let right = self.callback_value_hint(context, alternate)?;
            return Ok(left.or(right));
        }
        Ok(self.expression_type_hint(context, expression))
    }

    fn abstract_return(&mut self, contract: usize, context: &Context, span: Span) -> usize {
        if let Some(&id) = self.abstract_returns.get(&contract) {
            if !self.discovering_results {
                self.validated_closures.insert(id);
            }
            return id;
        }
        let callback = &self.callables.callbacks[contract];
        let invocation = if callback.kind.is_once() {
            Mode::Consume
        } else {
            Mode::Mutate
        };
        let capture = Binding { rest: false,
            name: "callback".into(),
            ty: Type::Callback(contract),
            parameter: true,
            input_ty: None,
            omission: crate::ParameterOmission::Required,
            declaration_span: None,
            initializer: None,
            rebindable: false,
            capture_reference: false,
            borrowed_element: None,
            borrowed_place: None,
            view_roots: Vec::new(),
            callback_invocation: Some(invocation),
            callback_reference: false,
            callback_kind: None,
            mode: Mode::Consume,
            mutable: false,
            transferred: None,
            moved_fields: BTreeMap::new(),
            used: Some(span),
        };
        let mut bindings = vec![capture.clone()];
        bindings.extend(
            callback
                .params
                .iter()
                .enumerate()
                .map(|(index, param)| Binding { rest: param.rest,
                    omission: param.omission,
                    name: format!("argument{index}"),
                    ty: param.ty,
                    mode: param.mode,
                    callback_invocation: None,
                    used: None,
                    ..capture.clone() }),
        );
        let id = self.closures.len();
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        self.closures.push(CheckedClosure {
            polymorphic: false,
            methods: Vec::new(),
            forward: Some(ClosureForward::Abstract(contract)),
            borrowed: false,
            lifetime_count: 0,
            depth: context.closure_depth + 1,
            parameters: (1..bindings.len()).collect(),
            function: CheckedFunction {
                template: true,
                specialization: false,
                name: context.function_name.clone(),
                bindings,
                parameter_count: 0,
                result: callback.result,
                body: Vec::new(),
            },
            captures: vec![Capture {
                outer: 0,
                inner: 0,
                storage: Mode::Consume,
                access: invocation,
                span,
            }],
            span,
            escapes: true,
            invocation,
        });
        self.abstract_returns.insert(contract, id);
        id
    }

    fn once_return(&mut self, origin: usize, context: &Context, span: Span) -> usize {
        if let Some(&id) = self.once_returns.get(&origin) {
            if !self.discovering_results {
                self.validated_closures.insert(id);
            }
            return id;
        }
        let inner = &self.closures[origin];
        let mut bindings = vec![Binding { rest: false,
            name: "callback".into(),
            ty: Type::Closure(origin),
            parameter: true,
            input_ty: None,
            omission: crate::ParameterOmission::Required,
            declaration_span: None,
            initializer: None,
            rebindable: false,
            capture_reference: false,
            borrowed_element: None,
            borrowed_place: None,
            view_roots: Vec::new(),
            callback_invocation: Some(Mode::Consume),
            callback_reference: false,
            callback_kind: None,
            mode: Mode::Consume,
            mutable: false,
            transferred: None,
            moved_fields: BTreeMap::new(),
            used: Some(span),
        }];
        bindings.extend(
            inner
                .parameters
                .iter()
                .map(|&id| inner.function.bindings[id].clone()),
        );
        let id = self.closures.len();
        let wrapper = CheckedClosure {
            polymorphic: inner.polymorphic,
            methods: Vec::new(),
            forward: Some(ClosureForward::Once(origin)),
            borrowed: inner.borrowed,
            lifetime_count: inner.lifetime_count,
            depth: context.closure_depth + 1,
            parameters: (1..bindings.len()).collect(),
            function: CheckedFunction {
                template: inner.function.template,
                specialization: false,
                name: context.function_name.clone(),
                bindings,
                parameter_count: 0,
                result: inner.function.result,
                // The emitter forwards directly through OnceCallback, without
                // duplicating or rechecking the original closure body.
                body: Vec::new(),
            },
            captures: vec![Capture {
                outer: 0,
                inner: 0,
                storage: Mode::Consume,
                access: Mode::Consume,
                span,
            }],
            span,
            escapes: true,
            invocation: Mode::Consume,
        };
        self.closures.push(wrapper);
        self.once_returns.insert(origin, id);
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        id
    }

    fn resolve_value_annotation(
        &self,
        annotation: &crate::type_syntax::ValueAnnotation,
        allow_void: bool,
    ) -> Result<Type, Diagnostic> {
        resolve_value_type(annotation, allow_void, &self.record_names, &self.unions)
    }

    fn type_name(&self, ty: Type) -> std::borrow::Cow<'_, str> {
        let name = match ty {
            Type::Int => "int",
            Type::Number => "number",
            Type::Boolean => "boolean",
            Type::String => "string",
            Type::Void => "void",
            Type::Null => "null",
            Type::Undefined => "undefined",
            Type::Closure(_) | Type::Callable(_) | Type::Callback(_) | Type::StoredCallable(_) => {
                "closure"
            }
            Type::Record(i) | Type::Enum(i, _) | Type::Set(i) | Type::Map(i) => &self.program.records[i].name.text,
            Type::Union(id) | Type::BorrowedUnion(id) => {
                return self
                    .unions
                    .members(id)
                    .into_iter()
                    .map(|member| self.type_name(member).into_owned())
                    .collect::<Vec<_>>()
                    .join(" | ")
                    .into();
            }
            Type::Array(element) => {
                let leaf = match element {
                    ArrayElement::Int(_) => "int",
                    ArrayElement::Number(_) => "number",
                    ArrayElement::Boolean(_) => "boolean",
                    ArrayElement::Null(_) => "null",
                    ArrayElement::Undefined(_) => "undefined",
                    ArrayElement::String(_) => "string",
                    ArrayElement::Record(id, _) | ArrayElement::Enum(id, _, _) | ArrayElement::Set(id, _) | ArrayElement::Map(id, _) => &self.program.records[id].name.text,
                    ArrayElement::StoredCallable(_, _) => "closure",
                    ArrayElement::Union(id, _) => {
                        return format!(
                            "({}){}",
                            self.type_name(Type::Union(id)),
                            "[]".repeat(usize::from(element.depth()) + 1)
                        )
                        .into();
                    }
                };
                return format!("{leaf}{}", "[]".repeat(usize::from(element.depth()) + 1)).into();
            }
        };
        name.into()
    }

    fn require(&self, actual: Type, expected: Option<Type>, span: Span) -> Result<(), Diagnostic> {
        if !self.deferred_generic_values.is_empty() {
            self.deferred_generic_requirement(actual, expected, span)?;
        }
        if expected.is_some_and(|ty| self.unions.accepts(ty.owning(), actual.owning())) {
            return Ok(());
        }
        if let Some(expected) = expected {
            if let Some(id) = expected.union() {
                let members = self.unions.members(id);
                let alternatives = actual
                    .union()
                    .map(|id| self.unions.members(id))
                    .unwrap_or_else(|| vec![actual]);
                if let Some(payload) = self.unions.payload(expected) {
                    if matches!(payload, Type::Callback(_) | Type::Callable(_)) {
                        for member in alternatives {
                            if matches!(member, Type::Null | Type::Undefined) {
                                if !members.contains(&member) {
                                    break;
                                }
                            } else {
                                self.require(member, Some(payload), span)?;
                            }
                        }
                        if actual
                            .union()
                            .map(|id| self.unions.members(id))
                            .unwrap_or_else(|| vec![actual])
                            .iter()
                            .all(|member| {
                                !matches!(member, Type::Null | Type::Undefined)
                                    || members.contains(member)
                            })
                        {
                            return Ok(());
                        }
                    }
                }
            }
            let contracts = crate::callable_compatibility::Contracts {
                storage: &self.callables.storage,
                signatures: &self.callables.shapes,
                callbacks: &self.callables.callbacks,
                results: &self.callables.results,
                closures: &self.closures,
                effects_only: self.effects_only,
            };
            if let Some(signature) = contracts.matching(actual, expected, span)? {
                self.require_parameter_source(signature.actual, signature.expected, span)?;
                return self.require(signature.result, Some(signature.expected_result), span);
            }
        }
        if let Some(expected) = expected
            && actual != expected
        {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Expected {}, found {}.",
                    self.type_name(expected),
                    self.type_name(actual)
                ),
                span,
            ));
        }
        Ok(())
    }

    fn run(mut self) -> Result<CheckedProgram, Diagnostic> {
        let main = *self.function_names.get("main").ok_or_else(|| {
            Diagnostic::new(
                "entry",
                "Define function main() with no parameters or return value.",
                Span::default(),
            )
        })?;
        if !self.signatures[main].params.is_empty() || self.signatures[main].result != Type::Void {
            return Err(Diagnostic::new(
                "entry",
                "main() must have no parameters or return value.",
                self.program.functions[main].name.span,
            ));
        }
        if self.assumed_modes.is_some() {
            self.discover_results()?;
        }
        let mut index = 0;
        while index < self.program.functions.len() {
            if self.checked[index].is_none() && self.capturing_function_is_unused(index)? {
                index += 1;
                continue;
            }
            self.function(index)?;
            index += 1;
        }
        if self.assumed_modes.is_some()
            && !self.effects_only
            && self.validated_closures.len() != self.closures.len()
        {
            let mut ids = vec![None; self.closures.len()];
            let mut next = 0;
            for (id, mapped) in ids.iter_mut().enumerate() {
                if self.validated_closures.contains(&id) {
                    *mapped = Some(next);
                    next += 1;
                }
            }
            self.callables.storage.remap_origins(&ids)?;
            self.unions.remap_closure_origins(&ids);
            self.closures = self
                .closures
                .into_iter()
                .enumerate()
                .filter_map(|(id, closure)| ids[id].map(|_| closure))
                .collect();
            for closure in &mut self.closures {
                // Symbolic template trees preserve inspectable source contracts,
                // like named templates below; no native operation reads them.
                if closure.function.template {
                    continue;
                }
                if let Some(ClosureForward::Once(origin)) = &mut closure.forward {
                    *origin = ids[*origin].ok_or_else(|| {
                        Diagnostic::new(
                            "unsupported",
                            "Callable result wrapper retains an unresolved origin.",
                            closure.span,
                        )
                    })?;
                }
                remap_function_closures(&mut closure.function, &ids)?;
                for method in &mut closure.methods {
                    if !method.function.template {
                        remap_function_closures(&mut method.function, &ids)?;
                    }
                }
            }
            for function in self.checked.iter_mut().flatten() {
                if !function.template {
                    remap_function_closures(function, &ids)?;
                }
            }
        }
        if !self.effects_only
            && (!self.closures.iter().all(|c| c.methods.is_empty())
                || self.recursive_callbacks.iter().any(|&id| {
                    let function = self.checked[id]
                        .as_ref()
                        .expect("checked recursive function");
                    !function.template
                        && function.bindings[..function.parameter_count]
                            .iter()
                            .any(|binding| matches!(binding.ty, Type::Callback(_)))
                }))
        {
            let functions = self
                .checked
                .iter()
                .map(|f| f.as_ref().expect("checked function"))
                .collect::<Vec<_>>();
            validate_callback_instances(
                &functions,
                &self.closures,
                &self.recursive_callbacks,
                &self.program.functions,
            )?;
        }
        let deferred_generic_nominals = self.deferred_generic_nominal_uses();
        let checked = CheckedProgram {
            named_capture_layouts: Vec::new(),
            named_capture_access: self.named_captures.summaries(&self.checked),
            local_capture_plans: self.program.local_capture_plans,
            local_declarations: self.program.local_declarations,
            function_specializations:self.specializations.iter().map(|((origin,_),instance)|(*instance,*origin)).collect(),
            error_effects: Default::default(),
            stored_contracts: self.callables.storage,
            callable_shapes: self.callables.shapes,
            generic_parameter_types: self
                .program
                .functions
                .iter()
                .enumerate()
                .filter_map(|(id, function)| {
                    function.application.as_ref()?;
                    let declaration = self.function_names.get(&function.name.text)?;
                    Some((id, self.signatures[*declaration].params.clone()))
                })
                .collect(),
            deferred_generic_nominals,
            deferred_generic_values: self.deferred_generic_values.keys().copied().collect(),
            deferred_generic_sources: self.deferred_generic_values.iter().map(|(origin, source)| (*origin, self.generic_functions.template(&source.declaration).expect("deferred generic source template").name.clone())).collect(),
            generic_declarations: self
                .program
                .functions
                .iter()
                .enumerate()
                .filter_map(|(id, function)| {
                    function
                        .application
                        .as_ref()
                        .map(|application| (id, application.declaration.clone()))
                })
                .collect(),
            unions: self.unions.snapshot(),
            records: self.program.records,
            closures: self.closures,
            callback_contracts: self.callables.callbacks,
            absent_callback_payloads: self.absent_callback_payloads.values().copied().collect(),
            functions: self
                .checked
                .into_iter()
                .map(|f| f.expect("all functions analyzed"))
                .collect(),
        };
        crate::callable_layout::validate_storage_layout(&checked)?;
        crate::callable_size::validate(&checked)?;
        Ok(checked)
    }

    fn absent_callback(&mut self, id: usize) -> Type {
        if let Some(&empty) = self.absent_callback_payloads.get(&id) {
            return Type::Callback(empty);
        }
        let c = &self.callables.callbacks[id];
        let contract = CallbackContract {
            structural: c.structural,
            parameter_names: c.parameter_names.clone(),
            params: c.params.clone(),
            result: c.result,
            kind: c.kind,
        };
        let result = if let Type::Callable(result) = contract.result {
            let c = &self.callables.results[result];
            let shape = crate::callable_resolution::ResultContract {
                structural: c.structural,
                params: c.params.clone(),
                result: c.result,
            };
            let source = if let Some(source) = self
                .callables
                .callbacks
                .iter()
                .position(|c| c.structural == shape.structural)
            {
                source
            } else {
                let source = self.callables.callbacks.len();
                let kind = self
                    .callables
                    .shapes
                    .get(shape.structural)
                    .kind
                    .unwrap_or(CallbackKind::Owned);
                self.callables.callbacks.push(CallbackContract {
                    structural: shape.structural,
                    parameter_names: (0..shape.params.len())
                        .map(|i| format!("argument{i}"))
                        .collect(),
                    params: shape.params,
                    result: shape.result,
                    kind,
                });
                source
            };
            self.absent_callback(source)
        } else {
            contract.result
        };
        let empty = self.callables.callbacks.len();
        self.callables.callbacks.push(CallbackContract {
            structural: contract.structural,
            parameter_names: contract.parameter_names,
            params: contract.params,
            result,
            kind: contract.kind,
        });
        self.absent_callback_payloads.insert(id, empty);
        Type::Callback(empty)
    }

    fn absent_callable_type(&self, ty: Type) -> bool {
        matches!(self.unions.payload(ty).unwrap_or(ty),Type::Callback(id) if self.absent_callback_payloads.values().any(|&empty|empty==id))
    }

    fn callable_result_type(&self, ty: Type) -> bool {
        matches!(ty, Type::Callable(_))
            || matches!(self.unions.payload(ty),Some(Type::Callback(id)) if self.callables.callbacks[id].kind.is_owned() && self.callback_needs_context(id))
    }

    fn callable_result_shape(&mut self, ty: Type) -> Type {
        if let Type::Callable(id) = ty {
            return self.result_shape(id);
        }
        let Some(Type::Callback(id)) = self.unions.payload(ty) else {
            return ty;
        };
        let c = &self.callables.callbacks[id];
        let contract = if let Some(index) = self
            .callables
            .results
            .iter()
            .position(|r| r.structural == c.structural)
        {
            index
        } else {
            let index = self.callables.results.len();
            self.callables
                .results
                .push(crate::callable_resolution::ResultContract {
                    structural: c.structural,
                    params: c.params.clone(),
                    result: c.result,
                });
            index
        };
        let shape = self.result_shape(contract);
        self.unions
            .intern(
                self.unions
                    .members(ty.union().unwrap())
                    .into_iter()
                    .map(|member| {
                        if matches!(member, Type::Callback(_)) {
                            shape
                        } else {
                            member
                        }
                    }),
                Span::default(),
            )
            .expect("same nullable result domain")
    }

    fn pending_result(span: Span) -> Diagnostic {
        Diagnostic::new(
            "callable-result-pending",
            "Callable result origin is not resolved yet.",
            span,
        )
    }

    fn has_pending_result(&self, ty: Type) -> bool {
        let mut pending = vec![ty];
        let mut visited = HashSet::new();
        while let Some(ty) = pending.pop() {
            if let Some(id) = ty.union() {
                if matches!(self.unions.payload(ty),Some(Type::Callback(callback)) if self.callables.callbacks[callback].kind.is_owned() && self.callback_needs_context(callback) && !self.absent_callable_type(ty))
                {
                    return true;
                }
                pending.extend(self.unions.members(id));
                continue;
            }
            let key = match ty {
                Type::Closure(id) => (0, id),
                Type::Callback(id) => (1, id),
                Type::Callable(id) => (2, id),
                _ => continue,
            };
            if !visited.insert(key) {
                continue;
            }
            match ty {
                Type::Closure(id) => {
                    if self.pending_shapes.contains(&id) {
                        return true;
                    }
                    let closure = &self.closures[id];
                    if let Some(ClosureForward::Once(origin)) = closure.forward {
                        pending.push(Type::Closure(origin));
                    }
                    pending.push(closure.function.result);
                    pending.extend(
                        closure
                            .parameters
                            .iter()
                            .map(|&p| closure.function.bindings[p].ty),
                    );
                    pending.extend(
                        closure
                            .captures
                            .iter()
                            .map(|c| closure.function.bindings[c.inner].ty),
                    );
                }
                Type::Callback(id) => {
                    let c = &self.callables.callbacks[id];
                    pending.push(c.result);
                    pending.extend(c.params.iter().map(|p| p.ty));
                }
                Type::Callable(id) => {
                    let c = &self.callables.results[id];
                    pending.push(c.result);
                    pending.extend(c.params.iter().map(|p| p.ty));
                }
                _ => (),
            }
        }
        false
    }

    /// Discovery uses declared callable shapes for local invocation only.
    /// A shape cannot become a return origin or an environment capture.
    fn result_shape(&mut self, contract: usize) -> Type {
        if let Some(&id) = self.result_shapes.get(&contract) {
            return Type::Closure(id);
        }
        let params = self.callables.results[contract].params.clone();
        let result = match self.callables.results[contract].result {
            Type::Callable(inner) => self.result_shape(inner),
            result => result,
        };
        let bindings = params
            .iter()
            .enumerate()
            .map(|(i, argument)| Binding { rest: false,
                name: format!("argument{i}"),
                ty: argument.ty,
                parameter: true,
                input_ty: None,
                omission: crate::ParameterOmission::Required,
                declaration_span: None,
                initializer: None,
                rebindable: false,
                capture_reference: false,
                borrowed_element: None,
                borrowed_place: None,
                view_roots: Vec::new(),
                callback_invocation: None,
                callback_reference: false,
                callback_kind: None,
                mode: argument.mode,
                mutable: false,
                transferred: None,
                moved_fields: BTreeMap::new(),
                used: None,
            })
            .collect::<Vec<_>>();
        let id = self.closures.len();
        self.closures.push(CheckedClosure {
            polymorphic: false,
            methods: Vec::new(),
            forward: None,
            borrowed: false,
            lifetime_count: 0,
            depth: 1,
            parameters: (0..bindings.len()).collect(),
            function: CheckedFunction {
                template: true,
                specialization: false,
                name: "result-shape".into(),
                bindings,
                parameter_count: 0,
                result,
                body: Vec::new(),
            },
            captures: Vec::new(),
            span: Span::default(),
            escapes: true,
            invocation: Mode::Read,
        });
        self.result_shapes.insert(contract, id);
        self.pending_shapes.insert(id);
        Type::Closure(id)
    }

    fn discovery_expression(
        &mut self,
        context: &mut Context,
        expression: &Expr,
        expected: Option<Type>,
    ) -> Result<Option<(Type, Option<usize>)>, Diagnostic> {
        self.discovery_expression_inner(context,expression,expected,false)
    }
    #[inline(never)]
    fn discovery_expression_inner(&mut self,context:&mut Context,expression:&Expr,expected:Option<Type>,discarded:bool)->Result<Option<(Type,Option<usize>)>,Diagnostic>{
        let before = context.clone();
        let depth = self.depth;
        let mut expression = expression.clone();
        let mode = if self
            .expression_type_hint(context, &expression)
            .is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_)))
        {
            Mode::Read
        } else {
            Mode::Consume
        };
        let checked=if discarded && matches!(expression.kind,ExprKind::Sequence(_)) {
            self.sequence_expression(context,&mut expression,Mode::Read,&Loans::new(),None,true)
        }else{self.expression(context,&mut expression,mode,&Loans::new(),expected)};
        match checked {
            Ok(ty) => Ok(Some((ty, expression.binding))),
            Err(error) if error.code == "callable-result-pending" => {
                *context = before;
                self.depth = depth;
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Explore source scopes for a concrete return seed without reordering
    /// executable syntax. Normal body checking validates every path afterward.
    fn discover_body(
        &mut self,
        context: &mut Context,
        body: &[Stmt],
        result: Type,
    ) -> Result<Option<Type>, Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Callable result discovery nesting exceeds the prototype limit of 128.",
                body.first().map_or(Span::default(), |s| s.span),
            ));
        }
        self.depth += 1;
        let found = self.discover_body_inner(context, body, result);
        self.depth -= 1;
        found
    }

    fn callable_result_value(
        &mut self,
        context: &Context,
        expected: Type,
        actual: Type,
        span: Span,
    ) -> Type {
        if let Type::Callable(contract) = expected
            && self
                .callables
                .shapes
                .get(self.callables.results[contract].structural)
                .kind
                == Some(CallbackKind::OwnedOnce)
            && let Type::Closure(origin) = actual
            && self.closures[origin].invocation != Mode::Consume
        {
            return Type::Closure(self.once_return(origin, context, span));
        }
        actual
    }

    #[inline(never)]
    fn materialize_discovery_runtime_error(&mut self)->Type {
        if let Some(id)=self.program.records.iter().position(|record|record.name.kind==TokenKind::Symbol && record.name.text=="#RuntimeError") {
            return Type::Record(id);
        }
        let record=crate::error_effects::runtime_error_schema();
        let id=self.program.records.len();
        self.fields.push(record.fields.iter().map(|field|(field.name.text.clone(),field.ty.expect("runtime error field type"))).collect());
        self.program.records.push(record);
        Type::Record(id)
    }

    #[inline(never)]
    fn discover_if_result(&mut self,context:&Context,condition:&Expr,then_body:&[Stmt],else_body:&[Stmt],result:Type)->Result<Option<Type>,Diagnostic> {
        let mut probe=context.clone();
        if let Some((actual,_))=self.discovery_expression(&mut probe,condition,Some(Type::Boolean))? {
            self.require(actual,Some(Type::Boolean),condition.span)?;
        }
        let (then_flow,else_flow)=Self::condition_outcomes(&probe,condition);
        let mut absent=None;
        for (body,flow) in [(then_body,then_flow),(else_body,else_flow)] {
            let mut inner=probe.clone();
            inner.flow=flow;
            if let Some(ty)=self.discover_body(&mut inner,body,result)? {
                if self.absent_callable_type(ty) {absent=Some(ty);}else{return Ok(Some(ty));}
            }
        }
        Ok(absent)
    }

    #[inline(never)]
    fn discover_checked_exception_result(&mut self,context:&Context,region:&ExceptionRegion,result:Type,runtime:Option<Type>)->Result<Option<Type>,Diagnostic> {
        let mut checked_body=region.body.clone();
        let mut inner=context.clone();
        let preliminary=self.discovery_error_effects();
        self.with_exception_environment_mode(&mut inner,&preliminary,runtime,true,true,|analyzer,context|
            analyzer.check_region_body(context,&mut checked_body,result))?;
        let effects=self.discovery_error_effects();
        let incoming=crate::error_effects::annotate_region_body(&mut checked_body,&effects);
        let runtime=if incoming.runtime && !incoming.opaque && region.catch.as_ref().is_some_and(|handler|handler.name.is_some()) {
            Some(runtime.unwrap_or_else(||self.materialize_discovery_runtime_error()))
        }else{runtime};
        let mut source=region.clone();
        if let Some(handler)=&mut source.catch {
            handler.binding=None;
        handler.error_type=if let Some(name)=&handler.name {incoming.catch_type(&self.unions,runtime,name.span)?}else{None};
            handler.error_contract=Some(Box::new(incoming));
        }
        self.discover_exception_result(context,&source,result,Span::default())
    }

    #[inline(never)]
    fn discover_exception_result(&mut self,context:&Context,region:&ExceptionRegion,result:Type,_span:Span)->Result<Option<Type>,Diagnostic> {
        let cleanup=if let Some(body)=&region.finally {
            let found=self.discover_body(&mut context.clone(),body,result)?;
            if crate::completion::definitely_abrupt(body,0) {return Ok(found);}
            found
        }else{None};
        if region.catch.as_ref().is_some_and(|handler|handler.name.is_some() && handler.error_type.is_none() && handler.error_contract.is_none()) {
            return self.discover_checked_exception_result(context,region,result,None);
        }
        let body=self.discover_body(&mut context.clone(),&region.body,result)?;
        let handler=if let Some(handler)=&region.catch
            && handler.error_contract.as_ref().is_none_or(|contract|contract.runtime || contract.opaque || !contract.explicit.is_empty()) {
            let mut inner=context.clone();
            if let Some(name)=&handler.name {
                let ty=handler.error_type.ok_or_else(||Diagnostic::new("error-effect","Catch result discovery requires a resolved payload type.",name.span))?;
                inner.names.remove(&name.text);
                Self::bind(&mut inner,name,ty,false)?;
            }
            self.discover_body(&mut inner,&handler.body,result)?
        }else{None};
        let candidates=[body,handler,cleanup];
        Ok(candidates.into_iter().flatten().find(|ty|!self.absent_callable_type(*ty))
            .or_else(||candidates.into_iter().flatten().next()))
    }

    #[inline(never)]
    fn discover_throw_value(&mut self,context:&mut Context,value:&Expr)->Result<(),Diagnostic> {
        let expected=self.expression_type_hint(context,value).filter(|ty|matches!(ty,Type::Closure(_)));
        if let Some((Type::Void,_))=self.discovery_expression(context,value,expected)? {
            return Err(Diagnostic::new("type","A thrown expression must produce a value.",value.span));
        }
        Ok(())
    }

    fn discover_body_inner(
        &mut self,
        context: &mut Context,
        body: &[Stmt],
        result: Type,
    ) -> Result<Option<Type>, Diagnostic> {
        let mut absent = None;
        for stmt in body {
            match &stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Try(region) => {
                    if let Some(ty)=self.discover_exception_result(context,region,result,stmt.span)? {
                        if self.absent_callable_type(ty) {absent=Some(ty);}else{return Ok(Some(ty));}
                    }
                    if crate::completion::definitely_abrupt(std::slice::from_ref(stmt),0) {return Ok(absent);}
                }
                StmtKind::Throw(value) => {
                    self.discover_throw_value(context,value)?;
                    return Ok(absent);
                }
                StmtKind::Return(Some(value)) => {
                    let previous_permission=context.returning_closure;
                    context.returning_closure = true;
                    let value = self.discovery_expression(context, value, Some(result));
                    context.returning_closure = previous_permission;
                    let value=value?;
                    if let Some((actual, _)) = value {
                        let actual = self.callable_result_value(context, result, actual, stmt.span);
                        self.require(actual, Some(result), stmt.span)?;
                        if !self.has_pending_result(actual) {
                            if self.absent_callable_type(actual) {
                                absent = Some(actual);
                            } else {
                                return Ok(Some(actual));
                            }
                        }
                    }
                }
                StmtKind::Let {
                    name,
                    annotation,
                    value,
                    rebindable,
                    ..
                } => {
                    let expected = annotation
                        .as_ref()
                        .map(|a| self.resolve_value_annotation(a, false))
                        .transpose()?
                        .or_else(|| match &value.kind {
                            ExprKind::Name(name) => context
                                .names
                                .get(name)
                                .map(|&id| context.bindings[id].ty)
                                .filter(|ty| matches!(ty, Type::Callback(_) | Type::Closure(_))),
                            _ => None,
                        });
                    if let Some((ty, source)) =
                        self.discovery_expression(context, value, expected)?
                    {
                        let callback = source.map(|id| {
                            (
                                context.bindings[id].callback_invocation,
                                context.bindings[id].callback_reference,
                                context.bindings[id].callback_kind,
                            )
                        });
                        let id = Self::bind(context, name, ty, false)?;
                        context.bindings[id].rebindable = *rebindable;
                        if let Some((invocation, reference, kind)) = callback {
                            context.bindings[id].callback_invocation = invocation;
                            context.bindings[id].callback_reference = reference;
                            context.bindings[id].callback_kind = kind;
                        }
                    } else {
                        context.pending_names.insert(name.text.clone());
                    }
                }
                StmtKind::If { condition,then_body,else_body } => {
                    if let Some(ty)=self.discover_if_result(context,condition,then_body,else_body,result)? {
                        if self.absent_callable_type(ty) {absent=Some(ty);}else{return Ok(Some(ty));}
                    }
                }
                StmtKind::While { body, .. } | StmtKind::Block(body) => {
                    if let Some(ty) = self.discover_body(&mut context.clone(), body, result)? {
                        if self.absent_callable_type(ty) {
                            absent = Some(ty);
                        } else {
                            return Ok(Some(ty));
                        }
                    }
                }
                StmtKind::Loop { body, update, .. } => {
                    let mut inner = context.clone();
                    if let Some(ty) = self.discover_body(&mut inner, body, result)? {
                        if self.absent_callable_type(ty) {
                            absent = Some(ty);
                        } else {
                            return Ok(Some(ty));
                        }
                    }
                    if let Some(ty) = self.discover_body(&mut context.clone(), update, result)? {
                        if self.absent_callable_type(ty) {
                            absent = Some(ty);
                        } else {
                            return Ok(Some(ty));
                        }
                    }
                }
                StmtKind::ForOf {
                    name,
                    rebindable,
                    iterable,
                    body,
                    ..
                } => {
                    let mut inner = context.clone();
                    if let Some((actual, _)) = self.discovery_expression(&mut inner, iterable, None)?
                        && let Some(element) = self.iterable_element(actual)
                    {
                        let id = Self::bind(&mut inner, name, element.ty(), false)?;
                        inner.bindings[id].rebindable = *rebindable;
                        if !element.ty().is_copy() {
                            inner.bindings[id].borrowed_element = Some(iterable.span);
                        }
                        if let Some(ty) = self.discover_body(&mut inner, body, result)? {
                            if self.absent_callable_type(ty) {
                                absent = Some(ty);
                            } else {
                                return Ok(Some(ty));
                            }
                        }
                    }
                }
                StmtKind::Assign {
                    target,
                    value,
                    operator,
                } => {
                    if let Some((op, span)) = *operator {
                        let depth = self.depth;
                        let before = context.clone();
                        let mut target = target.clone();
                        let mut value = value.clone();
                        if let Err(error) =
                            self.compound_assign(context, &mut target, &mut value, op, span)
                        {
                            if error.code != "callable-result-pending" {
                                return Err(error);
                            }
                            *context = before;
                            self.depth = depth;
                        }
                        continue;
                    }
                    if matches!(
                        self.expression_type_hint(context, target),
                        Some(Type::Int | Type::Number | Type::Boolean)
                    ) && !crate::flow::place(target, &context.names)
                        .and_then(|place| self.place_type(context, &place))
                        .is_some_and(|ty| matches!(ty, Type::Union(_)))
                    {
                        self.discover_scalar_assignment(context, target, value)?;
                        continue;
                    }
                    if let ExprKind::Name(name) = &target.kind {
                        let depth = self.depth;
                        let before = context.clone();
                        let mut target = target.clone();
                        let mut value = value.clone();
                        if context.names.contains_key(name)
                            && let Err(error) = self.reassign(context, &mut target, &mut value)
                        {
                            if error.code != "callable-result-pending" {
                                return Err(error);
                            }
                            *context = before;
                            self.depth = depth;
                        }
                    }
                }
                StmtKind::Expr(value) => {
                    self.discovery_expression_inner(context, value, None,true)?;
                }
                StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => (),
            }
        }
        Ok(absent)
    }

    fn discover_results(&mut self) -> Result<(), Diagnostic> {
        let effects = self.effects_only;
        self.effects_only = true;
        self.discovering_results = true;
        loop {
            let mut changed = false;
            let mut index = 0;
            while index < self.program.functions.len() {
                if self.callable_result_type(self.signatures[index].result)
                    && !self.known_results.contains_key(&index)
                {
                    let mut body = self.program.functions[index].body.clone();
                    let mut context = self.function_context(index, &mut body)?;
                    if let Some(ty) =
                        self.discover_body(&mut context, Self::uninitialized_body(&mut body), self.signatures[index].result)?
                    {
                        self.known_results.insert(index, ty);
                        changed = true;
                    }
                }
                index += 1;
            }
            if !changed {
                break;
            }
        }
        self.discovering_results = false;
        self.effects_only = effects;
        for (index, signature) in self.signatures.iter().enumerate() {
            if self.callable_result_type(signature.result)
                && !self.known_results.contains_key(&index)
                && !self.templates.contains_key(&index)
            {
                let mut error = Diagnostic::new(
                    "unsupported",
                    "Recursive callable results need one resolvable environment origin; recursive environment storage is unsupported.",
                    self.program.functions[index].name.span,
                );
                if let Some(&span) = self.result_waits.get(&index) {
                    error = error.related(
                        span,
                        "This callable result has no resolved concrete return origin.",
                    );
                }
                return Err(error);
            }
        }
        Ok(())
    }

    fn check_parameter_default(
        &mut self,
        context: &mut Context,
        initializer: &ParameterInitializer,
        expected: Type,
        data_mode: Mode,
    ) -> Result<Option<Box<Expr>>, Diagnostic> {
        let ParameterInitializer::Default { value, .. } = initializer else { return Ok(None); };
        let mut value = value.clone();
        let prior = context.bindings.iter().map(Availability::of).collect::<Vec<_>>();
        let supplied_flow = context.flow.clone();
        let kind = match self.unions.payload(expected).unwrap_or(expected) { Type::Callback(id) => Some(self.callables.callbacks[id].kind), _ => None };
        let mode = if kind == Some(CallbackKind::Read) { Mode::Read } else if kind == Some(CallbackKind::Reusable) { Mode::Mutate } else if kind.is_some() {Mode::Consume} else {data_mode};
        let mode = if self.expression_type_hint(context,&value).is_some_and(|ty|matches!(ty,Type::Null|Type::Undefined)) {Mode::Copy} else {mode};
        let actual = if mode==Mode::Mutate && expected.union().is_some() {
            if matches!(value.kind,ExprKind::Name(_)|ExprKind::Field(..)|ExprKind::Index(..)) {
                self.parameter_argument(context,&mut value,ArgumentContract{ rest: false,ty:expected,mode,omission:crate::ParameterOmission::Defaulted},&Loans::new())?
            } else {self.expression(context,&mut value,Mode::Consume,&Loans::new(),Some(expected))?}
        } else if mode==Mode::Read && expected.union().is_some() {
            if matches!(value.kind,ExprKind::Name(_)|ExprKind::Field(..)|ExprKind::Index(..)) {
                self.parameter_argument(context,&mut value,ArgumentContract{ rest: false,ty:expected,mode,omission:crate::ParameterOmission::Defaulted},&Loans::new())?
            } else {self.expression(context,&mut value,Mode::Consume,&Loans::new(),Some(expected))?}
        } else {self.expression(context, &mut value, mode, &Loans::new(), Some(expected))?};
        self.require(actual, Some(expected), value.span)?;
        for (binding, supplied) in context.bindings.iter_mut().zip(prior) {
            supplied.join(Availability::of(binding)).restore(binding);
        }
        context.flow = crate::flow::join(&supplied_flow, &context.flow);
        Ok(Some(value))
    }

    fn owning_default_family(&mut self, input: Type, body: Type, initializer: &mut Option<Box<Expr>>, span: Span) -> Result<Type,Diagnostic> {
        let Type::Callback(callback)=self.unions.payload(body).unwrap_or(body) else {return Ok(body)};
        if !self.callables.callbacks[callback].kind.is_owned() {return Ok(body)}
        let Some(value)=initializer.as_mut() else {return Ok(body)};
        let Some(payload)=self.unions.payload(input) else {return Ok(body)};
        if !matches!(payload,Type::Closure(_)) && !self.absent_callable_type(payload) {return Ok(body)}
        let source=if let ExprKind::Inject{value,..}=&value.kind {value.as_ref()}else{value.as_ref()};
        let default=self.unions.payload(source.ty.expect("checked default")).unwrap_or(source.ty.unwrap());
        if !matches!(default,Type::Closure(_)|Type::Null|Type::Undefined) && !self.absent_callable_type(default) {return Ok(body)}
        let family=self.callables.storage.intern(&self.callables.shapes,&self.callables.callbacks[callback],callback).expect("owning default contract");
        let Type::StoredCallable(id)=family else {unreachable!()};
        for origin in [payload,default] {
            if let Type::Closure(origin)=origin {self.callables.storage.register_origin(id,origin,&self.callables.callbacks,&self.callables.shapes,&self.closures,span)?;}
        }
        let selected=if let Some(union)=body.union(){self.unions.intern(self.unions.members(union).into_iter().map(|ty|if matches!(ty,Type::Callback(_)){family}else{ty}),span)?}else{family};
        let value_span=value.span;
        let mut inner=std::mem::replace(value,Box::new(Expr::new(ExprKind::Undefined,value_span)));
        if let ExprKind::Inject{..}=&inner.kind {
            let ExprKind::Inject{value,..}=inner.kind else {unreachable!()};inner=value;
        }
        let op=if selected.union().is_some(){UnaryOp::ParameterDefaultStore}else{UnaryOp::CallableStore};
        *value=Box::new(Expr::new(ExprKind::Unary{op,value:inner,operator:value_span},value_span));
        value.ty=Some(selected);
        Ok(selected)
    }

    fn initialized_parameter_fact(&self, context: &mut Context, binding: usize) {
        let parameter=&context.bindings[binding];
        let Some(union)=parameter.ty.union() else {return};
        let Some(initializer)=parameter.initializer.as_deref() else {return};
        let value=if let ExprKind::Inject{value,..}=&initializer.kind {value.as_ref()} else {initializer};
        let Some(ty)=value.ty else {return};
        let mut members=self.unions.members(union).into_iter().filter(|ty|*ty!=Type::Undefined).collect::<Vec<_>>();
        for member in ty.union().map_or_else(||vec![ty],|id|self.unions.members(id)) {if !members.contains(&member){members.push(member);}}
        if members.len()<self.unions.members(union).len() {
            context.flow.insert(crate::flow::Place{root:binding,fields:Vec::new()},crate::flow::Fact{members,origin:None});
        }
    }

    /// Pattern declarations remain in the checked body for all source walkers,
    /// but are initialized alongside their parameter rather than with the body.
    #[inline(never)]
    fn initialize_parameter_pattern(&mut self, context: &mut Context, parameter: &Token, body: &mut [Stmt], owned: bool) -> Result<(), Diagnostic> {
        if !parameter.text.starts_with("pattern#") { return Ok(()); }
        let start=body.iter().position(|stmt| stmt.span.start>=parameter.span.start && stmt.span.end<=parameter.span.end);
        let Some(start)=start else { return Ok(()); };
        let count=body[start..].iter().take_while(|stmt| stmt.span.start>=parameter.span.start && stmt.span.end<=parameter.span.end).count();
        if owned && context.names.get(&parameter.text).is_some_and(|&binding| matches!(context.bindings[binding].ty, Type::Array(_) | Type::Record(_))) {
            owned_array_patterns::lower(&mut body[start..start+count], &parameter.text, context.bindings[context.names[&parameter.text]].ty, &self.fields);
        }
        self.block_with_scope(context, &mut body[start..start+count], Type::Void, true)?;
        Ok(())
    }

    fn uninitialized_body(body: &mut [Stmt]) -> &mut [Stmt] {
        let prefix=body.iter().take_while(|stmt| matches!(stmt.kind,StmtKind::Let{binding:Some(_),..})).count();
        &mut body[prefix..]
    }

    fn function_context(&mut self, index: usize, body: &mut [Stmt]) -> Result<Context, Diagnostic> {
        let mut context = Context::new(self.program.functions[index].name.text.clone());
        context.effects_only = self.effects_only;
        context.template = self.templates.contains_key(&index);
        let parameters = self.program.functions[index].params.clone();
        let initializers = self.program.functions[index].parameter_initializers.clone();
        let signature = self.signatures[index].clone();
        // Reserve the ordinary contiguous parameter ABI before defaults create
        // any local temporaries; expose names only after their initialization.
        for (position, (param, _, _)) in parameters.iter().enumerate() {
            let body_ty = if matches!(initializers[position], ParameterInitializer::Optional(_)) { signature.params[position] } else { signature.body_params[position] };
            Self::bind(&mut context, param, body_ty, true)?;
        }
        context.names.clear();
        self.named_captures.attach(index, &mut context);
        for (position, ((param, _, annotation), input_ty)) in parameters.iter().zip(&signature.params).enumerate() {
            let initializer = &initializers[position];
            let binding = position;
            let mut body_ty = context.bindings[binding].ty;
            if self.assumed_modes.is_none() && matches!(initializer,ParameterInitializer::Default{..})
                && !body_ty.is_copy() && !matches!(self.unions.payload(body_ty).unwrap_or(body_ty),Type::Callback(_)|Type::Closure(_)|Type::StoredCallable(_)) {
                return Err(Diagnostic::new("recursion-inference","Default owner access requires initialized parameter access inference.",param.span));
            }
            let default_mode=self.assumed_modes.as_ref().and_then(|modes|modes.get(index)).and_then(|modes|modes.get(position)).copied().unwrap_or(Mode::Consume);
            let mut checked_initializer = self.check_parameter_default(&mut context, initializer, body_ty,default_mode)?;
            body_ty=self.owning_default_family(*input_ty,body_ty,&mut checked_initializer,param.span)?;
            context.bindings[binding].ty=body_ty;
            context.names.insert(param.text.clone(), binding);
            let ty = &body_ty;
            context.bindings[binding].input_ty = Some(*input_ty);
            context.bindings[binding].omission = initializer.omission();
            context.bindings[binding].rest=self.program.functions[index].rest_parameter && position+1==parameters.len();
            if context.bindings[binding].rest {
                if !matches!(body_ty,Type::Array(_)) {return Err(Diagnostic::new("rest-type","A rest parameter requires a supported homogeneous array type.",param.span));}
                context.bindings[binding].mode=Mode::Consume;
            }
            context.bindings[binding].declaration_span = Some(param.span);
            context.bindings[binding].initializer = checked_initializer;
            if matches!(default_mode,Mode::Read|Mode::Mutate)
                && let Some(value)=context.bindings[binding].initializer.as_deref()
                && let Some(source)=Self::root_binding(value)
                && source!=binding {
                // Pattern locals follow reserved argument IDs; visibility, not
                // numeric binding order, proves the source is initialized.
                context.bindings[binding].view_roots=Self::view_sources(&context,value);
            }
            self.initialized_parameter_fact(&mut context,binding);
            let source_kind = crate::callable_annotations::nullable_kind(
                &self.program.functions[index].params[binding].1,
            );
            let nullable_kind = match self.unions.payload(*ty).unwrap_or(*ty) {
                Type::Callback(id) => Some(self.callables.callbacks[id].kind),
                Type::StoredCallable(id) => self
                    .callables
                    .storage
                    .declaration(id)
                    .map(|id| self.callables.callbacks[id].kind),
                _ => source_kind,
            };
            if let Some(kind) = nullable_kind {
                context.bindings[binding].callback_kind = Some(kind);
                context.bindings[binding].callback_reference =
                    matches!(kind, CallbackKind::Reusable | CallbackKind::Read);
                context.bindings[binding].callback_invocation = Some(kind.invocation());
                if kind.is_once() || kind.is_owned() {
                    context.bindings[binding].mode = Mode::Consume;
                }
            }
            if let Some(annotation) = annotation {
                context.bindings[binding].callback_kind = Some(annotation.kind);
                context.bindings[binding].callback_reference =
                    matches!(annotation.kind, CallbackKind::Reusable | CallbackKind::Read);
                context.bindings[binding].callback_invocation = Some(annotation.kind.invocation());
            }
            if annotation
                .as_ref()
                .is_some_and(|c| c.kind.is_once() || c.kind.is_owned())
            {
                // The annotation transfers ownership even when never invoked.
                context.bindings[binding].mode = Mode::Consume;
            }
            if matches!(self.program.functions[index].parameter_access[position],Some((Mode::Read,_))) && !body_ty.is_copy() {
                context.readonly_parameters.insert(binding);
            }
            self.initialize_parameter_pattern(&mut context,param,body,matches!(self.program.functions[index].parameter_access[position],Some((Mode::Consume,_))))?;
        }
        Ok(context)
    }

    fn function(&mut self, index: usize) -> Result<(), Diagnostic> {
        if self.discovering_results {
            return Ok(());
        }
        if self.checked[index].is_some() {
            return Ok(());
        }
        if self.named_captures.needs_context(index) && !self.named_captures.ready(index) {
            return Err(self.program.local_capture_plans.iter().find_map(crate::local_capture_plan::Plan::pending_diagnostic).expect("pending named capture"));
        }
        let name = self.program.functions[index].name.clone();
        if self.visiting[index] {
            if self.signatures[index].inferred_result {
                return Err(Diagnostic::new("result-inference", "A recursive function needs an explicit result type. Add a result annotation to establish the recursive checking boundary.", name.span));
            }
            let start = self
                .function_stack
                .iter()
                .position(|&id| id == index)
                .expect("active function");
            for &id in &self.function_stack[start..] {
                if self.signatures[id]
                    .params
                    .iter()
                    .any(|ty| matches!(ty, Type::Callback(_)))
                {
                    self.recursive_callbacks.insert(id);
                }
            }
            return if self.assumed_modes.is_some() {
                Ok(())
            } else {
                Err(Diagnostic::new(
                    "recursion-inference",
                    "Recursive access inference is required.",
                    name.span,
                ))
            };
        }
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Call graph depth exceeds the prototype limit of 128.",
                name.span,
            ));
        }
        if self.callable_result_type(self.signatures[index].result)
            && self.assumed_modes.is_some()
            && !self.known_results.contains_key(&index)
        {
            self.discover_results()?;
        }
        self.visiting[index] = true;
        self.function_stack.push(index);
        self.depth += 1;
        let mut body = std::mem::take(&mut self.program.functions[index].body);
        let mut context = self.function_context(index, &mut body)?;
        for (position, access) in self.program.functions[index].parameter_access.iter().enumerate() {
            if matches!(access, Some((Mode::Read, _))) && !context.bindings[position].ty.is_copy() {
                context.readonly_parameters.insert(position);
            }
        }
        // Move the body into analysis rather than clone the syntax tree.
        let mut result_type = self.signatures[index].result;
        context.inferred_body_result = self.signatures[index].inferred_result;
        let returned = self
            .check_source_callable_body(&mut context, Self::uninitialized_body(&mut body), result_type)
            .map_err(|error| self.filter_ownership_fixes(&context, error))?;
        if context.inferred_body_result {
            result_type = self.inferred_body_result(&body)?;
            self.signatures[index].result = result_type;
        }
        if result_type != Type::Void && !returned {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Function {:?} must return {}.",
                    self.program.functions[index]
                        .application
                        .as_ref()
                        .map(|application| application.declaration.text.as_str())
                        .unwrap_or(&name.text),
                    self.type_name(result_type)
                ),
                name.span,
            ));
        }
        self.finalize_nullable_locals(&mut context.bindings, &mut body)?;
        let result_type = if self.callable_result_type(result_type) {
            context.returned_callable.expect("checked callable return")
        } else {
            result_type
        };
        self.finalize_nullable_returns(&mut body, result_type)?;
        self.declared_parameter_access(index, &mut context)?;
        if self.assumed_modes.is_some() && self.callable_result_type(self.signatures[index].result)
        {
            self.known_results.insert(index, result_type);
        }
        let uncalled = self.prepare_uncalled_capturing_functions(index, &context)?;
        self.checked[index] = Some(CheckedFunction {
            template: context.template,
            specialization: index >= self.original_count,
            name: name.text,
            bindings: context.bindings,
            parameter_count: self.signatures[index].params.len(),
            result: result_type,
            body,
        });
        self.visiting[index] = false;
        self.function_stack.pop();
        self.depth -= 1;
        for nested in uncalled {
            self.function(nested)?;
        }
        Ok(())
    }

    /// A nested function that no call reached still has a body to check. Its
    /// captured variables take their types from the finished enclosing body.
    #[inline(never)]
    fn prepare_uncalled_capturing_functions(&mut self, index: usize, context: &Context) -> Result<Vec<usize>, Diagnostic> {
        self.named_captures.complete(index, context);
        let ready = self.named_captures.prepare_completed(&self.program, &self.function_names)?;
        for &nested in &ready {
            // A discovery pass can have deferred this body before its lexical
            // owner had finished. Replace that placeholder with real checking.
            self.checked[nested] = None;
        }
        Ok(ready)
    }

    fn specialize(&mut self, index: usize, args: &[Expr], span: Span) -> Result<usize, Diagnostic> {
        if !self.templates.contains_key(&index) {
            return Ok(index);
        }
        let mut signature = self.signatures[index].clone();
        let mut types = Vec::new();
        for (ty, arg) in signature.params.iter_mut().zip(args) {
            if let Some(id)=ty.union() && self.unions.payload(*ty).is_some_and(|payload|matches!(payload,Type::Callback(callback) if self.callback_needs_context(callback))) {
                let origin=match &arg.kind {ExprKind::Inject{value,..}=>value.ty.unwrap_or(arg.ty.unwrap()),_=>arg.ty.unwrap()};
                let payload=if matches!(origin,Type::Undefined|Type::Null) {
                    let Type::Callback(callback)=self.unions.payload(*ty).expect("nullable callback signature") else {unreachable!()};
                    self.absent_callback(callback)
                } else {self.unions.payload(origin).unwrap_or(origin)};
                if !matches!(payload,Type::Closure(id) if !self.closures[id].function.template) && !self.absent_callable_type(payload) {return Ok(index);}
                *ty=self.unions.intern(self.unions.members(id).into_iter().map(|member|if matches!(member,Type::Callback(_)){payload}else{member}),arg.span)?;
                types.push(*ty);
            }
            if let Type::Callback(id) = *ty
                && self.callback_needs_context(id)
            {
                let actual = arg.ty.expect("checked argument");
                if self.discovering_results && self.has_pending_result(actual) {
                    return Ok(index);
                }
                // Template calls stay abstract until their caller is specialized.
                if !matches!(actual,Type::StoredCallable(_)) && !matches!(actual, Type::Closure(id) if !self.closures[id].function.template) {
                    return Ok(index);
                }
                *ty = actual;
                types.push(actual);
            }
        }
        for (position,initializer) in self.program.functions[index].parameter_initializers.iter().enumerate() {
            if !matches!(initializer,ParameterInitializer::Default{..}) {signature.body_params[position]=signature.params[position];}
        }
        let key = (index, types);
        if let Some(&id) = self.specializations.get(&key) {
            return Ok(id);
        }
        if self.specializations.len()
            + self
                .closure_specializations
                .keys()
                .filter(|(_, _, effects, discovery)| !effects && !discovery)
                .count()
            >= 128
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Callback origin specialization count exceeds the prototype limit of 128.",
                span,
            ));
        }
        let id = self.program.functions.len();
        self.program.functions.push(self.templates[&index].clone());
        self.signatures.push(signature);
        self.checked.push(None);
        self.visiting.push(false);
        // Owned recursion reuses a cached concrete function on back edges.
        // Mixed borrowed parameters still need the per-function vector guard.
        if self.recursive_callbacks.contains(&index) {
            self.recursive_callbacks.insert(id);
        }
        self.specializations.insert(key, id);
        self.named_captures.specialized(id, index);
        // Data results already have a complete declared type. Drain these
        // concrete bodies iteratively in run(), rather than unfolding native
        // type contexts on the compiler stack. Callable results still need
        // immediate checking to resolve their concrete returned environment.
        if !self.discovering_results && self.callable_result_type(self.signatures[id].result) {
            self.function(id)?;
        }
        Ok(id)
    }

    fn check_loop_back(context: &Context) -> Result<(), Diagnostic> {
        if context.effects_only {
            return Ok(());
        }
        Self::check_loop_back_at(context, context.loops.len()-1)
    }

    fn check_loop_back_at(context: &Context, index: usize) -> Result<(), Diagnostic> {
        if context.effects_only {return Ok(());}
        let frame = &context.loops[index];
        for (place, required) in &frame.flow_required {
            if !context.flow.get(place).is_some_and(|fact| {
                fact.members
                    .iter()
                    .all(|member| required.members.contains(member))
            }) {
                return Err(Diagnostic::new(
                    "loop-narrowing",
                    "A loop back edge invalidates a nullable proof used before the next guard; guard inside each iteration.",
                    frame.span,
                ));
            }
        }
        for (id, before) in &frame.entry {
            let binding = &context.bindings[*id];
            let transfer = if before.transferred.is_none() {
                binding.transferred
            } else {
                None
            };
            let transfer = transfer.or_else(|| {
                binding.moved_fields.iter().find_map(|(path, span)| {
                    (before.transferred.is_none()
                        && !before.fields.keys().any(|prior| path.starts_with(prior)))
                    .then_some(*span)
                })
            });
            if let Some(transfer) = transfer {
                return Err(Diagnostic::new("loop-transfer", format!(
                    "Cannot transfer {:?} on a path that may start another loop iteration.", context.bindings[*id].name), transfer)
                    .related(frame.span, "This loop may need the value again; reinitialize a let binding before continuing, or transfer only on a path that breaks or returns."));
            }
        }
        Ok(())
    }

    fn while_loop(
        &mut self,
        context: &mut Context,
        condition: &mut Expr,
        body: &mut [Stmt],
        result_type: Type,
        span: Span,
        labels: &[Token],
    ) -> Result<(), Diagnostic> {
        let entry_flow = context.flow.clone();
        let mut entry = context.names.values().copied().collect::<Vec<_>>();
        entry.sort_unstable();
        let entry = entry
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let actual = self.expression(
            context,
            condition,
            Mode::Copy,
            &Loans::new(),
            Some(Type::Boolean),
        )?;
        self.require(actual, Some(Type::Boolean), condition.span)?;
        // The condition runs even when the body never does. Return paths do not
        // reach the join after the loop; break paths do. Back edges must preserve
        // entry availability. A transferred let binding must be reinitialized
        // before a continuing edge; const bindings cannot be reinitialized.
        let after_condition = entry
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let (yes, no) = Self::condition_outcomes(context, condition);
        let mut places = Vec::new();
        crate::flow::projected_places(condition, &context.names, &mut places);
        let required = places
            .into_iter()
            .filter_map(|place| entry_flow.get(&place).cloned().map(|fact| (place, fact)))
            .collect();
        context.flow = yes;
        let breaks = self.loop_body(
            context,
            body,
            result_type,
            span,
            labels,
            (entry, after_condition),
            (entry_flow, required, no),
        )?;
        if !breaks && let Some((_, no)) = self.pure_guard_outcomes(context, condition) {
            context.flow = no;
        }
        Ok(())
    }

    fn loop_body(
        &mut self,
        context: &mut Context,
        body: &mut [Stmt],
        result_type: Type,
        span: Span,
        labels: &[Token],
        availability: (Vec<(usize, Availability)>, Vec<Availability>),
        flows: (crate::flow::Facts, crate::flow::Facts, crate::flow::Facts),
    ) -> Result<bool, Diagnostic> {
        let (entry, baseline) = availability;
        let (flow_entry, flow_required, zero_flow) = flows;
        Self::validate_control_labels(context,labels)?;
        context.loops.push(LoopFrame {
            labels: labels.to_vec(),
            target: ControlTarget::Loop(span.start),
            flow_entry,
            flow_required,
            break_flow: None,
            continue_flow: None,
            breaks: vec![Availability::default(); entry.len()],
            break_seen: false,
            defer_continue: false,
            continues: None,
            entry,
            span,
        });
        let exits = self.block(context, body, result_type)?;
        if !exits {
            Self::check_loop_back(context)?;
        }
        let frame = context.loops.pop().expect("matching loop frame");
        exception_state::consume_target(context,frame.target);
        context.flow = crate::flow::join(&zero_flow, &context.flow);
        if let Some(breaks) = frame.break_flow {
            context.flow = crate::flow::join(&context.flow, &breaks);
        }
        let break_seen = frame.break_seen;
        for ((id, _), (condition_state, break_state)) in frame
            .entry
            .into_iter()
            .zip(baseline.into_iter().zip(frame.breaks))
        {
            condition_state
                .join(break_state)
                .restore(&mut context.bindings[id]);
        }
        Ok(break_seen)
    }

    #[allow(clippy::too_many_arguments)]
    fn tail_loop(
        &mut self,
        context: &mut Context,
        condition: &mut Option<Expr>,
        body: &mut [Stmt],
        update: &mut [Stmt],
        post_test: bool,
        tail_reachable: &mut bool,
        result_type: Type,
        span: Span,
        labels: &[Token],
    ) -> Result<bool, Diagnostic> {
        let initial_flow = context.flow.clone();
        let mut required = HashMap::new();
        let mut zero_flow = None;
        let mut visible = context.names.values().copied().collect::<Vec<_>>();
        visible.sort_unstable();
        let entry = visible
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        if !post_test && let Some(condition) = condition {
            let actual = self.expression(
                context,
                condition,
                Mode::Copy,
                &Loans::new(),
                Some(Type::Boolean),
            )?;
            self.require(actual, Some(Type::Boolean), condition.span)?;
            let (yes, no) = Self::condition_outcomes(context, condition);
            let mut places = Vec::new();
            crate::flow::projected_places(condition, &context.names, &mut places);
            required = places
                .into_iter()
                .filter_map(|place| initial_flow.get(&place).cloned().map(|fact| (place, fact)))
                .collect();
            context.flow = yes;
            zero_flow = Some(no);
        }
        // Only a pre-test condition supplies a zero-body exit. do...while
        // inherits its first body effects on every path that reaches its exit.
        let mut exit_state = (!post_test && condition.is_some()).then(|| {
            entry
                .iter()
                .map(|(id, _)| Availability::of(&context.bindings[*id]))
                .collect::<Vec<_>>()
        });
        Self::validate_control_labels(context,labels)?;
        context.loops.push(LoopFrame {
            labels: labels.to_vec(),
            target: ControlTarget::Tail(span.start),
            flow_entry: initial_flow,
            flow_required: required,
            break_flow: None,
            continue_flow: None,
            breaks: vec![Availability::default(); entry.len()],
            break_seen: false,
            defer_continue: true,
            continues: None,
            entry,
            span,
        });
        let body_exits = self.block(context, body, result_type)?;
        let frame = context.loops.last_mut().expect("new loop frame");
        let mut continuing = frame.continues.take();
        if !body_exits {
            let states = frame
                .entry
                .iter()
                .map(|(id, _)| Availability::of(&context.bindings[*id]))
                .collect::<Vec<_>>();
            continuing = Some(match continuing {
                Some(previous) => previous
                    .into_iter()
                    .zip(states)
                    .map(|(a, b)| a.join(b))
                    .collect(),
                None => states,
            });
        }
        *tail_reachable = continuing.is_some();
        if let Some(states) = continuing {
            if let Some(continues) = context
                .loops
                .last()
                .and_then(|frame| frame.continue_flow.clone())
            {
                context.flow = if body_exits {
                    continues
                } else {
                    crate::flow::join(&context.flow, &continues)
                };
            }
            for ((id, _), state) in context
                .loops
                .last()
                .expect("new loop frame")
                .entry
                .iter()
                .zip(states)
            {
                state.restore(&mut context.bindings[*id]);
            }
            self.block(context, update, result_type)?;
            if post_test && let Some(condition) = condition {
                let actual = self.expression(
                    context,
                    condition,
                    Mode::Copy,
                    &Loans::new(),
                    Some(Type::Boolean),
                )?;
                self.require(actual, Some(Type::Boolean), condition.span)?;
            }
            Self::check_loop_back(context)?;
            if condition.is_some() {
                let states = context
                    .loops
                    .last()
                    .expect("new loop frame")
                    .entry
                    .iter()
                    .map(|(id, _)| Availability::of(&context.bindings[*id]))
                    .collect::<Vec<_>>();
                exit_state = Some(match exit_state {
                    Some(previous) => previous
                        .into_iter()
                        .zip(states)
                        .map(|(a, b)| a.join(b))
                        .collect(),
                    None => states,
                });
            }
        } else {
            // Type-check a syntactically present but unreachable tail without
            // adding its availability effects to any reachable exit.
            let mut unreachable = context.clone();
            unreachable.effects_only = true;
            self.block(&mut unreachable, update, result_type)?;
            if post_test && let Some(condition) = condition {
                let actual = self.expression(
                    &mut unreachable,
                    condition,
                    Mode::Copy,
                    &Loans::new(),
                    Some(Type::Boolean),
                )?;
                self.require(actual, Some(Type::Boolean), condition.span)?;
            }
        }
        let frame = context.loops.pop().expect("matching new loop frame");
        exception_state::consume_target(context,frame.target);
        if let Some(zero) = zero_flow {
            context.flow = crate::flow::join(&zero, &context.flow);
        }
        if let Some(breaks) = &frame.break_flow {
            context.flow = crate::flow::join(&context.flow, breaks);
        }
        if !frame.break_seen
            && let Some(condition) = condition
            && let Some((_, no)) = self.pure_guard_outcomes(context, condition)
        {
            context.flow = no;
        }
        if frame.break_seen {
            exit_state = Some(match exit_state {
                Some(previous) => previous
                    .into_iter()
                    .zip(frame.breaks)
                    .map(|(a, b)| a.join(b))
                    .collect(),
                None => frame.breaks,
            });
        }
        let exits = exit_state.is_none();
        if let Some(states) = exit_state {
            for ((id, _), state) in frame.entry.into_iter().zip(states) {
                state.restore(&mut context.bindings[id]);
            }
        }
        Ok(exits)
    }

    fn for_of(
        &mut self,
        context: &mut Context,
        variable: (&Token, bool, &mut Option<usize>),
        iterable: &mut Expr,
        body: &mut [Stmt],
        result_type: Type,
        span: Span,
        labels: &[Token],
    ) -> Result<(), Diagnostic> {
        let actual = self.expression(context, iterable, Mode::Read, &Loans::new(), None)?;
        let element = self.iterable_element(actual).ok_or_else(|| Diagnostic::new("iteration-type", "for...of requires an array, string, Set or Map.", iterable.span))?;
        // Iterable evaluation happens once, even for an empty array. Its moves
        // therefore belong to both the zero-iteration and repeated paths.
        let mut visible = context.names.values().copied().collect::<Vec<_>>();
        visible.sort_unstable();
        let entry = visible
            .into_iter()
            .map(|id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let baseline = entry.iter().map(|(_, state)| state.clone()).collect();
        let first_loan = context.iteration_loans.len();
        for id in Self::loan_roots(iterable, Mode::Read) {
            context.iteration_loans.push((id, iterable.span));
        }
        let (name, rebindable, binding) = variable;
        let id = Self::bind(context, name, element.ty(), false)?;
        context.bindings[id].rebindable = rebindable;
        if !element.ty().is_copy() {
            context.bindings[id].borrowed_element = Some(iterable.span);
        }
        *binding = Some(id);
        let entry_flow = context.flow.clone();
        self.loop_body(
            context,
            body,
            result_type,
            span,
            labels,
            (entry, baseline),
            (entry_flow.clone(), HashMap::new(), entry_flow),
        )?;
        context.iteration_loans.truncate(first_loan);
        context.names.remove(&name.text);
        Ok(())
    }

    // Branch snapshots belong only to an if statement. Keep their temporary
    // maps outside block frames retained by deeply nested ordinary calls.
    fn if_statement(
        &mut self,
        context: &mut Context,
        condition: &mut Expr,
        then_body: &mut [Stmt],
        else_body: &mut [Stmt],
        result_type: Type,
    ) -> Result<bool, Diagnostic> {
        let actual = self.expression(
            context,
            condition,
            Mode::Read,
            &Loans::new(),
            Some(Type::Boolean),
        )?;
        self.require(actual, Some(Type::Boolean), condition.span)?;
        // Snapshot only visible bindings. Keep contracts and mutation flags
        // accumulating across both arms; restore only availability.
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let (then_flow, else_flow) = Self::condition_outcomes(context, condition);
        context.flow = then_flow;
        let then_exits = self.block(context, then_body, result_type)?;
        let after_then_flow = context.flow.clone();
        let after_then = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        for (id, state) in &before {
            state.clone().restore(&mut context.bindings[*id]);
        }
        context.flow = else_flow;
        let else_exits = self.block(context, else_body, result_type)?;
        let after_else_flow = context.flow.clone();
        context.flow = match (then_exits, else_exits) {
            (false, false) => crate::flow::join(&after_then_flow, &after_else_flow),
            (false, true) => after_then_flow,
            (true, false) => after_else_flow,
            (true, true) => HashMap::new(),
        };
        for ((id, _), then_state) in before.into_iter().zip(after_then) {
            let else_state = Availability::of(&context.bindings[id]);
            let joined = match (then_exits, else_exits) {
                (false, false) => then_state.join(else_state),
                (false, true) => then_state,
                (true, false) => else_state,
                (true, true) => Availability::default(),
            };
            joined.restore(&mut context.bindings[id]);
        }
        Ok(then_exits && else_exits)
    }

    // Legacy replacement selection has its own temporary state. Ordinary
    // blocks and calls do not retain these maps and projections while recursing.
    fn assign_statement(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
        operator: Option<(BinaryOp, Span)>,
    ) -> Result<(), Diagnostic> {
        self.assign_statement_with_loans(context,target,value,operator,&Loans::new())
    }
    #[inline(never)]
    fn assign_statement_with_loans(&mut self,context:&mut Context,target:&mut Expr,value:&mut Expr,operator:Option<(BinaryOp,Span)>,loans:&Loans)->Result<(),Diagnostic>{
        self.reject_string_position_write(context, target)?;
        if let Some((op, span)) = operator {
            self.compound_assignment_value(context,target,value,(op,span),loans,true)?;
            return Ok(());
        }
        if !matches!(
            crate::flow::place(target, &context.names)
                .and_then(|place| self.place_type(context, &place)),
            Some(Type::Union(_))
        ) && matches!(
            self.expression_type_hint(context, target),
            Some(Type::Int | Type::Number | Type::Boolean)
        ) {
            self.scalar_assignment(context, target, value, loans, target.span)?;
            return Ok(());
        }
        if matches!(target.kind, ExprKind::Name(_)) {
            self.reassign_with_loans(context,target,value,loans)?;
            return Ok(());
        }
        if !Self::named_storage(target)
            || !matches!(&target.kind, ExprKind::Field(base, _) | ExprKind::Index(base, _) if matches!(base.kind, ExprKind::Name(_) | ExprKind::Index(_, _) | ExprKind::Field(_, _) | ExprKind::Project { .. }))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Only a named record field or array element can be replaced here.",
                target.span,
            ));
        }
        context.replacement = Some(target.span);
        let suppressed = crate::flow::place(target, &context.names);
        if let Some(place) = &suppressed {
            context.suppressed_flow.insert(place.clone());
        }
        let ty = self.mutation_target(context, target, loans, true);
        if let Some(place) = &suppressed {
            context.suppressed_flow.remove(place);
        }
        context.replacement = None;
        let ty = ty?;
        let selected = match &target.kind {
            ExprKind::Index(base, _) => base.as_ref(),
            ExprKind::Field(base, _) => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            _ => unreachable!("checked assignment target"),
        };
        let active = Self::selection_loans(selected, loans);
        let actual = if let Type::StoredCallable(family) = ty {
            self.stored_callable_value(context, value, family, &active)?
        } else {
            self.expression(context, value, Mode::Consume, &active, Some(ty))?
        };
        self.require(actual, Some(ty), value.span)?;
        let binding = Self::root_binding(target).expect("checked field has a binding");
        if let Some((_, path)) = Self::direct_fields(target) {
            Self::availability(
                &context.bindings[binding],
                Some((&path, true)),
                target.span,
                context.effects_only,
            )?;
            context.bindings[binding]
                .moved_fields
                .retain(|missing, _| !missing.starts_with(&path));
        } else if !context.effects_only && context.bindings[binding].transferred.is_some() {
            return Err(Self::transferred(&context.bindings[binding], target.span));
        }
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        Ok(())
    }

    fn block(
        &mut self,
        context: &mut Context,
        body: &mut [Stmt],
        result_type: Type,
    ) -> Result<bool, Diagnostic> {
        self.block_with_scope(context,body,result_type,false)
    }

    #[inline(never)]
    fn check_catch_body(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type)->Result<bool,Diagnostic> {
        let Some(name)=&handler.name else {return self.block(context,&mut handler.body,result);};
        let ty=handler.error_type.ok_or_else(||Diagnostic::new("error-effect","Catch payload type must be resolved before checking its binding.",name.span))?;
        let shadowed=context.names.remove(&name.text);
        let checked=(|| {
            let binding=Self::bind(context,name,ty,false)?;
            handler.binding=Some(binding);
            context.bindings[binding].declaration_span=Some(name.span);
            context.bindings[binding].rebindable=true;
            self.block(context,&mut handler.body,result)
        })();
        context.names.remove(&name.text);
        if let Some(outer)=shadowed {context.names.insert(name.text.clone(),outer);}
        checked
    }

    #[inline(never)]
    fn discovery_error_effects(&mut self)->crate::error_effects::ErrorEffects {
        let families=self.callables.storage.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
        crate::error_effects::annotate_partial_with_storage(&mut self.checked.iter_mut().map(Option::as_mut).collect::<Vec<_>>(),&mut self.closures,&families)
    }

    #[inline(never)]
    fn checked_error_effects(&mut self)->Result<crate::error_effects::ErrorEffects,Diagnostic> {
        let mut functions=self.checked.iter_mut().map(Option::as_mut).collect::<Option<Vec<_>>>()
            .ok_or_else(||Diagnostic::new("error-effect","Error contracts require every function body to be checked first.",Span::default()))?;
        let families=self.callables.storage.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
        Ok(crate::error_effects::annotate_parts_with_storage(&mut functions,&mut self.closures,&families))
    }

    #[inline(never)]
    fn check_source_callable_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<bool,Diagnostic> {
        context.callable_scope_depth = context.lexical_scopes.len();
        if has_exception_syntax(body) {self.check_exception_callable_body(context,body,result)}else{self.block(context,body,result)}
    }

    #[inline(never)]
    fn check_exception_callable_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<bool,Diagnostic> {
        let initial=self.discovery_error_effects();
        self.with_exception_environment_mode(context,&initial,None,true,true,|analyzer,context| {
            let (normal,completions,_errors)=analyzer.check_with_exit_frames(context,|analyzer,context|analyzer.block(context,body,result))?;
            if let Some((_,span,_))=completions.exits.iter().find(|(kind,_,_)|matches!(kind,crate::completion::CompletionKind::Break(_)|crate::completion::CompletionKind::Continue(_))) {
                return Err(Diagnostic::new("loop-control","Completion cannot leave its callable's control boundary.",*span));
            }
            Ok(normal.is_none())
        })
    }

    #[inline(never)]
    fn check_nested_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,span:Span)->Result<bool,Diagnostic> {
        let environment=context.exception_environment.clone().expect("typed region environment");
        let (completed,_)=if environment.partial_contracts {
            self.check_partial_exception_region(context,region,result,environment.runtime,&mut |_,_|{})?
        }else if environment.refresh_contracts {
            self.check_current_exception_region(context,region,result,environment.runtime,&mut |_,_|{})?
        }else{self.check_typed_exception_region(context,region,result,&environment.effects,environment.runtime,&mut |_,_|{})?};
        completed.forward_exits(self,context,span)
    }

    #[inline(never)]
    fn with_exception_environment<T>(&mut self,context:&mut Context,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>,check:impl FnOnce(&mut Self,&mut Context)->Result<T,Diagnostic>)->Result<T,Diagnostic> {
        self.with_exception_environment_mode(context,effects,runtime,false,false,check)
    }

    #[inline(never)]
    fn with_exception_environment_mode<T>(&mut self,context:&mut Context,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>,refresh_contracts:bool,partial_contracts:bool,check:impl FnOnce(&mut Self,&mut Context)->Result<T,Diagnostic>)->Result<T,Diagnostic> {
        let previous=context.exception_environment.replace(std::rc::Rc::new(exception_state::CheckEnvironment{effects:effects.clone(),runtime,refresh_contracts,partial_contracts}));
        let result=check(self,context);
        context.exception_environment=previous;
        result
    }

    #[inline(never)]
    fn check_typed_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState))->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        self.with_exception_environment(context,effects,runtime,|analyzer,context|{
            let checked=analyzer.check_typed_region_body(context,&mut region.body,result,effects)?;
            analyzer.finish_checked_exception_region(context,region,result,effects,runtime,unwind,checked,false)
        })
    }

    #[inline(never)]
    fn check_current_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState))->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        let initial=self.checked_error_effects()?;
        self.with_exception_environment_mode(context,&initial,runtime,true,false,|analyzer,context|{
            let (normal,frame,errors,incoming,effects)=analyzer.check_current_region_body(context,&mut region.body,result)?;
            analyzer.finish_checked_exception_region(context,region,result,&effects,runtime,unwind,(normal,frame,errors,incoming),true)
        })
    }

    #[inline(never)]
    fn region_error_effects(&mut self,context:&Context)->Result<crate::error_effects::ErrorEffects,Diagnostic> {
        if context.exception_environment.as_ref().is_some_and(|environment|environment.partial_contracts) {
            Ok(self.discovery_error_effects())
        }else{self.checked_error_effects()}
    }

    #[inline(never)]
    fn check_partial_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState))->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        let initial=self.discovery_error_effects();
        self.with_exception_environment_mode(context,&initial,runtime,true,true,|analyzer,context|{
            let (normal,frame,errors,incoming,effects)=analyzer.check_current_region_body(context,&mut region.body,result)?;
            analyzer.finish_checked_exception_region(context,region,result,&effects,runtime,unwind,(normal,frame,errors,incoming),true)
        })
    }

    #[inline(never)]
    fn canonical_cleanup_body(&mut self,entry:&Context,state:exception_state::ExitState,source:&[Stmt],result:Type,effects:&crate::error_effects::ErrorEffects,refresh:bool)->Result<(Vec<Stmt>,Vec<Binding>),Diagnostic> {
        let mut context=entry.clone();
        let mut body=source.to_vec();
        self.check_region_body_at_state(entry,&mut context,state,source,&mut body,result)?;
        let refreshed=if refresh {Some(self.region_error_effects(&context)?)}else{None};
        crate::error_effects::annotate_region_body(&mut body,refreshed.as_ref().unwrap_or(effects));
        Ok((body,context.bindings))
    }

    #[inline(never)]
    fn finish_checked_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState),checked:(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),refresh_contracts:bool)->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        let (normal,frame,errors,incoming)=checked;
        let runtime=if runtime.is_none() && incoming.runtime && !incoming.opaque
            && region.catch.as_ref().is_some_and(|handler|handler.name.is_some())
        {Some(self.materialize_discovery_runtime_error())}else{runtime};
        let outer=frame.outer;
        let mut pending=exception_state::CompletionStates::from_completions(normal,frame,unwind);
        pending.record_failures(errors,unwind);
        let mut outgoing=incoming.clone();
        if let Some(handler)=&mut region.catch {
            handler.binding=None;
            handler.error_type=None;
            handler.error_contract=Some(Box::new(incoming.clone()));
            outgoing=crate::error_effects::FailureSet::default();
            pending=pending.catch(&mut |state| {
                state.restore(context);
                let (normal,frame,errors,contract)=self.check_catch_contract_exits(context,handler,result,&incoming,(!refresh_contracts).then_some(effects),runtime)?;
                outgoing|=contract;
                let mut handled=exception_state::CompletionStates::from_completions(normal,frame,unwind);
                handled.record_failures(errors,unwind);
                handled.project_outer(outer);
                Ok(handled)
            })?;
        }
        if let Some(cleanup)=&mut region.finally {
            let entry=context.clone();
            let source=cleanup.clone();
            let canonical=pending.joined_entry().map(|state|self.canonical_cleanup_body(&entry,state,&source,result,effects,refresh_contracts)).transpose()?;
            let mut cleanup_contract=crate::error_effects::FailureSet::default();
            pending=pending.finally(&mut |state| {
                let (normal,frame,mut errors)=self.check_region_body_at_state(&entry,context,state,&source,cleanup,result)?;
                let refreshed=if refresh_contracts {Some(self.region_error_effects(context)?)}else{None};
                let current=refreshed.as_ref().unwrap_or(effects);
                cleanup_contract|=crate::error_effects::annotate_region_body(cleanup,current);
                let points=crate::error_effects::OperationEffects::for_body(cleanup);
                errors.refine(&mut |span|points.get(span));
                let mut cleaned=exception_state::CompletionStates::from_completions(normal,frame,unwind);
                cleaned.record_failures(errors,unwind);
                cleaned.project_outer(outer);
                Ok(cleaned)
            })?;
            if let Some((canonical,bindings))=canonical {
                *cleanup=canonical;
                context.bindings.truncate(entry.bindings.len());
                context.bindings.extend(bindings.into_iter().skip(entry.bindings.len()));
            }
            if crate::completion::definitely_abrupt(&source,0) {outgoing=cleanup_contract;}else{outgoing|=cleanup_contract;}
        }
        pending.restore_normal(context);
        Ok((pending,outgoing))
    }

    #[inline(never)]
    fn check_typed_catch_exits(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type,incoming:&crate::error_effects::FailureSet,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),Diagnostic> {
        self.check_catch_contract_exits(context,handler,result,incoming,Some(effects),runtime)
    }

    #[inline(never)]
    fn check_catch_contract_exits(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type,incoming:&crate::error_effects::FailureSet,effects:Option<&crate::error_effects::ErrorEffects>,runtime:Option<Type>)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),Diagnostic> {
        handler.binding=None;
        handler.error_type=None;
        handler.error_contract=Some(Box::new(incoming.clone()));
        handler.error_type=if let Some(name)=&handler.name {incoming.catch_type(&self.unions,runtime,name.span)?}else{None};
        let (normal,completions,mut errors)=self.check_catch_exits(context,handler,result)?;
        let refreshed=if effects.is_none() {Some(self.region_error_effects(context)?)}else{None};
        let current=effects.or(refreshed.as_ref()).expect("catch contracts");
        let outgoing=crate::error_effects::annotate_region_body(&mut handler.body,current);
        let points=crate::error_effects::OperationEffects::for_body(&handler.body);
        errors.refine(&mut |span|points.get(span));
        Ok((normal,completions,errors,outgoing))
    }

    #[inline(never)]
    fn check_catch_exits(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame),Diagnostic> {
        self.check_with_exit_frames(context,|analyzer,context|analyzer.check_catch_body(context,handler,result))
    }

    #[inline(never)]
    fn check_current_region_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet,crate::error_effects::ErrorEffects),Diagnostic> {
        let (normal,completions,mut errors)=self.check_region_body(context,body,result)?;
        self.check_pending_region_functions(body,result)?;
        let effects=if context.exception_environment.as_ref().is_some_and(|environment|environment.partial_contracts) {
            self.region_caller_error_effects(context,body,result)
        }else{self.region_error_effects(context)?};
        let contract=crate::error_effects::annotate_region_body(body,&effects);
        let points=crate::error_effects::OperationEffects::for_body(body);
        errors.refine(&mut |span|points.get(span));
        Ok((normal,completions,errors,contract,effects))
    }

    #[inline(never)]
    fn region_caller_error_effects(&mut self,context:&Context,body:&[Stmt],result:Type)->crate::error_effects::ErrorEffects {
        let caller=CheckedFunction{template:false,specialization:false,name:"#region".into(),bindings:context.bindings.clone(),parameter_count:0,result,body:body.to_vec()};
        let families=self.callables.storage.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
        crate::error_effects::annotate_partial_with_callers(&mut self.checked.iter_mut().map(Option::as_mut).collect::<Vec<_>>(),&mut self.closures,&families,&caller,&self.unions)
    }

    #[inline(never)]
    fn check_pending_region_functions(&mut self,body:&[Stmt],result:Type)->Result<(),Diagnostic> {
        let caller=CheckedFunction{template:false,specialization:false,name:String::new(),bindings:vec![],parameter_count:0,result,body:body.to_vec()};
        let targets=crate::error_effects::checked_expressions(&caller).into_iter().filter_map(|expression|match expression.target {Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>Some(id),_=>None}).collect::<HashSet<_>>();
        for id in targets {
            if self.checked[id].is_none() && !self.visiting[id] {self.function(id)?;}
        }
        Ok(())
    }

    #[inline(never)]
    fn check_typed_region_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type,effects:&crate::error_effects::ErrorEffects)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),Diagnostic> {
        let (normal,completions,mut errors)=self.check_region_body(context,body,result)?;
        let contract=crate::error_effects::annotate_region_body(body,effects);
        let points=crate::error_effects::OperationEffects::for_body(body);
        errors.refine(&mut |span|points.get(span));
        Ok((normal,completions,errors,contract))
    }

    #[inline(never)]
    fn check_region_body_at_state(&mut self,entry:&Context,context:&mut Context,state:exception_state::ExitState,source:&[Stmt],body:&mut Vec<Stmt>,result:Type)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame),Diagnostic> {
        // Every ownership replay starts at the same declaration prefix. Retaining
        // a prior path's local slots changes binding identities in the shared body.
        *context=entry.clone();
        *body=source.to_vec();
        state.restore(context);
        self.check_region_body(context,body,result)
    }

    #[inline(never)]
    fn check_region_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame),Diagnostic> {
        self.check_with_exit_frames(context,|analyzer,context|analyzer.block(context,body,result))
    }

    #[inline(never)]
    fn check_with_exit_frames(&mut self,context:&mut Context,check:impl FnOnce(&mut Self,&mut Context)->Result<bool,Diagnostic>)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame),Diagnostic> {
        let outer=context.bindings.len();
        let entry_loans=context.capture_loans.len();
        let previous_entry_loans=context.region_entry_loans.replace(entry_loans);
        let previous_loop_depth=context.region_loop_depth.replace(context.loops.len());
        let previous_completions=context.completion_frame.replace(Box::new(exception_state::CompletionFrame{outer,exits:vec![]}));
        let previous_errors=context.error_frame.replace(Box::new(exception_state::ErrorFrame{outer,points:vec![]}));
        let checked=check(self,context);
        let mut completions=*std::mem::replace(&mut context.completion_frame,previous_completions).expect("region completion frame");
        let mut errors=*std::mem::replace(&mut context.error_frame,previous_errors).expect("region error frame");
        context.region_loop_depth=previous_loop_depth;
        context.region_entry_loans=previous_entry_loans;
        let exited=checked?;
        // These exits carry no borrowed payload; region-local loans end here.
        for (kind,_,state) in &mut completions.exits {
            if matches!(kind,crate::completion::CompletionKind::Break(_)|crate::completion::CompletionKind::Continue(_)) {
                state.unwind_capture_loans(entry_loans,&[]);
            }
        }
        for (_,state) in &mut errors.points {state.unwind_capture_loans(entry_loans,&[]);}
        let normal=(!exited).then(||exception_state::ExitState::capture(context,outer));
        Ok((normal,completions,errors))
    }

    #[inline(never)]
    fn record_typed_completion(&self,context:&mut Context,kind:crate::completion::CompletionKind,ty:Type,span:Span) {
        exception_state::record_completion(context,kind,span);
        let capture_free=if let Type::Closure(origin)=ty {
            let closure=&self.closures[origin];
            !closure.borrowed && closure.captures.iter().all(|capture|
                capture.storage==Mode::Consume && !self.expression_result_retains_captures(closure.function.bindings[capture.inner].ty))
        }else{false};
        if (capture_free || !self.expression_result_retains_captures(ty))
            && let Some(entry)=context.region_entry_loans
            && let Some(frame)=&mut context.completion_frame
            && let Some((_,_,state))=frame.exits.last_mut()
        {state.unwind_capture_loans(entry,&[]);}
    }

    #[inline(never)]
    fn check_throw_value(&mut self,context:&mut Context,value:&mut Expr)->Result<Type,Diagnostic> {
        let expected=self.expression_type_hint(context,value).filter(|ty|matches!(ty,Type::Closure(_)));
        if let Some(Type::Closure(origin))=expected
            && self.closures[origin].borrowed
        {
            return Err(Diagnostic::new("closure-escape","A thrown callable must own its captured storage.",value.span)
                .related(self.closures[origin].span,"Borrowed callable is created here."));
        }
        let ty=self.expression(context,value,Mode::Consume,&Loans::new(),expected)?;
        if ty==Type::Void {return Err(Diagnostic::new("type","A thrown expression must produce a value.",value.span));}
        self.record_typed_completion(context,crate::completion::CompletionKind::Throw,ty,value.span);
        Ok(ty)
    }

    #[inline(never)]
    fn block_with_scope(&mut self, context: &mut Context, body: &mut [Stmt], result_type: Type, keep_names: bool) -> Result<bool, Diagnostic> {
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Combined analysis nesting exceeds the prototype limit of 128.",
                body.first().map_or(Span::default(), |stmt| stmt.span),
            ));
        }
        context.lexical_scopes.push(
            body.iter()
                .filter_map(|stmt| match &stmt.kind {
                    StmtKind::Let { name, .. } => Some(name.text.clone()),
                    _ => None,
                })
                .collect(),
        );
        // Remove only names actually shadowed by this block. Their typed slots
        // and source identities remain available to captured named bodies.
        let mut shadowed = Vec::new();
        for name in context.lexical_scopes.last().expect("entered lexical scope") {
            if let Some(&slot) = context.names.get(name)
                && (context.lexical_scopes.len() != context.callable_scope_depth + 1 || context.is_capture(slot)) {
                shadowed.push((name.clone(), slot));
            }
        }
        for (name, _) in &shadowed { context.names.remove(name); }
        self.depth += 1;
        let first_local = context.bindings.len();
        let first_loan = context.capture_loans.len();
        let mut exited = false;
        for stmt in body {
            if exited {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Statement is unreachable after all preceding paths exit this block.",
                    stmt.span,
                ));
            }
            match &mut stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Throw(value) if context.completion_frame.is_some() => {
                    self.check_throw_value(context,value)?;
                    exited=true;
                }
                StmtKind::Try(region) if context.exception_environment.is_some() && context.completion_frame.is_some() => {
                    exited=self.check_nested_exception_region(context,region,result_type,stmt.span)?;
                }
                StmtKind::Try(_) | StmtKind::Throw(_) => return Err(Diagnostic::new("unsupported","Exception syntax is parsed; error propagation and completion analysis are not implemented yet.",stmt.span)),
                StmtKind::ForOf {
                    name,
                    rebindable,
                    iterable,
                    body,
                    binding,
                } => {
                    self.for_of(
                        context,
                        (name, *rebindable, binding),
                        iterable,
                        body,
                        result_type,
                        stmt.span,
                        &stmt.labels,
                    )?;
                }
                StmtKind::While { condition, body } => {
                    self.while_loop(context, condition, body, result_type, stmt.span, &stmt.labels)?;
                }
                StmtKind::Block(body) => {
                    exited = if stmt.labels.is_empty() {self.block(context, body, result_type)?} else {self.labeled_block(context,body,result_type,stmt.span,&stmt.labels)?};
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    post_test,
                    tail_reachable,
                } => {
                    exited = self.tail_loop(
                        context,
                        condition,
                        body,
                        update,
                        *post_test,
                        tail_reachable,
                        result_type,
                        stmt.span,
                        &stmt.labels,
                    )?;
                }
                StmtKind::Break | StmtKind::Continue => {
                    self.control_exit(context, &stmt.labels, &mut stmt.control_target, matches!(stmt.kind,StmtKind::Continue), stmt.span)?;
                    let kind=if matches!(stmt.kind,StmtKind::Continue) {
                        crate::completion::CompletionKind::Continue(stmt.control_target)
                    } else {crate::completion::CompletionKind::Break(stmt.control_target)};
                    exception_state::record_completion(context,kind,stmt.span);
                    exited=true;
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    exited =
                        self.if_statement(context, condition, then_body, else_body, result_type)?;
                }
                StmtKind::Let {
                    rebindable,
                    name,
                    annotation,
                    value,
                    binding,
                } => self.let_statement(context, *rebindable, name, annotation, value, binding)?,
                StmtKind::Assign {
                    target,
                    value,
                    operator,
                } => self.assign_statement(context, target, value, *operator)?,
                StmtKind::Return(value) => {
                    self.return_statement(context, value, result_type, stmt.span)?;
                    exited = true;
                }
                StmtKind::Expr(value) => {
                    let view = self
                        .expression_type_hint(context, value)
                        .is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_)));
                    let place_view = self.borrowed_place_hint(context, value);
                    let ty = if matches!(value.kind,ExprKind::Sequence(_)) {
                        self.sequence_expression(context,value,Mode::Read,&Loans::new(),None,true)?
                    } else {
                        self.expression(context,value,if view || place_view {Mode::Read}else{Mode::Consume},&Loans::new(),None)?
                    };
                    if matches!(ty, Type::Closure(_)) && !matches!(value.kind, ExprKind::Call(..)) {
                        return Err(Diagnostic::new(
                            "unsupported",
                            "Bind a closure before using it.",
                            value.span,
                        ));
                    }
                }
            }
        }
        context.lexical_scopes.pop();
        if !keep_names {
            context.capture_loans.truncate(first_loan);
            for binding in &context.bindings[first_local..] {
                context.names.remove(&binding.name);
            }
        }
        if !keep_names {
            context.names.extend(shadowed);
        }
        self.depth -= 1;
        Ok(exited)
    }

    /// One named declaration. Kept out of the block loop, whose frame is
    /// retained across nested statement bodies.
    #[inline(never)]
    fn let_statement(
        &mut self,
        context: &mut Context,
        rebindable: bool,
        name: &Token,
        annotation: &Option<crate::type_syntax::ValueAnnotation>,
        value: &mut Expr,
        binding: &mut Option<usize>,
    ) -> Result<(), Diagnostic> {
        let expected = annotation
            .as_ref()
            .map(|a| match crate::callable_annotations::callback(a) {
                Some(callback) => self.resolve_callback(&callback),
                None => self.callable_resolver().resolve_value_annotation(a, false),
            })
            .transpose()?;
        if value.is_deferred()
            && self.deferred_let(context, name, expected, value, binding)?
        {
            return Ok(());
        }
        // Infer the concrete environment on a declaration move. Keep
        // callback contracts intact; this does not create an alias.
        let expected = expected.or_else(|| match &value.kind {
            ExprKind::Name(source) => context.names.get(source).and_then(|&id| {
                let ty = context.bindings[id].ty;
                matches!(ty, Type::Closure(_) | Type::Callback(_)).then_some(ty)
            }),
            _ => None,
        });
        let view = self
            .expression_type_hint(context, value)
            .is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_)));
        let place_view = self.borrowed_place_hint(context, value)
            || self.indexed_element_hint(context, value);
        let mut actual = self.expression(
            context,
            value,
            if view || place_view {
                Mode::Read
            } else {
                Mode::Consume
            },
            &Loans::new(),
            expected,
        )?;
        if actual == Type::Void {
            return Err(Diagnostic::new(
                "type",
                "Cannot bind a function with no return value.",
                value.span,
            ));
        }
        self.require(actual, expected, value.span)?;
        if let Some(Type::Callback(contract)) = expected
            && self.callables.callbacks[contract].kind.is_once()
            && let Type::Closure(origin) = actual
            && self.closures[origin].invocation != Mode::Consume
        {
            let wrapper = self.once_return(origin, context, value.span);
            value.wrap_once(wrapper);
            actual = Type::Closure(wrapper);
        }
        let nullable_callback = expected
            .and_then(|ty| self.unions.payload(ty))
            .and_then(|ty| {
                if let Type::Callback(id) = ty {
                    Some(self.callables.callbacks[id].kind)
                } else {
                    None
                }
            });
        if nullable_callback.is_some() {
            if let Some(concrete) = self.nullable_origin(value)? {
                let members = self
                    .unions
                    .members(actual.union().expect("nullable callable"));
                let specialized = self.unions.intern(
                    members.into_iter().map(|member| {
                        if matches!(member, Type::Callback(_)) {
                            concrete
                        } else {
                            member
                        }
                    }),
                    value.span,
                )?;
                Self::retag_nullable_origin(
                    value,
                    actual.union().expect("nullable callable"),
                    specialized.union().expect("nullable callable"),
                );
                actual = specialized;
            }
        }
        let callback = Self::conditional_callback_metadata(context, value)?;
        // A borrowed owned union is held as its readonly view.
        if place_view && let Type::Union(union) = actual && !union.copy {
            actual = Type::BorrowedUnion(union);
        }
        let id = Self::bind(context, name, actual, false)?;
        if let Some(kind) = nullable_callback {
            context.bindings[id].callback_kind = Some(kind);
            context.bindings[id].callback_invocation = Some(kind.invocation());
        }
        context.bindings[id].rebindable =
            rebindable && !matches!(actual, Type::BorrowedUnion(_)) && !place_view;
        if matches!(actual, Type::BorrowedUnion(_)) || (place_view && !actual.is_copy())
        {
            if place_view && !matches!(actual, Type::BorrowedUnion(_)) {
                context.bindings[id].borrowed_place = Some(value.span);
            }
            context.bindings[id].view_roots = Self::view_sources(context, value);
            context.bindings[id].used = Some(value.span);
        }
        if let Some((invocation, reference, kind)) = callback {
            context.bindings[id].callback_invocation = invocation;
            context.bindings[id].callback_reference = reference;
            context.bindings[id].callback_kind = kind;
        }
        if !context.bindings[id].rebindable && let Some(ty)=crate::flow::sentinel(value) {
            context.constant_absence.insert(id,ty);
        }
        *binding = Some(id);
        Ok(())
    }

    /// One return statement; the caller marks the block as exited.
    #[inline(never)]
    fn return_statement(
        &mut self,
        context: &mut Context,
        value: &mut Option<Expr>,
        result_type: Type,
        span: Span,
    ) -> Result<(), Diagnostic> {
        context.returning_closure = self.callable_result_type(result_type);
        let mut actual = match value {
            Some(value) => self.expression(
                context,
                value,
                Mode::Consume,
                &Loans::new(),
                if context.inferred_body_result { None } else { Some(result_type) },
            )?,
            None => Type::Void,
        };
        context.returning_closure = false;
        let converted =
            self.callable_result_value(context, result_type, actual, span);
        if converted != actual
            && let Some(value) = value
            && let Type::Closure(origin) = converted
        {
            value.wrap_once(origin);
        }
        actual = converted;
        if !context.inferred_body_result { self.require(actual, Some(result_type), span)?; }
        if self.discovering_results && self.has_pending_result(actual) {
            return Ok(());
        }
        if matches!(
            self.unions.payload(actual).unwrap_or(actual),
            Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_)
        ) {
            if !self.effects_only
                && context.returned_callable.is_some_and(|previous| {
                    previous != actual
                        && !self.absent_callable_type(previous)
                        && !self.absent_callable_type(actual)
                })
            {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A factory must return one closure origin; mixed environments are unsupported.",
                    span,
                ));
            }
            if !self.absent_callable_type(actual) || context.returned_callable.is_none()
            {
                context.returned_callable = Some(actual);
            }
        }
        self.record_typed_completion(context,crate::completion::CompletionKind::Return,actual,span);
        Ok(())
    }

    fn rebind_target(&self, context: &Context, target: &Expr) -> Result<usize, Diagnostic> {
        let ExprKind::Name(name) = &target.kind else {
            unreachable!("named assignment")
        };
        let id = *context.names.get(name).ok_or_else(|| {
            Diagnostic::new("name", format!("Unknown value {name:?}."), target.span)
        })?;
        if !context.effects_only {
            Self::check_element_access(&context.bindings[id], Mode::Mutate, target.span)?;
        }
        if !context.bindings[id].rebindable {
            return Err(Diagnostic::new(
                "unsupported",
                "Only let bindings can be reassigned; const bindings and parameters cannot be rebound.",
                target.span,
            ));
        }
        // Replacement needs no old value, but cannot invalidate an outstanding loan.
        if !context.effects_only
            && context
                .names
                .values()
                .any(|view| context.bindings[*view].view_roots.contains(&id))
        {
            return Err(Diagnostic::new(
                "view-conflict",
                "Cannot replace storage while a scoped readonly result borrows it.",
                target.span,
            ));
        }
        Self::check_iteration_access(context, id, target.span)?;
        for &(captured, _, creation) in &context.capture_loans {
            if !context.effects_only && captured == id {
                return Err(Diagnostic::new(
                    "capture-conflict",
                    format!("Cannot replace {name:?} while a closure holds access."),
                    target.span,
                )
                .related(creation, "Closure capture starts here."));
            }
        }
        Ok(id)
    }

    fn reassign(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
    ) -> Result<(), Diagnostic> {
        self.reassign_with_loans(context,target,value,&Loans::new())
    }
    #[inline(never)]
    fn reassign_with_loans(&mut self,context:&mut Context,target:&mut Expr,value:&mut Expr,loans:&Loans)->Result<(),Diagnostic>{
        let id = self.rebind_target(context, target)?;
        let ty = context.bindings[id].ty;
        if !context.effects_only && let Some(active)=loans.get(&id) && let Some((_,earlier))=active.first(){
            return Err(Diagnostic::new("conflicting-access",format!("Cannot replace {:?} through overlapping access.",context.bindings[id].name),target.span).related(*earlier,"Earlier access starts here."));
        }
        // Check the RHS against the current availability, allowing x = take(x)
        // but rejecting a read of x after an earlier transfer. Commit only after
        // evaluation and exact type checking complete.
        let actual = self.expression(context, value, Mode::Consume, loans, Some(ty))?;
        let nullable_callback = matches!(self.unions.payload(ty), Some(Type::Callback(_)));
        if matches!(ty, Type::Callback(_)) && actual != ty {
            return Err(Diagnostic::new(
                "callable-origin",
                "Callback replacement requires a value derived from the same parameter; matching signatures alone are insufficient.",
                value.span,
            ));
        }
        if let (Type::Closure(previous), Type::Closure(next)) = (ty, actual)
            && previous != next
        {
            return Err(Diagnostic::new("callable-origin",
                    "Callable replacement requires the same concrete environment origin; matching signatures alone are insufficient.", value.span)
                    .related(self.closures[previous].span, "Original environment is defined here.")
                    .related(self.closures[next].span, "Replacement environment is defined here."));
        }
        self.require(actual, Some(ty), value.span)?;
        let source_kind = value
            .binding
            .and_then(|source| context.bindings[source].callback_kind);
        if !nullable_callback && context.bindings[id].callback_kind != source_kind {
            return Err(Diagnostic::new(
                "callback-contract",
                "Callback replacement must preserve the source ownership and invocation contract.",
                value.span,
            ));
        }
        if context.is_capture(id) {
            context.bindings[id].used.get_or_insert(target.span);
            context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
        }
        crate::flow::invalidate(&mut context.flow, id);
        context.bindings[id].transferred = None;
        context.bindings[id].moved_fields.clear();
        context.bindings[id].mutable = true;
        target.binding = Some(id);
        target.ty = Some(ty);
        Ok(())
    }

    fn mutation_target(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        loans: &Loans,
        suppress_target: bool,
    ) -> Result<Type, Diagnostic> {
        let place = crate::flow::place(target, &context.names);
        let prior = context.mutation_place.clone();
        context.mutation_place = crate::flow::mutation_footprint(target, &context.names);
        let first_keep = context.keep_flow.len();
        // Selecting a stable replacement target has no write effect. Its old
        // proofs remain usable in the RHS; commit invalidates them afterward.
        if suppress_target && let Some(place) = &place {
            context.keep_flow.extend(
                context
                    .flow
                    .keys()
                    .filter(|prior| {
                        prior.root == place.root
                            && (prior.fields.starts_with(&place.fields)
                                || place.fields.starts_with(&prior.fields))
                    })
                    .cloned(),
            );
        }
        if suppress_target && let Some(place) = &place {
            context.suppressed_flow.insert(place.clone());
        }
        let result = self.expression(context, target, Mode::Mutate, loans, None);
        if suppress_target && let Some(place) = &place {
            context.suppressed_flow.remove(place);
        }
        context.mutation_place = prior;
        context.keep_flow.truncate(first_keep);
        result
    }

    fn update_expression(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        loans: &Loans,
        operator: Span,
    ) -> Result<Type, Diagnostic> {
        self.reject_string_position_write(context, target)?;
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "An update requires a let binding, named record field, or array element.",
                target.span,
            ));
        }
        // Forward enclosing loans: an expression update is an exclusive access.
        let ty = self.mutation_target(context, target, loans, false)?;
        if !matches!(ty, Type::Int | Type::Number) {
            return Err(Diagnostic::new(
                "type",
                "An update requires an int or number target without coercion.",
                operator,
            ));
        }
        if named {
            let id = target.binding.expect("checked update name");
            if context.is_capture(id) {
                context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
            }
        }
        if ty == Type::Int { exception_state::record_point(context, operator); }
        Ok(ty)
    }

    /// A const taken from an owned element of a named array, or from a field
    /// of such an element, borrows it for reading; indexing never moves.
    #[inline(never)]
    fn indexed_element_hint(&mut self, context: &Context, expr: &Expr) -> bool {
        if let ExprKind::Sequence(values)=&expr.kind{return values.last().is_some_and(|last|self.indexed_element_hint(context,last));}
        if let ExprKind::TypeAssertion { value, .. } = &expr.kind {
            return self.indexed_element_hint(context, value);
        }
        let mut place = expr;
        if let ExprKind::Unary{op:UnaryOp::PatternSelection,value,..}=&place.kind {place=value;}
        while let ExprKind::Field(base, _) = &place.kind {
            place = base;
        }
        let ExprKind::Index(base, _) = &place.kind else { return false };
        Self::named_storage(base)
            && matches!(self.expression_type_hint(context, base), Some(Type::Array(_)))
            && self.expression_type_hint(context, expr).is_some_and(|ty| !ty.is_copy())
    }

    /// `text[position]`: a new string, never a view of the text.
    #[inline(never)]
    pub(crate) fn string_position(&self, context: &Context, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Index(base, _)
            if self.expression_type_hint(context, base).is_some_and(|ty| self.unions.payload(ty).unwrap_or(ty) == Type::String))
    }

    fn borrowed_place_hint(&mut self, context: &Context, expr: &Expr) -> bool {
        if self.string_position(context, expr) {
            return false;
        }
        match &expr.kind {
            ExprKind::Sequence(values)=>values.last().is_some_and(|last|self.borrowed_place_hint(context,last) || self.indexed_element_hint(context,last)),
            ExprKind::TypeAssertion { value, .. } => self.borrowed_place_hint(context, value) || self.indexed_element_hint(context, value),
            ExprKind::Unary{op:UnaryOp::PatternSelection,value,..}=>self.borrowed_place_hint(context,value) || self.indexed_element_hint(context,value),
            ExprKind::Optional { body, .. } => self.borrowed_place_hint(context, body),
            ExprKind::OptionalChain(value) => self.optional_borrowed_hint(context, value),
            ExprKind::Selected { owned, .. } => !owned && !expr.ty.is_some_and(Type::is_copy),
            // A loop variable over owned elements is itself a readonly borrow.
            ExprKind::Name(name) => context.names.get(name).is_some_and(|id| {
                context.bindings[*id].borrowed_place.is_some()
                    || context.bindings[*id].borrowed_element.is_some()
                    || context.readonly_parameters.contains(id)
            }),
            ExprKind::CoalesceAssign { target, value, .. } => self
                .coalesce_hint(context, target, value, expr.span, false)
                .or_else(|| self.expression_type_hint(context, target))
                .is_some_and(|ty| !ty.is_copy()),
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                self.borrowed_place_hint(context, consequent)
                    || self.borrowed_place_hint(context, alternate)
            }
            ExprKind::Field(value, _) | ExprKind::Index(value, _) => {
                self.expression_type_hint(context, expr)
                    .is_none_or(|ty| !ty.is_copy())
                    && self.borrowed_place_hint(context, value)
            }
            ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                value,
                ..
            } => self.borrowed_place_hint(context, value),
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or,
                left,
                right,
                ..
            } => {
                self.borrowed_place_hint(context, left) || self.borrowed_place_hint(context, right)
            }
            _ => false,
        }
    }

    fn coalesce_assignment(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::CoalesceAssign {
            target,
            value,
            operator,
            branch,
        } = &mut expr.kind
        else {
            unreachable!()
        };
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(..) | ExprKind::Index(..))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Nullish assignment requires a mutable let binding, named field, or indexed storage.",
                target.span,
            ));
        }
        let place = crate::flow::place(target, &context.names);
        let prior_fact = place.as_ref().and_then(|p| context.flow.get(p)).cloned();
        let ty = self.mutation_target(context, target, loans, true)?;
        let members = ty
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![ty]);
        let possible = prior_fact
            .map(|fact| fact.members)
            .unwrap_or_else(|| members.clone());
        let absent = possible
            .iter()
            .any(|ty| matches!(ty, Type::Null | Type::Undefined));
        *branch = if !absent {
            crate::syntax::NullishBranch::Present
        } else if possible
            .iter()
            .all(|ty| matches!(ty, Type::Null | Type::Undefined))
        {
            crate::syntax::NullishBranch::Absent
        } else {
            crate::syntax::NullishBranch::Dynamic
        };
        let payload = self.unions.present(ty, expr.span)?;
        let before = context.clone();
        if let Some(place) = &place {
            let nil = possible
                .iter()
                .copied()
                .filter(|ty| matches!(ty, Type::Null | Type::Undefined))
                .collect::<Vec<_>>();
            if !nil.is_empty() {
                context.flow.insert(
                    place.clone(),
                    crate::flow::Fact {
                        members: nil,
                        origin: Some((expr.span.start, expr.span.end)),
                    },
                );
            }
        }
        let hint = self.expression_type_hint(context, value);
        let mut rhs_members = vec![payload.unwrap_or(ty)];
        if let Some(hint) = hint {
            for member in hint
                .union()
                .map(|id| self.unions.members(id))
                .unwrap_or_else(|| vec![hint])
            {
                if matches!(member, Type::Null | Type::Undefined) {
                    rhs_members.push(member);
                }
            }
        }
        // A sentinel-only destination cannot acquire a different payload.
        if payload.is_none() {
            rhs_members = members.clone();
        }
        let rhs_context = self.unions.intern(rhs_members, value.span)?;
        let selected_parent = match &target.kind {
            ExprKind::Index(base, _) => base.as_ref(),
            ExprKind::Field(base, _) => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            _ => target.as_ref(),
        };
        let active = Self::selection_loans(selected_parent, loans);
        let actual = self.expression(context, value, Mode::Consume, &active, Some(rhs_context))?;
        self.require(actual, Some(ty), value.span)?;
        if matches!(ty, Type::Closure(_) | Type::Callback(_)) && actual != ty {
            return Err(Diagnostic::new(
                "callable-origin",
                "Nullish assignment must preserve the callable's concrete origin.",
                value.span,
            ));
        }
        let actual_members = actual
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![actual]);
        let actual_members = if let Some(Type::Callback(wanted)) = payload {
            let actual_payload = self.unions.payload(actual).unwrap_or(actual);
            if let Type::Callback(found) = actual_payload
                && wanted != found
            {
                return Err(Diagnostic::new(
                    "callable-origin",
                    "Nullish assignment must preserve the callback parameter origin; matching signatures alone are insufficient.",
                    value.span,
                ));
            }
            actual_members
                .into_iter()
                .map(|member| {
                    if matches!(member, Type::Closure(_) | Type::Callback(_)) {
                        Type::Callback(wanted)
                    } else {
                        member
                    }
                })
                .collect::<Vec<_>>()
        } else {
            actual_members
        };
        let result_members = if absent {
            possible
                .iter()
                .copied()
                .filter(|ty| !matches!(ty, Type::Null | Type::Undefined))
                .chain(actual_members)
                .collect::<Vec<_>>()
        } else {
            possible.clone()
        };
        let logical = self.unions.intern(result_members, expr.span)?;
        if absent {
            Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
            if possible
                .iter()
                .any(|ty| !matches!(ty, Type::Null | Type::Undefined))
            {
                for id in before.names.values().copied() {
                    Availability::of(&before.bindings[id])
                        .join(Availability::of(&context.bindings[id]))
                        .restore(&mut context.bindings[id]);
                }
                context.flow = crate::flow::join(&before.flow, &context.flow);
            }
        } else {
            // Source-check an unreachable RHS, but retain no access or ownership effects.
            *context = before;
        }
        if let Some(place) = place {
            crate::flow::invalidate_place(&mut context.flow, &place);
            context.flow.insert(
                place,
                crate::flow::Fact {
                    members: logical
                        .union()
                        .map(|id| self.unions.members(id))
                        .unwrap_or_else(|| vec![logical]),
                    origin: Some((expr.span.start, expr.span.end)),
                },
            );
        }
        if named {
            let id = target.binding.expect("checked target");
            context.bindings[id].mutable = true;
        }
        if !logical.is_copy() && matches!(mode, Mode::Consume | Mode::Mutate) {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An owned assignment result borrows its destination; use a readonly consumer or clone() for independent ownership.",
                *operator,
            ));
        }
        self.require(logical, expected, expr.span)?;
        Ok(if !logical.is_copy() {
            logical.union().map(Type::BorrowedUnion).unwrap_or(logical)
        } else {
            logical
        })
    }

    fn scalar_assignment(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
        loans: &Loans,
        operator: Span,
    ) -> Result<Type, Diagnostic> {
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Assignment values require a let binding, named record field, or array element.",
                target.span,
            ));
        }
        // Replacement selects storage, but never reads its previous scalar.
        // Preserve enclosing loans and permit the existing direct-field rules.
        let prior = context.replacement.replace(target.span);
        let selected = self.mutation_target(context, target, loans, true);
        context.replacement = prior;
        let ty = selected?;
        if !ty.is_copy() {
            return Err(Diagnostic::new(
                "unsupported",
                "Assignment values support only int, number, and boolean; owned assignment statements remain supported.",
                operator,
            ));
        }
        // The final indexed record is reacquired from saved indices after RHS.
        // Preserve loans for its ancestors, rather than holding that final row.
        let selected_parent = match &target.kind {
            ExprKind::Index(base, _) => base.as_ref(),
            ExprKind::Field(base, _) => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            _ => target,
        };
        let active = Self::selection_loans(selected_parent, loans);
        let actual = self.expression(context, value, Mode::Copy, &active, Some(ty))?;
        self.require(actual, Some(ty), value.span)?;
        // RHS effects are unconditional. Recheck the destination after them.
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        if named {
            let id = target.binding.expect("checked scalar assignment target");
            if context.is_capture(id) {
                context.bindings[id].used.get_or_insert(target.span);
                context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
            }
            context.bindings[id].mutable = true;
        }
        Ok(ty)
    }

    fn discover_scalar_assignment(
        &mut self,
        context: &mut Context,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), Diagnostic> {
        let before = context.clone();
        let depth = self.depth;
        let mut target = target.clone();
        let mut value = value.clone();
        let span = target.span;
        if let Err(error) =
            self.scalar_assignment(context, &mut target, &mut value, &Loans::new(), span)
        {
            if error.code != "callable-result-pending" {
                return Err(error);
            }
            *context = before;
            self.depth = depth;
        }
        Ok(())
    }

    fn logical_assignment(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
        loans: &Loans,
        operator: Span,
    ) -> Result<Type, Diagnostic> {
        self.reject_string_position_write(context, target)?;
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Logical assignment requires a let binding, named record field, or array element.",
                target.span,
            ));
        }
        // Read existing availability and forward every enclosing expression loan.
        let ty = self.mutation_target(context, target, loans, false)?;
        self.require(ty, Some(Type::Boolean), operator)?;
        let active = match &target.kind {
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => {
                Self::selection_loans(base, loans)
            }
            _ => loans.clone(),
        };
        // Only availability is conditional; inferred effects/capture contracts accumulate.
        let skipped = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let rhs = self.expression(context, value, Mode::Copy, &active, Some(Type::Boolean))?;
        self.require(rhs, Some(Type::Boolean), operator)?;
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        for (id, before) in skipped {
            before
                .join(Availability::of(&context.bindings[id]))
                .restore(&mut context.bindings[id]);
        }
        if named {
            let id = target.binding.expect("checked logical target");
            if context.is_capture(id) {
                context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
            }
        }
        Ok(Type::Boolean)
    }

    fn compound_assign(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
        op: BinaryOp,
        operator: Span,
    ) -> Result<(), Diagnostic> {
        self.compound_assignment_value(context, target, value, (op, operator), &Loans::new(), true)
            .map(|_| ())
    }

    #[inline(never)]
    fn string_concatenation_operand(&self, context: &Context, value: &mut Expr) -> Result<(), Diagnostic> {
        let Some(ty) = self.expression_type_hint(context, value) else { return Ok(()); };
        let scalar = |ty| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String);
        let supported = ty.union().map_or_else(|| scalar(ty), |id| self.unions.members(id).iter().copied().all(scalar));
        if ty == Type::String { return Ok(()); }
        if !supported { return Err(Diagnostic::new("type", "String concatenation accepts only string or scalar data operands. Convert arrays or records explicitly with String() or a template literal; functions are unsupported.", value.span)); }
        let span = value.span;
        let original = std::mem::replace(value, Expr::new(ExprKind::Undefined, span));
        *value = Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("String".to_owned()), span)), vec![original]), span);
        Ok(())
    }

    fn compound_assignment_value(
        &mut self,
        context: &mut Context,
        target: &mut Expr,
        value: &mut Expr,
        operation: (BinaryOp, Span),
        loans: &Loans,
        allow_owned_statement: bool,
    ) -> Result<Type, Diagnostic> {
        self.reject_string_position_write(context, target)?;
        let (op, operator) = operation;
        let named = matches!(target.kind, ExprKind::Name(_));
        if named {
            self.rebind_target(context, target)?;
        } else if !Self::named_storage(target)
            || !matches!(target.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Compound assignment requires a let binding, named record field, or array element.",
                target.span,
            ));
        }
        // Compound assignment reads the old value; it cannot restore moved storage.
        let ty = self.mutation_target(context, target, loans, false)?;
        if !matches!(ty, Type::Int | Type::Number | Type::String)
            || (ty == Type::String && op != BinaryOp::Add)
        {
            return Err(Diagnostic::new(
                "type",
                "Compound assignment requires the matching implemented int, number, or string operator.",
                operator,
            ));
        }
        if ty == Type::String && !allow_owned_statement {
            return Err(Diagnostic::new(
                "unsupported",
                "Compound assignment values support int and number; standalone string += remains supported.",
                operator,
            ));
        }
        let selected_parent = match &target.kind {
            // Numeric destinations are reacquired after RHS; their final row
            // is not retained as a loan. String old values remain borrowed.
            ExprKind::Field(base, _) if ty != Type::String => match &base.kind {
                ExprKind::Index(parent, _) => parent.as_ref(),
                _ => base.as_ref(),
            },
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => base.as_ref(),
            _ => target,
        };
        let mut active = Self::selection_loans(selected_parent, loans);
        if ty == Type::String {
            let root = Self::root_binding(target).expect("checked compound target");
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, target.span));
        }
        if ty == Type::String { self.string_concatenation_operand(context, value)?; }
        let actual = self.expression(
            context,
            value,
            if ty == Type::String {
                Mode::Read
            } else {
                Mode::Copy
            },
            &active,
            Some(ty),
        )?;
        self.require(actual, Some(ty), operator)?;
        // A copied scalar can permit RHS mutation, but its destination must survive.
        Self::storage_access(context, target, Mode::Mutate, target.span, loans)?;
        if ty == Type::Int && crate::error_effects::integer_operation(op) {
            exception_state::record_point(context, operator);
        }
        if named {
            let id = target.binding.expect("checked compound name");
            if context.is_capture(id) {
                context.bindings[id].mode = context.bindings[id].mode.max(Mode::Mutate);
            }
        }
        Ok(ty)
    }

    fn bind(
        context: &mut Context,
        token: &Token,
        ty: Type,
        parameter: bool,
    ) -> Result<usize, Diagnostic> {
        if context.names.contains_key(&token.text)
            || [
                "clone", "String", "Boolean", "Array", "console", "Number", "Math", "null",
                "typeof", "void",
            ]
            .contains(&token.text.as_str())
        {
            return Err(Diagnostic::new(
                "duplicate",
                format!(
                    "Binding name {:?} is already defined or reserved.",
                    token.text
                ),
                token.span,
            ));
        }
        let index = context.bindings.len();
        context.names.insert(token.text.clone(), index);
        context.source_bindings.insert(token.span.start, index);
        context.bindings.push(Binding { rest: false,
            name: token.text.clone(),
            ty,
            parameter,
            input_ty: None,
            omission: crate::ParameterOmission::Required,
            declaration_span: None,
            initializer: None,
            rebindable: false,
            capture_reference: false,
            borrowed_element: None,
            borrowed_place: None,
            view_roots: Vec::new(),
            callback_invocation: None,
            callback_reference: false,
            callback_kind: None,
            mode: if ty.is_copy() { Mode::Copy } else { Mode::Read },
            mutable: false,
            transferred: None,
            moved_fields: BTreeMap::new(),
            used: None,
        });
        Ok(index)
    }

    // Filter at body boundaries so ownership access keeps its small recursive frame.
    fn filter_ownership_fixes(&mut self, context: &Context, mut error: Diagnostic) -> Diagnostic {
        if error.code != "transferred" {
            return error;
        }
        error.fixes.retain(|fix| {
            if fix.title != "Clone before transferring (copies data; may allocate)" {
                return true;
            }
            let binding = context.bindings.iter().find(|binding| {
                binding.transferred == Some(fix.span)
                    && fix.replacement == format!("clone({})", binding.name)
            });
            binding.is_none_or(|binding| self.require_cloneable(binding.ty, fix.span).is_ok())
        });
        error
    }

    fn transferred(binding: &Binding, span: Span) -> Diagnostic {
        let error = Diagnostic::new(
            match binding.ty {
                Type::Closure(_) | Type::StoredCallable(_) => "closure-consumed",
                Type::Callback(_) => "callback-consumed",
                _ => "transferred",
            },
            format!(
                "{:?} is no longer available because its value was transferred on a path reaching this use.",
                binding.name
            ),
            span,
        )
        .related(
            binding.transferred.expect("a transferred binding"),
            "Value transferred here.",
        );
        if matches!(binding.ty, Type::String | Type::Array(_) | Type::Record(_)) {
            error.fix(
                "Clone before transferring (copies data; may allocate)",
                binding.transferred.expect("a transferred binding"),
                format!("clone({})", binding.name),
            )
        } else {
            error
        }
    }

    fn check_iteration_access(
        context: &Context,
        index: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !context.effects_only
            && let Some(&(_, start)) = context.iteration_loans.iter().find(|&&(id, _)| id == index)
        {
            return Err(Diagnostic::new(
                "iteration-conflict",
                "Cannot mutate, replace, or transfer storage while for...of borrows it.",
                span,
            )
            .related(start, "Iteration borrows here."));
        }
        Ok(())
    }

    fn check_element_access(binding: &Binding, mode: Mode, span: Span) -> Result<(), Diagnostic> {
        if matches!(mode, Mode::Mutate | Mode::Consume)
            && let Some(start) = binding.borrowed_place
        {
            return Err(Diagnostic::new("borrowed-place","A scoped assignment result is readonly and cannot transfer or escape; use clone() for independent ownership.",span).related(start,"Assignment storage borrowed here."));
        }
        if matches!(mode, Mode::Mutate | Mode::Consume)
            && let Some(start) = binding.borrowed_element
        {
            return Err(Diagnostic::new("borrowed-element", "Iteration borrows this owned element for reading; use clone() to obtain an owned value.", span)
                .related(start, "Element borrow starts with this iteration."));
        }
        Ok(())
    }

    fn access(
        context: &mut Context,
        index: usize,
        mode: Mode,
        span: Span,
        loans: &Loans,
    ) -> Result<(), Diagnostic> {
        if matches!(mode, Mode::Mutate | Mode::Consume) {
            if !context.effects_only {
                if matches!(context.bindings[index].ty, Type::BorrowedUnion(_)) {
                    return Err(Diagnostic::new(
                        "borrowed-view",
                        "An optional element view is readonly and cannot escape or transfer ownership; use clone() to own its value.",
                        span,
                    ));
                }
                for &view in context.names.values() {
                    if context.bindings[view].view_roots.contains(&index) {
                        let holder = &context.bindings[view];
                        // A borrowed const names itself; other views keep the element wording.
                        let (message, note) = if holder.borrowed_place.is_some() {
                            (
                                format!("Cannot mutate, replace, or transfer this storage while {:?} borrows part of it. Use the borrow in an inner block, or clone() the part.", holder.name),
                                format!("{:?} borrows here.", holder.name),
                            )
                        } else {
                            (
                                "Cannot mutate, replace, or transfer storage while an optional element view borrows it.".to_owned(),
                                "Optional element view borrows here.".to_owned(),
                            )
                        };
                        return Err(Diagnostic::new("view-conflict", message, span).related(holder.used.unwrap_or(span), note));
                    }
                }
            }
            Self::check_iteration_access(context, index, span)?;
        }
        if matches!(mode, Mode::Mutate | Mode::Consume) {
            let retained = if mode == Mode::Mutate {
                context
                    .keep_flow
                    .iter()
                    .filter_map(|place| {
                        context
                            .flow
                            .get(place)
                            .cloned()
                            .map(|fact| (place.clone(), fact))
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if let Some(target) = context
                .mutation_place
                .as_ref()
                .filter(|place| place.root == index)
            {
                crate::flow::invalidate_place(&mut context.flow, target);
            } else {
                crate::flow::invalidate(&mut context.flow, index);
            }
            context.flow.extend(retained);
        }
        let projection = context
            .projection
            .as_ref()
            .filter(|projection| projection.root == context.bindings[index].name);
        let binding = &mut context.bindings[index];
        if !context.effects_only {
            Self::check_element_access(binding, mode, span)?;
        }
        // Copying a scalar reads it without transferring its binding.
        let mode = if (binding.ty.is_copy() && mode != Mode::Mutate) || mode == Mode::Copy {
            Mode::Read
        } else {
            mode
        };
        Self::availability(
            binding,
            projection.map(|p| (p.path.as_slice(), p.replacing)),
            projection.map_or(span, |p| p.span),
            context.effects_only,
        )?;
        for &(captured, previous, creation) in &context.capture_loans {
            if !context.effects_only
                && captured == index
                && (mode != Mode::Read || previous != Mode::Read)
            {
                return Err(Diagnostic::new(
                    "capture-conflict",
                    format!(
                        "Cannot access {:?} while a closure holds conflicting access.",
                        binding.name
                    ),
                    span,
                )
                .related(creation, "Closure capture starts here."));
            }
        }
        binding.used.get_or_insert(span);
        if !context.effects_only
            && let Some(active) = loans.get(&index)
        {
            for (previous, previous_span) in active {
                if mode != Mode::Read || *previous != Mode::Read {
                    return Err(Diagnostic::new("conflicting-access", format!("Cannot access {:?} through overlapping shared, exclusive, or consuming access.", binding.name), span)
                        .related(*previous_span, "Earlier access starts here."));
                }
            }
        }
        if binding.parameter && (!binding.ty.is_copy() || mode == Mode::Mutate) {
            // Moving a reusable callback transfers the local &mut reference,
            // not ownership of the caller's environment.
            let parameter_mode = if binding.callback_reference && mode == Mode::Consume {
                Mode::Mutate
            } else {
                mode
            };
            binding.mode = binding.mode.max(parameter_mode);
        }
        if mode == Mode::Mutate {
            binding.mutable = true;
        }
        if mode == Mode::Consume {
            if let Some(projection) = projection {
                context.has_field_moves = true;
                binding
                    .moved_fields
                    .retain(|path, _| !path.starts_with(&projection.path));
                binding
                    .moved_fields
                    .insert(projection.path.clone(), projection.span);
            } else {
                binding.transferred = Some(span);
            }
        }
        Ok(())
    }

    fn direct_fields(expr: &Expr) -> Option<(String, Vec<String>)> {
        let mut current = expr;
        let mut path = Vec::new();
        loop {
            match &current.kind {
                ExprKind::Sequence(values)=>current=values.last().expect("nonempty sequence"),
                ExprKind::Field(base, name) => {
                    path.push(name.clone());
                    current = base;
                }
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. }
                | ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    value,
                    ..
                } => current = value,
                _ => break,
            }
        }
        let ExprKind::Name(name) = &current.kind else {
            return None;
        };
        path.reverse();
        Some((name.clone(), path))
    }

    fn availability(
        binding: &Binding,
        projection: Option<(&[String], bool)>,
        span: Span,
        effects_only: bool,
    ) -> Result<(), Diagnostic> {
        if effects_only {
            return Ok(());
        }
        if binding.transferred.is_some() {
            return Err(Self::transferred(binding, span));
        }
        let missing = binding
            .moved_fields
            .iter()
            .find(|(missing, _)| match projection {
                None => true,
                Some((path, true)) => missing.len() < path.len() && path.starts_with(missing),
                Some((path, false)) => path.starts_with(missing) || missing.starts_with(path),
            });
        if let Some((path, origin)) = missing {
            return Err(Diagnostic::new("partial-transfer", format!("Cannot access {:?}: field {} was transferred on a path reaching this use. Restore the field before using it or the whole record.", binding.name, path.join(".")), span).related(*origin, "Field transferred here."));
        }
        Ok(())
    }

    fn loan_root(expr: &Expr, mode: Mode) -> Option<usize> {
        if mode == Mode::Consume && matches!(expr.kind, ExprKind::Field(_, _)) {
            let mut root = expr;
            while let ExprKind::Field(base, _) = &root.kind {
                root = base;
            }
            if matches!(root.kind, ExprKind::Name(_)) {
                return None;
            }
        }
        Self::root_binding(expr)
    }

    fn loan_roots(expr: &Expr, mode: Mode) -> Vec<usize> {
        fn collect(expr: &Expr, mode: Mode, roots: &mut Vec<usize>) {
            if let ExprKind::Sequence(values)=&expr.kind {
                if let Some(last)=values.last(){collect(last,mode,roots);}
            } else if let ExprKind::Selected { source, .. } = &expr.kind {
                for root in &source.roots {
                    if !roots.contains(root) {
                        roots.push(*root);
                    }
                }
            } else if let ExprKind::Optional { body, .. } = &expr.kind {
                collect(body, mode, roots);
            } else if matches!(
                expr.target,
                Some(CallTarget::MapMethod(MapMethod::Get) | CallTarget::ArrayAt | CallTarget::ArrayFind | CallTarget::ArrayFindLast)
            ) && matches!(expr.ty, Some(Type::BorrowedUnion(_)))
            {
                if let ExprKind::Call(callee, _) = &expr.kind
                    && let ExprKind::Field(base, _) = &callee.kind
                {
                    collect(base, Mode::Read, roots);
                }
            } else if let ExprKind::CoalesceAssign { target, .. } = &expr.kind {
                if !expr.ty.is_some_and(|ty| ty.owning().is_copy()) {
                    collect(target, Mode::Read, roots);
                }
            } else if let ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } = &expr.kind {
                collect(value, mode, roots);
            } else if let ExprKind::Inject { value, .. } = &expr.kind {
                if matches!(expr.ty, Some(Type::BorrowedUnion(_)))
                    || value.ty.is_some_and(Type::is_copy)
                {
                    collect(value, mode, roots);
                }
            } else if let ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull | UnaryOp::ParameterMutableInput,
                value,
                ..
            } = &expr.kind
            {
                collect(value, mode, roots);
            } else if let ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                left,
                right,
                ..
            } = &expr.kind
            {
                collect(left, mode, roots);
                collect(right, mode, roots);
            } else if let ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } = &expr.kind
            {
                collect(consequent, mode, roots);
                collect(alternate, mode, roots);
            } else if let Some(root) = Analyzer::loan_root(expr, mode)
                && !roots.contains(&root)
            {
                roots.push(root);
            }
        }
        let mut roots = Vec::new();
        collect(expr, mode, &mut roots);
        roots
    }

    fn storage_access(
        context: &mut Context,
        expr: &Expr,
        mode: Mode,
        span: Span,
        loans: &Loans,
    ) -> Result<(), Diagnostic> {
        let mut current = expr;
        let first_keep = context.keep_flow.len();
        loop {
            match &current.kind {
                ExprKind::Sequence(values)=>current=values.last().expect("nonempty sequence"),
                ExprKind::Project {
                    value,
                    union,
                    narrowed,
                } => {
                    if current.ty != Some(Type::Union(*union)) {
                        let place =
                            crate::flow::place(value, &context.names).expect("stable projection");
                        let fact=context.flow.get(&place).ok_or_else(||Diagnostic::new("narrowing-invalidated","The nullable guard no longer holds after the preceding effects; guard the value again.",span))?;
                        if !fact.members.iter().all(|ty| ty == narrowed)
                            && !matches!(narrowed, Type::Union(_))
                        {
                            return Err(Diagnostic::new(
                                "narrowing-invalidated",
                                "The nullable guard no longer proves this payload.",
                                span,
                            ));
                        }
                        context.keep_flow.push(place);
                    }
                    current = value;
                }
                ExprKind::Field(base, _) | ExprKind::Index(base, _) => current = base,
                _ => break,
            }
        }
        let previous = context.mutation_place.clone();
        context.mutation_place = crate::flow::mutation_footprint(expr, &context.names);
        let result = Self::storage_access_inner(context, expr, mode, span, loans);
        context.mutation_place = previous;
        context.keep_flow.truncate(first_keep);
        result
    }

    fn storage_access_inner(
        context: &mut Context,
        expr: &Expr,
        mode: Mode,
        span: Span,
        loans: &Loans,
    ) -> Result<(), Diagnostic> {
        let mut selected = expr;
        loop {
            match &selected.kind {
                ExprKind::Sequence(values) => selected = values.last().expect("nonempty sequence"),
                ExprKind::Field(base, _) | ExprKind::Index(base, _)
                | ExprKind::Project { value: base, .. } => selected = base,
                _ => break,
            }
        }
        if let ExprKind::SpreadValue(value)=&selected.kind {
            if value.source.readonly && mode==Mode::Mutate {
                return Err(Diagnostic::new("borrowed-view","A shared staged receiver cannot be mutated.",span));
            }
            if !value.owned && mode==Mode::Consume && !expr.ty.is_some_and(Type::is_copy) {
                return Err(Diagnostic::new("borrowed-place","A staged receiver borrows its original storage and cannot be transferred.",span));
            }
            if value.owned {return Ok(());}
            if let Some((root,fields))=&value.source.place {
                let prior=context.projection.take();
                if !fields.is_empty() {context.projection=Some(Projection{root:context.bindings[*root].name.clone(),path:fields.clone(),replacing:false,span});}
                let result=Self::access(context,*root,mode,span,loans);context.projection=prior;return result;
            }
            for root in &value.source.roots {Self::access(context,*root,mode,span,loans)?;}
            return Ok(());
        }
        if let ExprKind::Selected {
            slot,
            source,
            owned,
            ..
        } = &selected.kind
        {
            context
                .selected_access
                .entry(*slot)
                .and_modify(|prior| *prior = (*prior).max(mode))
                .or_insert(mode);
            if source.readonly && mode == Mode::Mutate {
                return Err(Diagnostic::new(
                    "borrowed-view",
                    "An optional readonly view cannot be mutated; use clone() for independent ownership.",
                    span,
                ));
            }
            if !owned && mode == Mode::Consume && !expr.ty.is_some_and(Type::is_copy) {
                return Err(Diagnostic::new(
                    "borrowed-place",
                    "An optional selection borrows its source; use clone() for independent ownership.",
                    span,
                ));
            }
            if *owned {
                return Ok(());
            }
            if source.roots.len() > 1 {
                for root in &source.roots {
                    Self::access(context, *root, mode, span, loans)?;
                }
                return Ok(());
            }
        }
        let Some(id) = Self::root_binding(expr) else {
            // Owning temporary selections do not alias source storage.
            return Ok(());
        };
        if context.bindings[id].moved_fields.is_empty() {
            return Self::access(context, id, mode, span, loans);
        }
        let mut current = expr;
        let mut steps = Vec::new();
        loop {
            match &current.kind {
                ExprKind::Sequence(values)=>current=values.last().expect("nonempty sequence"),
                ExprKind::Project { value, .. } | ExprKind::Unary { op: UnaryOp::NonNull, value, .. } => current = value,
                ExprKind::Field(base, name) => {
                    steps.push(Some(name.clone()));
                    current = base;
                }
                ExprKind::Index(base, _) => {
                    steps.push(None);
                    current = base;
                }
                _ => break,
            }
        }
        // No field inside an indexed element can be partially moved. Checking
        // the complete first array therefore covers every later selection.
        let path = steps
            .into_iter()
            .rev()
            .take_while(Option::is_some)
            .flatten()
            .collect::<Vec<_>>();
        let prior = context.projection.take();
        if !path.is_empty() {
            context.projection = Some(Projection {
                root: context.bindings[id].name.clone(),
                path,
                replacing: false,
                span,
            });
        }
        let result = Self::access(context, id, mode, span, loans);
        context.projection = prior;
        result
    }

    fn named_storage(mut expr: &Expr) -> bool {
        loop {
            match &expr.kind {
                ExprKind::Sequence(values)=>expr=values.last().expect("nonempty sequence"),
                ExprKind::Index(base, _) | ExprKind::Field(base, _) => expr = base,
                ExprKind::Project { value, .. }
                | ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    value,
                    ..
                } => expr = value,
                _ => break,
            }
        }
        matches!(expr.kind, ExprKind::Name(_))
            || matches!(&expr.kind, ExprKind::Selected { source, .. } if source.place.is_some() || !source.roots.is_empty())
            || matches!(&expr.kind, ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
    }

    fn root_binding(expr: &Expr) -> Option<usize> {
        match &expr.kind {
            ExprKind::Sequence(values)=>values.last().and_then(Self::root_binding),
            ExprKind::SpreadValue(selected) => selected.source.place.as_ref().map(|(root,_)|*root).or_else(||selected.source.roots.first().copied()),
            ExprKind::Selected { source, .. } => source
                .place
                .as_ref()
                .map(|(root, _)| *root)
                .or_else(|| source.roots.first().copied()),
            ExprKind::Name(_) => expr.binding,
            ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } | ExprKind::Inject {value,..}
            | ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull | UnaryOp::ParameterMutableInput,
                value,
                ..
            } => Self::root_binding(value),
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => Self::root_binding(base),
            _ => None,
        }
    }

    fn parameter_argument(
        &mut self,
        context: &mut Context,
        argument: &mut Expr,
        contract: ArgumentContract,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        if !contract.omission.is_omittable() && !(contract.mode==Mode::Mutate && matches!(contract.ty,Type::Union(id) if !id.copy) && self.unions.payload(contract.ty).is_none()) {
            return self.expression(context, argument, contract.mode, loans, Some(contract.ty));
        }
        self.omittable_parameter_argument(context, argument, contract, loans)
    }

    fn omittable_parameter_argument(&mut self, context: &mut Context, argument: &mut Expr, contract: ArgumentContract, loans: &Loans) -> Result<Type,Diagnostic> {
        if contract.mode == Mode::Mutate
            && let Type::Union(id) = contract.ty && !id.copy
            && (contract.omission.is_omittable() || self.unions.payload(contract.ty).is_none()) {
            let hint = self.expression_type_hint(context, argument);
            let expected = if hint.is_some_and(|ty| ty.union().is_some()) {None}
                else {hint.filter(|ty| matches!(ty,Type::Null|Type::Undefined)).or_else(||self.unions.payload(contract.ty))};
            let access = if hint.is_some_and(|ty| matches!(ty,Type::Null|Type::Undefined)) {Mode::Copy} else {Mode::Mutate};
            let actual = self.expression(context, argument, access, loans, expected)?;
            self.require(actual, Some(contract.ty), argument.span)?;
            let span = argument.span;
            let value = std::mem::replace(argument, Expr::new(ExprKind::Undefined, span));
            *argument = Expr::new(ExprKind::Unary {op:UnaryOp::ParameterMutableInput,value:Box::new(value),operator:span},span);
            let physical = if let Some(Type::Callback(callback)) = self.unions.payload(contract.ty) {
                let payload = if matches!(actual,Type::Null|Type::Undefined) {self.absent_callback(callback)} else {self.unions.payload(actual).unwrap_or(actual)};
                self.unions.intern(self.unions.members(id).into_iter().map(|member|if matches!(member,Type::Callback(_)){payload}else{member}),span)?
            } else {contract.ty};
            argument.ty = Some(physical);
            return Ok(physical);
        }
        if contract.omission.is_omittable() && contract.mode==Mode::Read
            && let Type::Union(id)=contract.ty && !id.copy
            && !matches!(argument.kind,ExprKind::Conditional{..}) {
                let hint=self.expression_type_hint(context,argument);
                let expected=if hint.is_some_and(|ty|ty.union().is_some()) {None} else {hint.filter(|ty|matches!(ty,Type::Null|Type::Undefined)).or_else(||self.unions.payload(contract.ty))};
                let actual=self.expression(context,argument,Mode::Read,loans,expected)?;
                self.require(actual,Some(contract.ty),argument.span)?;
                argument.inject(id);argument.ty=Some(Type::BorrowedUnion(id));
                return Ok(Type::BorrowedUnion(id));
        }
        self.expression(context,argument,contract.mode,loans,Some(contract.ty))
    }

    fn pack_rest_arguments(args: &mut Vec<Expr>, fixed: usize, span: Span) {
        if args.len() == fixed + 1
            && args.last().is_some_and(|argument| {
                matches!(
                    argument.kind,
                    ExprKind::Unary {
                        op: UnaryOp::RestPack,
                        ..
                    }
                )
            })
        {
            return;
        }
        let tail = if args.len() > fixed {
            args.split_off(fixed)
        } else {
            Vec::new()
        };
        while args.len() < fixed {
            args.push(Expr::new(ExprKind::Undefined, span));
        }
        let range = tail.first().zip(tail.last()).map_or(span, |(first, last)| {
            Span::new(first.span.start, last.span.end)
        });
        let value = Expr::new(ExprKind::Array(tail), range);
        args.push(Expr::new(
            ExprKind::Unary {
                op: UnaryOp::RestPack,
                value: Box::new(value),
                operator: range,
            },
            range,
        ));
    }

    fn prepare_arguments(
        args: &mut Vec<Expr>,
        params: &[ArgumentContract],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let rest = params.last().is_some_and(|parameter| parameter.rest);
        let fixed = params.len() - usize::from(rest);
        let minimum = crate::parameter_arity::minimum_arguments(&params[..fixed], |parameter| {
            parameter.omission.is_omittable()
        });
        let packed = rest
            && args.len() == params.len()
            && args.last().is_some_and(|argument| {
                matches!(
                    argument.kind,
                    ExprKind::Unary {
                        op: UnaryOp::RestPack,
                        ..
                    }
                )
            });
        if args.len() < minimum || (!rest && args.len() > fixed) {
            let maximum = if rest {
                "unbounded".into()
            } else {
                fixed.to_string()
            };
            return Err(Diagnostic::new(
                "arity",
                format!(
                    "Expected {minimum}..={maximum} arguments, found {}.",
                    args.len()
                ),
                span,
            ));
        }
        if rest && !packed {
            Self::pack_rest_arguments(args, fixed, span);
        }
        while args.len() < params.len() {
            args.push(Expr::new(ExprKind::Undefined, span));
        }
        Ok(())
    }

    fn arity(count: usize, expected: usize, span: Span) -> Result<(), Diagnostic> {
        if count != expected {
            return Err(Diagnostic::new(
                "arity",
                format!("Expected {expected} arguments, found {count}."),
                span,
            ));
        }
        Ok(())
    }

    fn mutation_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
        place: crate::flow::Place,
    ) -> Result<Type, Diagnostic> {
        context.mutation_place = Some(place);
        let result = self.expression(context, expr, Mode::Mutate, loans, expected);
        context.mutation_place = None;
        result
    }

    // Already planned spread nodes keep their Result temporaries out of the
    // shared recursive expression frame.
    #[inline(never)]
    fn checked_spread_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        match &mut expr.kind {
            ExprKind::SpreadValue(_) => self.spread_value_expression(context, expr, mode, expected),
            ExprKind::SpreadCall(_) => {
                let ty = expr.ty.expect("checked spread call result");
                self.require(ty, expected, expr.span)?;
                Ok(ty)
            }
            ExprKind::Unary { value, .. } => {
                let ty = self.spread_value_expression(context, value, Mode::Consume, expected)?;
                expr.ty = Some(ty);
                Ok(ty)
            }
            _ => unreachable!("checked spread expression"),
        }
    }

    #[inline(never)]
    fn lower_enum_member(&self, context: &Context, expr: &mut Expr, mode: Mode) -> Result<(), Diagnostic> {
        let ExprKind::Field(base, member) = &expr.kind else { return Ok(()); };
        let ExprKind::Name(name) = &base.kind else { return Ok(()); };
        if context.names.contains_key(name) { return Ok(()); }
        let Some(&id) = self.record_names.get(name) else { return Ok(()); };
        let Some(definition) = &self.program.records[id].enum_definition else { return Ok(()); };
        if mode == Mode::Mutate { return Err(Diagnostic::new("enum", "Enum members are read-only constants.", expr.span)); }
        let Some(index) = definition.members.iter().position(|value| &value.name.text == member) else {
            return Err(Diagnostic::new("enum", format!("Unknown enum member {name}.{member}."), expr.span));
        };
        expr.kind = ExprKind::EnumMember { enumeration: id, member: index };
        Ok(())
    }

    #[inline(never)]
    fn type_assertion_expression(
        &mut self, context: &mut Context, expr: &mut Expr, mode: Mode,
        loans: &Loans, expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::TypeAssertion { value, annotation, satisfies } = &mut expr.kind else { unreachable!() };
        let target = self.callable_resolver().resolve_value_annotation(annotation, false)?;
        let numeric_widening = target == Type::Number
            && self.expression_type_hint(context, value) == Some(Type::Int);
        let needs_context = matches!(&value.kind, ExprKind::Record(_) | ExprKind::Closure { .. })
            || matches!(&value.kind, ExprKind::Array(items) if items.is_empty());
        let contextual = ((!*satisfies || needs_context) && !numeric_widening).then_some(target);
        let borrowed_injection = !*satisfies && mode == Mode::Read
            && matches!(target, Type::Union(id) if !id.copy)
            && self.expression_type_hint(context, value).is_some_and(|ty| ty.union().is_none())
            && (self.borrowed_place_hint(context, value) || self.indexed_element_hint(context, value));
        let actual = if borrowed_injection {
            let actual = self.expression(context, value, mode, loans, None)?;
            self.require(actual, Some(target), annotation.span)?;
            let union = target.union().expect("borrowed union target");
            value.inject(union);
            value.ty = Some(Type::BorrowedUnion(union));
            Type::BorrowedUnion(union)
        } else {
            self.expression(context, value, mode, loans, contextual)?
        };
        if !(actual == Type::Int && target == Type::Number) {
            self.require(actual, Some(target), annotation.span).map_err(|mut error| {
                error.message.push_str(" Type assertions cannot narrow an incompatible value; use typeof to narrow a union.");
                error
            })?;
        }
        let result = if *satisfies || matches!(actual, Type::BorrowedUnion(_)) || matches!((actual, target), (Type::Closure(_), Type::Callback(_))) { actual } else { target };
        self.require(result, expected, expr.span)?;
        expr.binding = value.binding;
        expr.callable_origin = value.callable_origin;
        expr.ty = Some(result);
        if let Some(Type::Union(id)) = expected {
            if result != Type::Union(id) {
                expr.inject(id);
                expr.ty = Some(Type::Union(id));
                return Ok(Type::Union(id));
            }
        }
        Ok(result)
    }

    #[inline(never)]
    fn lower_scalar_unary_plus(&self, context: &Context, expr: &mut Expr) {
        let ExprKind::Unary { op: UnaryOp::Positive, value, .. } = &mut expr.kind else { return; };
        if self.expression_type_hint(context, value).is_none_or(|ty| matches!(ty, Type::Int | Type::Number)) { return; }
        let span = expr.span;
        let operand = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
        expr.kind = ExprKind::Call(Box::new(Expr::new(ExprKind::Name("Number".to_owned()), span)), vec![*operand]);
    }

    fn expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        if matches!(expr.kind,ExprKind::Sequence(_)){return self.sequence_expression(context,expr,mode,loans,expected,false);}
        if let Some(expected) = expected
            && self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).is_some()
        {
            if self.lower_selected_generic_factory(context, expr)? {
                return self.stage_selected_generic_expression(context, expr, expected, loans);
            }
        }
        if let Some(expected) = expected
            && self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).is_some()
            && self.fresh_generic_origin(context, expr)?.is_some()
        {
            return self.stage_generic_expression(context, expr, expected, mode, loans);
        }
        self.lower_enum_member(context, expr, mode)?;
        if matches!(&expr.kind,ExprKind::TypeAssertion{..}) {return self.type_assertion_expression(context,expr,mode,loans,expected);}
        if matches!(&expr.kind, ExprKind::SpreadValue(_) | ExprKind::SpreadCall(_))
            || matches!(&expr.kind, ExprKind::Unary { op: UnaryOp::RestPack, value, .. } if matches!(value.kind, ExprKind::SpreadValue(_)))
        {
            return self.checked_spread_expression(context, expr, mode, expected);
        }
        if let ExprKind::Unary {
            op: UnaryOp::RestPack,
            value,
            ..
        } = &mut expr.kind
        {
            let ty = if let ExprKind::Array(values) = &mut value.kind {
                self.array_literal(context, values, loans, expected, expr.span)?
            } else {
                self.expression(context, value, mode, loans, expected)?
            };
            value.ty = Some(ty);
            expr.ty = Some(ty);
            return Ok(ty);
        }
        // A declared nongeneric call already returns its exact tagged type.
        // Do not retain an injection frame while checking its nested arguments.
        let expected = if matches!(expected, Some(Type::Union(_)))
            && matches!(&expr.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Name(name) if !context.names.contains_key(name) && self.function_names.contains_key(name) && self.generic_functions.template(name).is_none()))
            && self.expression_type_hint(context, expr) == expected
        { None } else { expected };
        self.lower_scalar_unary_plus(context, expr);
        self.lower_tuple_expression(context, expr, expected)?;
        self.checked_expression(context, expr, mode, loans, expected)
    }

    fn lower_tuple_expression(&self, context: &Context, expr: &mut Expr, expected: Option<Type>) -> Result<(), Diagnostic> {
        match &mut expr.kind {
            ExprKind::Array(values) => {
                if let Some(Type::Record(id)) = expected && self.program.records[id].is_tuple() {
                    let record = &self.program.records[id];
                    if values.len() != record.fields.len() {
                        return Err(Diagnostic::new("type", format!("Tuple requires {} positions, received {}.", record.fields.len(), values.len()), expr.span));
                    }
                    let fields = std::mem::take(values).into_iter().enumerate().map(|(index,value)|
                        (Token { kind: TokenKind::Symbol, text: index.to_string(), span:value.span }, value)).collect();
                    expr.kind = ExprKind::Record(fields);
                }
            }
            ExprKind::Index(base,index) => {
                if let Some(Type::Record(id)) = self.expression_type_hint(context, base) && self.program.records[id].is_tuple() {
                    let Some(position) = Self::flat_literal_depth(index) else {
                        return Err(Diagnostic::new("tuple-index", "A tuple index must be a constant integer position.", index.span));
                    };
                    if position < 0 || position as usize >= self.program.records[id].fields.len() {
                        return Err(Diagnostic::new("bounds", "Tuple index is outside its declared positions.", index.span));
                    }
                    let placeholder = Box::new(Expr::new(ExprKind::Undefined,base.span));
                    let receiver = std::mem::replace(base,placeholder);
                    expr.kind = ExprKind::Field(receiver,position.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    // Rare early outcomes and result bookkeeping own their temporaries in
    // dedicated helpers; this frame is retained across expression nesting.
    fn checked_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        if let Some(ty) = self.checked_expression_prelude(context, expr, mode, loans, expected)? {
            return Ok(ty);
        }
        if self.depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Combined analysis nesting exceeds the prototype limit of 128.",
                expr.span,
            ));
        }
        self.depth += 1;
        let invocation=matches!(expr.kind,ExprKind::Call(_, _));
        let bounds_or_nonnull=matches!(expr.kind,ExprKind::Index(_,_)|ExprKind::Unary{op:UnaryOp::NonNull,..});
        let integer_operation=matches!(&expr.kind,ExprKind::Unary{op:UnaryOp::Negate,..}) || matches!(&expr.kind,ExprKind::Binary{op,..} if crate::error_effects::integer_operation(*op));
        let result = if invocation {
            self.expression_call(context, expr, loans, expected)
        } else {
            self.expression_kind(context, expr, mode, loans, expected)
        };
        if result.is_ok() && (invocation || bounds_or_nonnull || (integer_operation && result.as_ref().is_ok_and(|ty|*ty==Type::Int))) {exception_state::record_point(context,expr.span);}
        let result = self.checked_expression_result(context, expr, mode, expected, result);
        self.depth -= 1;
        result
    }

    #[inline(never)]
    fn checked_expression_prelude(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        if mode == Mode::Consume
            && let Some(Type::StoredCallable(family)) = expected
        {
            if !matches!(expr.kind, ExprKind::Conditional { .. })
                && self.expression_type_hint(context, expr) != expected
            {
                return (self.stored_callable_value(context, expr, family, loans)).map(Some);
            }
        }
        if mode == Mode::Mutate
            && context.mutation_place.is_none()
            && let Some(place) = crate::flow::mutation_footprint(expr, &context.names)
        {
            return (self.mutation_expression(context, expr, loans, expected, place)).map(Some);
        }
        context
            .condition_flows
            .remove(&(expr.span.start, expr.span.end));
        if !matches!(
            expr.kind,
            ExprKind::Project { .. } | ExprKind::Selected { .. }
        ) && let Some((place, union, narrowed)) = self.flow_view(context, expr)
        {
            return (self
                .project_expression(context, expr, mode, loans, expected, place, union, narrowed)).map(Some);
        }
        if let Some(Type::BorrowedUnion(id)) = expected {
            if mode != Mode::Read {
                return (Err(Diagnostic::new(
                    "borrowed-view",
                    "Borrowed optional views cannot be consumed or escape; use clone() for independent ownership.",
                    expr.span,
                ))).map(Some);
            }
            let actual = self.expression(context, expr, Mode::Read, loans, None)?;
            self.require(actual, Some(Type::Union(id)), expr.span)?;
            if !matches!(
                actual,
                Type::BorrowedUnion(_) | Type::Null | Type::Undefined
            ) {
                return (Err(Diagnostic::new(
                    "borrowed-view",
                    "A borrowed view branch requires another view or an absent sentinel; clone the view for an owning join.",
                    expr.span,
                ))).map(Some);
            }
            if actual != Type::BorrowedUnion(id) {
                expr.inject(id);
                expr.ty = Some(Type::BorrowedUnion(id));
            }
            return (Ok(Type::BorrowedUnion(id))).map(Some);
        }
        if let Some(Type::Union(id)) = expected
            && !matches!(
                expr.kind,
                ExprKind::Conditional { .. }
                    | ExprKind::Binary {
                        op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                        ..
                    }
                    | ExprKind::Inject { .. }
                    | ExprKind::Project { .. }
            )
        {
            return (self.inject_expression(context, expr, mode, loans, id)).map(Some);
        }
        Ok(None)
    }

    #[inline(never)]
    fn checked_expression_result(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        expected: Option<Type>,
        result: Result<Type, Diagnostic>,
    ) -> Result<Type, Diagnostic> {
        let result = match result {
            Ok(actual)=>self.rest_callable_adapter(context,expr,actual,expected,mode),
            Err(error)=>Err(error),
        };
        let result = if result
            .as_ref()
            .is_ok_and(|ty| matches!(ty, Type::BorrowedUnion(_)))
            && mode != Mode::Read
        {
            Err(Diagnostic::new(
                "borrowed-view",
                "Optional element views require readonly use; use clone() to own the selected value.",
                expr.span,
            ))
        } else {
            result
        };
        if let Ok(ty) = &result {
            expr.ty = Some(*ty);
            if *ty == Type::Boolean {
                context
                    .condition_flows
                    .entry((expr.span.start, expr.span.end))
                    .or_insert_with(|| (context.flow.clone(), context.flow.clone()));
            }
        }
        result
    }

    fn non_null_assertion(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let ExprKind::Unary { value, operator, .. } = &mut expr.kind else { unreachable!() };
        let actual = self.expression(context, value, mode, loans, None)?;
        let result = if let Some(union) = actual.union() {
            let present = self.unions.members(union).into_iter().filter(|ty| !matches!(ty,Type::Null|Type::Undefined)).collect::<Vec<_>>();
            if present.is_empty() { return Err(Diagnostic::new("non-null", "This value can only be null or undefined; `!` cannot produce a present value. Use `??` with a fallback instead.", *operator)); }
            let payload = self.unions.intern(present, *operator)?;
            if matches!(actual, Type::BorrowedUnion(_)) && !payload.is_copy() && mode != Mode::Read {
                return Err(Diagnostic::new("borrowed-view", "A non-null assertion does not make a borrowed payload owned. Use clone() to retain it.", *operator));
            }
            let placeholder = value.clone();
            let source = std::mem::replace(value, placeholder);
            let mut projected = Expr::new(ExprKind::Project { value: source, union, narrowed: payload }, value.span);
            projected.ty = Some(payload);
            *value = Box::new(projected);
            payload
        } else if matches!(actual, Type::Null | Type::Undefined | Type::Void) {
            return Err(Diagnostic::new("non-null", "This value has no present payload; `!` cannot produce one. Use `??` with a fallback instead.", *operator));
        } else { actual };
        self.require(result, expected, expr.span)?;
        expr.binding = value.binding;
        Ok(result)
    }

    #[inline(never)]
    fn wrapper_expression(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>, previous_view: Option<Type>) -> Result<Type, Diagnostic> {
        match &mut expr.kind {
            ExprKind::Unary {
                op: UnaryOp::CallableStore,
                value,
                ..
            } => {
                let Type::StoredCallable(family) = expr.ty.expect("checked storage wrapper") else {
                    unreachable!("stored wrapper type");
                };
                let actual = self.expression(context, value, Mode::Consume, loans, None)?;
                let Type::Closure(origin) = actual else {
                    return Err(Diagnostic::new(
                        "callable-layout",
                        "Stored callable construction requires a concrete owned environment.",
                        value.span,
                    ));
                };
                if !self.discovering_results && !self.effects_only {
                    self.callables.storage.register_origin(
                        family,
                        origin,
                        &self.callables.callbacks,
                        &self.callables.shapes,
                        &self.closures,
                        expr.span,
                    )?;
                } else if self.speculative_access_run && !self.discovering_results {
                    self.callables.storage.note_speculative_origin(family, origin);
                }
                Ok(Type::StoredCallable(family))
            }
            ExprKind::Unary {op:UnaryOp::CallableRestAdapter,value,..} => {
                let Type::Closure(id)=expr.ty.expect("adapter type") else {unreachable!()};
                let access=self.closures[id].captures[0].storage;
                self.expression(context,value,access,loans,None)?;
                if !self.discovering_results {self.validated_closures.insert(id);}
                Ok(Type::Closure(id))
            }
            ExprKind::Unary {
                op: UnaryOp::CallableOnce,
                value,
                ..
            } => {
                let actual = self.expression(context, value, Mode::Consume, loans, None)?;
                let Type::Closure(origin) = actual else {
                    unreachable!("checked once environment");
                };
                Ok(Type::Closure(self.once_return(origin, context, expr.span)))
            }
            ExprKind::Unary {op:UnaryOp::ParameterDefaultStore,value,..} => {
                self.expression(context,value,Mode::Consume,loans,None)?;
                Ok(expr.ty.expect("checked default storage"))
            }
            ExprKind::Unary {
                op: UnaryOp::ParameterMutableInput,
                value,
                ..
            } => {
                let actual = self.expression(context,value,Mode::Mutate,loans,None)?;
                self.require(actual,expected,expr.span)?;
                Ok(expr.ty.expect("checked mutable input"))
            }
            ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected,
                value,
                ..
            } => self.expression(context, value, mode, loans, expected),
            ExprKind::Project {
                value,
                union,
                narrowed,
            } => {
                let place = crate::flow::place(value, &context.names);
                if let Some(place) = &place {
                    context.suppressed_flow.insert(place.clone());
                }
                let actual =
                    self.expression(context, value, mode, loans, Some(Type::Union(*union)));
                if let Some(place) = &place {
                    context.suppressed_flow.remove(place);
                }
                actual?;
                Ok(previous_view.unwrap_or(*narrowed))
            }
            ExprKind::Inject { value, union } => {
                let actual = self.expression(context, value, mode, loans, None)?;
                self.require(actual, Some(Type::Union(*union)), value.span)?;
                Ok(Type::Union(*union))
            }
            _ => self.expression_inner(context, expr, mode, loans, expected),
        }
    }

    fn expression_kind(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        // Aggregate nesting must not retain the larger general-expression frame.
        let previous_view = expr.ty;
        match &mut expr.kind {
            ExprKind::Unary { op: UnaryOp::PatternSelection, .. } => self.pattern_selection(context,expr,mode,loans,expected),
            ExprKind::Unary { op: UnaryOp::NonNull, .. } => self.non_null_assertion(context, expr, mode, loans, expected),
            ExprKind::OptionalChain(_) => self.optional_chain(context, expr, mode, loans, expected),
            ExprKind::Selected { .. } => {
                self.selected_expression(context, expr, mode, loans, expected)
            }
            ExprKind::CoalesceAssign { .. } => {
                self.coalesce_assignment(context, expr, mode, loans, expected)
            }
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                ..
            } => self.coalesce(context, expr, mode, loans, expected),
            ExprKind::Binary {
                op,
                operator,
                left,
                right,
            } if matches!(op, BinaryOp::And | BinaryOp::Or) => {
                if self.expression_type_hint(context, left) == Some(Type::Boolean)
                    && self.expression_type_hint(context, right).is_none_or(|ty| ty == Type::Boolean) {
                    self.logical_expression(context, (expr.span, *operator), *op, left, right, loans)
                } else { self.logical_value_expression(context, expr, mode, loans, expected) }
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => self.conditional(
                context,
                predicate,
                (consequent, alternate),
                mode,
                loans,
                expected,
            ),
            ExprKind::CompoundAssign {
                target,
                value,
                op,
                operator,
            } => self.compound_assignment_value(
                context,
                target,
                value,
                (*op, *operator),
                loans,
                false,
            ),
            ExprKind::Assign {
                target,
                value,
                operator,
            } => self.scalar_assignment(context, target, value, loans, *operator),
            ExprKind::LogicalAssign {
                target,
                value,
                operator,
                ..
            } => self.logical_assignment(context, target, value, loans, *operator),
            ExprKind::Update {
                target, operator, ..
            } => self.update_expression(context, target, loans, *operator),
            ExprKind::Record(fields) if fields.iter().any(|(key,_)|key.text=="record#spread") => self.object_spread(context,expr,loans,expected),
            ExprKind::Record(fields) => {
                self.record_literal(context, fields, loans, expected, expr.span)
            }
            ExprKind::ArrayOf(_, _) => self.array_constructor(context, expr, loans, expected),
            ExprKind::SetConstructor(_, _) => self.set_constructor(context, expr, loans, expected),
            ExprKind::MapConstructor(_, _) => self.map_constructor(context, expr, loans, expected),
            ExprKind::Array(values) if values.iter().any(|value| matches!(value.kind, ExprKind::Unary { op: UnaryOp::Spread, .. })) => self.spread_array_construction(context, expr, loans, expected),
            ExprKind::Array(values) => {
                self.array_literal(context, values, loans, expected, expr.span)
            }
            ExprKind::Template(parts) => self.template_literal(context, parts, loans),
            // Synthesized wrappers and other unary forms keep their
            // temporaries out of this recursively retained dispatcher.
            ExprKind::Unary {
                op: UnaryOp::CallableStore
                    | UnaryOp::CallableRestAdapter
                    | UnaryOp::CallableOnce
                    | UnaryOp::ParameterDefaultStore
                    | UnaryOp::ParameterMutableInput
                    | UnaryOp::RestPack
                    | UnaryOp::CoalesceSelected
                    | UnaryOp::OptionalSelected,
                ..
            }
            | ExprKind::Project { .. }
            | ExprKind::Inject { .. } => {
                self.wrapper_expression(context, expr, mode, loans, expected, previous_view)
            }
            ExprKind::Index(_, _) => self.index_expression(context,expr,mode,loans,expected),
            ExprKind::Field(base, name) => {
                self.field_access(context, base, name, mode, loans, expr.span)
            }
            ExprKind::Name(name)
                if name == "undefined"
                    && !context.names.contains_key(name)
                    && !context.pending_names.contains(name)
                    && !context
                        .lexical_scopes
                        .iter()
                        .any(|scope| scope.contains(name))
                    && !self.function_names.contains_key(name) =>
            {
                if mode == Mode::Mutate {
                    Err(Diagnostic::new(
                        "readonly",
                        "The global undefined value cannot be assigned.",
                        expr.span,
                    ))
                } else {
                    expr.kind = ExprKind::Undefined;
                    Ok(Type::Undefined)
                }
            }
            ExprKind::Name(name)
                if !context.names.contains_key(name)
                    && !context.pending_names.contains(name)
                    && self.function_names.contains_key(name) =>
            {
                self.function_value(name, expr.span)
            }
            ExprKind::Call(_, _) => unreachable!("call kind uses its own frame"),
            ExprKind::Name(_) => self.expression_name(context, expr, mode, loans, expected),
            _ => self.expression_inner(context, expr, mode, loans, expected),
        }
    }

    fn place_type(&self, context: &Context, place: &crate::flow::Place) -> Option<Type> {
        let mut ty = context.bindings.get(place.root)?.ty;
        let mut parent = crate::flow::Place {
            root: place.root,
            fields: Vec::new(),
        };
        for field in &place.fields {
            if matches!(ty, Type::Union(_) | Type::BorrowedUnion(_)) {
                let alternatives = context.flow.get(&parent)?;
                if alternatives.members.len() != 1 {
                    return None;
                }
                ty = alternatives.members[0];
            }
            let Type::Record(record) = ty else {
                return None;
            };
            ty = *self.fields[record].get(field)?;
            parent.fields.push(field.clone());
        }
        Some(ty)
    }

    fn flow_view(
        &self,
        context: &Context,
        expr: &Expr,
    ) -> Option<(crate::flow::Place, UnionId, Type)> {
        let place = crate::flow::place(expr, &context.names)?;
        if context.suppressed_flow.contains(&place) {
            return None;
        }
        let union = self.place_type(context, &place)?.union()?;
        let members = context.flow.get(&place)?;
        if members.members.is_empty() {
            return None;
        }
        let narrowed = self
            .unions
            .intern(members.members.iter().copied(), expr.span)
            .ok()?;
        (narrowed != Type::Union(union)).then_some((place, union, narrowed))
    }

    #[allow(clippy::too_many_arguments)]
    fn project_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
        place: crate::flow::Place,
        union: UnionId,
        narrowed: Type,
    ) -> Result<Type, Diagnostic> {
        let physical = self
            .place_type(context, &place)
            .unwrap_or(Type::Union(union));
        // Same-tag transport and whole residual observations borrow the original
        // enum. A smaller owning tag can only be reconstructed by moving it.
        let actual = if expected == Some(physical)
            || context.observer_places.contains(&place)
            || (mode == Mode::Read
                && matches!(narrowed, Type::Union(id) if !id.copy)
                && expected.is_none())
        {
            physical
        } else {
            narrowed
        };
        if actual != physical
            && let Some(fact) = context.flow.get(&place).cloned()
        {
            for frame in &mut context.loops {
                if frame
                    .flow_entry
                    .get(&place)
                    .is_some_and(|entry| entry.origin == fact.origin)
                {
                    frame.flow_required.insert(place.clone(), fact.clone());
                }
            }
        }
        let retag = matches!(actual, Type::Union(id) if id != union && !id.copy);
        let injecting_owned =
            matches!(expected, Some(Type::Union(id)) if !id.copy && actual != physical);
        let access = if matches!(physical, Type::BorrowedUnion(_))
            && mode == Mode::Consume
            && !actual.is_copy()
        {
            return Err(Diagnostic::new(
                "borrowed-view",
                "A borrowed optional payload cannot be transferred; use clone() to own it.",
                expr.span,
            ));
        } else if actual.is_copy() && !union.copy && mode != Mode::Mutate {
            // Copying an absent variant does not transfer the owning enum.
            Mode::Read
        } else if (retag || injecting_owned) && !matches!(mode, Mode::Mutate | Mode::Read) {
            Mode::Consume
        } else {
            mode
        };
        if mode == Mode::Mutate && matches!(actual, Type::Union(id) if id != union) {
            return Err(Diagnostic::new(
                "unsupported",
                "Replacing a narrowed tag requires assignment to its declared nullable storage.",
                expr.span,
            ));
        }
        context.suppressed_flow.insert(place.clone());
        let keep_start = context.keep_flow.len();
        if access == Mode::Mutate {
            context.keep_flow.push(place.clone());
        }
        let checked = self.expression(context, expr, access, loans, None);
        context.keep_flow.truncate(keep_start);
        context.suppressed_flow.remove(&place);
        let checked = checked?;
        self.require(checked, Some(physical), expr.span)?;
        expr.project(union, narrowed, actual);
        if let Some(Type::Union(target)) = expected
            && actual != Type::Union(target)
        {
            self.require(actual, expected, expr.span)?;
            expr.inject(target);
            return Ok(Type::Union(target));
        }
        Ok(actual)
    }

    fn pure_guard_outcomes(&self, context: &Context, expr: &Expr) -> Option<crate::flow::Outcomes> {
        match &expr.kind {
            ExprKind::Unary {
                op: UnaryOp::Not,
                value,
                ..
            } => self
                .pure_guard_outcomes(context, value)
                .map(|(yes, no)| (no, yes)),
            ExprKind::Binary {
                op, left, right, ..
            } if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) => {
                let valid = crate::flow::guard_pair(left, right, &context.names).is_some();
                if !valid {
                    return None;
                }
                let mut probe = context.clone();
                self.remember_guard(&mut probe, expr.span, *op, left, right);
                Some(Self::condition_outcomes(&probe, expr))
            }
            _ => None,
        }
    }

    fn observed_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let place = crate::flow::place(expr, &context.names);
        if let Some(place) = &place {
            context.observer_places.insert(place.clone());
        }
        let result = self.expression(context, expr, Mode::Read, loans, None);
        if let Some(place) = &place {
            context.observer_places.remove(place);
        }
        result
    }

    fn condition_outcomes(context: &Context, expr: &Expr) -> crate::flow::Outcomes {
        context
            .condition_flows
            .get(&(expr.span.start, expr.span.end))
            .cloned()
            .unwrap_or_else(|| (context.flow.clone(), context.flow.clone()))
    }

    fn remember_guard(
        &self,
        context: &mut Context,
        expr_span: Span,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) {
        let pair = crate::flow::guard_pair(left, right, &context.names);
        let mut yes = context.flow.clone();
        let mut no = context.flow.clone();
        if let Some((place, guard)) = pair
            && let Some(union) = self.place_type(context, &place).and_then(Type::union)
        {
            let members = context
                .flow
                .get(&place)
                .map(|fact| fact.members.clone())
                .unwrap_or_else(|| self.unions.members(union));
            let equal = members
                .iter()
                .copied()
                .filter(|ty| guard.matches(*ty))
                .collect::<Vec<_>>();
            let unequal = members
                .iter()
                .copied()
                .filter(|ty| !guard.matches(*ty))
                .collect::<Vec<_>>();
            let (a, b) = if op == BinaryOp::Equal {
                (equal, unequal)
            } else {
                (unequal, equal)
            };
            if !a.is_empty() {
                yes.insert(
                    place.clone(),
                    crate::flow::Fact {
                        members: a,
                        origin: Some((expr_span.start, expr_span.end)),
                    },
                );
            }
            if !b.is_empty() {
                no.insert(
                    place,
                    crate::flow::Fact {
                        members: b,
                        origin: Some((expr_span.start, expr_span.end)),
                    },
                );
            }
        }
        context
            .condition_flows
            .insert((expr_span.start, expr_span.end), (yes, no));
    }

    /// A private descendant fact survives only while the observed source is stable.
    /// Existing mutation invalidation and branch joins also apply to this fact.
    #[inline(never)]
    fn guard_snapshot(context: &mut Context, value: &Expr, span: Span) -> Option<crate::flow::Place> {
        let mut value = value;
        while let ExprKind::Sequence(values) = &value.kind { value = values.last()?; }
        if let ExprKind::Unary { op: UnaryOp::Typeof, value: inner, .. } = &value.kind { value = inner; }
        let mut marker = crate::flow::place(value, &context.names)?;
        // NUL cannot occur in a source field identifier.
        marker.fields.push(format!("\0comparison:{}:{}", span.start, span.end));
        context.flow.insert(marker.clone(), crate::flow::Fact { members: vec![Type::Void], origin: None });
        Some(marker)
    }

    #[inline(never)]
    fn finish_guard_snapshot(context: &mut Context, marker: Option<crate::flow::Place>) -> bool {
        let Some(marker) = marker else { return true; };
        let stable = context.flow.remove(&marker).is_some();
        for (yes, no) in context.condition_flows.values_mut() { yes.remove(&marker); no.remove(&marker); }
        stable
    }

    #[inline(never)]
    fn is_typeof_guard(context: &Context, left: &Expr, right: &Expr) -> bool {
        matches!(crate::flow::guard_pair(left, right, &context.names), Some((_, crate::flow::Guard::Typeof(_))))
    }

    #[inline(never)]
    fn typeof_equality(&mut self, context: &mut Context, span: Span, op: BinaryOp, left: &mut Expr, right: &mut Expr, loans: &Loans) -> Result<Type, Diagnostic> {
        let left_type = self.expression(context, left, Mode::Read, loans, Some(Type::String))?;
        self.require(left_type, Some(Type::String), left.span)?;
        let mut active = loans.clone();
        for root in Self::loan_roots(left, Mode::Read) { active.entry(root).or_default().push((Mode::Read, left.span)); }
        let marker = Self::guard_snapshot(context, left, span);
        let right_result = self.expression(context, right, Mode::Read, &active, Some(Type::String));
        let stable = Self::finish_guard_snapshot(context, marker);
        self.require(right_result?, Some(Type::String), right.span)?;
        if stable { self.remember_guard(context, span, op, left, right); }
        Ok(Type::Boolean)
    }

    fn nullish_equality(
        &mut self,
        context: &mut Context,
        span: Span,
        op: BinaryOp,
        left: &mut Expr,
        right: &mut Expr,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let first_capture = context.capture_loans.len();
        let observing = self.observing_values;
        self.observing_values = true;
        let result = (|| {
            let left_type = self.observed_expression(context, left, loans)?;
            if left_type == Type::Void {
                return Err(Diagnostic::new(
                    "type",
                    "A void result is not a value for equality.",
                    left.span,
                ));
            }
            let mut active = loans.clone();
            if !left_type.is_copy() {
                for root in Self::loan_roots(left, Mode::Read) {
                    active
                        .entry(root)
                        .or_default()
                        .push((Mode::Read, left.span));
                }
            }
            let marker = Self::guard_snapshot(context, left, span);
            let right_result = self.observed_expression(context, right, &active);
            let stable = Self::finish_guard_snapshot(context, marker);
            let right_type = right_result?;
            if right_type == Type::Void {
                return Err(Diagnostic::new(
                    "type",
                    "A void result is not a value for equality.",
                    right.span,
                ));
            }
            Ok(stable)
        })();
        self.observing_values = observing;
        context.capture_loans.truncate(first_capture);
        if result? { self.remember_guard(context, span, op, left, right); }
        Ok(Type::Boolean)
    }

    /// Strict equality where an operand is a data union. The other operand is
    /// an int, number, boolean or string, or another union of primitive
    /// alternatives. Values of different alternatives are unequal, except that
    /// int and number compare by value.
    #[inline(never)]
    fn union_equality(
        &mut self,
        context: &mut Context,
        spans: (Span, Span),
        op: BinaryOp,
        left: &mut Expr,
        right: &mut Expr,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let (expr_span, operator) = spans;
        let first_capture = context.capture_loans.len();
        let observing = self.observing_values;
        self.observing_values = true;
        let result = (|| {
            let left_type = self.observed_expression(context, left, loans)?;
            let mut active = loans.clone();
            if !left_type.is_copy() {
                for root in Self::loan_roots(left, Mode::Read) {
                    active.entry(root).or_default().push((Mode::Read, left.span));
                }
            }
            let marker = Self::guard_snapshot(context, left, expr_span);
            let right_result = self.observed_expression(context, right, &active);
            let stable = Self::finish_guard_snapshot(context, marker);
            let right_type = right_result?;
            Ok((left_type, right_type, stable))
        })();
        self.observing_values = observing;
        context.capture_loans.truncate(first_capture);
        let (left_type, right_type, stable) = result?;
        let primitive = |ty: Type| {
            matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined)
        };
        let comparable = |ty: Type| match ty.union() {
            Some(id) => self.unions.members(id).into_iter().all(primitive),
            None => primitive(ty),
        };
        let both_unions = left_type.union().is_some() && right_type.union().is_some();
        let one_primitive = (left_type.union().is_none() && primitive(left_type))
            || (right_type.union().is_none() && primitive(right_type));
        let callable = |ty: Type| matches!(self.unions.payload(ty), Some(Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::Callable(_)));
        if callable(left_type) || callable(right_type)
            || !(one_primitive || both_unions && comparable(left_type) && comparable(right_type))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Strict equality on a union compares it with an int, number, boolean or string, or with another union of such values. Narrow records and arrays with typeof first.",
                operator,
            ));
        }
        // Equality with one primitive value proves the alternative of a
        // stable union place on the equal path.
        let guarded = match (left_type.union(), right_type.union()) {
            (Some(union), None) if stable => crate::flow::place(left, &context.names).map(|place| (place, union, right_type)),
            (None, Some(union)) => crate::flow::place(right, &context.names).map(|place| (place, union, left_type)),
            _ => None,
        };
        if let Some((place, union, value)) = guarded {
            let numeric = |ty: Type| matches!(ty, Type::Int | Type::Number);
            let matching = context
                .flow
                .get(&place)
                .map(|fact| fact.members.clone())
                .unwrap_or_else(|| self.unions.members(union))
                .into_iter()
                .filter(|member| *member == value || numeric(*member) && numeric(value))
                .collect::<Vec<_>>();
            if !matching.is_empty() {
                let mut equal = context.flow.clone();
                equal.insert(
                    place,
                    crate::flow::Fact { members: matching, origin: Some((expr_span.start, expr_span.end)) },
                );
                let other = context.flow.clone();
                context.condition_flows.insert(
                    (expr_span.start, expr_span.end),
                    if op == BinaryOp::Equal { (equal, other) } else { (other, equal) },
                );
            }
        }
        Ok(Type::Boolean)
    }

    fn logical_expression(
        &mut self,
        context: &mut Context,
        spans: (Span, Span),
        op: BinaryOp,
        left: &mut Expr,
        right: &mut Expr,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let (expr_span, operator) = spans;
        let actual = self.expression(context, left, Mode::Copy, loans, Some(Type::Boolean))?;
        self.require(actual, Some(Type::Boolean), operator)?;
        let (left_true, left_false) = Self::condition_outcomes(context, left);
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let skipped = if op == BinaryOp::And {
            left_false.clone()
        } else {
            left_true.clone()
        };
        context.flow = if op == BinaryOp::And {
            left_true
        } else {
            left_false
        };
        let actual = self.expression(context, right, Mode::Copy, loans, Some(Type::Boolean))?;
        self.require(actual, Some(Type::Boolean), operator)?;
        let (right_true, right_false) = Self::condition_outcomes(context, right);
        for (id, state) in before {
            state
                .join(Availability::of(&context.bindings[id]))
                .restore(&mut context.bindings[id]);
        }
        let outcomes = if op == BinaryOp::And {
            (right_true, crate::flow::join(&skipped, &right_false))
        } else {
            (crate::flow::join(&skipped, &right_true), right_false)
        };
        context.flow = crate::flow::join(&outcomes.0, &outcomes.1);
        context
            .condition_flows
            .insert((expr_span.start, expr_span.end), outcomes);
        Ok(Type::Boolean)
    }

    /// A record literal in a union context takes the record alternative whose
    /// field names match it. A lone record alternative is used otherwise, so
    /// its field diagnostics apply.
    #[inline(never)]
    fn record_alternative(
        &self,
        union: UnionId,
        fields: &[(Token, Expr)],
        span: Span,
    ) -> Result<Option<Type>, Diagnostic> {
        let records = self
            .unions
            .members(union)
            .into_iter()
            .filter(|member| matches!(member, Type::Record(record) if !self.program.records[*record].is_tuple()))
            .collect::<Vec<_>>();
        let matching = records
            .iter()
            .copied()
            .filter(|member| {
                let Type::Record(record) = member else { return false };
                self.fields[*record].len() == fields.len()
                    && fields.iter().all(|(key, _)| self.fields[*record].contains_key(&key.text))
            })
            .collect::<Vec<_>>();
        match (matching.as_slice(), records.as_slice()) {
            ([only], _) | ([], [only]) => Ok(Some(*only)),
            ([], _) => Ok(None),
            _ => Err(Diagnostic::new(
                "type",
                "This record literal matches more than one record alternative of the union; build it through a binding or function with the intended record type.",
                span,
            )),
        }
    }

    fn inject_expression(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        union: UnionId,
    ) -> Result<Type, Diagnostic> {
        let expected = Type::Union(union);
        let hint = self.expression_type_hint(context, expr);
        if let Some(Type::BorrowedUnion(source)) = hint {
            if mode != Mode::Read {
                return Err(Diagnostic::new(
                    "borrowed-view",
                    "A borrowed optional element requires a readonly consumer; use clone() to own its value.",
                    expr.span,
                ));
            }
            let actual = self.expression(context, expr, Mode::Read, loans, None)?;
            self.require(actual, Some(expected), expr.span)?;
            if source != union {
                expr.inject(union);
                expr.ty = Some(Type::BorrowedUnion(union));
                return Ok(Type::BorrowedUnion(union));
            }
            return Ok(actual);
        }
        if matches!(expr.kind, ExprKind::OptionalChain(_)) {
            // The present branch receives payload context inside optional_chain;
            // the complete expression still satisfies the nullable expectation.
            return self.optional_chain(context, expr, mode, loans, Some(expected));
        }
        let member_hint = if hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Null | Type::Undefined | Type::Union(_) | Type::BorrowedUnion(_)
            )
        }) {
            hint
        } else {
            self.unions.payload(expected)
        };
        let member_hint = member_hint.filter(|ty| !matches!(ty, Type::Union(_)));
        let member_hint = match &expr.kind {
            ExprKind::Record(fields) if member_hint.is_none() => {
                self.record_alternative(union, fields, expr.span)?
            }
            _ => member_hint,
        };
        // An integer literal is a number where the union has number and no int.
        let member_hint = if member_hint.is_none() && matches!(expr.kind, ExprKind::Integer(_)) {
            let members = self.unions.members(union);
            (members.contains(&Type::Number) && !members.contains(&Type::Int)).then_some(Type::Number)
        } else {
            member_hint
        };
        if mode == Mode::Mutate
            && self
                .unions
                .payload(expected)
                .is_some_and(|ty| matches!(ty, Type::Callback(_)))
            && hint.is_some_and(|ty| ty.union().is_some())
        {
            // A mutable callback borrow retains the existing tag and environment.
            // The concrete call specialization uses that physical tag identity;
            // abstract template bodies are never emitted as native operations.
            let actual = self.expression(context, expr, mode, loans, None)?;
            self.require(actual, Some(expected), expr.span)?;
            return Ok(actual);
        }
        let injecting_owned = hint != Some(expected) && !expected.is_copy();
        if injecting_owned && mode == Mode::Mutate {
            return Err(Diagnostic::new(
                "type",
                "A nullable mutable parameter requires existing tagged storage.",
                expr.span,
            ));
        }
        let borrowing_callback=mode==Mode::Read && self.unions.payload(expected).is_some_and(|ty|matches!(ty,Type::Callback(id) if self.callables.callbacks[id].kind==CallbackKind::Read || hint.is_some_and(|ty|ty.union().is_some())));
        let mut actual = if let Some(Type::StoredCallable(family)) = member_hint {
            self.stored_callable_value(context, expr, family, loans)?
        } else {
            self.expression(
                context,
                expr,
                if injecting_owned && !borrowing_callback {
                    Mode::Consume
                } else {
                    mode
                },
                loans,
                member_hint,
            )?
        };
        if let Some(Type::Callback(callback)) = member_hint {
            self.require(actual, Some(Type::Callback(callback)), expr.span)?;
            if self.callables.callbacks[callback].kind.is_once()
                && let Type::Closure(origin) = actual
                && self.closures[origin].invocation != Mode::Consume
            {
                let wrapper = self.once_return(origin, context, expr.span);
                expr.wrap_once(wrapper);
                actual = Type::Closure(wrapper);
            }
        } else {
            self.require(actual, Some(expected), expr.span)?;
        }
        if actual == expected {
            return Ok(actual);
        }
        if context.returning_closure
            && matches!(self.unions.payload(expected),Some(Type::Callback(id)) if self.callables.callbacks[id].kind.is_owned())
            && let Some(payload) = self.unions.payload(actual)
            && (matches!(payload, Type::Closure(_)) || self.absent_callable_type(actual))
        {
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        payload
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            if actual != concrete {
                expr.inject(concrete.union().expect("nullable result"));
            }
            return Ok(concrete);
        }
        if context.returning_closure
            && matches!(actual, Type::Null | Type::Undefined)
            && let Some(Type::Callback(id)) = self.unions.payload(expected)
            && self.callables.callbacks[id].kind.is_owned()
        {
            let empty = self.absent_callback(id);
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        empty
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            expr.inject(concrete.union().expect("nullable absent result"));
            return Ok(concrete);
        }
        if context.returning_closure
            && matches!(self.unions.payload(expected),Some(Type::Callback(id)) if self.callables.callbacks[id].kind.is_owned())
            && let Type::Closure(_) = actual
        {
            let concrete = self.unions.intern(
                self.unions.members(union).into_iter().map(|member| {
                    if matches!(member, Type::Callback(_)) {
                        actual
                    } else {
                        member
                    }
                }),
                expr.span,
            )?;
            expr.inject(concrete.union().expect("nullable result"));
            return Ok(concrete);
        }
        expr.inject(union);
        if borrowing_callback {
            expr.ty = Some(Type::BorrowedUnion(union));
            return Ok(Type::BorrowedUnion(union));
        }
        Ok(expected)
    }

    // Appending ends each operand borrow before the next interpolation runs.
    // Incoming consumer loans remain active throughout the template.
    fn template_literal(
        &mut self,
        context: &mut Context,
        parts: &mut [Expr],
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        for part in parts {
            let actual = self.expression(context, part, Mode::Read, loans, None)?;
            if !matches!(
                actual,
                Type::String
                    | Type::Int
                    | Type::Number
                    | Type::Boolean
                    | Type::Null
                    | Type::Undefined
            ) {
                return Err(Diagnostic::new(
                    "type",
                    "Template interpolation requires string, int, number, or boolean.",
                    part.span,
                ));
            }
        }
        Ok(Type::String)
    }

    fn finalize_nullable_returns(&self, body: &mut [Stmt], result: Type) -> Result<(), Diagnostic> {
        let Some(to) = result.union() else {
            return Ok(());
        };
        let mut nodes = body.iter_mut().collect::<Vec<_>>();
        while let Some(stmt) = nodes.pop() {
            match &mut stmt.kind {
                StmtKind::Try(region) => nodes.extend(region.statements_mut()),
                StmtKind::Return(Some(value))
                    if self.absent_callable_type(value.ty.unwrap_or(Type::Void)) =>
                {
                    if let Some(from) = value.ty.and_then(Type::union) {
                        Self::retag_nullable_origin(value, from, to);
                    }
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    nodes.extend(then_body);
                    nodes.extend(else_body);
                }
                StmtKind::Block(body)
                | StmtKind::While { body, .. }
                | StmtKind::ForOf { body, .. } => nodes.extend(body),
                StmtKind::Loop { body, update, .. } => {
                    nodes.extend(body);
                    nodes.extend(update);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn finalize_nullable_locals(
        &self,
        bindings: &mut [Binding],
        body: &mut [Stmt],
    ) -> Result<(), Diagnostic> {
        fn statements<'a>(body: &'a mut [Stmt], out: &mut Vec<&'a mut Stmt>) {
            out.extend(body.iter_mut());
        }
        let mut origins = HashMap::<UnionId, Type>::new();
        let collect = |origins: &mut HashMap<UnionId, Type>,
                       binding: Option<usize>,
                       value: &Expr|
         -> Result<(), Diagnostic> {
            let Some(binding) = binding
                .and_then(|id| bindings.get(id))
                .filter(|binding| !binding.parameter)
            else {
                return Ok(());
            };
            let Some(from) = binding.ty.union() else {
                return Ok(());
            };
            if !matches!(self.unions.payload(binding.ty), Some(Type::Callback(_))) {
                return Ok(());
            }
            if let Some(origin) = self.nullable_origin(value)? {
                if origins
                    .insert(from, origin)
                    .is_some_and(|prior| prior != origin)
                {
                    return Err(Diagnostic::new(
                        "callable-origin",
                        "Nullable callback replacement must preserve one concrete environment origin; matching signatures alone are insufficient.",
                        value.span,
                    ));
                }
            }
            Ok(())
        };
        let mut pending = Vec::new();
        statements(body, &mut pending);
        while let Some(stmt) = pending.pop() {
            match &mut stmt.kind {
                StmtKind::Try(region) => pending.extend(region.statements_mut()),
                StmtKind::Let { binding, value, .. } => collect(&mut origins, *binding, value)?,
                StmtKind::Assign { target, value, .. } => {
                    collect(&mut origins, target.binding, value)?
                }
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    statements(then_body, &mut pending);
                    statements(else_body, &mut pending);
                }
                StmtKind::Block(body)
                | StmtKind::While { body, .. }
                | StmtKind::ForOf { body, .. } => statements(body, &mut pending),
                StmtKind::Loop { body, update, .. } => {
                    statements(body, &mut pending);
                    statements(update, &mut pending);
                }
                _ => (),
            }
        }
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |expr| {
            if let ExprKind::CoalesceAssign { target, value, .. }
            | ExprKind::Assign { target, value, .. } = &expr.kind
            {
                collect(&mut origins, target.binding, value)?;
            }
            Ok(())
        })?;
        let replacements = origins
            .into_iter()
            .map(|(from, origin)| {
                let concrete = self.unions.intern(
                    self.unions.members(from).into_iter().map(|member| {
                        if matches!(member, Type::Callback(_)) {
                            origin
                        } else {
                            member
                        }
                    }),
                    Span::default(),
                )?;
                Ok((from, (concrete.union().unwrap(), origin)))
            })
            .collect::<Result<HashMap<_, _>, Diagnostic>>()?;
        for binding in bindings {
            if let Some(from) = binding.ty.union()
                && let Some((to, _)) = replacements.get(&from)
            {
                binding.ty = if matches!(binding.ty, Type::BorrowedUnion(_)) {
                    Type::BorrowedUnion(*to)
                } else {
                    Type::Union(*to)
                };
            }
        }
        crate::source_walk::body(body, &mut |_| Ok(()), &mut |expr| {
            if let Some(from) = expr.ty.and_then(Type::union)
                && let Some((to, _)) = replacements.get(&from)
            {
                expr.ty = Some(if matches!(expr.ty, Some(Type::BorrowedUnion(_))) {
                    Type::BorrowedUnion(*to)
                } else {
                    Type::Union(*to)
                });
            }
            match &mut expr.kind {
                ExprKind::Inject { union, .. } | ExprKind::Project { union, .. } => {
                    if let Some((to, _)) = replacements.get(union) {
                        *union = *to;
                    }
                }
                _ => (),
            }
            // Successful projections and assignment views use the same environment.
            if let Some(Type::Callback(id)) = expr.ty {
                if let Some((_, (_, origin))) = replacements.iter().find(|(from, _)| {
                    self.unions.payload(Type::Union(**from)) == Some(Type::Callback(id))
                }) {
                    expr.ty = Some(*origin);
                }
            }
            if let ExprKind::Project { narrowed, .. } = &mut expr.kind
                && let Type::Callback(id) = *narrowed
            {
                if let Some((_, (_, origin))) = replacements.iter().find(|(from, _)| {
                    self.unions.payload(Type::Union(**from)) == Some(Type::Callback(id))
                }) {
                    *narrowed = *origin;
                }
            }
            Ok(())
        })
    }

    fn nullable_origin(&self, expr: &Expr) -> Result<Option<Type>, Diagnostic> {
        let mut pending = vec![expr];
        let mut origin = None;
        while let Some(value) = pending.pop() {
            let payload = value.ty.and_then(|ty| self.unions.payload(ty).or(Some(ty)));
            if let Some(ty @ Type::Closure(_)) = payload {
                if origin.is_some_and(|previous| previous != ty) {
                    return Err(Diagnostic::new(
                        "callable-origin",
                        "Nullable borrowed callback construction must preserve one concrete environment origin.",
                        value.span,
                    ));
                }
                origin = Some(ty);
                continue;
            }
            match &value.kind {
                ExprKind::Inject { value, .. }
                | ExprKind::Project { value, .. }
                | ExprKind::Unary { value, .. } => pending.push(value),
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(alternate);
                    pending.push(consequent);
                }
                _ => (),
            }
        }
        Ok(origin)
    }
    fn retag_nullable_origin(expr: &mut Expr, from: UnionId, to: UnionId) {
        let mut pending = vec![expr];
        while let Some(expr) = pending.pop() {
            if expr.ty == Some(Type::Union(from)) {
                expr.ty = Some(Type::Union(to));
            }
            match &mut expr.kind {
                ExprKind::Inject { value, union } => {
                    if *union == from {
                        *union = to;
                    }
                    pending.push(value);
                }
                ExprKind::Unary { value, .. } => pending.push(value),
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(alternate);
                    pending.push(consequent);
                }
                _ => (),
            }
        }
    }

    fn conditional_callback_metadata(
        context: &Context,
        expr: &Expr,
    ) -> Result<Option<CallbackMetadata>, Diagnostic> {
        if let ExprKind::Project { value, .. } | ExprKind::Inject { value, .. } = &expr.kind {
            return Self::conditional_callback_metadata(context, value);
        }
        if let ExprKind::Unary {
            op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
            value,
            ..
        } = &expr.kind
        {
            return Self::conditional_callback_metadata(context, value);
        }
        if let ExprKind::Binary {
            op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
            right,
            ..
        } = &expr.kind
        {
            return Self::conditional_callback_metadata(context, right);
        }
        if let ExprKind::Conditional {
            consequent,
            alternate,
            ..
        } = &expr.kind
        {
            let left = Self::conditional_callback_metadata(context, consequent)?;
            let right = Self::conditional_callback_metadata(context, alternate)?;
            if left != right && matches!(expr.ty, Some(Type::Closure(_) | Type::Callback(_))) {
                return Err(Diagnostic::new(
                    "callback-contract",
                    "Conditional callable branches require matching ownership and invocation contracts.",
                    expr.span,
                ));
            }
            return Ok(left);
        }
        let binding = expr.binding.or_else(|| {
            if let ExprKind::Name(name) = &expr.kind {
                context.names.get(name).copied()
            } else {
                None
            }
        });
        if binding.is_none()
            && let ExprKind::Name(name) = &expr.kind
            && crate::native_bindings::by_symbol(name).is_some()
        {
            return Ok(Some((None, false, None)));
        }
        Ok(binding.map(|id| {
            let b = &context.bindings[id];
            (b.callback_invocation, b.callback_reference, b.callback_kind)
        }))
    }

    #[inline(never)]
    fn pattern_selection_hint(&self,context:&Context,value:&Expr)->Option<Type>{
        match &value.kind {
            ExprKind::Index(base,_)=>match self.expression_type_hint(context,base)? {
                Type::Array(element)=>{let ty=self.unions.intern([element.ty(),Type::Undefined],value.span).ok()?;Some(if !ty.is_copy(){Type::BorrowedUnion(ty.union()?)}else{ty})},
                Type::Record(id)=>Some(Self::flat_literal_depth(match &value.kind{ExprKind::Index(_,index)=>index,_=>unreachable!()}).and_then(|position|self.fields[id].get(&position.to_string()).copied()).unwrap_or(Type::Undefined)),
                _=>None,
            },
            ExprKind::Field(base,name)=>match self.expression_type_hint(context,base)?{Type::Record(id)=>Some(self.fields[id].get(name).copied().unwrap_or(Type::Undefined)),_=>None},
            _=>self.expression_type_hint(context,value),
        }
    }
    #[inline(never)]
    fn pattern_selection(&mut self,context:&mut Context,expr:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Unary{value,..}=&mut expr.kind else{unreachable!()};
        let span=expr.span;
        let mut selected=std::mem::replace(value,Box::new(Expr::new(ExprKind::Undefined,span)));
        let missing=self.pattern_selection_hint(context,&selected)==Some(Type::Undefined);
        if missing {
            let selected_name=match &selected.kind{ExprKind::Field(_,name)=>name.clone(),ExprKind::Index(_,index)=>Self::flat_literal_depth(index).unwrap_or(0).to_string(),_=>String::new()};
            if let ExprKind::Field(base,_)|ExprKind::Index(base,_)=&mut selected.kind {
                let prior=context.projection.take();
                if let Some((root,mut path))=Self::direct_fields(base){path.push(selected_name);context.projection=Some(Projection{root,path,replacing:false,span});}
                let checked=self.expression(context,base,Mode::Read,loans,None);
                context.projection=prior;
                checked?;
            }
        }
        match &mut selected.kind {
            ExprKind::Index(base,index) if matches!(self.expression_type_hint(context,base),Some(Type::Array(_)))=>{
                let callee=Expr::new(ExprKind::Field(base.clone(),"at".into()),span);
                selected.kind=ExprKind::Call(Box::new(callee),vec![*index.clone()]);
            },
            ExprKind::Field(_,_)|ExprKind::Index(_,_) if missing=>selected.kind=ExprKind::Undefined,
            _=>{},
        }
        *expr=*selected;
        self.expression(context,expr,mode,loans,expected)
    }

    fn coalesce_hint(
        &self,
        context: &Context,
        left: &Expr,
        right: &Expr,
        span: Span,
        undefined_only:bool,
    ) -> Option<Type> {
        let lhs = self.expression_type_hint(context, left)?;
        if lhs==Type::Undefined || (!undefined_only && lhs==Type::Null) {
            return self.expression_type_hint(context, right);
        }
        let Some(_) = lhs.union() else {
            return Some(lhs);
        };
        let payload = if undefined_only {let members=self.unions.members(lhs.union()?).into_iter().filter(|ty|*ty!=Type::Undefined).collect::<Vec<_>>();if members.is_empty(){None}else{self.unions.intern(members,span).ok()}}else{self.unions.present(lhs.owning(), span).ok().flatten()};
        let rhs = self.expression_type_hint(context, right);
        let joined = match (payload, rhs) {
            (None, rhs) => return rhs,
            (Some(payload), None) => payload,
            (Some(payload), Some(rhs)) => self.unions.intern([payload, rhs.owning()], span).ok()?,
        };
        Some(
            if matches!(lhs, Type::BorrowedUnion(_)) || matches!(rhs, Some(Type::BorrowedUnion(_)))
            {
                joined.union().map(Type::BorrowedUnion).unwrap_or(joined)
            } else {
                joined
            },
        )
    }

    fn coalesce(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let undefined_only=matches!(expr.kind,ExprKind::Binary{op:BinaryOp::PatternDefault,..});
        let ExprKind::Binary {
            left,
            right,
            operator,
            ..
        } = &mut expr.kind
        else {
            unreachable!()
        };
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let before_flow = context.flow.clone();
        let hint = self.expression_type_hint(context, left);
        let left_expected = if hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Closure(_) | Type::Callback(_) | Type::Callable(_) | Type::StoredCallable(_)
            )
        }) {
            expected.or(hint)
        } else if hint.is_none()
            || number_literal_context(left)
            || matches!(left.kind, ExprKind::Record(_) | ExprKind::Array(_))
        {
            expected.map(|ty| self.unions.payload(ty).unwrap_or(ty))
        } else {
            None
        };
        let view = matches!(hint, Some(Type::BorrowedUnion(_)));
        let lhs_mode = if view { Mode::Read } else { mode };
        let lhs = self.expression(context, left, lhs_mode, loans, left_expected)?;
        let members = lhs
            .union()
            .map(|id| self.unions.members(id))
            .unwrap_or_else(|| vec![lhs]);
        let payload = if undefined_only {let present=members.iter().copied().filter(|ty|*ty!=Type::Undefined).collect::<Vec<_>>();if present.is_empty(){None}else{Some(self.unions.intern(present,left.span)?)} }else{self.unions.present(lhs.owning(), left.span)?.filter(|_| !self.absent_callable_type(lhs))};
        let absent = members
            .iter()
            .any(|ty| *ty==Type::Undefined || (!undefined_only && *ty==Type::Null));
        if lhs == Type::Void {
            return Err(Diagnostic::new(
                "type",
                "A void expression cannot be a coalescing value.",
                left.span,
            ));
        }
        if !absent || payload.is_none() {
            let choose_left = !absent;
            let result = if choose_left {
                let mut unreachable = Box::new(context.clone());
                for (id, state) in &before {
                    state.clone().restore(&mut unreachable.bindings[*id]);
                }
                unreachable.flow = before_flow;
                let rhs_hint = self.expression_type_hint(&unreachable, right).filter(|ty| {
                    matches!(
                        ty,
                        Type::Closure(_)
                            | Type::Callback(_)
                            | Type::Callable(_)
                            | Type::StoredCallable(_)
                    )
                });
                self.expression(&mut unreachable, right, Mode::Consume, loans, rhs_hint)?;
                lhs
            } else {
                self.expression(context, right, mode, loans, expected)?
            };
            if !choose_left {
                return Ok(result);
            }
            let placeholder = Box::new(left.as_ref().clone());
            let value = if choose_left {
                std::mem::replace(left, placeholder)
            } else {
                std::mem::replace(right, placeholder)
            };
            expr.binding = value.binding;
            expr.kind = ExprKind::Unary {
                op: UnaryOp::CoalesceSelected,
                operator: *operator,
                value,
            };
            return Ok(result);
        }
        let payload = payload.unwrap();
        self.coalesce_joined_arms(context, expr, mode, loans, expected, (lhs, lhs_mode, payload, view, undefined_only), members, before, before_flow)
    }

    /// Both arms can run: checks the fallback after the present arm and
    /// joins their ownership and flow states. Kept out of `coalesce`, whose
    /// frame is retained across a nested chain of single-arm selections.
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn coalesce_joined_arms(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
        (lhs, lhs_mode, payload, view, undefined_only): (Type, Mode, Type, bool, bool),
        members: Vec<Type>,
        before: Vec<(usize, Availability)>,
        before_flow: crate::flow::Facts,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Binary { left, right, .. } = &mut expr.kind else {
            unreachable!()
        };
        self.require(payload, expected, left.span)?;
        let fallback_context = expected
            .map(|ty| self.unions.payload(ty).unwrap_or(ty))
            .unwrap_or(payload);
        let rhs_hint = self.expression_type_hint(context, right);
        let mut contextual_members = vec![fallback_context];
        // An int array literal beside a number array payload is a number
        // array, so `numbers ?? [1, 2]` stays one array type.
        let literal_fallback = matches!(right.kind, ExprKind::Array(_))
            && matches!((payload, rhs_hint), (Type::Array(ArrayElement::Number(wanted)), Some(Type::Array(ArrayElement::Int(found)))) if wanted == found);
        if expected.is_none() && !literal_fallback && !number_literal_context(right) && let Some(ty)=rhs_hint
            && !ty.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![ty]).iter().any(|member|matches!(member,Type::Closure(_)|Type::Callable(_)|Type::Callback(_)|Type::StoredCallable(_))) {contextual_members.push(ty.owning());}
        if let Some(ty) = rhs_hint {
            for member in ty
                .union()
                .map(|id| self.unions.members(id))
                .unwrap_or_else(|| vec![ty])
            {
                if matches!(member, Type::Null | Type::Undefined) {
                    contextual_members.push(member);
                }
            }
        }
        let rhs_context = self.unions.intern(contextual_members, right.span)?;
        let left_state = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let left_flow = context.flow.clone();
        // The absent arm contains no owned payload to transfer. A stable source
        // place is available in that arm even when the present arm moves it.
        if lhs_mode == Mode::Consume
            && !lhs.is_copy()
            && let Some(place) = crate::flow::place(left, &context.names)
        {
            if let Some((_, state)) = before.iter().find(|(id, _)| *id == place.root) {
                state.clone().restore(&mut context.bindings[place.root]);
            }
            for (place, fact) in &before_flow {
                if place.root == Self::root_binding(left).unwrap_or(usize::MAX) {
                    context.flow.insert(place.clone(), fact.clone());
                }
            }
        }
        if let Some(place) = crate::flow::place(left, &context.names) {
            context.flow.insert(
                place,
                crate::flow::Fact {
                    members: members
                        .iter()
                        .copied()
                        .filter(|ty| *ty==Type::Undefined || (!undefined_only && *ty==Type::Null))
                        .collect(),
                    origin: Some((span.start, span.end)),
                },
            );
        }
        let mut fallback_loans=loans.clone();
        if undefined_only && view {
            for root in Self::loan_roots(left,Mode::Read){fallback_loans.entry(root).or_default().push((Mode::Read,left.span));}
        }
        let rhs = self.expression(
            context,
            right,
            if view || matches!(rhs_hint, Some(Type::BorrowedUnion(_))) {
                Mode::Read
            } else {
                mode
            },
            &fallback_loans,
            Some(rhs_context),
        )?;
        let joined = self.unions.intern([payload, rhs.owning()], right.span)?;
        self.require(joined, expected, span)?;
        let borrowed = view || matches!(rhs, Type::BorrowedUnion(_));
        let result = if borrowed {
            joined.union().map(Type::BorrowedUnion).unwrap_or(joined)
        } else {
            expected.unwrap_or(joined)
        };
        if borrowed && !result.is_copy() && mode == Mode::Consume {
            return Err(Diagnostic::new(
                "borrowed-view",
                "Coalescing a borrowed payload cannot transfer ownership; use clone() or a readonly consumer.",
                span,
            ));
        }
        if borrowed
            && !rhs.is_copy()
            && !matches!(rhs, Type::BorrowedUnion(_))
            && !undefined_only
            && !Self::stable_at_receiver(right)
            && !matches!(right.kind, ExprKind::String(_))
        {
            return Err(Diagnostic::new(
                "borrowed-view",
                "Join borrowed values only with stable readonly sources or string literals; clone the view before selecting a fresh owner.",
                right.span,
            ));
        }
        context.flow = crate::flow::join(&left_flow, &context.flow);
        for ((id, _), state) in before.into_iter().zip(left_state) {
            state
                .join(Availability::of(&context.bindings[id]))
                .restore(&mut context.bindings[id]);
        }
        Ok(result)
    }

    // Branch planning, the state between arms and the final join own their
    // temporaries in dedicated helpers. This frame is retained across nested
    // conditional arms and keeps only the plan and arm results.
    fn conditional(
        &mut self,
        context: &mut Context,
        predicate: &mut Expr,
        arms: (&mut Expr, &mut Expr),
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let (consequent, alternate) = arms;
        self.conditional_predicate(context, predicate, loans)?;
        let mut plan = self.conditional_plan(context, predicate, consequent, alternate, mode, expected)?;
        let left = if let Some(Type::StoredCallable(family)) = plan.left_context {
            self.stored_callable_value(context, consequent, family, loans)
        } else {
            self.expression(context, consequent, mode, loans, plan.left_context)
        };
        let left = self.conditional_between(context, consequent, alternate, &mut plan, left)?;
        let right = if let Some(Type::StoredCallable(family)) = plan.right_context {
            self.stored_callable_value(context, alternate, family, loans)
        } else {
            self.expression(context, alternate, mode, loans, plan.right_context)
        };
        self.conditional_join(context, consequent, alternate, plan, left, right)
    }

    #[inline(never)]
    fn conditional_predicate(&mut self, context: &mut Context, predicate: &mut Expr, loans: &Loans) -> Result<(), Diagnostic> {
        let predicate_type =
            self.expression(context, predicate, Mode::Copy, loans, Some(Type::Boolean))?;
        self.require(predicate_type, Some(Type::Boolean), predicate.span)
    }

    #[inline(never)]
    fn conditional_plan(
        &mut self,
        context: &mut Context,
        predicate: &Expr,
        consequent: &mut Expr,
        alternate: &mut Expr,
        mode: Mode,
        expected: Option<Type>,
    ) -> Result<Box<ConditionalPlan>, Diagnostic> {
        let (then_flow, else_flow) = Self::condition_outcomes(context, predicate);
        context.flow = then_flow.clone();
        self.prepare_nullable_factory_hint(context, consequent, alternate)?;
        let left_hint = self.expression_type_hint(context, consequent);
        context.flow = else_flow.clone();
        self.prepare_nullable_factory_hint(context, alternate, consequent)?;
        let right_hint = self.expression_type_hint(context, alternate);
        context.flow = then_flow.clone();
        let nullable = left_hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Null | Type::Undefined | Type::Union(_) | Type::BorrowedUnion(_)
            )
        }) || right_hint.is_some_and(|ty| {
            matches!(
                ty,
                Type::Null | Type::Undefined | Type::Union(_) | Type::BorrowedUnion(_)
            )
        });
        let assignment_alias = mode == Mode::Read
            && (self.borrowed_place_hint(context, consequent)
                || self.borrowed_place_hint(context, alternate));
        let borrowed = matches!(left_hint, Some(Type::BorrowedUnion(_)))
            || matches!(right_hint, Some(Type::BorrowedUnion(_)))
            || (assignment_alias
                && [left_hint, right_hint]
                    .into_iter()
                    .flatten()
                    .any(|ty| ty.union().is_some_and(|union| !union.copy)));
        if borrowed
            && !assignment_alias
            && [left_hint, right_hint]
                .into_iter()
                .flatten()
                .any(|ty| !matches!(ty, Type::BorrowedUnion(_) | Type::Null | Type::Undefined))
        {
            return Err(Diagnostic::new(
                "borrowed-view",
                "Conditional borrowed views can join views and absent sentinels; use clone() before joining an owning value.",
                alternate.span,
            ));
        }
        let joined = if nullable && (expected.is_none() || borrowed) {
            let left = left_hint.or(self.callback_value_hint(context, consequent)?);
            let right = right_hint.or(self.callback_value_hint(context, alternate)?);
            match (left, right) {
                (Some(left), Some(right)) => {
                    let joined = self
                        .unions
                        .intern([left.owning(), right.owning()], alternate.span)?;
                    Some(if borrowed {
                        Type::BorrowedUnion(joined.union().expect("view union"))
                    } else {
                        joined
                    })
                }
                _ => None,
            }
        } else {
            None
        };
        let hint = if borrowed && assignment_alias {
            expected.or(joined).map(Type::owning)
        } else if borrowed {
            expected
                .and_then(Type::union)
                .map(Type::BorrowedUnion)
                .or(joined)
        } else {
            expected.or(joined)
        }
        .or_else(|| self.conditional_hint(context, consequent, alternate));
        let infer_nullable_closure = nullable
            && expected.is_none()
            && ((left_hint.is_none() && matches!(consequent.kind, ExprKind::Closure { .. }))
                || (right_hint.is_none() && matches!(alternate.kind, ExprKind::Closure { .. })));
        let hint = if infer_nullable_closure { None } else { hint };
        let hint = if hint.is_none() && !infer_nullable_closure {
            self.callback_value_hint(context, consequent)?
                .or(self.callback_value_hint(context, alternate)?)
        } else {
            hint
        };
        let before = context
            .names
            .values()
            .map(|&id| (id, Availability::of(&context.bindings[id])))
            .collect::<Vec<_>>();
        let first_loan = context.capture_loans.len();
        let left_context =
            if assignment_alias && borrowed && self.borrowed_place_hint(context, consequent) {
                hint.and_then(Type::union).map(Type::BorrowedUnion)
            } else {
                hint
            };
        Ok(Box::new(ConditionalPlan {
            else_flow,
            hint,
            assignment_alias,
            borrowed,
            infer_nullable_closure,
            before,
            first_loan,
            left_context,
            right_context: None,
            after_left: Vec::new(),
            after_left_flow: HashMap::new(),
            left_loans: Vec::new(),
        }))
    }

    #[inline(never)]
    fn conditional_between(
        &mut self,
        context: &mut Context,
        consequent: &Expr,
        alternate: &Expr,
        plan: &mut ConditionalPlan,
        left: Result<Type, Diagnostic>,
    ) -> Result<Type, Diagnostic> {
        let left = left?;
        let (hint, assignment_alias, borrowed, infer_nullable_closure, first_loan) = (plan.hint, plan.assignment_alias, plan.borrowed, plan.infer_nullable_closure, plan.first_loan);
        let else_flow = std::mem::take(&mut plan.else_flow);
        let before = &plan.before;
        self.require(left, hint, consequent.span)?;
        let after_left = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let after_left_flow = context.flow.clone();
        let left_loans = context.capture_loans[first_loan..].to_vec();
        context.capture_loans.truncate(first_loan);
        for (id, state) in before {
            state.clone().restore(&mut context.bindings[*id]);
        }
        context.flow = else_flow;
        let right_context =
            if assignment_alias && borrowed && self.borrowed_place_hint(context, alternate) {
                hint.and_then(Type::union).map(Type::BorrowedUnion)
            } else {
                if infer_nullable_closure {
                    None
                } else {
                    hint.or(Some(left))
                }
            };
        plan.after_left = after_left;
        plan.after_left_flow = after_left_flow;
        plan.left_loans = left_loans;
        plan.right_context = right_context;
        Ok(left)
    }

    #[inline(never)]
    fn conditional_join(
        &mut self,
        context: &mut Context,
        consequent: &mut Expr,
        alternate: &mut Expr,
        plan: Box<ConditionalPlan>,
        left: Type,
        right: Result<Type, Diagnostic>,
    ) -> Result<Type, Diagnostic> {
        let mut left = left;
        let mut right = right?;
        let ConditionalPlan { assignment_alias, borrowed, infer_nullable_closure, before, after_left, after_left_flow, left_loans, .. } = *plan;
        context.flow = crate::flow::join(&after_left_flow, &context.flow);
        if infer_nullable_closure && left != right {
            let joined = self.unions.intern([left, right], alternate.span)?;
            let id = joined.union().expect("different nullable alternatives");
            if left != joined {
                consequent.inject(id);
                left = joined;
            }
            if right != joined {
                alternate.inject(id);
                right = joined;
            }
        }
        if left != right && !(assignment_alias && left.owning() == right.owning()) {
            return Err(Diagnostic::new(
                "type",
                "Conditional branches require the exact same value type and callable origin.",
                alternate.span,
            ));
        }
        for ((id, _), left_state) in before.into_iter().zip(after_left) {
            left_state
                .join(Availability::of(&context.bindings[id]))
                .restore(&mut context.bindings[id]);
        }
        for loan in left_loans {
            if !context.capture_loans.contains(&loan) {
                context.capture_loans.push(loan);
            }
        }
        Ok(if assignment_alias && borrowed {
            Type::BorrowedUnion(left.union().expect("conditional alias union"))
        } else {
            left
        })
    }

    fn conditional_hint(
        &self,
        context: &Context,
        consequent: &Expr,
        alternate: &Expr,
    ) -> Option<Type> {
        let left = self.expression_type_hint(context, consequent);
        let right = self.expression_type_hint(context, alternate);
        if let (Some(left), Some(right)) = (left, right)
            && (left.union().is_some()
                || right.union().is_some()
                || matches!(left, Type::Null | Type::Undefined)
                || matches!(right, Type::Null | Type::Undefined))
        {
            let joined = self
                .unions
                .intern([left.owning(), right.owning()], alternate.span)
                .ok()?;
            return Some(
                if matches!(left, Type::BorrowedUnion(_)) || matches!(right, Type::BorrowedUnion(_))
                {
                    Type::BorrowedUnion(joined.union()?)
                } else {
                    joined
                },
            );
        }
        if matches!(
            left,
            Some(Type::Number | Type::Array(ArrayElement::Number(_)))
        ) {
            left
        } else if matches!(
            right,
            Some(Type::Number | Type::Array(ArrayElement::Number(_)))
        ) {
            right
        } else {
            left.or(right)
        }
    }

    /// An omitted `name?: Type` field holds `undefined`. The record keeps
    /// its fixed shape; omitted fields follow the written ones.
    #[inline(never)]
    fn fill_omitted_optional_fields(&self, index: usize, fields: &mut Vec<(Token, Expr)>, span: Span) {
        for field in &self.program.records[index].fields {
            if field.optional && !fields.iter().any(|(key, _)| key.text == field.name.text) {
                fields.push((
                    Token { text: field.name.text.clone(), span, kind: TokenKind::Name },
                    Expr::new(ExprKind::Undefined, span),
                ));
            }
        }
    }

    fn record_literal(
        &mut self,
        context: &mut Context,
        fields: &mut Vec<(Token, Expr)>,
        loans: &Loans,
        expected: Option<Type>,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if expected.is_none() && self.builtin_literals {
            let mut inferred = Vec::new();
            let mut types = HashMap::new();
            for (key, value) in fields.iter_mut() {
                if types.contains_key(&key.text) {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Duplicate field {:?}.", key.text),
                        key.span,
                    ));
                }
                let actual = self.expression(context, value, Mode::Consume, loans, None)?;
                if !matches!(
                    actual,
                    Type::Int
                        | Type::Number
                        | Type::Boolean
                        | Type::Null
                        | Type::Undefined
                        | Type::String
                        | Type::Array(_)
                        | Type::Record(_)
                ) {
                    return Err(Diagnostic::new(
                        "type",
                        "Builtin object fields must be printable values.",
                        value.span,
                    ));
                }
                types.insert(key.text.clone(), actual);
                inferred.push(RecordField {
                    name: key.clone(),
                    annotation: crate::type_syntax::ValueAnnotation {
                        kind: crate::type_syntax::ValueAnnotationKind::Named(
                            crate::type_syntax::TypeName {
                                text: key.text.clone(),
                                span: key.span,
                            },
                        ),
                        span: key.span,
                    },
                    ty: Some(actual),
                    callable_declaration: None, optional: false,
                });
            }
            let id = self.program.records.len();
            self.program.records.push(Record {
            map_definition: None, set_definition: None,                enum_definition: None,
                application: None,
                name: Token {
                    text: format!("BuiltinObject{id}"),
                    span,
                    kind: TokenKind::Name,
                },
                type_parameters: Vec::new(),
                fields: inferred,
            });
            self.fields.push(types);
            return Ok(Type::Record(id));
        }
        let index = match expected {
            Some(Type::Record(index)) => index,
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "A record literal needs a named record type from its annotation or call/return context.",
                    span,
                ));
            }
        };
        self.fill_omitted_optional_fields(index, fields, span);
        let mut seen = HashSet::new();
        for (key, value) in fields {
            if !seen.insert(key.text.clone()) {
                return Err(Diagnostic::new(
                    "duplicate",
                    format!("Duplicate field {:?}.", key.text),
                    key.span,
                ));
            }
            let field_type = self.fields[index].get(&key.text).copied().ok_or_else(|| {
                Diagnostic::new(
                    "type",
                    format!(
                        "Unknown field {:?} on {}.",
                        key.text,
                        self.type_name(Type::Record(index))
                    ),
                    key.span,
                )
            })?;
            let actual = if let Type::StoredCallable(family) = field_type {
                self.stored_callable_value(context, value, family, loans)?
            } else {
                self.expression(context, value, Mode::Consume, loans, Some(field_type))?
            };
            self.require(actual, Some(field_type), value.span)?;
        }
        let mut missing = self.fields[index]
            .keys()
            .filter(|name| !seen.contains(*name))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            missing.sort();
            return Err(Diagnostic::new(
                "type",
                format!("Missing fields: {}.", missing.join(", ")),
                span,
            ));
        }
        Ok(Type::Record(index))
    }

    /// Convert a concrete owned environment once, after checking its contract.
    fn stored_callable_value(
        &mut self,
        context: &mut Context,
        value: &mut Expr,
        family: usize,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        if matches!(value.kind, ExprKind::Conditional { .. }) {
            return self.expression(
                context,
                value,
                Mode::Consume,
                loans,
                Some(Type::StoredCallable(family)),
            );
        }
        let declaration = self.callables.storage.declaration(family).ok_or_else(|| {
            Diagnostic::new(
                "callable-layout",
                "Unknown callable storage signature.",
                value.span,
            )
        })?;
        if let ExprKind::Closure { id, .. } = &value.kind {
            self.escaping.insert(*id);
        }
        let expected = Type::Callback(declaration);
        // A value that is already stored moves as its own family; only a
        // fresh environment is checked against the declared signature.
        let stored = matches!(self.expression_type_hint(context, value), Some(Type::StoredCallable(_)));
        let actual = self.expression(
            context,
            value,
            Mode::Consume,
            loans,
            Some(if stored { Type::StoredCallable(family) } else { expected }),
        )?;
        if matches!(actual, Type::StoredCallable(_)) {
            self.require(actual, Some(Type::StoredCallable(family)), value.span)?;
            return Ok(actual);
        }
        self.require(actual, Some(expected), value.span)?;
        let Type::Closure(mut origin) = actual else {
            return Err(Diagnostic::new(
                "callable-layout",
                "Stored callable construction requires a concrete owned environment.",
                value.span,
            ));
        };
        if self.callables.callbacks[declaration].kind.is_once()
            && self.closures[origin].invocation != Mode::Consume
        {
            origin = self.once_return(origin, context, value.span);
            value.wrap_once(origin);
        }
        if !context.template && !self.discovering_results && !self.effects_only {
            self.callables.storage.register_origin(
                family,
                origin,
                &self.callables.callbacks,
                &self.callables.shapes,
                &self.closures,
                value.span,
            )?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        value.wrap_storage(family);
        Ok(Type::StoredCallable(family))
    }

    // Integer literals are contextual; names and typed storage/call results are
    // concrete int anchors. Walk syntax iteratively so this probe does not add a
    // recursive frame for every nested operator in the analysis stack.
    fn int_operator_anchor(&self, context: &Context, expr: &Expr) -> bool {
        let mut pending = vec![expr];
        while let Some(expr) = pending.pop() {
            match &expr.kind {
                ExprKind::Project { value, .. }
                | ExprKind::Inject { value, .. }
                | ExprKind::Unary { value, .. } => pending.push(value),
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(alternate);
                    pending.push(consequent);
                }
                ExprKind::Binary { left, right, .. } => {
                    pending.push(right);
                    pending.push(left);
                }
                ExprKind::TypeAssertion { .. }
                | ExprKind::Name(_)
                | ExprKind::Call(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Index(_, _)
                | ExprKind::Update { .. }
                | ExprKind::Assign { .. }
                | ExprKind::CoalesceAssign { .. }
                | ExprKind::CompoundAssign { .. }
                    if self.expression_type_hint(context, expr) == Some(Type::Int) =>
                {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn bitwise_or_power_type(
        &self,
        context: &Context,
        left: &Expr,
        right: Option<&Expr>,
        expected: Option<Type>,
    ) -> Type {
        if let Some(ty @ (Type::Int | Type::Number)) = expected {
            ty
        } else if self.int_operator_anchor(context, left)
            || right.is_some_and(|right| self.int_operator_anchor(context, right))
        {
            Type::Int
        } else {
            // Keep the existing Number behavior of unanchored literal syntax.
            Type::Number
        }
    }

    fn expression_type_hint(&self, context: &Context, expr: &Expr) -> Option<Type> {
        if matches!(expr.kind, ExprKind::Selected { .. }) {
            return expr.ty;
        }
        if let ExprKind::Project { narrowed, .. } = &expr.kind {
            return Some(*narrowed);
        }
        if let Some((_, _, narrowed)) = self.flow_view(context, expr) {
            return Some(narrowed);
        }
        if let Some(ty) = expr.ty {
            return Some(ty);
        }
        if let ExprKind::Field(base, _) = &expr.kind
            && let ExprKind::Name(name) = &base.kind
            && !context.names.contains_key(name)
            && let Some(&id) = self.record_names.get(name)
            && let Some(definition) = &self.program.records[id].enum_definition {
            return Some(Type::Enum(id, definition.kind));
        }
        match &expr.kind {
            ExprKind::Sequence(values)=>values.last().and_then(|value|self.expression_type_hint(context,value)),
            ExprKind::EnumMember { enumeration, .. } => self.program.records[*enumeration].enum_definition.as_ref().map(|definition| Type::Enum(*enumeration, definition.kind)),
            ExprKind::Optional { .. } | ExprKind::Selected { .. } | ExprKind::SpreadCall(_) | ExprKind::SpreadValue(_) => expr.ty,
            ExprKind::TypeAssertion {value,annotation,satisfies} => if *satisfies {self.expression_type_hint(context,value)}else{self.resolve_value_annotation(annotation,false).ok()},
            ExprKind::OptionalChain(value) => self.optional_hint(context, value),
            ExprKind::Unary { op: UnaryOp::NonNull, value, .. } => {
                let ty = self.expression_type_hint(context, value)?;
                self.unions.payload(ty.owning()).or_else(|| (!matches!(ty, Type::Null | Type::Undefined | Type::Void) && ty.union().is_none()).then_some(ty))
            }
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary,
                value,
                ..
            } => self
                .expression_type_hint(context, value)
                .and_then(|ty| self.unions.payload(ty).or(Some(ty))),
            ExprKind::Integer(_) => Some(Type::Int),
            ExprKind::Number(_) => Some(Type::Number),
            ExprKind::Boolean(_) => Some(Type::Boolean),
            ExprKind::Null => Some(Type::Null),
            ExprKind::Undefined => Some(Type::Undefined),
            ExprKind::String(_) | ExprKind::Template(_) => Some(Type::String),
            ExprKind::Array(values) if values.is_empty() && self.builtin_literals => {
                Some(Type::Array(ArrayElement::Int(0)))
            }
            ExprKind::Name(name) => context
                .names
                .get(name)
                .map(|id| context.bindings[*id].ty)
                .or_else(|| {
                    if context.pending_names.contains(name) {
                        return None;
                    }
                    if !self.function_names.contains_key(name)
                        && crate::syntax::global_number_member(name, false).is_some()
                    {
                        return Some(Type::Number);
                    }
                    if name == "undefined"
                        && !context
                            .lexical_scopes
                            .iter()
                            .any(|scope| scope.contains(name))
                        && !self.function_names.contains_key(name)
                    {
                        return Some(Type::Undefined);
                    }
                    self.function_names
                        .get(name)
                        .and_then(|id| self.function_values.get(id))
                        .map(|&id| Type::Closure(id))
                }),
            ExprKind::Project { narrowed, .. } => Some(*narrowed),
            ExprKind::Inject { union, .. } => Some(Type::Union(*union)),
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => self.conditional_hint(context, consequent, alternate),
            ExprKind::CoalesceAssign { target, value, .. } => {
                self.coalesce_hint(context, target, value, expr.span, false)
            }
            ExprKind::LogicalAssign { .. } => Some(Type::Boolean),
            ExprKind::Update { target, .. }
            | ExprKind::Assign { target, .. }
            | ExprKind::CompoundAssign { target, .. } => self.expression_type_hint(context, target),
            ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected,
                value,
                ..
            } => self.expression_type_hint(context, value),
            ExprKind::Binary {
                op,
                left,
                right,
                ..
            } if matches!(op,BinaryOp::Coalesce|BinaryOp::PatternDefault) => self.coalesce_hint(context, left, right, expr.span,*op==BinaryOp::PatternDefault),
            ExprKind::Unary { op, value, .. } => {
                if *op==UnaryOp::PatternSelection {return self.pattern_selection_hint(context,value);}
                if *op == UnaryOp::Typeof {
                    return Some(Type::String);
                }
                if *op == UnaryOp::Void {
                    return Some(Type::Undefined);
                }
                if *op == UnaryOp::Positive && self.expression_type_hint(context, value).is_some_and(|ty| !matches!(ty, Type::Int | Type::Number)) {
                    Some(Type::Number)
                } else if *op == UnaryOp::BitNot {
                    Some(self.bitwise_or_power_type(context, value, None, None))
                } else if *op == UnaryOp::Not {
                    Some(Type::Boolean)
                } else {
                    self.expression_type_hint(context, value)
                }
            }
            ExprKind::Binary {
                op, left, right, ..
            } => match op {
                BinaryOp::And | BinaryOp::Or => self.logical_value_hint(context, left, right, *op, expr.span),
                op if op.bitwise_or_power() => {
                    Some(self.bitwise_or_power_type(context, left, Some(right), None))
                }
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Remainder => {
                    let left = self.expression_type_hint(context, left);
                    let right = self.expression_type_hint(context, right);
                    if *op == BinaryOp::Add
                        && (left == Some(Type::String) || right == Some(Type::String))
                    {
                        Some(Type::String)
                    } else if left == Some(Type::Number) || right == Some(Type::Number) {
                        Some(Type::Number)
                    } else {
                        Some(Type::Int)
                    }
                }
                _ => Some(Type::Boolean),
            },
            ExprKind::Field(base, name) if matches!(&base.kind, ExprKind::Name(namespace) if NumericConstant::lookup(namespace, name).is_some()) => {
                Some(Type::Number)
            }
            ExprKind::Field(base, name) => {
                let hint = self.expression_type_hint(context, base)?;
                let direct = matches!(base.kind, ExprKind::Unary { op: UnaryOp::OptionalBoundary, .. });
                let optional = Self::optional_hint_boundary(base);
                let receiver = if direct {self.unions.payload(hint.owning()).unwrap_or(hint.owning())}
                    else if optional && let Some(id)=hint.union(){self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),base.span).ok()?}
                    else{hint};
                let result = match receiver {
                    Type::Set(_) | Type::Map(_) if name == "size" => Some(Type::Int),
                    Type::Record(id) if name == "length" && self.program.records[id].is_tuple() => Some(Type::Int),
                    Type::Record(id) => self.fields[id].get(name).copied(),
                    Type::Array(_) | Type::String if name == "length" => Some(Type::Int),
                    _ => None,
                }?;
                if optional {
                    let result = self.unions.intern([result, Type::Undefined], expr.span).ok()?;
                    Some(if matches!(hint, Type::BorrowedUnion(_)) && !result.is_copy() { Type::BorrowedUnion(result.union()?) } else { result })
                } else { Some(result) }
            },
            ExprKind::Index(base, index) => {
                let hint=self.expression_type_hint(context,base)?;
                let optional=Self::optional_hint_boundary(base);
                let receiver=if optional && let Some(id)=hint.union(){self.unions.intern(self.unions.members(id).into_iter().filter(|ty|*ty!=Type::Undefined),base.span).ok()?}else{hint};
                let result=match receiver {
                    Type::Record(id) if self.program.records[id].is_tuple()=>self.fields[id].get(&Self::flat_literal_depth(index)?.to_string()).copied(),
                    Type::Array(element)=>Some(element.ty()),
                    Type::String=>Some(Type::String),
                    _=>None,
                }?;
                if optional{self.unions.intern([result,Type::Undefined],expr.span).ok()}else{Some(result)}
            },
            ExprKind::SetConstructor(_, Some(annotation)) | ExprKind::MapConstructor(_, Some(annotation)) => self.resolve_value_annotation(annotation, false).ok(),
            ExprKind::SetConstructor(_, None) | ExprKind::MapConstructor(_, None) => None,
            ExprKind::ArrayOf(_, Some(element)) => self
                .resolve_value_annotation(element, false)
                .ok()
                .and_then(|ty| self.constructor_array_type(ty, expr.span).ok()),
            ExprKind::Array(values) | ExprKind::ArrayOf(values, None) => {
                let hints = values
                    .iter()
                    .filter_map(|value| {
                        if let ExprKind::Unary { op: UnaryOp::Spread, value, .. } = &value.kind {
                            match self.expression_type_hint(context, value)? { Type::Array(element) => Some(element.ty()), _ => None }
                        } else { self.expression_type_hint(context, value) }
                    })
                    .collect::<Vec<_>>();
                let leaf = hints
                    .iter()
                    .copied()
                    .find(|ty| matches!(ty, Type::Number | Type::Array(ArrayElement::Number(_))))
                    .or_else(|| hints.first().copied())?;
                ArrayElement::from_value_type(leaf).map(Type::Array)
            }
            ExprKind::Call(callee, arguments) => {
                if matches!(
                    callee.kind,
                    ExprKind::Conditional { .. }
                        | ExprKind::Binary {
                            op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                            ..
                        }
                        | ExprKind::Unary {
                            op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                            ..
                        }
                ) {
                    return match self.expression_type_hint(context, callee)? {
                        Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => None,
                        Type::Closure(id) => Some(self.closures[id].function.result),
                        Type::Callback(id) => Some(self.callables.callbacks[id].result),
                        _ => None,
                    };
                }
                if let ExprKind::Name(name) = &callee.kind {
                    if let Some((_, binding)) = crate::native_bindings::by_symbol(name) {
                        if binding.result == crate::native_bindings::NativeType::SharedCallback { return None; }
                        return Some(native_type(binding.result));
                    }
                    if !context.names.contains_key(name)
                        && !context.pending_names.contains(name)
                        && !self.function_names.contains_key(name)
                        && let Some(member) = crate::syntax::global_number_member(name, true)
                    {
                        return Some(if member.starts_with("parse") {
                            Type::Number
                        } else {
                            Type::Boolean
                        });
                    }
                    if name == "clone" && arguments.len() == 1 {
                        return self
                            .expression_type_hint(context, &arguments[0])
                            .map(Type::owning);
                    }
                    if name == "String" {
                        return Some(Type::String);
                    }
                    if name == "Boolean" {
                        return Some(Type::Boolean);
                    }
                    if name == "Number" {
                        return Some(Type::Number);
                    }
                    if let Some(id) = context.names.get(name) {
                        return match context.bindings[*id].ty {
                            Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => None,
                        Type::Closure(id) => Some(self.closures[id].function.result),
                            Type::Callback(id) => Some(self.callables.callbacks[id].result),
                            Type::StoredCallable(id) => self
                                .callables
                                .storage
                                .declaration(id)
                                .map(|declaration| self.callables.callbacks[declaration].result),
                            _ => None,
                        };
                    }
                    return self
                        .function_names
                        .get(name)
                        .map(|id| self.signatures[*id].result);
                }
                if let ExprKind::Field(base, name) = &callee.kind {
                    let receiver_hint = self.expression_type_hint(context, base);
                    if let Some(Type::Map(id)) = receiver_hint {
                        let result = self.map_method_type(id, name, expr.span).ok()?;
                        return Some(if name == "get" && Self::stable_at_receiver(base) && !result.is_copy() { Type::BorrowedUnion(result.union()?) } else { result });
                    }
                    if let Some(Type::Set(id)) = receiver_hint {
                        return self.set_call_type_hint(context,id,name,arguments);
                    }
                    if name == "isArray"
                        && matches!(&base.kind,ExprKind::Name(namespace) if namespace=="Array")
                    {
                        return Some(Type::Boolean);
                    }

                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Object") && name == "keys" {
                        return Some(Type::Array(ArrayElement::String(0)));
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Object") && name == "is" {
                        return Some(Type::Boolean);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "String" && !context.names.contains_key(namespace))
                        && matches!(name.as_str(), "fromCodePoint" | "fromCharCode")
                    {
                        return Some(Type::String);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                        && matches!(name.as_str(), "parseFloat" | "parseInt")
                    {
                        return Some(Type::Number);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                        && number_predicate_target(name).is_some()
                    {
                        return Some(Type::Boolean);
                    }
                    if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Math")
                        && math_target(name).is_some()
                    {
                        return Some(Type::Number);
                    }
                    return match name.as_str() {
                        "trim" | "trimStart" | "trimEnd" | "trimLeft" | "trimRight"
                        | "toLowerCase" | "toUpperCase" | "toLocaleLowerCase" | "toLocaleUpperCase"
                        | "charAt" | "repeat" | "replace"
                        | "replaceAll" | "padStart" | "padEnd" | "substring" | "substr"
                        | "toFixed" | "toExponential" | "toPrecision" => Some(Type::String),
                        "isWellFormed" => Some(Type::Boolean),
                        "toWellFormed" => Some(Type::String),
                        STRING_CODE_POINT_SLOT => Some(Type::Array(ArrayElement::Int(0))),
                        "charCodeAt" => Some(Type::Number),
                        method if string_html_target(method).is_some() => Some(Type::String),
                        "valueOf" => receiver_hint.filter(|ty| {
                            matches!(ty, Type::String | Type::Boolean | Type::Number | Type::Int)
                        }),
                        "toString" => receiver_hint
                            .filter(|ty| {
                                matches!(
                                    ty,
                                    Type::String | Type::Boolean | Type::Int | Type::Array(_)
                                )
                            })
                            .map(|_| Type::String),
                        "split" => Some(Type::Array(ArrayElement::String(0))),
                        "join" => Some(Type::String),
                        "some" | "every" => Some(Type::Boolean),
                        "findIndex" | "findLastIndex" | "unshift" => Some(Type::Int),
                        "concat" => receiver_hint
                            .filter(|ty| matches!(ty, Type::String | Type::Array(_))),
                        "entries" => {
                            let Type::Array(element) = receiver_hint? else { return None; };
                            let label = format!("[int,{}]",self.type_name(element.ty()));
                            self.record_names.get(&label).copied().map(|pair|Type::Array(ArrayElement::Record(pair,0)))
                        }
                        "keys" => receiver_hint
                            .filter(|ty| matches!(ty, Type::Array(_)))
                            .map(|_| Type::Array(ArrayElement::Int(0))),
                        "filter" | "toReversed" | "toSorted" | "with" | "toSpliced" | "splice" | "values" => {
                            receiver_hint
                                .filter(|ty| matches!(ty, Type::Array(_)))
                        }
                        "reduce" | "reduceRight" => {
                            let argument = arguments.first()?;
                            match &argument.kind {
                                ExprKind::Closure {
                                    result: ResultAnnotation::Named(token),
                                    ..
                                } => self.resolve_value_annotation(token, true).ok(),
                                _ => match self.expression_type_hint(context, argument)? {
                                    Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => None,
                        Type::Closure(id) => Some(self.closures[id].function.result),
                                    Type::Callback(id) => Some(self.callables.callbacks[id].result),
                                    Type::Callable(id) => Some(self.callables.results[id].result),
                                    _ => None,
                                },
                            }
                        }
                        "flat" => {
                            let Type::Array(element) = receiver_hint?
                            else {
                                return None;
                            };
                            let depth = match arguments.first() {
                                None => 1,
                                Some(arg) => Self::flat_literal_depth(arg)
                                    .or_else(|| (element.depth() == 0).then_some(0))?,
                            };
                            Some(Type::Array(element.with_depth(
                                element.depth().saturating_sub(depth.clamp(0, 127) as u8),
                            )))
                        }
                        "map" | "flatMap" => {
                            let argument = arguments.first()?;
                            let result = match &argument.kind {
                                ExprKind::Closure {
                                    result: ResultAnnotation::Named(token),
                                    ..
                                } => self.resolve_value_annotation(token, true).ok()?,
                                _ => match self.expression_type_hint(context, argument)? {
                                    Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => return None,
                                    Type::Closure(id) => self.closures[id].function.result,
                                    Type::Callback(id) => self.callables.callbacks[id].result,
                                    Type::Callable(id) => self.callables.results[id].result,
                                    _ => return None,
                                },
                            };
                            if name == "flatMap" && matches!(result, Type::Array(_)) {
                                Some(result)
                            } else {
                                Self::transform_array_type(result, argument.span).ok()
                            }
                        }
                        "slice" => receiver_hint
                            .filter(|ty| matches!(ty, Type::String | Type::Array(_))),
                        "at" | "find" | "findLast" => {
                            match receiver_hint? {
                                Type::Array(element) => {
                                    let result = self
                                        .unions
                                        .intern([element.ty(), Type::Undefined], expr.span)
                                        .ok()?;
                                    Some(if Self::stable_at_receiver(base) && !result.is_copy() {
                                        Type::BorrowedUnion(result.union()?)
                                    } else {
                                        result
                                    })
                                }
                                _ => None,
                            }
                        }
                        "pop" | "shift" => match receiver_hint? {
                            Type::Array(element) => self
                                .unions
                                .intern([element.ty(), Type::Undefined], expr.span)
                                .ok(),
                            _ => None,
                        },
                        "remove" => match receiver_hint? {
                            Type::Array(element) => Some(element.ty()),
                            _ => None,
                        },
                        _ => None,
                    };
                }
                match self.expression_type_hint(context, callee)? {
                    Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => None,
                        Type::Closure(id) => Some(self.closures[id].function.result),
                    Type::Callback(id) => Some(self.callables.callbacks[id].result),
                    Type::Callable(id) => Some(self.callables.results[id].result),
                    Type::StoredCallable(id) => self
                        .callables
                        .storage
                        .declaration(id)
                        .map(|declaration| self.callables.callbacks[declaration].result),
                    _ => None,
                }
            }
            ExprKind::Closure { .. } | ExprKind::Record(_) => None,
        }
    }

    fn array_constructor(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        if matches!(&expr.kind,ExprKind::ArrayOf(args,_) if args.iter().any(|arg|matches!(arg.kind,ExprKind::Unary{op:UnaryOp::Spread,..}))) {return self.spread_array_construction(context,expr,loans,expected);}
        let ExprKind::ArrayOf(values, element) = &mut expr.kind else {
            unreachable!()
        };
        let annotated = element
            .as_ref()
            .map(|element| self.resolve_value_annotation(element, false))
            .transpose()?;
        let array = annotated
            .map(|ty| self.constructor_array_type(ty, expr.span))
            .transpose()?;
        if let Some(array) = array {
            self.require(array, expected, expr.span)?;
        }
        if values.is_empty() && !matches!(array.or(expected), Some(Type::Array(_))) {
            return Err(Diagnostic::new(
                "array-type",
                "An empty Array.of needs an explicit element type or array context.",
                expr.span,
            ));
        }
        let result = self.array_literal(context, values, loans, array.or(expected), expr.span)?;
        let values = std::mem::take(values);
        expr.kind = ExprKind::Array(values);
        expr.target = Some(CallTarget::ArrayOf);
        Ok(result)
    }

    fn constructor_array_type(&self, element: Type, span: Span) -> Result<Type, Diagnostic> {
        let element = ArrayElement::from_value_type(element).ok_or_else(|| {
            if matches!(element, Type::Array(_)) {
                Diagnostic::new(
                    "unsupported",
                    "Array type nesting exceeds the prototype limit of 128.",
                    span,
                )
            } else {
                Diagnostic::new(
                    "array-element",
                    "Array.of elements must use one concrete supported dense value type.",
                    span,
                )
            }
        })?;
        Ok(Type::Array(element))
    }

    fn array_literal(
        &mut self,
        context: &mut Context,
        values: &mut [Expr],
        loans: &Loans,
        expected: Option<Type>,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let mixed = if expected.is_none() {
            self.mixed_literal_element(context, values, span)?
        } else {
            None
        };
        let mut element = match expected {
            Some(Type::Array(element)) => Some(element),
            _ if mixed.is_some() => mixed,
            _ => values
                .iter()
                .filter_map(|value| self.expression_type_hint(context, value))
                .find_map(|ty| match ty {
                    Type::Number => Some(ArrayElement::Number(0)),
                    Type::Array(ArrayElement::Number(depth)) if depth < 127 => {
                        Some(ArrayElement::Number(depth + 1))
                    }
                    _ => None,
                }),
        };
        if values.is_empty() && element.is_none() && self.builtin_literals {
            element = Some(ArrayElement::Int(0));
        }
        for value in values {
            if element.is_none() && let ExprKind::Closure { id, .. } = value.kind { self.escaping.insert(id); }
            let actual = if let Some(ArrayElement::StoredCallable(family, 0)) = element {
                self.stored_callable_value(context, value, family, loans)?
            } else {
                self.expression(
                    context,
                    value,
                    Mode::Consume,
                    loans,
                    element.map(ArrayElement::ty),
                )?
            };
            let actual = if element.is_none() { self.inferred_callable_array_element(context, value, actual)? } else { actual };
            let Type::Array(inferred) = Self::transform_array_type(actual, value.span)? else {
                unreachable!("array element conversion returns an array");
            };
            self.require(actual, element.map(ArrayElement::ty), value.span)?;
            element.get_or_insert(inferred);
        }
        Ok(Type::Array(element.ok_or_else(|| {
            Diagnostic::new(
                "array-type",
                "An empty array needs an array annotation or call/return context.",
                span,
            )
        })?))
    }

    /// An array literal without a declared type whose values have different
    /// data types takes the union of those types as its element type. Values
    /// of one type, and int values beside number values, keep the plain rules.
    #[inline(never)]
    fn mixed_literal_element(
        &mut self,
        context: &Context,
        values: &[Expr],
        span: Span,
    ) -> Result<Option<ArrayElement>, Diagnostic> {
        // An empty array literal has no element type of its own; it takes the
        // type of its neighbours under the plain rules.
        fn has_empty_literal(value: &Expr) -> bool {
            match &value.kind {
                ExprKind::Array(values) => values.is_empty() || values.iter().any(has_empty_literal),
                _ => false,
            }
        }
        if values.iter().any(has_empty_literal) {
            return Ok(None);
        }
        let mut types = Vec::new();
        for value in values {
            let Some(ty) = self.expression_type_hint(context, value) else { return Ok(None) };
            let ty = ty.owning();
            let data = match ty {
                Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined | Type::Record(_) | Type::Array(_) => true,
                Type::Union(id) => !self.unions.members(id).iter().any(|member| {
                    matches!(member, Type::Closure(_) | Type::Callable(_) | Type::Callback(_) | Type::StoredCallable(_))
                }),
                _ => false,
            };
            if !data {
                return Ok(None);
            }
            if !types.contains(&ty) {
                types.push(ty);
            }
        }
        // An int beside a number is a number, as in a plain number array.
        if types.contains(&Type::Number) {
            types.retain(|ty| *ty != Type::Int);
        }
        if types.len() < 2 {
            return Ok(None);
        }
        Ok(match self.unions.intern(types, span)? {
            Type::Union(id) => Some(ArrayElement::Union(id, 0)),
            _ => None,
        })
    }

    fn numeric_constant_access(mode: Mode, span: Span) -> Result<Type, Diagnostic> {
        if mode == Mode::Mutate {
            Err(Diagnostic::new(
                "type",
                "Numeric namespace constants are read-only.",
                span,
            ))
        } else {
            Ok(Type::Number)
        }
    }

    fn field_access(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        name: &str,
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if matches!(&base.kind, ExprKind::Name(namespace) if NumericConstant::lookup(namespace, name).is_some())
        {
            return Self::numeric_constant_access(mode, span);
        }
        if self.map_receiver_hint(context, base) { return self.map_size(context, base, name, mode, loans, span); }
        if self.set_receiver_hint(context, base) {
            return self.set_size(context, base, name, mode, loans, span);
        }
        if name == "length"
            && !matches!(
                base.kind,
                ExprKind::Name(_)
                    | ExprKind::Selected { .. }
                    | ExprKind::Index(_, _)
                    | ExprKind::Field(_, _)
                    | ExprKind::Project { .. }
            )
        {
            let actual = self.expression(context, base, Mode::Read, loans, None)?;
            if !matches!(actual, Type::String | Type::Array(_)) && !matches!(actual,Type::Record(id) if self.program.records[id].is_tuple()) {
                return Err(Diagnostic::new(
                    "type",
                    "Length requires a string or array.",
                    base.span,
                ));
            }
            if mode == Mode::Mutate {
                return Err(Diagnostic::new(
                    "unsupported",
                    "String and array length are read-only; use push() to append an array element.",
                    span,
                ));
            }
            return Ok(Type::Int);
        }
        if !Self::named_storage(base) {
            return self.temporary_field(context, base, name, mode, loans, span);
        }
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Sequence(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(base.kind, ExprKind::Sequence(_) | ExprKind::Unary { op: UnaryOp::NonNull, .. }) && Self::named_storage(base)) {
            return Err(Diagnostic::new(
                "unsupported",
                "Field access requires named, indexed, or field storage.",
                span,
            ));
        }
        let top_projection = context.projection.is_none();
        // Ordinary fully available field reads need no owned path allocation.
        let partial = if top_projection && context.has_field_moves {
            let mut root: &Expr = base;
            loop {
                match &root.kind {
                    ExprKind::Field(parent, _) => root=parent,
                    ExprKind::Sequence(values) => root=values.last().expect("nonempty sequence"),
                    _ => break,
                }
            }
            matches!(&root.kind, ExprKind::Name(root) if context.names.get(root).is_some_and(|&id| !context.bindings[id].moved_fields.is_empty()))
        } else {
            false
        };
        let mut direct = if top_projection && partial {
            Self::direct_fields(base)
        } else {
            None
        };
        if let Some((_, path)) = &mut direct {
            path.push(name.to_owned());
        }
        if top_projection && let Some((root, path)) = &direct {
            context.projection = Some(Projection {
                root: root.clone(),
                path: path.clone(),
                replacing: context.replacement == Some(span),
                span,
            });
        }
        let base_mode = if mode == Mode::Mutate {
            Mode::Mutate
        } else {
            Mode::Read
        };
        let base_type = self.expression(context, base, base_mode, loans, None);
        if top_projection {
            context.projection = None;
        }
        let base_type = base_type?;
        if (matches!(base_type, Type::Array(_) | Type::String) || matches!(base_type, Type::Record(id) if self.program.records[id].is_tuple())) && name == "length" {
            if mode == Mode::Mutate {
                return Err(Diagnostic::new(
                    "unsupported",
                    "String and array length are read-only; use push() to append an array element.",
                    span,
                ));
            }
            return Ok(Type::Int);
        }
        let field_type = match base_type {
            Type::Record(index) => self.fields[index].get(name).copied(),
            _ => None,
        }
        .ok_or_else(|| {
            Diagnostic::new(
                "type",
                format!(
                    "Type {:?} has no field {name:?}.",
                    self.type_name(base_type)
                ),
                span,
            )
        })?;
        if mode == Mode::Consume && !field_type.is_copy() {
            let mut parent: &Expr = base;
            loop {
                match &parent.kind {
                    ExprKind::Sequence(values)=>parent=values.last().expect("nonempty sequence"),
                    ExprKind::Project {
                        narrowed: Type::Record(_),
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            "unsupported",
                            "Move the whole narrowed record payload to a local before consuming its owned fields.",
                            span,
                        ));
                    }
                    ExprKind::Project { value, .. }
                    | ExprKind::Field(value, _)
                    | ExprKind::Index(value, _) => parent = value,
                    _ => break,
                }
            }
            let Some((root, mut path)) = Self::direct_fields(base) else {
                return Err(Diagnostic::new(
                    "borrowed-element",
                    "Moving a field through indexed storage is not allowed; remove the element first or clone its field.",
                    span,
                ));
            };
            path.push(name.to_owned());
            context.projection = Some(Projection {
                root,
                path,
                replacing: false,
                span,
            });
            let checked = Self::access(
                context,
                Self::root_binding(base).expect("direct field root"),
                Mode::Consume,
                span,
                loans,
            );
            context.projection = None;
            checked?;
        }
        Ok(field_type)
    }

    fn temporary_field(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        name: &str,
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let borrowed = self.borrowed_place_hint(context, base);
        if borrowed && mode == Mode::Mutate {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An assignment result is readonly.",
                span,
            ));
        }
        let ty = self.expression(
            context,
            base,
            if borrowed { Mode::Read } else { Mode::Consume },
            loans,
            None,
        )?;
        if matches!(ty, Type::String | Type::Array(_)) && name == "length" {
            if mode == Mode::Mutate {
                return Err(Diagnostic::new(
                    "unsupported",
                    "String and array length are read-only; use push() to append an array element.",
                    span,
                ));
            }
            return Ok(Type::Int);
        }
        let field = match ty {
            Type::Record(record) => self.fields[record].get(name).copied(),
            _ => None,
        };
        if borrowed && mode == Mode::Consume && field.is_some_and(|ty| !ty.is_copy()) {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An owned field of an assignment result is borrowed; use clone().",
                span,
            ));
        }
        field.ok_or_else(|| {
            Diagnostic::new(
                "type",
                format!("Type {:?} has no field {name:?}.", self.type_name(ty)),
                span,
            )
        })
    }

    fn checked_tuple_index(&self, ty:Type, index:&Expr) -> Result<Option<(String,Type)>,Diagnostic> {
        let Type::Record(id)=ty else {return Ok(None)};
        if !self.program.records[id].is_tuple() {return Ok(None);}
        let Some(position)=Self::flat_literal_depth(index) else {return Err(Diagnostic::new("tuple-index","A tuple index must be a constant integer position.",index.span));};
        if position<0 || position as usize>=self.program.records[id].fields.len() {return Err(Diagnostic::new("bounds","Tuple index is outside its declared positions.",index.span));}
        let name=position.to_string();Ok(Some((name.clone(),self.fields[id][&name])))
    }

    /// A string position is a value, not storage: it cannot be assigned.
    #[inline(never)]
    fn reject_string_position_write(&mut self, context: &Context, target: &Expr) -> Result<(), Diagnostic> {
        if let ExprKind::Index(base, _) = &target.kind
            && self.expression_type_hint(context, base) == Some(Type::String)
        {
            return Err(Diagnostic::new(
                "unsupported",
                "A string cannot be changed in place. Build a new string, for example with slice() and +.",
                target.span,
            ));
        }
        Ok(())
    }

    /// `text[position]`: the character at a position as a new string. The
    /// text is only read; the result owns its character.
    #[inline(never)]
    fn string_index(&mut self,context:&mut Context,base:&mut Expr,index:&mut Expr,loans:&Loans)->Result<Type,Diagnostic> {
        let ty=self.expression(context,base,Mode::Read,loans,None)?;
        debug_assert_eq!(ty,Type::String);
        let mut active=loans.clone();
        for root in Self::loan_roots(base,Mode::Read){active.entry(root).or_default().push((Mode::Read,base.span));}
        let actual=self.expression(context,index,Mode::Copy,&active,Some(Type::Int))?;
        self.require(actual,Some(Type::Int),index.span)?;
        Ok(Type::String)
    }

    fn index_expression(&mut self,context:&mut Context,expr:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic> {
        let ExprKind::Index(base,index)=&mut expr.kind else {unreachable!()};
        if !Self::optional_hint_boundary(base) && self.expression_type_hint(context,base)==Some(Type::String) {
            return self.string_index(context,base,index,loans);
        }
        let result=if Self::named_storage(base) {self.array_index(context,base,index,mode,loans,expr.span)} else {self.temporary_array_index(context,base,index,mode,loans,expected)}?;
        if let Some((name,_))=self.checked_tuple_index(base.ty.expect("checked index receiver"),index)? {
            let span=base.span;let receiver=std::mem::replace(base,Box::new(Expr::new(ExprKind::Undefined,span)));
            expr.kind=ExprKind::Field(receiver,name);
        }
        Ok(result)
    }

    fn temporary_array_index(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        index: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        // Lift the selected value's context through each temporary index. An
        // inner index then lifts it again before checking its literal receiver.
        let array_context = expected
            .and_then(ArrayElement::from_value_type)
            .map(Type::Array);
        let borrowed = self.borrowed_place_hint(context, base);
        let ty = self.expression(
            context,
            base,
            if borrowed { Mode::Read } else { Mode::Consume },
            loans,
            array_context,
        )?;
        if let Some((_,field))=self.checked_tuple_index(ty,index)? {
            if borrowed && (mode==Mode::Mutate || mode==Mode::Consume && !field.is_copy()) {return Err(Diagnostic::new("borrowed-place","An owned tuple position of an assignment result is borrowed; use clone().",base.span));}
            return Ok(field);
        }
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                "Indexing requires an array, a tuple or a string.",
                base.span,
            ));
        };
        if borrowed && matches!(mode, Mode::Mutate | Mode::Consume) && !element.ty().is_copy() {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An owned element of an assignment result is borrowed; use clone().",
                base.span,
            ));
        }
        let mut active = loans.clone();
        if borrowed {
            for root in Self::loan_roots(base, Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((Mode::Read, base.span));
            }
        }
        let actual = self.expression(context, index, Mode::Copy, &active, Some(Type::Int))?;
        self.require(actual, Some(Type::Int), index.span)?;
        Ok(element.ty())
    }

    fn selection_loans(base: &Expr, loans: &Loans) -> Loans {
        let mut active = loans.clone();
        let mut selected = base;
        loop {
            match &selected.kind {
                ExprKind::Sequence(values) => { let Some(last)=values.last() else { break; }; selected=last; }
                ExprKind::Unary { op: UnaryOp::NonNull, value, .. } => selected=value,
                _ => break,
            }
        }
        if matches!(
            selected.kind,
            ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) {
            for root in Self::loan_roots(base, Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((Mode::Read, base.span));
            }
        }
        active
    }

    fn array_index(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        index: &mut Expr,
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        // Nested indexes recurse through this frame. Receiver checks and the
        // final access keep their temporaries in separate helpers.
        let element = match self.array_index_receiver(context, base, index, mode, loans, span)? {
            Ok(position) => return Ok(position),
            Err(element) => element,
        };
        let active = Self::selection_loans(base, loans);
        let actual = self.expression(context, index, Mode::Copy, &active, Some(Type::Int))?;
        self.array_index_access(context, base, (actual, index.span), mode, loans, span)?;
        Ok(element)
    }

    /// Checks the receiver. `Ok` is a tuple position, which is complete;
    /// `Err` is the element type of an array whose index is checked next.
    #[inline(never)]
    fn array_index_receiver(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        index: &mut Expr,
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<Result<Type, Type>, Diagnostic> {
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(base.kind, ExprKind::Sequence(_) | ExprKind::Unary { op: UnaryOp::NonNull, .. }) && Self::named_storage(base)) {
            return Err(Diagnostic::new(
                "unsupported",
                "Array indexing requires a named, indexed, or field array.",
                base.span,
            ));
        }
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        if let Some((_,field))=self.checked_tuple_index(ty,index)? {
            if mode==Mode::Consume && !field.is_copy() {return Err(Diagnostic::new("borrowed-element","Indexed tuple positions stay borrowed; bind the owner before transferring a position, or use clone().",span));}
            Self::storage_access(context,base,if mode==Mode::Mutate {Mode::Mutate}else{Mode::Read},span,loans)?;
            return Ok(Ok(field));
        }
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                "Indexing requires an array, a tuple or a string.",
                base.span,
            ));
        };
        if mode == Mode::Consume && !element.ty().is_copy() {
            return Err(Diagnostic::new(
                "borrowed-element",
                if matches!(element, ArrayElement::StoredCallable(_, _)) {
                    "Indexing borrows a callable environment; use remove() to transfer ownership out of the array."
                } else {
                    "Indexing borrows an owned element; use clone() to obtain an owned value."
                },
                span,
            ));
        }
        Ok(Err(element.ty()))
    }

    #[inline(never)]
    fn array_index_access(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        (actual, index_span): (Type, Span),
        mode: Mode,
        loans: &Loans,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.require(actual, Some(Type::Int), index_span)?;
        let access = if mode == Mode::Mutate {
            Mode::Mutate
        } else {
            Mode::Read
        };
        Self::storage_access(context, base, access, span, loans)
    }

    // Optional element views retain their selected receiver roots through aliases.
    fn view_sources(context: &Context, expr: &Expr) -> Vec<usize> {
        fn collect(context: &Context, expr: &Expr, out: &mut Vec<usize>) {
            match &expr.kind {
                ExprKind::Sequence(values)=>{if let Some(last)=values.last(){collect(context,last,out);}},
                ExprKind::Optional { body, .. } => collect(context, body, out),
                ExprKind::Selected { source, .. } => out.extend(source.roots.iter().copied()),
                ExprKind::CoalesceAssign { target, .. } => collect(context, target, out),
                ExprKind::Name(_) => {
                    if let Some(id) = expr.binding {
                        if context.bindings[id].view_roots.is_empty() {
                            out.push(id);
                        } else {
                            out.extend(context.bindings[id].view_roots.iter().copied());
                        }
                    }
                }
                ExprKind::Inject { value, .. }
                    if matches!(expr.ty, Some(Type::BorrowedUnion(_))) =>
                {
                    collect(context, value, out)
                }
                ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    value,
                    ..
                } => collect(context, value, out),
                ExprKind::Binary {
                    op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                    left,
                    right,
                    ..
                } => {
                    collect(context, left, out);
                    collect(context, right, out);
                }
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. }
                | ExprKind::Field(value, _)
                | ExprKind::Index(value, _) => collect(context, value, out),
                ExprKind::Call(callee, _)
                    if matches!(
                        expr.target,
                        Some(
                            CallTarget::MapMethod(MapMethod::Get) | CallTarget::ArrayAt | CallTarget::ArrayFind | CallTarget::ArrayFindLast
                        )
                    ) =>
                {
                    if let ExprKind::Field(base, _) = &callee.kind {
                        collect(context, base, out);
                    }
                }
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    collect(context, consequent, out);
                    collect(context, alternate, out);
                }
                _ => {}
            }
        }
        let mut roots = Vec::new();
        collect(context, expr, &mut roots);
        roots.sort_unstable();
        roots.dedup();
        roots
    }

    fn stable_at_receiver(expr: &Expr) -> bool {
        if let ExprKind::Sequence(values)=&expr.kind{return values.last().is_some_and(Self::stable_at_receiver);}
        match &expr.kind {
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => Self::stable_at_receiver(consequent) && Self::stable_at_receiver(alternate),
            _ => Self::named_storage(expr),
        }
    }

    fn array_at(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        index: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let hint = expected
            .and_then(|ty| self.unions.payload(ty.owning()))
            .and_then(|ty| Self::transform_array_type(ty, index.span).ok());
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let named = Self::stable_at_receiver(base);
        let actual = self.expression(
            context,
            base,
            if named { Mode::Read } else { Mode::Consume },
            loans,
            hint,
        );
        self.builtin_literals = previous;
        let Type::Array(element) = actual? else {
            return Err(Diagnostic::new(
                "type",
                "at() requires an array or string receiver.",
                base.span,
            ));
        };
        let mut active = Self::selection_loans(base, loans);
        if named && matches!(base.kind, ExprKind::Conditional { .. }) {
            for root in Self::loan_roots(base, Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((Mode::Read, base.span));
            }
        }
        let actual = self.expression(context, index, Mode::Copy, &active, Some(Type::Int))?;
        self.require(actual, Some(Type::Int), index.span)?;
        if named {
            Self::storage_access(context, base, Mode::Read, base.span, loans)?;
        }
        let result = self
            .unions
            .intern([element.ty(), Type::Undefined], base.span)?;
        Ok(if named && !result.is_copy() {
            Type::BorrowedUnion(result.union().expect("owned optional union"))
        } else {
            result
        })
    }

    fn array_optional_removal(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), 0, span)?;
        let result_hint = expected.and_then(|ty| match ty {
            Type::Union(_) => self.unions.payload(ty),
            Type::Undefined => None,
            other => Some(other),
        });
        let receiver_hint =
            result_hint.and_then(|ty| Self::transform_array_type(ty, base.span).ok());
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let checked = self.expression(context, base, Mode::Mutate, loans, receiver_hint);
        self.builtin_literals = previous;
        let Type::Array(element) = checked? else {
            return Err(Diagnostic::new(
                "type",
                "pop() and shift() require an array.",
                base.span,
            ));
        };
        Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        self.unions.intern([element.ty(), Type::Undefined], span)
    }

    fn array_unshift(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let mut root = &*base;
        loop {
            match &root.kind {
                ExprKind::Field(parent, _) | ExprKind::Index(parent, _) => root = parent,
                ExprKind::Project { value, .. } => root = value,
                _ => break,
            }
        }
        let named = Self::named_storage(root);
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let checked = self.expression(
            context,
            base,
            if args.is_empty() || named {
                Mode::Read
            } else {
                Mode::Consume
            },
            loans,
            None,
        );
        self.builtin_literals = previous;
        let Type::Array(element) = checked? else {
            return Err(Diagnostic::new(
                "type",
                "unshift() requires an array.",
                base.span,
            ));
        };
        let active = Self::selection_loans(base, loans);
        for arg in args.iter_mut() {
            let actual = if let Type::StoredCallable(family) = element.ty() {
                self.stored_callable_value(context, arg, family, &active)?
            } else {
                self.expression(context, arg, Mode::Consume, &active, Some(element.ty()))?
            };
            self.require(actual, Some(element.ty()), arg.span)?;
        }
        Self::storage_access(
            context,
            base,
            if args.is_empty() {
                Mode::Read
            } else {
                Mode::Mutate
            },
            span,
            loans,
        )?;
        Ok(Type::Int)
    }

    fn array_splice(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let mut root = &*base;
        loop {
            match &root.kind {
                ExprKind::Field(parent, _) | ExprKind::Index(parent, _) => root = parent,
                ExprKind::Project { value, .. } => root = value,
                _ => break,
            }
        }
        let named = Self::named_storage(root);
        let hint = if matches!(&base.kind,ExprKind::Array(values) if values.is_empty()) {
            expected
                .filter(|ty| matches!(ty, Type::Array(_)))
                .or_else(|| {
                    let infer = |argument: &Expr| {
                        Self::transform_array_type(
                            self.expression_type_hint(context, argument)?,
                            argument.span,
                        )
                        .ok()
                    };
                    args.iter()
                        .skip(2)
                        .filter_map(infer)
                        .find(|ty| matches!(ty, Type::Array(ArrayElement::Number(_))))
                        .or_else(|| args.iter().skip(2).find_map(infer))
                })
        } else {
            None
        };
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let checked = self.expression(
            context,
            base,
            if args.is_empty() || named {
                Mode::Read
            } else {
                Mode::Consume
            },
            loans,
            hint,
        );
        self.builtin_literals = previous;
        let Type::Array(element) = checked? else {
            return Err(Diagnostic::new(
                "type",
                "splice() requires an array.",
                base.span,
            ));
        };
        let active = Self::selection_loans(base, loans);
        for (index, arg) in args.iter_mut().enumerate() {
            let expected = if index < 2 { Type::Int } else { element.ty() };
            let mode = if index < 2 { Mode::Copy } else { Mode::Consume };
            let actual = if index >= 2
                && let Type::StoredCallable(family) = expected
            {
                self.stored_callable_value(context, arg, family, &active)?
            } else {
                self.expression(context, arg, mode, &active, Some(expected))?
            };
            self.require(actual, Some(expected), arg.span)?;
        }
        Self::storage_access(
            context,
            base,
            if args.is_empty() {
                Mode::Read
            } else {
                Mode::Mutate
            },
            span,
            loans,
        )?;
        Ok(Type::Array(element))
    }

    fn array_mutation(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        remove: bool,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), 1, span)?;
        let method = if remove { "remove" } else { "push" };
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(base.kind, ExprKind::Sequence(_) | ExprKind::Unary { op: UnaryOp::NonNull, .. }) && Self::named_storage(base)) {
            return Err(Diagnostic::new(
                "unsupported",
                format!("{method}() requires a named, indexed, or field array."),
                base.span,
            ));
        }
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                format!("{method}() requires an array."),
                base.span,
            ));
        };
        let active = Self::selection_loans(base, loans);
        let argument_type = if remove { Type::Int } else { element.ty() };
        let actual = if !remove && let Type::StoredCallable(family) = argument_type {
            self.stored_callable_value(context, &mut args[0], family, &active)?
        } else {
            self.expression(
                context,
                &mut args[0],
                if remove { Mode::Copy } else { Mode::Consume },
                &active,
                Some(argument_type),
            )?
        };
        self.require(actual, Some(argument_type), args[0].span)?;
        Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        Ok(if remove { element.ty() } else { Type::Void })
    }

    fn sequence_selection(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        method: &str,
    ) -> Result<Type, Diagnostic> {
        let slice = method == "slice";
        let range = slice || matches!(method, "substring" | "substr");
        if method == "substring" && args.is_empty() {
            return Err(Diagnostic::new(
                "arity",
                "substring() accepts one or two arguments.",
                span,
            ));
        }
        if args.len() > if range { 2 } else { 1 } {
            return Err(Diagnostic::new(
                "arity",
                format!(
                    "{method}() accepts at most {} arguments.",
                    if range { 2 } else { 1 }
                ),
                span,
            ));
        }
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        if slice {
            if !matches!(ty, Type::String | Type::Array(_)) {
                return Err(Diagnostic::new(
                    "type",
                    "slice() requires a string or array.",
                    base.span,
                ));
            }
        } else {
            self.require(ty, Some(Type::String), base.span)?;
        }
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        for arg in args {
            let actual = self.expression(context, arg, Mode::Copy, &active, Some(Type::Int))?;
            self.require(actual, Some(Type::Int), arg.span)?;
        }
        Ok(ty)
    }

    fn transform_array_type(ty: Type, span: Span) -> Result<Type, Diagnostic> {
        let element = ArrayElement::from_value_type(ty).ok_or_else(|| {
            if matches!(ty, Type::Array(_)) {
                Diagnostic::new(
                    "unsupported",
                    "Array type nesting exceeds the prototype limit of 128.",
                    span,
                )
            } else {
                Diagnostic::new(
                    "array-element",
                    "Array leaves must be int, number, boolean, string, or a named record.",
                    span,
                )
            }
        })?;
        Ok(Type::Array(element))
    }

    fn flat_literal_depth(expr: &Expr) -> Option<i64> {
        fn integer(expr: &Expr) -> Option<i128> {
            match &expr.kind {
                ExprKind::Integer(text) => text.parse().ok(),
                ExprKind::SpreadValue(selected) => selected.literal_integer.map(i128::from),
                ExprKind::Unary {
                    op: UnaryOp::Negate,
                    value,
                    ..
                } => integer(value)?.checked_neg(),
                ExprKind::Unary {
                    op: UnaryOp::Positive,
                    value,
                    ..
                } => integer(value),
                _ => None,
            }
        }
        i64::try_from(integer(expr)?).ok()
    }

    fn flat_literal_rank(expr: &Expr) -> u8 {
        if let ExprKind::Array(items) = &expr.kind {
            1 + items.iter().map(Self::flat_literal_rank).max().unwrap_or(0)
        } else {
            0
        }
    }

    // Flat owns depth and contextual-rank checking outside recursive call dispatch.
    fn array_flat(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, u8), Diagnostic> {
        if args.len() > 1 {
            return Err(Diagnostic::new(
                "arity",
                "flat() accepts zero or one int depth.",
                base.span,
            ));
        }
        let depth = args
            .first()
            .map(Self::flat_literal_depth)
            .unwrap_or(Some(1));
        let hint = if matches!(base.kind, ExprKind::Array(_)) {
            expected.and_then(|ty| {
                if let Type::Array(element) = ty {
                    let rank = if element.depth() == 0 {
                        Self::flat_literal_rank(base).saturating_sub(1)
                    } else {
                        element
                            .depth()
                            .checked_add(depth.unwrap_or(0).clamp(0, 127) as u8)?
                    };
                    (rank < 128).then_some(Type::Array(element.with_depth(rank)))
                } else {
                    None
                }
            })
        } else {
            None
        };
        let stable = Self::stable_at_receiver(base);
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let receiver = self.expression(
            context,
            base,
            if stable { Mode::Read } else { Mode::Consume },
            loans,
            hint,
        );
        self.builtin_literals = previous;
        let Type::Array(element) = receiver? else {
            return Err(Diagnostic::new(
                "type",
                "flat() requires an array.",
                base.span,
            ));
        };
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        if let Some(arg) = args.first_mut() {
            let actual = self.expression(context, arg, Mode::Copy, &active, Some(Type::Int))?;
            self.require(actual, Some(Type::Int), arg.span)?;
            if element.depth() > 0 && depth.is_none() {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Nested flat() requires a signed int literal depth because its return array rank is static.",
                    arg.span,
                ));
            }
        }
        if matches!(element,ArrayElement::Record(id,_) if self.program.records[id].is_tuple())
            && depth.is_none_or(|depth| depth > i64::from(element.depth())) {
            return Err(Diagnostic::new("unsupported", "Flattening tuple positions requires a homogeneous result array; this tuple traversal is not yet supported.", base.span));
        }
        let levels = depth.unwrap_or(0).max(0).min(i64::from(element.depth())) as u8;
        Ok((
            Type::Array(element.with_depth(element.depth() - levels)),
            levels,
        ))
    }

    // Callback checking and iterator contracts stay outside recursive expression frames.
    fn array_callback_hint_type(ty: Type, rest: bool) -> Type {
        if rest && let Type::Array(element) = ty { element.ty() } else { ty }
    }

    fn array_callback_rest_position(&self, expression: &Expr, hint: Option<Type>) -> Option<usize> {
        if let ExprKind::Closure { params, .. } = &expression.kind {
            return params.iter().position(|parameter| parameter.rest);
        }
        match hint {
            Some(Type::Closure(id)) => {
                let closure = &self.closures[id];
                closure.parameters.iter().position(|&id| closure.function.bindings[id].rest)
            }
            Some(Type::Callback(id)) => self.callables.callbacks[id].params.iter().position(|parameter| parameter.rest),
            Some(Type::Callable(id)) => self.callables.results[id].params.iter().position(|parameter| parameter.rest),
            _ => None,
        }
    }

    fn array_callback_parameter_types(parameters: &[ArgumentContract], supplied: &[Type]) -> Vec<Type> {
        parameters.iter().enumerate().map(|(index, parameter)| {
            if parameter.rest { parameter.ty } else { supplied.get(index).copied().unwrap_or(Type::Undefined) }
        }).collect()
    }

    /// Preserve the actual supplied tuple; padding belongs only to fixed slots.
    fn require_array_callback_parameters(
        &self,
        parameters: &[ArgumentContract],
        supplied: &[ArgumentContract],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let fixed = parameters.iter().position(|parameter| parameter.rest).unwrap_or(parameters.len());
        let minimum = crate::parameter_arity::minimum_arguments(&parameters[..fixed], |parameter| parameter.omission.is_omittable());
        if minimum > supplied.len() {
            return Err(Diagnostic::new("arity", "A required callback position exceeds the array method's supplied argument tuple.", span));
        }
        for (index, parameter) in parameters[..fixed].iter().enumerate() {
            if let Some(argument) = supplied.get(index) {
                self.require(argument.ty, Some(parameter.ty), span)?;
                if !self.effects_only && parameter.mode != argument.mode {
                    return Err(Diagnostic::new("callback-access", format!("Expected {} argument access, found {}. Array callback access contracts must match.", argument.mode.as_str(), parameter.mode.as_str()), span));
                }
            } else {
                self.require(Type::Undefined, Some(parameter.ty), span)?;
            }
        }
        if let Some(parameter) = parameters.get(fixed) {
            let Type::Array(element) = parameter.ty else {
                return Err(Diagnostic::new("rest-parameter", "A rest parameter requires a supported homogeneous array type.", span));
            };
            for argument in supplied.iter().skip(fixed) {
                self.require(argument.ty, Some(element.ty()), span)?;
                if !self.effects_only && !argument.ty.is_copy() && argument.mode != Mode::Consume {
                    return Err(Diagnostic::new("callback-access", "A rest pack cannot transfer an array callback's borrowed element or source array; use fixed borrowed parameters.", span));
                }
            }
        }
        Ok(())
    }

    /// Whether a contextual type needs an already materialized nominal name.
    fn names_materialized_type(&self, ty: Type) -> bool {
        match ty {
            Type::Record(id) => self.program.records[id].is_tuple() || self.program.records[id].application.is_some(),
            Type::Set(_) | Type::Map(_) => true,
            Type::Array(element) => {
self.names_materialized_type(element.with_depth(0).ty())
            }
            Type::Union(id) | Type::BorrowedUnion(id) => self.unions.members(id).into_iter().any(|member| self.names_materialized_type(member)),
            _ => false,
        }
    }

    fn contextual_arrow_parameters(&self, callback: &mut Expr, types: &[Type]) -> Result<(), Diagnostic> {
        let ExprKind::Closure { params, .. } = &mut callback.kind else { return Ok(()); };
        for (position, param) in params.iter_mut().enumerate() {
            if matches!(param.ty, ParameterAnnotation::Inferred(_)) && matches!(param.initializer, ParameterInitializer::Required) {
                let ty = *types.get(position).ok_or_else(|| Diagnostic::new("arity", "Too many callback parameters for this method.", param.name.span))?;
                // Preserve checked nominal descriptors rather than parsing fresh
                // tuple or generic applications after materialization has run.
                let mut annotation = if self.names_materialized_type(ty) {
                    crate::generic_inference::Types { callable_annotations: None,
                        callable_shapes: None,
                        inline_shapes: None,
                        records: &self.program.records,
                        names: &self.record_names,
                        unions: &self.unions,
                    }
                    .annotation(ty, param.name.span)?
                } else {
                    crate::type_syntax::parse(&self.type_name(ty)).map_err(|_| Diagnostic::new("parameter-inference", "This callback parameter needs an explicit callable annotation.", param.name.span))?
                };
                annotation.span = param.name.span;
                param.ty = ParameterAnnotation::Named(Box::new(annotation));
            }
        }
        Ok(())
    }

    fn array_transform(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        options: (bool, bool, Mode, bool, bool, bool, bool),
    ) -> Result<Type, Diagnostic> {
        let (filter, discard, receiver_mode, receiver_literals, flatten, callable_elements, from_array) =
            options;
        let set_for_each=discard && self.set_receiver_hint(context,base);
        let map_for_each=discard && self.map_receiver_hint(context,base);
        let collection_for_each=set_for_each||map_for_each;
        let collection_from=from_array && (self.set_receiver_hint(context,base)||self.map_receiver_hint(context,base));
        Self::arity(args.len(), 1, span)?;
        args[0].traversal_method = None;
        let callback_hint = self.callback_value_hint(context, &args[0])?;
        let value_hint = match &args[0].kind {
            ExprKind::Closure { params, .. } => params
                .first()
                .map(|p| match &p.ty {
                    ParameterAnnotation::Named(token) => {
                        self.resolve_value_annotation(token, false)
                    }
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation),
                    ParameterAnnotation::Inferred(token) => match &p.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value).ok_or_else(|| Diagnostic::new("parameter-inference", "This parameter needs its checked initializer or contextual type.", token.span)),
                        _ => Ok(Type::Void),
                    },
                })
                .transpose()?,
            _ => match callback_hint {
                Some(Type::Closure(id)) => self.closures[id]
                    .parameters
                    .first()
                    .map(|&p| self.closures[id].function.bindings[p].ty),
                Some(Type::Callback(id)) => {
                    self.callables.callbacks[id].params.first().map(|p| p.ty)
                }
                Some(Type::Callable(id)) => self.callables.results[id]
                    .params
                    .first()
                    .map(|parameter| parameter.ty),
                _ => None,
            },
        };
        let value_hint = value_hint.map(|ty| Self::array_callback_hint_type(ty, self.array_callback_rest_position(&args[0], callback_hint) == Some(0)));
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let absent_elements = matches!(self.expression_type_hint(context, base), Some(Type::Array(element)) if matches!(element.ty(), Type::Null | Type::Undefined)) || matches!(&base.kind, ExprKind::Array(values) if !values.is_empty() && values.iter().all(|value| matches!(value.kind, ExprKind::Null | ExprKind::Undefined)));
        let receiver_hint = value_hint
            .filter(|_| !collection_for_each && !collection_from)
            .filter(|ty| *ty != Type::Void)
            .filter(|_| !absent_elements)
            .filter(|ty| !matches!(ty, Type::Callback(_)))
            .map(|ty| Self::transform_array_type(ty, base.span))
            .transpose()?;
        let previous = self.builtin_literals;
        self.builtin_literals |= receiver_literals;
        let receiver = self.expression(context, base, receiver_mode, loans, if collection_for_each || collection_from{None}else{receiver_hint});
        self.builtin_literals = previous;
        let receiver = receiver?;
        let (value,position_type)=match receiver {
            Type::Array(element)=>(element.ty(),Type::Int),
            Type::Set(id) if from_array => (self.program.records[id].set_definition.as_ref().unwrap().key,Type::Int),
            Type::Map(id) if from_array => (Type::Record(self.map_iteration_entry(id,base.span)?),Type::Int),
            Type::Set(id) if set_for_each=>{let key=self.program.records[id].set_definition.as_ref().unwrap().key;(key,key)},
            Type::Map(id) if map_for_each=>{let map=self.program.records[id].map_definition.as_ref().unwrap();(map.value,map.key)},
            _=>return Err(Diagnostic::new("type","Array callback methods require an array.",base.span)),
        };
        let supplied_types=if from_array{vec![value,position_type]}else{vec![value,position_type,receiver]};
        self.contextual_arrow_parameters(&mut args[0], &supplied_types)?;
        let lends_callable = callable_elements
            && matches!(value, Type::StoredCallable(_))
            && matches!(value_hint, Some(Type::Callback(id)) if self.callables.callbacks[id].kind == CallbackKind::Reusable);
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active.entry(root).or_default().push((
                if lends_callable {
                    Mode::Mutate
                } else {
                    Mode::Read
                },
                base.span,
            ));
        }
        let first_capture = context.capture_loans.len();
        let checked_callback_hint = callback_hint.filter(|ty| {
            !(matches!(args[0].kind, ExprKind::Call(_, _))
                && matches!(ty, Type::Closure(id) if self.closures[*id].polymorphic))
        });
        let ty = if let Some(ty) = self.adapt_generic_traversal(context, &mut args[0], &supplied_types, &active)? {
            ty
        } else {
            self.expression(context, &mut args[0], Mode::Mutate, &active, checked_callback_hint)?
        };
        let (parameters, mut result, once) = match ty {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                let once = closure.invocation == Mode::Consume
                    || args[0].binding.is_some_and(|binding| {
                        context.bindings[binding].callback_invocation == Some(Mode::Consume)
                    });
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    once,
                )
            }
            Type::Callback(id) | Type::StoredCallable(id) => {
                let id = if matches!(ty, Type::StoredCallable(_)) {
                    self.callables.storage.declaration(id).expect("checked traversal storage")
                } else { id };
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.kind.is_once(),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "Array callback methods require a typed reusable callback.",
                    args[0].span,
                ));
            }
        };
        if once {
            return Err(Diagnostic::new(
                "callback-access",
                "Array callbacks must be reusable; a consuming callback cannot run for each element.",
                args[0].span,
            ));
        }

        let expected = [
            ArgumentContract { rest: false,
                ty: value,
                mode: if lends_callable {
                    Mode::Mutate
                } else if value.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
             omission: crate::ParameterOmission::Required, },
            ArgumentContract { rest: false,
                ty: position_type,
                mode: if position_type.is_copy(){Mode::Copy}else{Mode::Read},
                omission: crate::ParameterOmission::Required,
            },
            ArgumentContract { rest: false,
                ty: receiver,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
            },
        ];
        if lends_callable {
            if parameters.iter().skip(2).any(|parameter| !parameter.rest && !parameter.omission.is_omittable()) {
                return Err(Diagnostic::new(
                    "conflicting-access",
                    "A mutable callable element cannot be borrowed together with its source array.",
                    args[0].span,
                ));
            }
            self.require(value, Some(parameters[0].ty), args[0].span)?;
            if parameters[0].mode != Mode::Mutate {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable element invocation requires a mutate callback parameter.",
                    args[0].span,
                ));
            }
            self.require_array_callback_parameters(&parameters[1..], &expected[1..2], args[0].span)?;
            let Type::Closure(origin) = ty else {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Callable element traversal currently requires a concrete callback environment.",
                    args[0].span,
                ));
            };
            let (method, specialized_result) = self.materialize_closure_method_types(
                origin,
                Self::array_callback_parameter_types(&parameters, &expected[..2].iter().map(|parameter| parameter.ty).collect::<Vec<_>>()),
                args[0].span,
            )?;
            args[0].traversal_method = Some(TraversalMethod { id: method, access: Mode::Mutate });
            result = specialized_result;
            Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        } else {
            let supplied=if from_array{&expected[..2]}else{&expected[..]};
            self.require_array_callback_parameters(&parameters, supplied, args[0].span)?;
            if let Type::Closure(origin) = ty
                && self.closures[origin].polymorphic {
                let types = Self::array_callback_parameter_types(&parameters, &supplied.iter().map(|parameter| parameter.ty).collect::<Vec<_>>());
                let (id, specialized_result) = self.materialize_closure_method_types(origin, types, args[0].span)?;
                args[0].traversal_method = Some(TraversalMethod { id, access: Mode::Read });
                result = specialized_result;
            }
        }
        if flatten && matches!(result,Type::Record(id) if self.program.records[id].is_tuple()) {
            return Err(Diagnostic::new("unsupported", "flatMap() cannot yet flatten a tuple result into a homogeneous array.", args[0].span));
        }
        let result = if discard {
            // Any otherwise valid callback result (including callable owners) is
            // evaluated and dropped at the end of this invocation.
            Type::Void
        } else if filter {
            self.require(result, Some(Type::Boolean), args[0].span)?;
            receiver
        } else if flatten && matches!(result, Type::Array(_)) {
            result
        } else {
            Self::transform_array_type(result, args[0].span)?
        };
        // Temporary callback captures end after traversal. Named closures retain their existing loans.
        context.capture_loans.truncate(first_capture);
        Ok(result)
    }

    // Homogeneous concat spreads only arguments at the full receiver rank.
    // Lower-rank arrays cannot be treated as scalar array elements in JavaScript.
    fn collection_concat(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let hint = if let ExprKind::Array(values) = &base.kind {
            expected
                .filter(|ty| matches!(ty, Type::Array(_)))
                .or_else(|| {
                    if !values.is_empty() {
                        return None;
                    }
                    let mut inferred = None;
                    for arg in args.iter() {
                        let Some(ty) = self.expression_type_hint(context, arg) else {
                            continue;
                        };
                        let hint = match ty {
                            Type::Array(_) => Some(ty),
                            Type::Int
                            | Type::Number
                            | Type::Boolean
                            | Type::String
                            | Type::Record(_) => Self::transform_array_type(ty, arg.span).ok(),
                            _ => None,
                        };
                        // Like array literals, a Number hint supplies context for
                        // integer literals, without converting typed int values.
                        if matches!(hint, Some(Type::Array(ArrayElement::Number(_)))) {
                            return hint;
                        }
                        if inferred.is_none() {
                            inferred = hint;
                        }
                    }
                    inferred
                })
        } else {
            None
        };
        let receiver = self.expression(context, base, Mode::Read, loans, hint)?;
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        // Dispatch on the checked receiver, not an optional pure type hint.
        // Factory-returned callbacks can produce strings without a precheck hint.
        if receiver == Type::String {
            for arg in args {
                let actual =
                    self.expression(context, arg, Mode::Read, &active, Some(Type::String))?;
                self.require(actual, Some(Type::String), arg.span)?;
                for root in Self::loan_roots(arg, Mode::Read) {
                    active.entry(root).or_default().push((Mode::Read, arg.span));
                }
            }
            return Ok(Type::String);
        }
        let Type::Array(element) = receiver else {
            return Err(Diagnostic::new(
                "type",
                "concat() requires a string or array.",
                base.span,
            ));
        };
        let value = element.ty();
        for arg in args {
            let array = matches!(
                self.expression_type_hint(context, arg),
                Some(Type::Array(_))
            ) || matches!(arg.kind, ExprKind::Array(_));
            let target = if array { receiver } else { value };
            let mode = if target.is_copy() {
                Mode::Copy
            } else {
                Mode::Read
            };
            let actual = self.expression(context, arg, mode, &active, Some(target))?;
            if let Type::Array(_) = actual {
                if actual != receiver {
                    return Err(Diagnostic::new(
                        "type",
                        "concat() spreads array arguments one level; use an array at the receiver's full rank (wrap a nested element in an outer array).",
                        arg.span,
                    ));
                }
            } else {
                if matches!(value, Type::Array(_)) {
                    return Err(Diagnostic::new(
                        "type",
                        "Nested concat() requires array arguments at the receiver's full rank.",
                        arg.span,
                    ));
                }
                self.require(actual, Some(value), arg.span)?;
            }
            if !actual.is_copy() {
                for root in Self::loan_roots(arg, Mode::Read) {
                    active.entry(root).or_default().push((Mode::Read, arg.span));
                }
            }
        }
        Ok(receiver)
    }

    // Reduction owns its accumulator independently from the borrowed source.
    // Keep callback and argument temporaries outside recursive expression frames.
    fn array_reduce(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if !(1..=2).contains(&args.len()) {
            return Err(Diagnostic::new(
                "arity",
                "reduce() and reduceRight() expect a callback and optional initial value.",
                span,
            ));
        }
        args[0].traversal_method = None;
        let callback_hint = self.callback_value_hint(context, &args[0])?;
        let (parameter_hints, result_hint) = match &args[0].kind {
            ExprKind::Closure {
                params,
                result: annotation,
                ..
            } => (
                params.iter().map(|parameter| match &parameter.ty {
                    ParameterAnnotation::Named(token) => self.callable_resolver().resolve_value_annotation(token, false).map(Some),
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation).map(Some),
                    ParameterAnnotation::Inferred(_) => Ok(match &parameter.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value),
                        _ => None,
                    }),
                }).collect::<Result<Vec<_>, Diagnostic>>()?,
                if matches!(annotation, ResultAnnotation::Inferred(_)) { None } else { Some(self.resolve_result(annotation)?) },
            ),
            _ => match callback_hint {
                Some(Type::Closure(id)) if !self.deferred_generic_values.contains_key(&id) => (
                    self.closures[id]
                        .parameters
                        .iter()
                        .map(|&parameter| Some(self.closures[id].function.bindings[parameter].ty))
                        .collect(),
                    Some(self.closures[id].function.result),
                ),
                Some(Type::Callback(id)) => (
                    self.callables.callbacks[id]
                        .params
                        .iter()
                        .map(|parameter| Some(parameter.ty))
                        .collect(),
                    Some(self.callables.callbacks[id].result),
                ),
                Some(Type::Callable(id)) => (
                    self.callables.results[id]
                        .params
                        .iter()
                        .map(|parameter| Some(parameter.ty))
                        .collect(),
                    Some(self.callables.results[id].result),
                ),
                _ => (Vec::new(), None),
            },
        };
        let rest_position = self.array_callback_rest_position(&args[0], callback_hint);
        let parameter_hints = parameter_hints.into_iter().enumerate().map(|(index, ty)| ty.map(|ty| Self::array_callback_hint_type(ty, rest_position == Some(index)))).collect::<Vec<_>>();
        let accumulator_hint = parameter_hints.first().copied().flatten().or(result_hint);
        let value_hint = parameter_hints
            .get(1)
            .copied()
            .flatten()
            .or_else(|| (args.len() == 1 || rest_position == Some(0)).then_some(accumulator_hint).flatten());
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let absent_elements = matches!(self.expression_type_hint(context, base), Some(Type::Array(element)) if matches!(element.ty(), Type::Null | Type::Undefined)) || matches!(&base.kind, ExprKind::Array(values) if !values.is_empty() && values.iter().all(|value| matches!(value.kind, ExprKind::Null | ExprKind::Undefined)));
        let receiver_hint = value_hint
            .filter(|_| !absent_elements)
            .filter(|value| !matches!(value, Type::Callback(_)))
            .map(|value| Self::transform_array_type(value, base.span))
            .transpose()?;
        let receiver = self.expression(context, base, Mode::Read, loans, receiver_hint)?;
        let Type::Array(element) = receiver else {
            return Err(Diagnostic::new(
                "type",
                "Array reduction requires an array.",
                base.span,
            ));
        };
        if let ExprKind::Closure { params, .. } = &args[0].kind
            && params.iter().any(|param| matches!(param.ty, ParameterAnnotation::Inferred(_))) {
            let accumulator = accumulator_hint.or_else(|| args.get(1).and_then(|initial| self.expression_type_hint(context, initial))).unwrap_or(element.ty());
            self.contextual_arrow_parameters(&mut args[0], &[accumulator, element.ty(), Type::Int, receiver])?;
        }
        let lends_callable = matches!(element, ArrayElement::StoredCallable(_, _))
            && value_hint.is_some_and(|value| matches!(value, Type::Callback(id) if self.callables.callbacks[id].kind == CallbackKind::Reusable));
        let receiver_mode = if lends_callable {
            Mode::Mutate
        } else {
            Mode::Read
        };
        let mut active = loans.clone();
        for root in Self::loan_roots(base, receiver_mode) {
            active
                .entry(root)
                .or_default()
                .push((receiver_mode, base.span));
        }
        let first_capture = context.capture_loans.len();
        let checked_hint = callback_hint.filter(|ty| !matches!(ty, Type::Closure(id) if self.closures[*id].polymorphic && matches!(args[0].kind, ExprKind::Call(_, _))));
        let accumulator = accumulator_hint
            .or_else(|| args.get(1).and_then(|initial| self.expression_type_hint(context, initial)))
            .unwrap_or(element.ty());
        let callback = if let Some(callback) = self.adapt_generic_traversal(
            context, &mut args[0], &[accumulator, element.ty(), Type::Int, receiver], &active,
        )? {
            callback
        } else {
            self.expression(context, &mut args[0], Mode::Mutate, &active, checked_hint)?
        };
        let (parameters, mut result, once) = match callback {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&parameter| {
                            ArgumentContract::from(&closure.function.bindings[parameter])
                        })
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    closure.invocation == Mode::Consume
                        || args[0].binding.is_some_and(|binding| {
                            context.bindings[binding].callback_invocation == Some(Mode::Consume)
                        }),
                )
            }
            Type::Callback(id) | Type::StoredCallable(id) => {
                let id = if matches!(callback, Type::StoredCallable(_)) {
                    self.callables.storage.declaration(id).expect("checked traversal storage")
                } else { id };
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.kind.is_once(),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "Array reduction requires a typed reusable callback.",
                    args[0].span,
                ));
            }
        };
        if once {
            return Err(Diagnostic::new(
                "callback-access",
                "Reduction callbacks must be reusable; their captured environment cannot be consumed.",
                args[0].span,
            ));
        }

        if !matches!(
            result,
            Type::Int
                | Type::Number
                | Type::Boolean
                | Type::Null
                | Type::Undefined
                | Type::String
                | Type::Record(_)
                | Type::Array(_)
        ) {
            return Err(Diagnostic::new(
                "type",
                "Reduction accumulators must be int, number, boolean, string, named records, or arrays.",
                args[0].span,
            ));
        }
        if let Some(initial) = args.get_mut(1) {
            let actual = self.expression(context, initial, Mode::Consume, &active, Some(result))?;
            self.require(actual, Some(result), initial.span)?;
        } else {
            self.require(result, Some(element.ty()), args[0].span)?;
        }
        if let Some(accumulator) = parameters.first().filter(|parameter| !parameter.rest) {
            self.require(result, Some(accumulator.ty), args[0].span)?;
            if !self.effects_only && result.is_copy() && accumulator.mode != Mode::Copy {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Copy reduction accumulators require copy access.",
                    args[0].span,
                ));
            }
        }
        let value = element.ty();
        let expected = [
            ArgumentContract { rest: false,
                ty: value,
                mode: if lends_callable {
                    Mode::Mutate
                } else if value.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
             omission: crate::ParameterOmission::Required, },
            ArgumentContract { rest: false,
                ty: Type::Int,
                mode: Mode::Copy,
                omission: crate::ParameterOmission::Required,
            },
            ArgumentContract { rest: false,
                ty: receiver,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
            },
        ];
        if lends_callable {
            if args.len() != 2 {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable array reductions require an independent initial accumulator.",
                    span,
                ));
            }
            if parameters.iter().skip(3).any(|parameter| !parameter.rest && !parameter.omission.is_omittable()) {
                return Err(Diagnostic::new(
                    "conflicting-access",
                    "A mutable callable element cannot be borrowed together with its source array.",
                    args[0].span,
                ));
            }
            self.require(value, Some(parameters[1].ty), args[0].span)?;
            if parameters[1].mode != Mode::Mutate {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable element invocation requires a mutate callback parameter.",
                    args[0].span,
                ));
            }
            self.require_array_callback_parameters(&parameters[2..], &expected[1..2], args[0].span)?;
            let Type::Closure(origin) = callback else {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Callable element reduction requires a concrete arrow environment.",
                    args[0].span,
                ));
            };
            let mut types = vec![result];
            types.extend(
                Self::array_callback_parameter_types(&parameters[1..], &expected[..2].iter().map(|parameter| parameter.ty).collect::<Vec<_>>()),
            );
            let (method, specialized_result) =
                self.materialize_closure_method_types(origin, types, args[0].span)?;
            self.require(specialized_result, Some(result), args[0].span)?;
            result = specialized_result;
            args[0].traversal_method = Some(TraversalMethod { id: method, access: Mode::Mutate });
            Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        } else if parameters.first().is_some_and(|parameter| parameter.rest) {
            let mut supplied = vec![ArgumentContract { ty: result, mode: if result.is_copy() { Mode::Copy } else { Mode::Consume }, omission: crate::ParameterOmission::Required, rest: false }];
            supplied.extend_from_slice(&expected);
            self.require_array_callback_parameters(&parameters, &supplied, args[0].span)?;
        } else if parameters.len() > 1 {
            self.require_array_callback_parameters(&parameters[1..], &expected, args[0].span)?;
        }
        if !lends_callable && let Type::Closure(origin) = callback
            && self.closures[origin].polymorphic {
            let mut supplied = vec![result];
            supplied.extend(expected.iter().map(|parameter| parameter.ty));
            let types = Self::array_callback_parameter_types(&parameters, &supplied);
            let (id, specialized_result) = self.materialize_closure_method_types(origin, types, args[0].span)?;
            self.require(specialized_result, Some(result), args[0].span)?;
            result = specialized_result;
            args[0].traversal_method = Some(TraversalMethod { id, access: Mode::Read });
        }
        context.capture_loans.truncate(first_capture);
        Ok(result)
    }

    // Predicate callbacks share filter's boolean, readonly receiver contract.
    // Keep their argument checking outside the recursive expression frame.
    fn array_find(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let stable = Self::stable_at_receiver(base);
        let receiver = self.array_transform(
            context,
            base,
            args,
            loans,
            span,
            (
                true,
                false,
                if stable { Mode::Read } else { Mode::Consume },
                true,
                false,
                false,
                false,
            ),
        );
        let Type::Array(element) = receiver? else {
            unreachable!("predicate array")
        };
        let result = self.unions.intern([element.ty(), Type::Undefined], span)?;
        Ok(if stable && !result.is_copy() {
            Type::BorrowedUnion(result.union().expect("owned optional"))
        } else {
            result
        })
    }

    fn array_predicate(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        index_result: bool,
    ) -> Result<Type, Diagnostic> {
        self.array_transform(
            context,
            base,
            args,
            loans,
            span,
            (true, false, Mode::Read, false, false, true, false),
        )?;
        Ok(if index_result {
            Type::Int
        } else {
            Type::Boolean
        })
    }

    // Width-preserving mutation binds selected receiver access before operands,
    // but checks final exclusive access only after operands have completed.
    fn array_range_mutation(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        fill: bool,
    ) -> Result<Type, Diagnostic> {
        let method = if fill { "fill" } else { "copyWithin" };
        let minimum = if fill { 1 } else { 2 };
        if args.len() < minimum || args.len() > 3 {
            return Err(Diagnostic::new(
                "arity",
                format!("{method}() requires {minimum} to 3 arguments."),
                span,
            ));
        }
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(base.kind, ExprKind::Sequence(_) | ExprKind::Unary { op: UnaryOp::NonNull, .. }) && Self::named_storage(base)) {
            return Err(Diagnostic::new(
                "unsupported",
                format!("{method}() requires a named, indexed, or field array."),
                base.span,
            ));
        }
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                format!("{method}() requires an array."),
                base.span,
            ));
        };
        let active = Self::selection_loans(base, loans);
        for (index, argument) in args.iter_mut().enumerate() {
            let value = fill && index == 0;
            let expected = if value { element.ty() } else { Type::Int };
            let actual = self.expression(
                context,
                argument,
                if value { Mode::Consume } else { Mode::Copy },
                &active,
                Some(expected),
            )?;
            self.require(actual, Some(expected), argument.span)?;
        }
        Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        Ok(Type::Void)
    }

    fn array_to_spliced(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let hint = if let ExprKind::Array(values) = &base.kind {
            expected
                .filter(|ty| matches!(ty, Type::Array(_)))
                .or_else(|| {
                    if !values.is_empty() {
                        return None;
                    }
                    let infer = |argument: &Expr| {
                        Self::transform_array_type(
                            self.expression_type_hint(context, argument)?,
                            argument.span,
                        )
                        .ok()
                    };
                    args.iter()
                        .skip(2)
                        .filter_map(infer)
                        .find(|ty| matches!(ty, Type::Array(ArrayElement::Number(_))))
                        .or_else(|| args.iter().skip(2).find_map(infer))
                })
        } else {
            None
        };
        let ty = self.expression(context, base, Mode::Read, loans, hint)?;
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                "toSpliced() requires an array.",
                base.span,
            ));
        };
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        for (index, argument) in args.iter_mut().enumerate() {
            let expected = if index < 2 { Type::Int } else { element.ty() };
            let mode = if index < 2 { Mode::Copy } else { Mode::Consume };
            let actual = self.expression(context, argument, mode, &active, Some(expected))?;
            self.require(actual, Some(expected), argument.span)?;
        }
        Ok(ty)
    }

    #[allow(clippy::too_many_arguments)]
    fn array_copying(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        replacement: bool,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), if replacement { 2 } else { 0 }, span)?;
        let hint = if matches!(base.kind, ExprKind::Array(_)) {
            expected.filter(|ty| matches!(ty, Type::Array(_)))
        } else {
            None
        };
        let ty = self.expression(context, base, Mode::Read, loans, hint)?;
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                "Array copying methods require an array.",
                base.span,
            ));
        };
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        for (index, argument) in args.iter_mut().enumerate() {
            let expected = if index == 0 { Type::Int } else { element.ty() };
            let mode = if index == 0 {
                Mode::Copy
            } else {
                Mode::Consume
            };
            let actual = self.expression(context, argument, mode, &active, Some(expected))?;
            self.require(actual, Some(expected), argument.span)?;
        }
        Ok(ty)
    }

    // Sorting owns its callback contracts outside the recursive call dispatcher.
    fn array_sort(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        options: (bool, Option<Type>),
    ) -> Result<Type, Diagnostic> {
        let (copying, expected) = options;
        if args.len() > 1 {
            return Err(Diagnostic::new(
                "arity",
                "Sorting accepts zero or one comparator.",
                span,
            ));
        }
        let callback_hint = args
            .first()
            .map(|arg| self.callback_value_hint(context, arg))
            .transpose()?
            .flatten();
        let value_hint = args
            .first()
            .and_then(|arg| match &arg.kind {
                ExprKind::Closure { params, .. } => params.first().map(|p| match &p.ty {
                    ParameterAnnotation::Named(token) => {
                        self.resolve_value_annotation(token, false)
                    }
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation),
                    ParameterAnnotation::Inferred(token) => match &p.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value).ok_or_else(|| Diagnostic::new("parameter-inference", "This parameter needs its checked initializer or contextual type.", token.span)),
                        _ => Ok(Type::Void),
                    },
                }),
                _ => None,
            })
            .transpose()?
            .or_else(|| match callback_hint {
                Some(Type::Closure(id)) => self.closures[id]
                    .parameters
                    .first()
                    .map(|&p| self.closures[id].function.bindings[p].ty),
                Some(Type::Callback(id)) => {
                    self.callables.callbacks[id].params.first().map(|p| p.ty)
                }
                _ => None,
            });
        let value_hint = value_hint.map(|ty| Self::array_callback_hint_type(ty, args.first().and_then(|arg| self.array_callback_rest_position(arg, callback_hint)) == Some(0)));
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let hint = value_hint
            .filter(|ty| *ty != Type::Void)
            .map(|value| Self::transform_array_type(value, base.span))
            .transpose()?
            .or_else(|| expected.filter(|ty| matches!(ty, Type::Array(_))));
        let named = Self::stable_at_receiver(base);
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let receiver = self.expression(
            context,
            base,
            if copying || named {
                Mode::Read
            } else {
                Mode::Consume
            },
            loans,
            hint,
        );
        self.builtin_literals = previous;
        let Type::Array(element) = receiver? else {
            return Err(Diagnostic::new(
                "type",
                "Sorting requires an array.",
                base.span,
            ));
        };
        if let Some(callback) = args.first_mut() {
            self.contextual_arrow_parameters(callback, &[element.ty(), element.ty()])?;
            let mut active = loans.clone();
            for root in Self::loan_roots(base, Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((if copying { Mode::Read } else { Mode::Mutate }, base.span));
            }
            let first_capture = context.capture_loans.len();
            callback.traversal_method = None;
            let callback_type = if let Some(ty) = self.adapt_generic_traversal(
                context, callback, &[element.ty(), element.ty()], &active,
            )? {
                ty
            } else {
                self.expression(context, callback, Mode::Mutate, &active, callback_hint)?
            };
            let (parameters, result, once) = match callback_type {
                Type::Closure(id) => {
                    let closure = &self.closures[id];
                    (
                        closure
                            .parameters
                            .iter()
                            .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                            .collect::<Vec<_>>(),
                        closure.function.result,
                        closure.invocation == Mode::Consume
                            || callback.binding.is_some_and(|binding| {
                                context.bindings[binding].callback_invocation == Some(Mode::Consume)
                            }),
                    )
                }
                Type::Callback(id) | Type::StoredCallable(id) => {
                    let id = if matches!(callback_type, Type::StoredCallable(_)) {
                        self.callables.storage.declaration(id).expect("checked traversal storage")
                    } else { id };
                    let contract = &self.callables.callbacks[id];
                    (
                        contract.params.clone(),
                        contract.result,
                        contract.kind.is_once(),
                    )
                }
                _ => {
                    return Err(Diagnostic::new(
                        "type",
                        "Sorting requires a typed reusable comparator.",
                        callback.span,
                    ));
                }
            };
            if once {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Sorting comparators must be reusable.",
                    callback.span,
                ));
            }

            if !matches!(result, Type::Int | Type::Number) {
                return Err(Diagnostic::new(
                    "type",
                    "Sorting comparators return int or number.",
                    callback.span,
                ));
            }
            let value = element.ty();
            let parameter = ArgumentContract { rest: false,
                ty: value,
                mode: if value.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
             omission: crate::ParameterOmission::Required, };
            self.require_array_callback_parameters(&parameters, &[parameter, parameter], callback.span)?;
            if let Type::Closure(origin) = callback_type
                && self.closures[origin].polymorphic {
                let types = Self::array_callback_parameter_types(&parameters, &[value, value]);
                let (id, specialized_result) = self.materialize_closure_method_types(origin, types, callback.span)?;
                self.require(specialized_result, Some(result), callback.span)?;
                callback.traversal_method = Some(TraversalMethod { id, access: Mode::Read });
            }
            context.capture_loans.truncate(first_capture);
        } else if element.depth() > 0
            || matches!(
                element,
                ArrayElement::Record(_, _) | ArrayElement::StoredCallable(_, _)
            )
        {
            return Err(Diagnostic::new(
                "type",
                "Nested arrays, records and callable elements require an explicit sorting comparator.",
                span,
            ));
        }
        if !copying && named {
            Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        }
        Ok(if copying {
            Type::Array(element)
        } else {
            Type::Void
        })
    }

    fn array_reverse(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), 0, span)?;
        if !matches!(
            base.kind,
            ExprKind::Name(_)
                | ExprKind::Selected { .. }
                | ExprKind::Index(_, _)
                | ExprKind::Field(_, _)
                | ExprKind::Project { .. }
        ) && !matches!(&base.kind,ExprKind::SpreadValue(selected) if selected.stable && selected.selection==SpreadSelection::Receiver)
            && !(matches!(base.kind, ExprKind::Sequence(_) | ExprKind::Unary { op: UnaryOp::NonNull, .. }) && Self::named_storage(base)) {
            return Err(Diagnostic::new(
                "unsupported",
                "reverse() requires a named, indexed, or field array.",
                base.span,
            ));
        }
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        if !matches!(ty, Type::Array(_)) {
            return Err(Diagnostic::new(
                "type",
                "reverse() requires an array.",
                base.span,
            ));
        }
        Self::storage_access(context, base, Mode::Mutate, span, loans)?;
        Ok(Type::Void)
    }

    // Constructing string operations keep all selected inputs borrowed until
    // the final operand is checked, but return independent owning outputs.
    fn string_construct(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        method: &str,
    ) -> Result<Type, Diagnostic> {
        match method {
            "padStart" | "padEnd" if (1..=2).contains(&args.len()) => {}
            "padStart" | "padEnd" => {
                return Err(Diagnostic::new(
                    "arity",
                    format!("Expected one or two arguments, found {}.", args.len()),
                    span,
                ));
            }
            method if string_html_target(method).is_some() => {
                let Some(CallTarget::StringHtml(wrapper)) = string_html_target(method) else {
                    unreachable!("checked HTML wrapper")
                };
                Self::arity(args.len(), usize::from(wrapper.markup().1.is_some()), span)?;
            }
            "repeat" => Self::arity(args.len(), 1, span)?,
            "replace" | "replaceAll" => Self::arity(args.len(), 2, span)?,
            "split" if args.len() <= 2 => {}
            "split" => {
                return Err(Diagnostic::new(
                    "arity",
                    format!("Expected zero to two arguments, found {}.", args.len()),
                    span,
                ));
            }
            _ => unreachable!("checked string construction method"),
        }
        let receiver = self.expression(context, base, Mode::Read, loans, None)?;
        self.require(receiver, Some(Type::String), base.span)?;
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        for (index, argument) in args.iter_mut().enumerate() {
            let integer = method == "repeat"
                || (matches!(method, "padStart" | "padEnd") && index == 0)
                || (method == "split" && index == 1);
            let expected = if integer { Type::Int } else { Type::String };
            let actual = self.expression(
                context,
                argument,
                if integer { Mode::Copy } else { Mode::Read },
                &active,
                Some(expected),
            )?;
            self.require(actual, Some(expected), argument.span)?;
            if !integer {
                for root in Self::loan_roots(argument, Mode::Read) {
                    active
                        .entry(root)
                        .or_default()
                        .push((Mode::Read, argument.span));
                }
            }
        }
        Ok(if method == "split" {
            Type::Array(ArrayElement::String(0))
        } else {
            Type::String
        })
    }

    #[inline(never)]
    fn string_normalize(&mut self, context: &mut Context, base: &mut Expr, args: &mut [Expr], loans: &Loans, span: Span) -> Result<Type, Diagnostic> {
        if args.len() > 1 { return Err(Diagnostic::new("arity", "normalize expects zero or one string form argument.", span)); }
        let receiver=self.expression(context,base,Mode::Read,loans,Some(Type::String))?;
        self.require(receiver,Some(Type::String),base.span)?;
        if let Some(form)=args.first_mut() {
            let mut active=loans.clone();
            for root in Self::loan_roots(base, Mode::Read) { active.entry(root).or_default().push((Mode::Read,base.span)); }
            let actual=self.expression(context,form,Mode::Read,&active,None)?;
            let valid=matches!(actual,Type::String|Type::Undefined) || match actual {
                Type::Union(id)|Type::BorrowedUnion(id)=>self.unions.members(id).iter().all(|member|matches!(member,Type::String|Type::Undefined)),
                _=>false,
            };
            if !valid { return Err(Diagnostic::new("type", "normalize form must be a string or undefined.", form.span)); }
        }
        Ok(Type::String)
    }

    fn array_join(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if args.len() > 1 {
            return Err(Diagnostic::new(
                "arity",
                "join() expects zero or one argument.",
                span,
            ));
        }
        // Empty literal arrays use the same harmless int[] context as String([]).
        let previous = self.builtin_literals;
        self.builtin_literals = true;
        let receiver = self.expression(context, base, Mode::Read, loans, None);
        self.builtin_literals = previous;
        if !matches!(receiver?, Type::Array(_)) {
            return Err(Diagnostic::new(
                "type",
                "join() requires an array.",
                base.span,
            ));
        }
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        if let Some(separator) = args.first_mut() {
            let actual =
                self.expression(context, separator, Mode::Read, &active, Some(Type::String))?;
            self.require(actual, Some(Type::String), separator.span)?;
        }
        Ok(Type::String)
    }

    fn collection_search(
        &mut self,
        context: &mut Context,
        base: &mut Expr,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
        method: &str,
    ) -> Result<Type, Diagnostic> {
        if !(1..=2).contains(&args.len()) {
            return Err(Diagnostic::new(
                "arity",
                format!("Expected 1 or 2 arguments, found {}.", args.len()),
                span,
            ));
        }
        let index_of = method == "indexOf" || method == "lastIndexOf";
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        let element_type = if ty == Type::String {
            Type::String
        } else {
            if method == "startsWith" || method == "endsWith" {
                return Err(Diagnostic::new(
                    "type",
                    format!("{method}() requires a string."),
                    base.span,
                ));
            }
            let Type::Array(element) = ty else {
                return Err(Diagnostic::new(
                    "type",
                    format!("{method}() requires a string or array."),
                    base.span,
                ));
            };
            if let ArrayElement::Union(union, 0) = element {
                return self.union_collection_search(context, base, args, loans, union, method);
            }
            if !matches!(
                element,
                ArrayElement::Int(0)
                    | ArrayElement::Number(0)
                    | ArrayElement::Boolean(0)
                    | ArrayElement::Null(0)
                    | ArrayElement::Undefined(0)
                    | ArrayElement::String(0)
                    | ArrayElement::Enum(_, _, 0)
            ) {
                return Err(Diagnostic::new(
                    "unsupported",
                    format!(
                        "{method}() accepts int[], number[], boolean[], or string[]; owned record and array identity is not defined."
                    ),
                    base.span,
                ));
            }
            element.ty()
        };
        // Keep the selected receiver stable while all later operands run.
        // A temporary owns independent storage and has no source-root loan.
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active
                .entry(root)
                .or_default()
                .push((Mode::Read, base.span));
        }
        let actual = self.expression(
            context,
            &mut args[0],
            Mode::Read,
            &active,
            Some(element_type),
        )?;
        self.require(actual, Some(element_type), args[0].span)?;
        if !actual.is_copy() {
            for root in Self::loan_roots(&args[0], Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((Mode::Read, args[0].span));
            }
        }
        if args.len() == 2 {
            let actual =
                self.expression(context, &mut args[1], Mode::Copy, &active, Some(Type::Int))?;
            self.require(actual, Some(Type::Int), args[1].span)?;
        }
        Ok(if index_of { Type::Int } else { Type::Boolean })
    }

    /// Search in an array whose elements are a union of primitive values. The
    /// searched value has one primitive type; elements with another active
    /// alternative never match, except that int and number compare by value.
    #[inline(never)]
    fn union_collection_search(
        &mut self,
        context: &mut Context,
        base: &Expr,
        args: &mut [Expr],
        loans: &Loans,
        union: UnionId,
        method: &str,
    ) -> Result<Type, Diagnostic> {
        let members = self.unions.members(union);
        let primitive = |ty: &Type| {
            matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined)
        };
        if !members.iter().all(primitive) {
            return Err(Diagnostic::new(
                "unsupported",
                format!("{method}() needs every element alternative to be int, number, boolean, string, null or undefined; record and array identity is not defined."),
                base.span,
            ));
        }
        let mut active = loans.clone();
        for root in Self::loan_roots(base, Mode::Read) {
            active.entry(root).or_default().push((Mode::Read, base.span));
        }
        // An integer literal searches number elements when no int alternative exists.
        let numeric_hint = (members.contains(&Type::Number) && !members.contains(&Type::Int)).then_some(Type::Number);
        let actual = self.expression(context, &mut args[0], Mode::Read, &active, numeric_hint)?;
        let numeric = |ty: Type| matches!(ty, Type::Int | Type::Number);
        if !primitive(&actual) || !(members.contains(&actual) || numeric(actual) && members.iter().any(|member| numeric(*member))) {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "{method}() searches {} for one of its alternatives, found {}. Narrow a union value with typeof first.",
                    self.type_name(Type::Array(ArrayElement::Union(union, 0))),
                    self.type_name(actual)
                ),
                args[0].span,
            ));
        }
        if !actual.is_copy() {
            for root in Self::loan_roots(&args[0], Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, args[0].span));
            }
        }
        if args.len() == 2 {
            let from = self.expression(context, &mut args[1], Mode::Copy, &active, Some(Type::Int))?;
            self.require(from, Some(Type::Int), args[1].span)?;
        }
        Ok(if method == "includes" { Type::Boolean } else { Type::Int })
    }

    // Keep builtin argument and diagnostic temporaries outside the recursive
    // general-expression frame retained by unary and binary operands.
    fn string_well_formed(
        &mut self,
        context: &mut Context,
        receiver: &mut Expr,
        arguments: usize,
        loans: &Loans,
        span: Span,
    ) -> Result<(), Diagnostic> {
        Self::arity(arguments, 0, span)?;
        let actual = self.expression(context, receiver, Mode::Read, loans, None)?;
        self.require(actual, Some(Type::String), receiver.span)
    }

    fn number_parse_float(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), 1, span)?;
        let actual =
            self.expression(context, &mut args[0], Mode::Read, loans, Some(Type::String))?;
        self.require(actual, Some(Type::String), args[0].span)?;
        Ok(Type::Number)
    }

    /// Both operands are borrowed and evaluated once, left to right. Values of
    /// different kinds are never the same; arrays and records are the same
    /// only when both operands are the same storage.
    fn object_is(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if args.len() != 2 {
            return Err(Diagnostic::new(
                "arity",
                format!("Object.is expects two values, found {} arguments.", args.len()),
                span,
            ));
        }
        let mut active = loans.clone();
        for argument in args.iter_mut() {
            let previous = self.builtin_literals;
            self.builtin_literals = true;
            let ty = self.expression(context, argument, Mode::Read, &active, None);
            self.builtin_literals = previous;
            let ty = ty?;
            let unsupported = |member: Type| matches!(member, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::Callable(_) | Type::Void);
            if unsupported(ty) || ty.union().is_some_and(|id| self.unions.members(id).into_iter().any(unsupported)) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Object.is compares data values, including union alternatives; functions and void are not supported.",
                    argument.span,
                ));
            }
            for root in Self::loan_roots(argument, Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, argument.span));
            }
        }
        Ok(Type::Boolean)
    }

    /// One borrowed string and an optional int radix; the result is a number
    /// because a missing integer prefix yields NaN.
    fn number_parse_int(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                "arity",
                format!("Number.parseInt expects a string and an optional radix, found {} arguments.", args.len()),
                span,
            ));
        }
        let actual =
            self.expression(context, &mut args[0], Mode::Read, loans, Some(Type::String))?;
        self.require(actual, Some(Type::String), args[0].span)?;
        if args.len() == 2 {
            let mut active = loans.clone();
            for root in Self::loan_roots(&args[0], Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, args[0].span));
            }
            let radix = self.expression(context, &mut args[1], Mode::Copy, &active, Some(Type::Int))?;
            self.require(radix, Some(Type::Int), args[1].span)?;
        }
        Ok(Type::Number)
    }

    fn number_predicate(
        &mut self,
        context: &mut Context,
        name: &str,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        Self::arity(args.len(), 1, span)?;
        let actual =
            self.expression(context, &mut args[0], Mode::Copy, loans, Some(Type::Number))?;
        if actual != Type::Number {
            return Err(Diagnostic::new(
                "type",
                format!(
                    "Number.{name}() requires a number argument; use Number(...) for explicit conversion."
                ),
                args[0].span,
            ));
        }
        Ok(Type::Boolean)
    }

    fn math_arguments(
        &mut self,
        context: &mut Context,
        name: &str,
        args: &mut [Expr],
        loans: &Loans,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        match name {
            "min" | "max" | "hypot" => {}
            "imul" | "atan2" | "pow" => Self::arity(args.len(), 2, span)?,
            "random" => Self::arity(args.len(), 0, span)?,
            _ => Self::arity(args.len(), 1, span)?,
        }
        for argument in args {
            let actual =
                self.expression(context, argument, Mode::Copy, loans, Some(Type::Number))?;
            if actual != Type::Number {
                return Err(Diagnostic::new(
                    "type",
                    format!(
                        "Math.{name}() requires a number argument; use Number(...) for explicit conversion."
                    ),
                    argument.span,
                ));
            }
        }
        Ok(Type::Number)
    }

    #[allow(clippy::too_many_arguments)]
    fn closure_method_call(
        &mut self,
        context: &mut Context,
        binding: usize,
        origin: usize,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        loans: &Loans,
        span: Span,
        expected: Option<Type>,
    ) -> Result<(usize, Type), Diagnostic> {
        let (origin, once) = match self.closures[origin].forward {
            Some(ClosureForward::Once(inner)) => (inner, true),
            _ => (origin, false),
        };
        if self.deferred_generic_values.contains_key(&origin) {
            return self.deferred_generic_call(context, Some(binding), origin, callee, args, loans, span, once, expected, false);
        }
        let closure = &self.closures[origin];
        let invocation = if once {
            Mode::Consume
        } else {
            context.bindings[binding]
                .callback_invocation
                .unwrap_or(closure.invocation)
        };
        let params = closure
            .parameters
            .iter()
            .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
            .collect::<Vec<_>>();
        Self::prepare_arguments(args, &params, span)?;
        Self::access(context, binding, invocation, callee.span, loans)?;
        callee.binding = Some(binding);
        callee.ty = Some(context.bindings[binding].ty);
        let first_capture = context.capture_loans.len();
        let mut active = loans.clone();
        active
            .entry(binding)
            .or_default()
            .push((invocation, callee.span));
        for (arg, contract) in args.iter_mut().zip(params) {
            if let Type::Callback(id) = contract.ty
                && self.callables.callbacks[id].kind.is_owned()
                && let ExprKind::Closure { id, .. } = &arg.kind
            {
                self.escaping.insert(*id);
            }
            let actual =
                self.parameter_argument(context, arg, contract, &active)?;
            self.require(actual, Some(contract.ty), arg.span)?;
            if !actual.is_copy() {
                for root in Self::loan_roots(arg, contract.mode) {
                    active
                        .entry(root)
                        .or_default()
                        .push((contract.mode, arg.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture);
        let (method, result) = self.materialize_closure_method(origin, args, span)?;
        let result = self.closure_method_result(context, origin, method, result, args, span);
        Ok((method, result))
    }

    fn closure_method_result(
        &mut self,
        context: &Context,
        origin: usize,
        method: usize,
        mut result: Type,
        args: &[Expr],
        span: Span,
    ) -> Type {
        if let Type::Closure(id) = result
            && let Some(ClosureForward::Abstract(contract)) = self.closures[id].forward
        {
            let method = &self.closures[origin].methods[method];
            let position = method
                .parameters
                .iter()
                .position(|&p| method.function.bindings[p].ty == Type::Callback(contract))
                .expect("returned method parameter path");
            result = args[position].ty.expect("checked callback argument");
            if let Type::Callback(id) = result {
                result = Type::Closure(self.abstract_return(id, context, span));
            } else if self.callables.callbacks[contract].kind.is_once()
                && let Type::Closure(id) = result
                && self.closures[id].invocation != Mode::Consume
            {
                result = Type::Closure(self.once_return(id, context, span));
            }
        }
        result
    }

    fn resolved_call(
        &mut self,
        context: &Context,
        index: usize,
        args: &[Expr],
        span: Span,
    ) -> Result<(usize, Type), Diagnostic> {
        let index = self.specialize(index, args, span)?;
        let result = if let Some(target) = self.checked[index].as_ref() {
            target.result
        } else if let Some(&result) = self.known_results.get(&index) {
            result
        } else if self.callable_result_type(self.signatures[index].result) {
            if self.discovering_results {
                self.result_waits.insert(index, span);
                self.callable_result_shape(self.signatures[index].result)
            } else if context.template {
                self.callable_result_shape(self.signatures[index].result)
            } else {
                return Err(Self::pending_result(span));
            }
        } else {
            self.signatures[index].result
        };
        if context.template && self.checked.get(index).and_then(Option::as_ref).is_none_or(|function|function.template)
            && self.program.functions[index].parameter_initializers.iter().any(|initializer|matches!(initializer,ParameterInitializer::Default{..}))
            && matches!(result,Type::Closure(id) if matches!(self.closures[id].forward,Some(ClosureForward::Abstract(_)))) {
            return Ok((index,self.callable_result_shape(self.signatures[index].result)));
        }
        // An abstract direct return denotes one of this template's
        // parameters. Substitute its argument origin in the caller,
        // including through chains of forwarding templates.
        let result = if let Type::Closure(id) = result
            && let Some(ClosureForward::Abstract(contract)) = self.closures[id].forward
        {
            let result = self.signatures[index]
                .params
                .iter()
                .zip(args)
                .find(|(ty, _)| **ty == Type::Callback(contract))
                .expect("returned callback parameter origin")
                .1
                .ty
                .expect("checked callback argument");
            if let Type::Callback(id) = result {
                Type::Closure(self.abstract_return(id, context, span))
            } else if self.callables.callbacks[contract].kind.is_once()
                && let Type::Closure(id) = result
                && self.closures[id].invocation != Mode::Consume
            {
                Type::Closure(self.once_return(id, context, span))
            } else {
                result
            }
        } else {
            result
        };
        Ok((index, result))
    }

    // The environment origin remains stable; specialization/result belong to this call.
    // Keep its loan live before evaluating every argument and truncate only temporary captures.
    #[allow(clippy::too_many_arguments)]
    fn forward_call(
        &mut self,
        context: &mut Context,
        binding: usize,
        function: usize,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        loans: &Loans,
        span: Span,
    ) -> Result<(usize, Type), Diagnostic> {
        let Type::Closure(origin) = context.bindings[binding].ty else {
            unreachable!()
        };
        let closure = &self.closures[origin];
        let invocation = context.bindings[binding]
            .callback_invocation
            .unwrap_or(closure.invocation);
        let params = closure
            .parameters
            .iter()
            .map(|&id| ArgumentContract::from(&closure.function.bindings[id]))
            .collect::<Vec<_>>();
        Self::prepare_arguments(args, &params, span)?;
        Self::access(context, binding, invocation, callee.span, loans)?;
        callee.binding = Some(binding);
        callee.ty = Some(context.bindings[binding].ty);
        let first_capture = context.capture_loans.len();
        let mut active = loans.clone();
        active
            .entry(binding)
            .or_default()
            .push((invocation, callee.span));
        for (arg, contract) in args.iter_mut().zip(params) {
            if let Type::Callback(id) = contract.ty
                && self.callables.callbacks[id].kind.is_owned()
                && let ExprKind::Closure { id, .. } = &arg.kind
            {
                self.escaping.insert(*id);
            }
            let actual =
                self.parameter_argument(context, arg, contract, &active)?;
            self.require(actual, Some(contract.ty), arg.span)?;
            if !actual.is_copy() {
                for root in Self::loan_roots(arg, contract.mode) {
                    active
                        .entry(root)
                        .or_default()
                        .push((contract.mode, arg.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture);
        self.resolved_call(context, function, args, span)
    }

    #[inline(never)]
    fn conditional_call(
        &mut self,
        context: &mut Context,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        loans: &Loans,
        span: Span,
        expected: Option<Type>,
    ) -> Result<(CallTarget, Type), Diagnostic> {
        let first_capture=context.capture_loans.len();
        let checked_callee=matches!(callee.kind,ExprKind::Closure{..}|ExprKind::Call(_, _));
        if matches!(callee.kind,ExprKind::Closure{..}) {
            self.contextual_expression_callee(context,callee,args)?;
        }
        let hint = if checked_callee {Some(self.expression(context,callee,Mode::Consume,loans,None)?)} else {self.callback_value_hint(context, callee)?};
        let hint=hint.map(|hint|if let Type::Callable(id)=hint {self.result_shape(id)}else{hint});
        let hint = hint.ok_or_else(|| {
            Diagnostic::new(
                "unsupported",
                "Conditional invocation requires a resolved callable origin.",
                callee.span,
            )
        })?;
        let hint = if let Type::Closure(origin) = hint
            && let Some(ClosureForward::Native(id)) = self.closures[origin].forward
        {
            self.native_function_shape(id, span, args.len())?
        } else {
            hint
        };
        let result=self.checked_callable_expression_call(context,callee,args,loans,span,hint,checked_callee,expected)?;
        if checked_callee && !self.expression_result_retains_captures(result.1){context.capture_loans.truncate(first_capture);}
        Ok(result)
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn checked_callable_expression_call(
        &mut self, context:&mut Context, callee:&mut Expr, args:&mut Vec<Expr>,
        loans:&Loans, span:Span, hint:Type, checked_callee:bool, expected: Option<Type>,
    )->Result<(CallTarget,Type),Diagnostic>{
        if let Type::Closure(origin) = hint {
            let (origin, once) = match self.closures[origin].forward {
                Some(ClosureForward::Once(inner)) => (inner, true),
                _ => (origin, false),
            };
            if self.deferred_generic_values.contains_key(&origin) {
                let (method, result) = self.deferred_generic_call(context, None, origin, callee, args, loans, span, once, expected, checked_callee)?;
                return Ok((CallTarget::ClosureMethod(method), result));
            }
        }
        let (params, result, invocation, target) = match hint {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                let invocation = Self::conditional_callback_metadata(context, callee)?
                    .and_then(|x| x.0)
                    .unwrap_or(closure.invocation);
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    invocation,
                    CallTarget::ConditionalClosure(id),
                )
            }
            Type::Callback(id) => {
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.invocation(&self.callables.shapes),
                    CallTarget::ConditionalCallback(id),
                )
            }
            Type::StoredCallable(family) => {
                let declaration = self
                    .callables
                    .storage
                    .declaration(family)
                    .expect("stored signature");
                let contract = &self.callables.callbacks[declaration];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.invocation(&self.callables.shapes),
                    CallTarget::StoredCallableField(family),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "This expression must produce a callable value with a compatible signature.",
                    callee.span,
                ));
            }
        };
        Self::prepare_arguments(args, &params, span)?;
        let actual = if checked_callee {callee.ty.expect("checked IIFE closure")} else {self.expression(context, callee, invocation, loans, Some(hint))?};
        self.require(actual, Some(hint), callee.span)?;
        Self::conditional_callback_metadata(context, callee)?;
        let mut active = loans.clone();
        for root in Self::loan_roots(callee, invocation) {
            active
                .entry(root)
                .or_default()
                .push((invocation, callee.span));
        }
        let first_capture = context.capture_loans.len();
        for (arg, contract) in args.iter_mut().zip(params) {
            let actual =
                self.parameter_argument(context, arg, contract, &active)?;
            self.require(actual, Some(contract.ty), arg.span)?;
            if !actual.is_copy() {
                for root in Self::loan_roots(arg, contract.mode) {
                    active
                        .entry(root)
                        .or_default()
                        .push((contract.mode, arg.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture);
        if let Type::Closure(origin) = hint {
            let (origin, _) = match self.closures[origin].forward {
                Some(ClosureForward::Once(inner)) => (inner, true),
                _ => (origin, false),
            };
            if let Some(ClosureForward::Function(function)) = self.closures[origin].forward {
                let (function, result) = self.resolved_call(context, function, args, span)?;
                return Ok((CallTarget::ForwardFunction(function), result));
            }
            if self.closures[origin].polymorphic {
                let (method, result) = self.materialize_closure_method(origin, args, span)?;
                let result =
                    self.closure_method_result(context, origin, method, result, args, span);
                return Ok((CallTarget::ClosureMethod(method), result));
            }
        }
        let result = if let Type::Callable(contract) = result {
            if context.template || self.discovering_results {
                self.result_shape(contract)
            } else {
                return Err(Self::pending_result(span));
            }
        } else {
            result
        };
        Ok((target, result))
    }

    fn console_inspectable(&self, ty: Type) -> bool {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::String | Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined => {},
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.unions.members(id)),
                Type::Array(element) => pending.push(element.ty()),
                Type::Set(id) => pending.push(self.program.records[id].set_definition.as_ref().unwrap().key),
                Type::Map(id) => { let map = self.program.records[id].map_definition.as_ref().unwrap(); pending.extend([map.key,map.value]); },
                Type::Enum(_, _) => {},
                Type::Record(id) => pending.extend(self.program.records[id].fields.iter().map(|field| field.ty.expect("checked field"))),
                _ => return false,
            }
        }
        true
    }

    fn value_builtin_arguments(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        loans: &Loans,
        string_conversion: bool,
    ) -> Result<(), Diagnostic> {
        let first_capture = context.capture_loans.len();
        let mut active = loans.clone();
        for (index, argument) in args.iter_mut().enumerate() {
            let previous = self.builtin_literals;
            self.builtin_literals = true;
            let observing = self.observing_values;
            let refined_absence = string_conversion
                && index == 0
                && self
                    .flow_view(context, argument)
                    .is_some_and(|(_, _, narrowed)| {
                        matches!(narrowed, Type::Null | Type::Undefined)
                    });
            self.observing_values = !refined_absence;
            let result = if refined_absence {
                self.expression(context, argument, Mode::Read, &active, None)
            } else {
                self.observed_expression(context, argument, &active)
            };
            self.observing_values = observing;
            self.builtin_literals = previous;
            let actual = result?;
            if string_conversion
                && index == 0
                && matches!(
                    self.unions.payload(actual).unwrap_or(actual),
                    Type::Closure(_)
                        | Type::Callback(_)
                        | Type::Callable(_)
                        | Type::StoredCallable(_)
                )
            {
                return Err(Diagnostic::new(
                    "unsupported",
                    "String() function-source conversion is not yet implemented.",
                    argument.span,
                ));
            }
            if !actual.is_copy() {
                for root in Self::loan_roots(argument, Mode::Read) {
                    active
                        .entry(root)
                        .or_default()
                        .push((Mode::Read, argument.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture);
        Ok(())
    }

    // Keep builtin method temporaries off the call-graph recursion path.
    /// `text.at(index)` selects one character counted from either end. It is
    /// checked as the array `at` of the string's characters.
    #[inline(never)]
    fn string_at(&mut self, context: &Context, expr: &mut Expr) {
        let ExprKind::Call(callee, _) = &mut expr.kind else { return };
        let ExprKind::Field(base, name) = &mut callee.kind else { return };
        if name != "at"
            || !(matches!(base.kind, ExprKind::String(_))
                || matches!(self.expression_type_hint(context, base), Some(Type::String)))
        {
            return;
        }
        let span = base.span;
        let text = std::mem::replace(base.as_mut(), Expr::new(ExprKind::Undefined, span));
        let split = Expr::new(ExprKind::Field(Box::new(text), "split".to_owned()), span);
        **base = Expr::new(
            ExprKind::Call(Box::new(split), vec![Expr::new(ExprKind::String(String::new()), span)]),
            span,
        );
    }

    /// `toLocaleLowerCase()` and `toLocaleUpperCase()` without a locale are the
    /// locale-independent case mappings. Locale-specific rules are not provided.
    #[inline(never)]
    fn string_locale_case(&mut self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()) };
        let ExprKind::Field(base, name) = &mut callee.kind else { return Ok(()) };
        let plain = match name.as_str() {
            "toLocaleLowerCase" => "toLowerCase",
            "toLocaleUpperCase" => "toUpperCase",
            _ => return Ok(()),
        };
        if !(matches!(base.kind, ExprKind::String(_))
            || matches!(self.expression_type_hint(context, base), Some(Type::String)))
        {
            return Ok(());
        }
        if let Some(locale) = args.first() {
            return Err(Diagnostic::new(
                "unsupported",
                format!("{name}() takes no locale here; it applies the locale-independent mapping. Remove the argument."),
                locale.span,
            ));
        }
        *name = plain.to_owned();
        Ok(())
    }

    /// `text.codePointAt(position)` reads through a slot array that holds the
    /// code when the position names a character, then takes element zero. The
    /// array `at` contract supplies the checked `int | undefined` result.
    #[inline(never)]
    fn string_code_point_at(&mut self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()) };
        let ExprKind::Field(base, name) = &mut callee.kind else { return Ok(()) };
        if name != "codePointAt"
            || !(matches!(base.kind, ExprKind::String(_))
                || matches!(self.expression_type_hint(context, base), Some(Type::String)))
        {
            return Ok(());
        }
        if args.len() > 1 {
            return Err(Diagnostic::new(
                "arity",
                format!("codePointAt expects an optional int position, found {} arguments.", args.len()),
                span,
            ));
        }
        *name = STRING_CODE_POINT_SLOT.to_owned();
        let slot_span = callee.span;
        let slot_callee = std::mem::replace(callee.as_mut(), Expr::new(ExprKind::Undefined, slot_span));
        let slot = Expr::new(ExprKind::Call(Box::new(slot_callee), std::mem::take(args)), slot_span);
        **callee = Expr::new(ExprKind::Field(Box::new(slot), "at".to_owned()), slot_span);
        args.push(Expr::new(ExprKind::Integer("0".to_owned()), Span::new(span.end, span.end)));
        Ok(())
    }

    /// `String.fromCodePoint(a, ...b)` takes its codes as one int array, so
    /// ordinary and spread arguments share the array checks. A single spread
    /// array source is read in place; other iterables use ordered expansion.
    #[inline(never)]
    fn string_from_codes(&self, context: &Context, expr: &mut Expr) {
        let call = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { return };
        // The code array spans the argument list, so it never shares a span
        // with the call or with one argument.
        let span = Span::new(callee.span.end, call.end);
        let ExprKind::Field(base, name) = &callee.kind else { return };
        if !matches!(name.as_str(), "fromCodePoint" | "fromCharCode")
            || !matches!(&base.kind, ExprKind::Name(name) if name == "String" && !context.names.contains_key(name))
        {
            return;
        }
        let mut codes = std::mem::take(args);
        let single_spread = codes.len() == 1
            && matches!(&codes[0].kind, ExprKind::Unary { op: UnaryOp::Spread,value,.. } if matches!(self.expression_type_hint(context,value),Some(Type::Array(_))) || matches!(value.kind,ExprKind::Array(_)));
        if single_spread {
            let ExprKind::Unary { value, .. } = codes.pop().expect("one spread argument").kind else {
                unreachable!("matched spread argument")
            };
            args.push(*value);
        } else {
            args.push(Expr::new(ExprKind::Array(codes), span));
        }
    }

    /// Lower String.raw onto a borrowed segment array and owned string substitutions.
    /// Keep this temporary state out of the ordinary recursive call frame.
    #[inline(never)]
    fn string_raw(&self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let call_span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()) };
        let ExprKind::Field(base, name) = &callee.kind else { return Ok(()) };
        if name != "raw"
            || !matches!(&base.kind, ExprKind::Name(name) if name == "String" && !context.names.contains_key(name))
        { return Ok(()); }
        if args.is_empty() {
            return Err(Diagnostic::new("arity", "String.raw expects a template record with a raw string[] field.", call_span));
        }
        if matches!(args[0].kind, ExprKind::Unary { op: UnaryOp::Spread, .. }) {
            return Err(Diagnostic::new("unsupported", "String.raw template cannot be a spread argument.", args[0].span));
        }
        let mut original = std::mem::take(args).into_iter();
        let template = original.next().expect("checked first argument");
        let raw_span = Span::new(callee.span.end, template.span.end);
        args.push(Expr::new(ExprKind::Field(Box::new(template), "raw".to_owned()), raw_span));
        let substitutions = original.map(|mut value| {
            if let ExprKind::Unary { op: UnaryOp::Spread, value: source, .. } = &mut value.kind {
                let span = source.span;
                let original = std::mem::replace(source.as_mut(), Expr::new(ExprKind::Undefined, span));
                let method=if matches!(original.kind,ExprKind::SetConstructor(_, _)) || matches!(self.expression_type_hint(context,&original),Some(Type::Set(_))) {"values"}else if matches!(original.kind,ExprKind::MapConstructor(_, _)) || matches!(self.expression_type_hint(context,&original),Some(Type::Map(_))){"entries"}else{"slice"};
                **source = Expr::new(ExprKind::Call(Box::new(Expr::new(
                    ExprKind::Field(Box::new(original), method.to_owned()), span)), vec![]), span);
                value
            } else {
                let span = value.span;
                Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("String".to_owned()), span)), vec![value]), span)
            }
        }).collect();
        args.push(Expr::new(ExprKind::Array(substitutions), Span::new(callee.span.end, call_span.end)));
        Ok(())
    }

    #[inline(never)]
    fn string_raw_arguments(&mut self, context: &mut Context, args: &mut [Expr], loans: &Loans) -> Result<(), Diagnostic> {
        let strings = Type::Array(ArrayElement::String(0));
        let mut active = loans.clone();
        for argument in args.iter_mut() {
            let previous = self.builtin_literals;
            self.builtin_literals = true;
            let checked = self.expression(context, argument, Mode::Read, &active, Some(strings));
            self.builtin_literals = previous;
            let actual = checked?;
            self.require(actual, Some(strings), argument.span)?;
            for root in Self::loan_roots(argument, Mode::Read) {
                active.entry(root).or_default().push((Mode::Read, argument.span));
            }
        }
        Ok(())
    }

    /// `array.values()` yields the elements in order; it is the checked
    /// independent copy that `slice()` produces.
    #[inline(never)]
    fn array_values(&mut self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(()) };
        let ExprKind::Field(base, name) = &mut callee.kind else { return Ok(()) };
        if name != "values"
            || !(matches!(base.kind, ExprKind::Array(_))
                || matches!(self.expression_type_hint(context, base), Some(Type::Array(_))))
        {
            return Ok(());
        }
        Self::arity(args.len(), 0, span)?;
        *name = "slice".to_owned();
        Ok(())
    }

    fn array_from_callee(context: &Context, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Field(base, name)
                if name == "from" && matches!(&base.kind, ExprKind::Name(name) if name == "Array" && !context.names.contains_key(name))))
    }

    /// `Array.from(source)` copies an array or splits a string into its
    /// characters; a second argument maps each value and its index. The call
    /// is rewritten onto the checked copy, split and map operations.
    #[inline(never)]
    fn array_from(&mut self, context: &Context, expr: &mut Expr) -> Result<(), Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!("Array.from call") };
        if args.is_empty() || args.len() > 2 {
            return Err(Diagnostic::new(
                "arity",
                format!("Array.from expects a source and an optional mapping function, found {} arguments.", args.len()),
                span,
            ));
        }
        if args.iter().any(|arg| matches!(arg.kind, ExprKind::Unary { op: UnaryOp::Spread, .. })) {
            return Err(Diagnostic::new("unsupported", "Array.from does not accept spread arguments; pass the source array directly.", span));
        }
        let mut arguments = std::mem::take(args);
        let mapper = (arguments.len() == 2).then(|| arguments.pop().expect("mapping function"));
        let mut source = arguments.pop().expect("Array.from source");
        // A nested Array.from source is rewritten first so its type is known.
        if Self::array_from_callee(context, &source) {
            self.array_from(context, &mut source)?;
        }
        let source_span = source.span;
        let method = |receiver: Expr, name: &str, arguments: Vec<Expr>| {
            let callee = Expr::new(ExprKind::Field(Box::new(receiver), name.to_owned()), source_span);
            Expr::new(ExprKind::Call(Box::new(callee), arguments), span)
        };
        let array = matches!(source.kind, ExprKind::Array(_))
            || matches!(self.expression_type_hint(context, &source), Some(Type::Array(_)))
            || matches!(&source.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(base,name) if name=="map#arrayFrom" || matches!(name.as_str(),"values"|"entries") && (self.set_receiver_hint(context,base)||self.map_receiver_hint(context,base))));
        let text = matches!(self.expression_type_hint(context, &source), Some(Type::String));
        let rewritten = if array {
            match mapper {
                Some(mapper) => method(source, "map#arrayFrom", vec![mapper]),
                None => method(source, "slice", Vec::new()),
            }
        } else if text {
            let characters = method(source, "split", vec![Expr::new(ExprKind::String(String::new()), source_span)]);
            match mapper {
                Some(mapper) => method(characters, "map#arrayFrom", vec![mapper]),
                None => characters,
            }
        } else if self.set_receiver_hint(context,&source)||self.map_receiver_hint(context,&source) {
            let copy_method=if self.set_receiver_hint(context,&source){"values"}else{"entries"};
            match mapper {Some(mapper)=>method(source,"map#arrayFrom",vec![mapper]),None=>method(source,copy_method,Vec::new())}
        } else {
            return Err(Diagnostic::new("type", "Array.from requires an array, string, Set or Map source.", source_span));
        };
        *expr = rewritten;
        Ok(())
    }

    #[inline(never)]
    fn expression_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        // A direct nongeneric declaration needs no builtin or conditional state.
        let declared = matches!(&expr.kind, ExprKind::Call(callee, args)
            if matches!(&callee.kind, ExprKind::Name(name)
                if self.function_names.contains_key(name)
                    && self.generic_functions.template(name).is_none()
                    && !context.names.contains_key(name)
                    && !context.pending_names.contains(name)
                    && !matches!(name.as_str(), "String" | "Boolean" | "Number" | "clone"))
                && !args.iter().any(|argument| matches!(&argument.kind,
                    ExprKind::Unary { op: UnaryOp::Spread, .. })));
        if declared {
            return self.declared_named_call(context, expr, loans);
        }
        self.expression_call_other(context, expr, loans, expected)
    }

    fn expression_call_other(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        if let ExprKind::Call(callee, _) = &mut expr.kind {
            self.global_number_name(context, callee, true);
        }
        if let ExprKind::Call(callee, _) = &mut expr.kind {
            let mut receiver = callee.as_mut();
            while matches!(receiver.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _)) {
                receiver = match &mut receiver.kind {
                    ExprKind::Field(base, _) | ExprKind::Index(base, _) => base.as_mut(),
                    _ => unreachable!("member receiver"),
                };
            }
            // Resolve generic factories before inspecting a returned callable
            // field. Preparation establishes types without evaluating arguments.
            self.prepare_generic_call(context, receiver, None)?;
        }
        if Self::map_group_by_callee(context,expr) {return self.map_group_by(context,expr,loans,expected);}
        if Self::array_from_callee(context, expr) {
            self.array_from(context, expr)?;
        }
        if matches!(&expr.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(_,name) if name=="map#arrayFrom")) {
            return self.array_from_mapping(context,expr,loans,expected);
        }
        if self.map_method_receiver(context, expr) { return self.map_method_call(context, expr, loans, expected); }
        if self.set_method_receiver(context, expr) { return self.set_method_call(context, expr, loans, expected); }
        self.array_values(context, expr)?;
        self.string_at(context, expr);
        self.string_raw(context, expr)?;
        self.string_from_codes(context, expr);
        self.string_code_point_at(context, expr)?;
        self.string_locale_case(context, expr)?;
        if matches!(&expr.kind, ExprKind::Call(_, args) if args.iter().any(|arg| matches!(arg.kind, ExprKind::Unary {op:UnaryOp::Spread,..}))) {
            return self.spread_call(context, expr, loans, expected);
        }
        if let ExprKind::Call(callee, _) = &expr.kind
            && matches!(callee.kind, ExprKind::Field(_, _))
            && self
                .expression_type_hint(context, callee)
                .is_some_and(|ty| matches!(ty, Type::StoredCallable(_)))
        {
            return self.builtin_or_method_call(context, expr, loans, expected);
        }
        if let Some(result) = self.conditional_value_call(context, expr, loans, expected)? {
            return Ok(result);
        }
        // Recursive concat arguments must not retain the full builtin dispatch frame.
        if matches!(&expr.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Field(_, name) if name == "concat"))
        {
            return self.concat_call(context, expr, loans, expected);
        }
        let ordinary_name = matches!(&expr.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Name(name)
                if !matches!(name.as_str(), "String" | "Boolean" | "Number" | "clone")));
        if ordinary_name {
            self.prepare_generic_call(context, expr, expected)?;
            self.named_call(context, expr, loans, expected)
        } else {
            self.builtin_or_method_call(context, expr, loans, expected)
        }
    }

    // Keep conditional-call target construction off ordinary recursive call frames.
    #[inline(never)]
    fn conditional_value_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        if let ExprKind::Call(callee, args) = &mut expr.kind
            && self
                .flow_view(context, callee)
                .is_some_and(|(_, _, payload)| {
                    matches!(
                        payload,
                        Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_)
                    )
                })
        {
            let span = expr.span;
            let (target, result) = self.conditional_call(context, callee, args, loans, span, expected)?;
            expr.target = Some(target);
            return Ok(Some(result));
        }
        Ok(None)
    }

    fn concat_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("concat call dispatch");
        };
        let ExprKind::Field(base, _) = &mut callee.kind else {
            unreachable!("concat receiver dispatch");
        };
        let ty = self.collection_concat(context, base, args, loans, expected)?;
        expr.target = Some(if ty == Type::String {
            CallTarget::StringConcat
        } else {
            CallTarget::ArrayConcat
        });
        if expr.target == Some(CallTarget::ArrayConcat) {
            self.require_cloneable(base.ty.expect("checked array receiver"), base.span)?;
            for arg in args {
                self.require_cloneable(arg.ty.expect("checked concat argument"), arg.span)?;
            }
        }
        Ok(ty)
    }

    /// Binds `let name: Type;`. A type that admits `undefined` starts as
    /// `undefined` and is handled as an ordinary declaration (returns false).
    /// Any other type stays unassigned until its first assignment; reads are
    /// checked by `definite_assignment`.
    #[inline(never)]
    fn deferred_let(
        &mut self,
        context: &mut Context,
        name: &Token,
        expected: Option<Type>,
        value: &mut Expr,
        binding: &mut Option<usize>,
    ) -> Result<bool, Diagnostic> {
        let ty = expected.expect("parsed deferred annotation");
        let admits_undefined = ty == Type::Undefined
            || ty
                .union()
                .is_some_and(|union| self.unions.members(union).contains(&Type::Undefined));
        if admits_undefined {
            value.span = name.span;
            return Ok(false);
        }
        if matches!(ty, Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::Void) {
            return Err(Diagnostic::new(
                "unsupported",
                "A function-typed variable needs an initializer.",
                name.span,
            ));
        }
        let id = Self::bind(context, name, ty, false)?;
        context.bindings[id].rebindable = true;
        value.ty = Some(ty);
        *binding = Some(id);
        Ok(true)
    }

    /// Rewrites an undeclared global number name to its `Number` member, so
    /// one implementation serves both spellings. Declarations take precedence.
    #[inline(never)]
    fn global_number_name(&self, context: &Context, expr: &mut Expr, callee: bool) -> bool {
        let ExprKind::Name(name) = &expr.kind else {
            return false;
        };
        let Some(member) = crate::syntax::global_number_member(name, callee) else {
            return false;
        };
        if context.names.contains_key(name)
            || context.pending_names.contains(name)
            || self.function_names.contains_key(name)
            || crate::native_bindings::by_symbol(name).is_some()
        {
            return false;
        }
        let namespace = Expr::new(ExprKind::Name("Number".into()), expr.span);
        expr.kind = ExprKind::Field(Box::new(namespace), member.into());
        true
    }

    fn named_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee, _) = &expr.kind else {
            unreachable!("named call dispatch");
        };
        let ExprKind::Name(name) = &callee.kind else {
            unreachable!("named callee dispatch");
        };
        if crate::native_bindings::by_symbol(name).is_some()
            || context.names.contains_key(name)
            || (self.discovering_results && context.pending_names.contains(name))
        {
            return self.bound_named_call(context, expr, loans, expected);
        }
        self.declared_named_call(context, expr, loans)
    }

    fn bound_named_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("named call dispatch");
        };
        let ExprKind::Name(name) = &mut callee.kind else {
            unreachable!("named callee dispatch");
        };
        let stored_native = context.names.get(name).copied().and_then(|local| {
            let Type::Closure(origin) = context.bindings[local].ty else {
                return None;
            };
            let Some(ClosureForward::Native(id)) = self.closures[origin].forward else {
                return None;
            };
            Some((local, id))
        });
        let native = crate::native_bindings::by_symbol(name)
            .or_else(|| stored_native.map(|(_, id)| (id, &crate::native_bindings::BINDINGS[id])));
        if let Some((id, binding)) = native {
            if binding.parameters.first().is_some_and(|p| p.ty == crate::native_bindings::NativeType::ObservedValue) {
                if args.len() != binding.parameters.len() { return Err(Diagnostic::new("arity", format!("{} expects {} arguments.", binding.name, binding.parameters.len()), span)); }
                for argument in args.iter_mut() { self.expression(context, argument, Mode::Read, loans, None)?; }
                if binding.name == "containsValue" {
                    let actual = args[0].ty.expect("checked container");
                    let expected = args[1].ty.expect("checked expected");
                    let element = match actual {
                        Type::String => Some(Type::String),
                        Type::Array(element) if element.depth() == 0 && matches!(element.ty(), Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined) => Some(element.ty()),
                        _ => None,
                    };
                    let scalar_union_search = match actual {
                        Type::Array(ArrayElement::Union(union, 0)) => {
                            let primitive = |ty| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined);
                            primitive(expected) && self.unions.members(union).into_iter().all(primitive)
                        }
                        _ => false,
                    };
                    if element != Some(expected) && !scalar_union_search { return Err(Diagnostic::new("unsupported", "Private containment currently requires a string or primitive array with a matching expected type, or a primitive union array and scalar expected value.", span)); }
                }
                expr.target = Some(CallTarget::NativeBinding(id));
                expr.ty = Some(native_type(binding.result));
                return Ok(native_type(binding.result));
            }
            if binding.result == crate::native_bindings::NativeType::SharedCallback {
                return self.shared_callback_call(context, expr, loans);
            }
            if let Some((local, _)) = stored_native {
                if context.is_capture(local)
                    && context.bindings[local]
                        .callback_kind
                        .is_some_and(|kind| !kind.is_owned())
                {
                    return Err(Diagnostic::new(
                        "callback-escape",
                        "Capturing a callback requires an owned parameter contract.",
                        callee.span,
                    ));
                }
                let invocation = context.bindings[local]
                    .callback_invocation
                    .unwrap_or(Mode::Read);
                Self::access(context, local, invocation, callee.span, loans)?;
                callee.binding = Some(local);
                callee.ty = Some(context.bindings[local].ty);
            }
            if !binding.accepts_arity(args.len()) {
                return Err(Diagnostic::new(
                    "arity",
                    format!(
                        "{} expects {} to {} arguments, found {}.",
                        binding.name,
                        binding
                            .parameters
                            .iter()
                            .filter(|parameter| !parameter.optional)
                            .count(),
                        binding.parameters.len(),
                        args.len()
                    ),
                    span,
                ));
            }
            let contracts = binding.parameters[..args.len()].iter().map(|parameter| {
                Ok(ArgumentContract { ty: self.native_parameter_type(parameter.ty)?,
                    mode: parameter.access, omission: crate::ParameterOmission::Required, rest: false })
            }).collect::<Result<Vec<_>, Diagnostic>>()?;
            self.check_call_arguments(context, args, &contracts, loans)?;
            let result = native_type(binding.result);
            expr.target = Some(CallTarget::NativeBinding(id));
            expr.ty = Some(result);
            return Ok(result);
        }
        if self.discovering_results && context.pending_names.contains(name) {
            return Err(Self::pending_result(span));
        }
        if let Some(&binding) = context.names.get(name) {
            if context.is_capture(binding)
                && context.bindings[binding]
                    .callback_kind
                    .is_some_and(|kind| !kind.is_owned())
            {
                return Err(Diagnostic::new(
                    "callback-escape",
                    "Capturing a callback requires an owned parameter contract.",
                    callee.span,
                ));
            }
            if let Type::Closure(origin) = context.bindings[binding].ty
                && let Some(ClosureForward::Function(function)) = self.closures[origin].forward
            {
                let (function, result) =
                    self.forward_call(context, binding, function, callee, args, loans, span)?;
                expr.target = Some(CallTarget::ForwardFunction(function));
                expr.ty = Some(result);
                return Ok(result);
            }
            if let Type::Closure(origin) = context.bindings[binding].ty
                && let Some(ClosureForward::Once(inner)) = self.closures[origin].forward
                && let Some(ClosureForward::Function(function)) = self.closures[inner].forward
            {
                let (function, result) =
                    self.forward_call(context, binding, function, callee, args, loans, span)?;
                expr.target = Some(CallTarget::ForwardFunction(function));
                expr.ty = Some(result);
                return Ok(result);
            }
            if let Type::Closure(origin) = context.bindings[binding].ty
                && (self.closures[origin].polymorphic
                    || matches!(self.closures[origin].forward, Some(ClosureForward::Once(inner)) if self.closures[inner].polymorphic))
            {
                let (method, result) =
                    self.closure_method_call(context, binding, origin, callee, args, loans, span, expected)?;
                expr.target = Some(CallTarget::ClosureMethod(method));
                expr.ty = Some(result);
                return Ok(result);
            }
            let (params, result, invocation, target) = match context.bindings[binding].ty {
                Type::StoredCallable(family) => {
                    let declaration =
                        self.callables.storage.declaration(family).ok_or_else(|| {
                            Diagnostic::new(
                                "callable-layout",
                                "Unknown callable storage family.",
                                callee.span,
                            )
                        })?;
                    let contract = &self.callables.callbacks[declaration];
                    (
                        contract.params.clone(),
                        contract.result,
                        contract.invocation(&self.callables.shapes),
                        CallTarget::StoredCallable(binding),
                    )
                }
                Type::Callback(id) => {
                    if context.is_capture(binding)
                        && !self.callables.callbacks[id].kind.is_owned()
                    {
                        return Err(Diagnostic::new(
                            "callback-escape",
                            "Capturing a callback requires an owned parameter contract.",
                            callee.span,
                        ));
                    }
                    let contract = &self.callables.callbacks[id];
                    let invocation = contract.invocation(&self.callables.shapes);
                    (
                        contract.params.clone(),
                        contract.result,
                        invocation,
                        CallTarget::Callback(binding),
                    )
                }
                Type::Closure(id) => {
                    let closure = &self.closures[id];
                    let params = closure
                        .parameters
                        .iter()
                        .map(|&id| ArgumentContract::from(&closure.function.bindings[id]))
                        .collect();
                    (
                        params,
                        closure.function.result,
                        context.bindings[binding]
                            .callback_invocation
                            .unwrap_or(closure.invocation),
                        CallTarget::Closure(binding),
                    )
                }
                _ => {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "Only closure bindings are callable values.",
                        callee.span,
                    ));
                }
            };
            Self::prepare_arguments(args, &params, span)?;
            Self::access(context, binding, invocation, callee.span, loans)?;
            let mut active = loans.clone();
            active
                .entry(binding)
                .or_default()
                .push((invocation, callee.span));
            for (arg, contract) in args.iter_mut().zip(params) {
                let actual =
                    self.parameter_argument(context, arg, contract, &active)?;
                self.require(actual, Some(contract.ty), arg.span)?;
                if !actual.is_copy() {
                    for binding in Self::loan_roots(arg, contract.mode) {
                        active
                            .entry(binding)
                            .or_default()
                            .push((contract.mode, arg.span));
                    }
                }
            }
            let result = if let Type::Callable(contract) = result {
                if context.template || self.discovering_results {
                    self.result_shape(contract)
                } else {
                    return Err(Self::pending_result(span));
                }
            } else {
                result
            };
            // Preserve origin without treating the callee name as an evaluated value.
            if let Type::Closure(origin)=context.bindings[binding].ty {
                expr.callable_origin=Some(CallableOrigin::Closure(origin));
            }
            if let Type::StoredCallable(family)=context.bindings[binding].ty {
                expr.callable_origin=Some(CallableOrigin::Storage(family));
            }
            if let Type::Callback(contract)=context.bindings[binding].ty {
                expr.callable_origin=Some(CallableOrigin::Callback(contract));
            }
            expr.target = Some(target);
            expr.ty = Some(result);
            return Ok(result);
        }
        unreachable!("bound named-call dispatch");
    }

    fn shared_callback_call(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans) -> Result<Type, Diagnostic> {
        let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!() };
        if args.len() != 1 {
            return Err(Diagnostic::new("arity", "shareCallback expects one concrete callback.", expr.span));
        }
        let value = &mut args[0];
        if let ExprKind::Closure { id, .. } = &value.kind { self.escaping.insert(*id); }
        let actual = self.expression(context, value, Mode::Consume, loans, None)?;
        let Type::Closure(origin) = actual else {
            return Err(Diagnostic::new("callable-layout", "Sharing requires a concrete owned readonly callback environment.", value.span));
        };
        let closure = &self.closures[origin];
        let params = closure.parameters.iter().map(|&id| ArgumentContract::from(&closure.function.bindings[id])).collect::<Vec<_>>();
        if params.iter().any(|p| p.rest || p.omission != crate::ParameterOmission::Required) {
            return Err(Diagnostic::new("unsupported", "Shared callback construction currently requires fixed required parameters.", value.span));
        }
        let result = closure.function.result;
        let minimum = params.len();
        let structural = self.callables.shapes.intern_rest(params.iter().map(|p| (p.ty.into(), p.mode)).collect(), result.into(), Some(CallbackKind::SharedRead), minimum, false);
        let callback = CallbackContract { structural, parameter_names: closure.parameters.iter().map(|&id| closure.function.bindings[id].name.clone()).collect(), params, result, kind: CallbackKind::SharedRead };
        let declaration = self.callables.callbacks.len();
        let ty = self.callables.storage.intern_shared(&self.callables.shapes, &callback, declaration).expect("shared contract");
        self.callables.callbacks.push(callback);
        let Type::StoredCallable(family) = ty else { unreachable!() };
        if !context.template && !self.discovering_results && !self.effects_only {
            self.callables.storage.register_origin(family, origin, &self.callables.callbacks, &self.callables.shapes, &self.closures, value.span)?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        let mut lowered = args.remove(0);
        lowered.wrap_storage(family);
        *expr = lowered;
        Ok(ty)
    }

    fn native_parameter_type(&mut self, ty: crate::native_bindings::NativeType) -> Result<Type, Diagnostic> {
        if ty == crate::native_bindings::NativeType::ObservedValue {
            return Err(Diagnostic::new("unsupported", "Private value observation requires a direct call.", Span::default()));
        }
        if ty == crate::native_bindings::NativeType::SharedCallback {
            return Err(Diagnostic::new("unsupported", "The private sharing intrinsic requires a direct call.", Span::default()));
        }
        if ty == crate::native_bindings::NativeType::OwnedVoidCallback {
            let annotation = crate::type_syntax::parse("Owned<Function<void>>")?;
            let callback = crate::callable_annotations::callback(&annotation).expect("native callback annotation");
            self.resolve_callback(&callback)
        } else {
            Ok(native_type(ty))
        }
    }

    fn check_call_arguments(
        &mut self,
        context: &mut Context,
        args: &mut [Expr],
        params: &[ArgumentContract],
        loans: &Loans,
    ) -> Result<(), Diagnostic> {
        // Inline callback captures stay live through subsequent argument
        // evaluation and the call, then release. Existing named closure
        // loans keep their enclosing block lifetime.
        let first_capture_loan = context.capture_loans.len();
        let mut active = loans.clone();
        for (arg, contract) in args.iter_mut().zip(params.iter().copied()) {
            let (mode, ty) = (contract.mode, contract.ty);
            if let Type::Callback(id) = ty
                && self.callables.callbacks[id].kind.is_owned()
                && let ExprKind::Closure { id, .. } = &arg.kind
            {
                self.escaping.insert(*id);
            }
            let actual = self.parameter_argument(context, arg, contract, &active)?;
            self.require(actual, Some(ty), arg.span)?;
            if !actual.is_copy() {
                for binding in Self::loan_roots(arg, mode) {
                    active.entry(binding).or_default().push((mode, arg.span));
                }
            }
        }
        context.capture_loans.truncate(first_capture_loan);
        Ok(())
    }

    fn declared_named_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("declared call dispatch");
        };
        let ExprKind::Name(name) = &mut callee.kind else {
            unreachable!("declared callee dispatch");
        };
        let index = *self.function_names.get(name).ok_or_else(|| {
            Diagnostic::new("name", format!("Unknown function {name:?}."), callee.span)
        })?;
        if name == "main" {
            return Err(Diagnostic::new(
                "unsupported",
                "Calling the entry point is outside this milestone.",
                callee.span,
            ));
        }
        self.named_captures.prepare(index, context, &self.program)?;
        self.function(index)?;
        // Propagate body access through static named edges. Runtime loan formation
        // remains the separate environment-lowering stage behind the public guard.
        // A direct call passes each captured binding as a hidden argument:
        // a copy, or a borrow that lasts for the call. Usage and forwarded
        // access are recorded here; the borrows are checked after the
        // written arguments, which are evaluated first.
        let capture_accesses = self.named_captures.access(index, self.checked[index].as_ref(), &self.assumed_capture_access, self.signatures[index].params.len());
        let mut capture_slots = Vec::with_capacity(capture_accesses.len());
        for access in &capture_accesses {
            let Some(&slot) = context.source_bindings.get(&access.binding.declaration_start) else {
                return Err(self.program.local_capture_plans.iter().find_map(crate::local_capture_plan::Plan::pending_diagnostic).expect("pending named capture"));
            };
            let binding = &mut context.bindings[slot];
            binding.used.get_or_insert(callee.span);
            if binding.parameter && (!binding.ty.is_copy() || access.mode == Mode::Mutate) {
                binding.mode = binding.mode.max(access.mode);
            }
            capture_slots.push((slot, access.mode));
        }
        let params = if let Some(target) = &self.checked[index] {
            target.bindings[..target.parameter_count]
                .iter()
                .map(ArgumentContract::from)
                .collect::<Vec<_>>()
        } else {
            self.assumed_modes
                .as_ref()
                .expect("recursive access summaries")[index]
                .iter()
                .copied()
                .zip(self.signatures[index].params.iter().copied())
                .zip(&self.program.functions[index].parameter_initializers)
                .enumerate().map(|(position,((mode,ty),initializer))|ArgumentContract {rest:self.program.functions[index].rest_parameter && position+1==self.signatures[index].params.len(),ty,mode,omission:initializer.omission()})
                .collect()
        };
        Self::prepare_arguments(args, &params, span)?;
        self.check_call_arguments(context, args, &params, loans)?;
        if !capture_slots.is_empty() {
            self.named_capture_arguments(context, &capture_slots, args, &params, callee.span, loans)?;
        }
        let (index, result) = self.resolved_call(context, index, args, span)?;
        for (argument, parameter) in args.iter_mut().zip(&self.signatures[index].params) {
            if parameter.union().is_some()
                && self
                    .unions
                    .payload(*parameter)
                    .is_some_and(|ty| matches!(ty, Type::Closure(_)))
            {
                if let ExprKind::Inject { value, .. } = &argument.kind {
                    if value.ty.map(Type::owning) == Some(*parameter) {
                        let ExprKind::Inject { value, .. } =
                            std::mem::replace(&mut argument.kind, ExprKind::Null)
                        else {
                            unreachable!()
                        };
                        *argument = *value;
                    }
                }
            }
        }
        expr.target = Some(CallTarget::Function(index));
        if !capture_slots.is_empty() {
            expr.capture_slots = Some(capture_slots.iter().map(|&(slot, _)| slot).collect());
        }
        Ok(result)
    }

    fn builtin_or_method_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("call dispatched from expression")
        };
        if matches!(callee.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
            && let Some(Type::StoredCallable(family)) = self.expression_type_hint(context, callee)
        {
            let declaration = self
                .callables
                .storage
                .declaration(family)
                .expect("checked field storage signature");
            let contract = &self.callables.callbacks[declaration];
            let params = contract.params.clone();
            let result = contract.result;
            let invocation = contract.invocation(&self.callables.shapes);
            Self::prepare_arguments(args, &params, span)?;
            self.expression(
                context,
                callee,
                invocation,
                loans,
                Some(Type::StoredCallable(family)),
            )?;
            let mut active = loans.clone();
            for root in Self::loan_roots(callee, invocation) {
                active
                    .entry(root)
                    .or_default()
                    .push((invocation, callee.span));
            }
            for (arg, parameter) in args.iter_mut().zip(params) {
                let actual =
                    self.parameter_argument(context,arg,parameter,&active)?;
                self.require(actual, Some(parameter.ty), arg.span)?;
                if !actual.is_copy() {
                    for root in Self::loan_roots(arg, parameter.mode) {
                        active
                            .entry(root)
                            .or_default()
                            .push((parameter.mode, arg.span));
                    }
                }
            }
            expr.target = Some(CallTarget::StoredCallableField(family));
            expr.ty = Some(result);
            return Ok(result);
        }
        if matches!(
            callee.kind,
            ExprKind::Selected { .. }
                | ExprKind::CoalesceAssign { .. }
                | ExprKind::SpreadValue(_)
                | ExprKind::Call(_, _)
                | ExprKind::Closure { .. }
                | ExprKind::Conditional { .. }
                | ExprKind::Binary {
                    op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                    ..
                }
                | ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    ..
                }
        ) {
            let (target, result) = self.conditional_call(context, callee, args, loans, span, expected)?;
            expr.target = Some(target);
            expr.ty = Some(result);
            return Ok(result);
        }
        let ty = match &mut callee.kind {
            ExprKind::Name(name) if matches!(name.as_str(), "String" | "Boolean") => {
                self.value_builtin_arguments(context, args, loans, name == "String")?;
                expr.target = Some(if name == "String" {
                    CallTarget::StringConversion
                } else {
                    CallTarget::BooleanConversion
                });
                if name == "String" {
                    Type::String
                } else {
                    Type::Boolean
                }
            }
            ExprKind::Field(base, name)
                if name == "raw"
                    && matches!(&base.kind, ExprKind::Name(name) if name == "String" && !context.names.contains_key(name)) =>
            {
                self.string_raw_arguments(context, args, loans)?;
                expr.target = Some(CallTarget::StringRaw);
                Type::String
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "fromCodePoint" | "fromCharCode")
                    && matches!(&base.kind, ExprKind::Name(name) if name == "String" && !context.names.contains_key(name)) =>
            {
                let target = if name == "fromCodePoint" {
                    CallTarget::StringFromCodePoint
                } else {
                    CallTarget::StringFromCharCode
                };
                let codes = Type::Array(ArrayElement::Int(0));
                let actual = self.expression(context, &mut args[0], Mode::Read, loans, Some(codes))?;
                if actual != codes {
                    return Err(Diagnostic::new(
                        "type",
                        format!("String.{name} takes int codes; convert other values first."),
                        args[0].span,
                    ));
                }
                expr.target = Some(target);
                Type::String
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "values" | "entries")
                    && matches!(&base.kind, ExprKind::Name(name) if name == "Object" && !context.names.contains_key(name)) =>
            {
                let entries = name == "entries";
                let (ty, pair) = self.object_values(context, args, loans, expected, span, entries)?;
                expr.target = Some(match pair {
                    Some(pair) => CallTarget::ObjectEntries(pair),
                    None => CallTarget::ObjectValues,
                });
                ty
            }
            ExprKind::Field(base, name)
                if (name == "hasOwn#in" && matches!(&base.kind,ExprKind::Name(name) if name=="Object#in")) || (name == "hasOwn" && matches!(&base.kind,ExprKind::Name(name) if name=="Object" && !context.names.contains_key(name))) =>
            {
                let ty=self.object_has_own(context,args,loans,span,name == "hasOwn#in")?;
                expr.target=Some(CallTarget::ObjectHasOwn);
                ty
            }
            ExprKind::Field(base, name)
                if (name == "keys#in" && matches!(&base.kind,ExprKind::Name(name) if name=="Object#in")) || (name == "keys" && matches!(&base.kind,ExprKind::Name(name) if name=="Object")) =>
            {
                let ty = self.object_keys(context, args, loans, span)?;
                expr.target = Some(CallTarget::ObjectKeys);
                ty
            }
            ExprKind::Field(base, name)
                if name == "is" && matches!(&base.kind,ExprKind::Name(name) if name=="Object") =>
            {
                let ty = self.object_is(context, args, loans, span)?;
                expr.target = Some(CallTarget::ObjectIs);
                ty
            }
            ExprKind::Field(base, name)
                if name == "isArray"
                    && matches!(&base.kind,ExprKind::Name(name) if name=="Array") =>
            {
                self.value_builtin_arguments(context, args, loans, false)?;
                expr.target = Some(CallTarget::ArrayIsArray);
                Type::Boolean
            }
            ExprKind::Name(name) if name == "Number" => {
                if args.len() > 1 {
                    return Err(Diagnostic::new(
                        "arity",
                        format!("Number() expects 0 or 1 arguments, found {}.", args.len()),
                        span,
                    ));
                }
                if let Some(argument) = args.first_mut() {
                    let scalar = |ty| matches!(ty, Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String | Type::Enum(_, _));
                    let accepted = |ty: Type| ty.union().map(|id| self.unions.members(id).iter().copied().all(scalar)).unwrap_or_else(|| scalar(ty));
                    if self.expression_type_hint(context, argument).is_some_and(|ty| !accepted(ty)) {
                        return Err(Diagnostic::new("type", "Number() and unary + accept only scalar data, including scalar unions; arrays, records and functions need an explicit scalar value.", argument.span));
                    }
                    let expected = number_literal_context(argument).then_some(Type::Number);
                    let actual = self.expression(context, argument, Mode::Read, loans, expected)?;
                    let accepted = actual.union().map(|id| self.unions.members(id).iter().copied().all(scalar)).unwrap_or_else(|| scalar(actual));
                    if !accepted {
                        return Err(Diagnostic::new("type", "Number() and unary + accept only scalar data, including scalar unions; arrays, records and functions need an explicit scalar value.", argument.span));
                    }
                }
                expr.target = Some(CallTarget::Number);
                Type::Number
            }
            ExprKind::Name(name) if name == "clone" => {
                Self::arity(args.len(), 1, span)?;
                let actual = self.expression(context, &mut args[0], Mode::Read, loans, None)?;
                let ty = match &args[0].kind {
                    ExprKind::Project { narrowed, .. } => *narrowed,
                    _ => actual,
                };
                if !matches!(ty, Type::String | Type::Record(_) | Type::Set(_) | Type::Map(_) | Type::Array(_))
                    && !matches!(ty,Type::Union(id) | Type::BorrowedUnion(id) if !id.copy)
                    && !self.shared_callable_handle(ty)
                {
                    return Err(Diagnostic::new(
                        "unsupported",
                        "clone() accepts strings, records, arrays, Sets, Maps, and nullable unions with owned payloads.",
                        args[0].span,
                    ));
                }
                self.require_cloneable(ty, args[0].span)?;
                expr.target = Some(CallTarget::Clone);
                ty.owning()
            }
            ExprKind::Field(base, name)
                if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                    && name == "parseInt" =>
            {
                let ty = self.number_parse_int(context, args, loans, span)?;
                expr.target = Some(CallTarget::NumberParseInt);
                ty
            }
            ExprKind::Field(base, name)
                if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                    && name == "parseFloat" =>
            {
                let ty = self.number_parse_float(context, args, loans, span)?;
                expr.target = Some(CallTarget::NumberParseFloat);
                ty
            }
            ExprKind::Field(base, name)
                if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Number")
                    && number_predicate_target(name).is_some() =>
            {
                let ty = self.number_predicate(context, name, args, loans, span)?;
                expr.target = number_predicate_target(name);
                ty
            }
            ExprKind::Field(base, name)
                if matches!(&base.kind, ExprKind::Name(namespace) if namespace == "Math")
                    && math_target(name).is_some() =>
            {
                let ty = self.math_arguments(context, name, args, loans, span)?;
                expr.target = math_target(name);
                ty
            }
            ExprKind::Field(base, name)
                if matches!(
                    name.as_str(),
                    "log"
                        | "info"
                        | "debug"
                        | "warn"
                        | "error"
                        | "table"
                        | "assert"
                        | "dir"
                        | "dirxml"
                        | "group"
                        | "groupCollapsed"
                        | "groupEnd"
                        | "count"
                        | "countReset"
                        | "time"
                        | "timeLog"
                        | "timeEnd"
                        | "clear"
                        | "trace"
                        | "profile"
                        | "profileEnd"
                        | "timeStamp"
                ) && matches!(&base.kind, ExprKind::Name(name) if name == "console") =>
            {
                if name == "table" && args.len() > 2 {
                    return Err(Diagnostic::new(
                        "arity",
                        "console.table() expects optional data and an optional string[] column list.",
                        span,
                    ));
                }
                let first_capture = context.capture_loans.len();
                let mut active = loans.clone();
                for (index, argument) in args.iter_mut().enumerate() {
                    let expected = (name == "table"
                        && index == 1
                        && self.expression_type_hint(context, argument) != Some(Type::Undefined))
                    .then_some(Type::Array(ArrayElement::String(0)));
                    let previous = self.builtin_literals;
                    self.builtin_literals = true;
                    let result = self.expression(context, argument, Mode::Read, &active, expected);
                    self.builtin_literals = previous;
                    let actual = result?;
                    self.require(actual, expected, argument.span)?;
                    if name == "dir"
                        && index == 1
                        && !matches!(actual, Type::Record(_) | Type::Null | Type::Undefined)
                    {
                        return Err(Diagnostic::new(
                            "type",
                            "console.dir() options must be a record.",
                            argument.span,
                        ));
                    }
                    if name == "dir"
                        && index == 1
                        && let Type::Record(id) = actual
                    {
                        for (field, ty) in &self.fields[id] {
                            let valid = match field.as_str() {
                                "depth" => matches!(
                                    ty,
                                    Type::Int | Type::Number | Type::Null | Type::Undefined
                                ),
                                "colors" | "showHidden" => {
                                    matches!(ty, Type::Boolean | Type::Null | Type::Undefined)
                                }
                                _ => true,
                            };
                            if !valid {
                                return Err(Diagnostic::new(
                                    "type",
                                    format!("Invalid type for console.dir() option {field}."),
                                    argument.span,
                                ));
                            }
                        }
                    }
                    if !self.console_inspectable(actual) {
                        return Err(Diagnostic::new(
                            "type",
                            format!(
                                "console.{name}() accepts primitives, arrays, or records; callable and void values are unsupported."
                            ),
                            argument.span,
                        ));
                    }
                    if !actual.is_copy() {
                        for root in Self::loan_roots(argument, Mode::Read) {
                            active
                                .entry(root)
                                .or_default()
                                .push((Mode::Read, argument.span));
                        }
                    }
                }
                context.capture_loans.truncate(first_capture);
                expr.target = Some(CallTarget::Log);
                Type::Void
            }
            ExprKind::Field(base, name) if name == "normalize" => {
                let ty=self.string_normalize(context,base,args,loans,span)?;
                expr.target=Some(CallTarget::StringNormalize);
                ty
            }
            ExprKind::Field(base, name) if name == "toLocaleString" => {
                if !args.is_empty() {
                    return Err(Diagnostic::new("unsupported", "toLocaleString locale and options arguments are unsupported; remove the arguments.", args[0].span));
                }
                let actual = self.expression(context, base, Mode::Read, loans, None)?;
                expr.target = Some(match actual {
                    Type::Int => CallTarget::IntToLocaleString,
                    Type::Number => CallTarget::NumberToLocaleString,
                    Type::Array(ArrayElement::StoredCallable(_, _)) => return Err(Diagnostic::new("unsupported", "toLocaleString function-source elements are unsupported.", base.span)),
                    Type::Array(_) => CallTarget::ArrayToLocaleString,
                    _ => return Err(Diagnostic::new("type", "toLocaleString requires an int, number or array receiver.", base.span)),
                });
                Type::String
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "toFixed" | "toExponential" | "toPrecision") =>
            {
                if args.len() > 1 {
                    return Err(Diagnostic::new(
                        "arity",
                        format!("Number.{name}() expects zero or one argument."),
                        span,
                    ));
                }
                let actual = self.expression(context, base, Mode::Copy, loans, None)?;
                self.require(actual, Some(Type::Number), base.span)?;
                if let Some(precision) = args.first_mut() {
                    let actual =
                        self.expression(context, precision, Mode::Copy, loans, Some(Type::Int))?;
                    self.require(actual, Some(Type::Int), precision.span)?;
                }
                expr.target = Some(match name.as_str() {
                    "toFixed" => CallTarget::NumberToFixed,
                    "toExponential" => CallTarget::NumberToExponential,
                    "toPrecision" => CallTarget::NumberToPrecision,
                    _ => unreachable!("checked number formatting"),
                });
                Type::String
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "isWellFormed" | "toWellFormed") =>
            {
                self.string_well_formed(context, base, args.len(), loans, span)?;
                let predicate = name == "isWellFormed";
                expr.target = Some(if predicate {
                    CallTarget::StringIsWellFormed
                } else {
                    CallTarget::StringIdentity
                });
                if predicate {
                    Type::Boolean
                } else {
                    Type::String
                }
            }
            ExprKind::Field(base, name) if matches!(name.as_str(), "valueOf" | "toString") => {
                let previous = self.builtin_literals;
                self.builtin_literals |= name == "toString";
                let receiver = self.expression(context, base, Mode::Read, loans, None);
                self.builtin_literals = previous;
                let actual = receiver?;
                if name == "toString" && matches!(actual, Type::Int | Type::Number) {
                    if args.len() > 1 {
                        return Err(Diagnostic::new(
                            "arity",
                            if actual == Type::Int {
                                "int.toString() expects zero or one argument."
                            } else {
                                "number.toString() expects zero or one argument."
                            },
                            span,
                        ));
                    }
                    if let Some(radix) = args.first_mut() {
                        let actual =
                            self.expression(context, radix, Mode::Copy, loans, Some(Type::Int))?;
                        self.require(actual, Some(Type::Int), radix.span)?;
                    }
                    expr.target = Some(if actual == Type::Int {
                        CallTarget::IntToString
                    } else {
                        CallTarget::NumberToString
                    });
                    return Ok(Type::String);
                }
                Self::arity(args.len(), 0, span)?;
                expr.target = Some(match (name.as_str(), actual) {
                    (_, Type::String) => CallTarget::StringIdentity,
                    ("valueOf", Type::Boolean) => CallTarget::BooleanIdentity,
                    ("toString", Type::Boolean) => CallTarget::BooleanToString,
                    ("toString", Type::Array(_)) => CallTarget::ArrayJoin,
                    ("valueOf", Type::Number) => CallTarget::NumberIdentity,
                    ("valueOf", Type::Int) => CallTarget::IntIdentity,
                    ("toString", Type::Number) => {
                        return Err(Diagnostic::new(
                            "unsupported",
                            "Number.toString() is not supported.",
                            callee.span,
                        ));
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            "type",
                            "valueOf() requires a string, boolean, number, or int receiver; toString() requires string, boolean, int, or array.",
                            callee.span,
                        ));
                    }
                });
                if name == "toString" {
                    Type::String
                } else {
                    actual
                }
            }
            ExprKind::Field(base, name)
                if matches!(
                    name.as_str(),
                    "trim"
                        | "trimStart"
                        | "trimEnd"
                        | "trimLeft"
                        | "trimRight"
                        | "toLowerCase"
                        | "toUpperCase"
                ) =>
            {
                Self::arity(args.len(), 0, span)?;
                let actual = self.expression(context, base, Mode::Read, loans, None)?;
                self.require(actual, Some(Type::String), callee.span)?;
                expr.target = Some(match name.as_str() {
                    "trim" => CallTarget::Trim,
                    "trimStart" | "trimLeft" => CallTarget::TrimStart,
                    "trimEnd" | "trimRight" => CallTarget::TrimEnd,
                    "toLowerCase" => CallTarget::Lower,
                    "toUpperCase" => CallTarget::Upper,
                    _ => unreachable!("checked string transform"),
                });
                Type::String
            }
            ExprKind::Field(base, name)
                if (name == STRING_CODE_POINT_SLOT || name == "charCodeAt")
                    && (matches!(base.kind, ExprKind::String(_))
                        || matches!(self.expression_type_hint(context, base), Some(Type::String))) =>
            {
                let slot = name == STRING_CODE_POINT_SLOT;
                if args.len() > 1 {
                    return Err(Diagnostic::new(
                        "arity",
                        format!("charCodeAt expects an optional int position, found {} arguments.", args.len()),
                        span,
                    ));
                }
                let receiver = self.expression(context, base, Mode::Read, loans, Some(Type::String))?;
                self.require(receiver, Some(Type::String), base.span)?;
                if let Some(position) = args.first_mut() {
                    let mut active = loans.clone();
                    for root in Self::loan_roots(base, Mode::Read) {
                        active.entry(root).or_default().push((Mode::Read, base.span));
                    }
                    let actual = self.expression(context, position, Mode::Copy, &active, Some(Type::Int))?;
                    self.require(actual, Some(Type::Int), position.span)?;
                }
                expr.target = Some(if slot {
                    CallTarget::StringCodePointSlot
                } else {
                    CallTarget::StringCharCodeAt
                });
                if slot { Type::Array(ArrayElement::Int(0)) } else { Type::Number }
            }
            ExprKind::Field(base,name) if name=="pattern#record-rest" => {
                let (ty,target)=self.record_pattern_rest(context,base,args,loans,span)?;
                expr.target=Some(target);ty
            }
            ExprKind::Field(base, name) if name=="pattern#rest" => {
                let (ty,target)=self.pattern_rest(context,base,args,loans,span)?;
                expr.target=Some(target);ty
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "slice" | "charAt" | "substring" | "substr") =>
            {
                let ty = self.sequence_selection(context, base, args, loans, span, name)?;
                expr.target = Some(if matches!(ty, Type::Array(_)) {
                    CallTarget::ArraySlice
                } else if name == "substring" {
                    CallTarget::StringSubstring
                } else if name == "substr" {
                    CallTarget::StringSubstr
                } else if name == "slice" {
                    CallTarget::StringSlice
                } else {
                    CallTarget::StringCharAt
                });
                ty
            }
            ExprKind::Field(base, name)
                if matches!(
                    name.as_str(),
                    "repeat" | "replace" | "replaceAll" | "split" | "padStart" | "padEnd"
                ) || string_html_target(name).is_some() =>
            {
                let ty = self.string_construct(context, base, args, loans, span, name)?;
                expr.target = Some(match name.as_str() {
                    "padStart" => CallTarget::StringPadStart,
                    "padEnd" => CallTarget::StringPadEnd,
                    "repeat" => CallTarget::StringRepeat,
                    "replace" => CallTarget::StringReplace,
                    "replaceAll" => CallTarget::StringReplaceAll,
                    "split" => CallTarget::StringSplit,
                    _ => string_html_target(name).expect("checked string construction method"),
                });
                ty
            }
            ExprKind::Field(base, name) if name == "reduce" || name == "reduceRight" => {
                let ty = self.array_reduce(context, base, args, loans, span)?;
                expr.target = Some(if name == "reduceRight" {
                    CallTarget::ArrayReduceRight
                } else {
                    CallTarget::ArrayReduce
                });
                ty
            }
            ExprKind::Field(base, name) if name == "join" => {
                let ty = self.array_join(context, base, args, loans, span)?;
                expr.target = Some(CallTarget::ArrayJoin);
                ty
            }
            ExprKind::Field(base, name) if name == "flat" => {
                let (ty, levels) = self.array_flat(context, base, args, loans, expected)?;
                expr.target = Some(CallTarget::ArrayFlat(levels));
                ty
            }
            ExprKind::Field(base, name) if name == "forEach" => {
                let ty = self.array_transform(
                    context,
                    base,
                    args,
                    loans,
                    span,
                    (false, true, Mode::Read, false, false, true, false),
                )?;
                expr.target = Some(CallTarget::ArrayForEach);
                ty
            }
            ExprKind::Field(base, name)
                if matches!(name.as_str(), "map" | "filter" | "flatMap") =>
            {
                let filter = name == "filter";
                let ty = self.array_transform(
                    context,
                    base,
                    args,
                    loans,
                    span,
                    (
                        filter,
                        false,
                        Mode::Read,
                        name == "flatMap",
                        name == "flatMap",
                        !filter,
                        false,
                    ),
                )?;
                expr.target = Some(if filter {
                    CallTarget::ArrayFilter
                } else if name == "flatMap" {
                    CallTarget::ArrayFlatMap
                } else {
                    CallTarget::ArrayMap
                });
                ty
            }
            ExprKind::Field(base, name) if name == "find" || name == "findLast" => {
                let result = self.array_find(context, base, args, loans, span)?;
                expr.target = Some(if name == "find" {
                    CallTarget::ArrayFind
                } else {
                    CallTarget::ArrayFindLast
                });
                result
            }
            ExprKind::Field(base, name)
                if matches!(
                    name.as_str(),
                    "some" | "every" | "findIndex" | "findLastIndex"
                ) =>
            {
                let ty = self.array_predicate(
                    context,
                    base,
                    args,
                    loans,
                    span,
                    name == "findIndex" || name == "findLastIndex",
                )?;
                expr.target = Some(match name.as_str() {
                    "some" => CallTarget::ArraySome,
                    "every" => CallTarget::ArrayEvery,
                    "findIndex" => CallTarget::ArrayFindIndex,
                    "findLastIndex" => CallTarget::ArrayFindLastIndex,
                    _ => unreachable!("checked array predicate"),
                });
                ty
            }
            ExprKind::Field(base, name) if name == "fill" || name == "copyWithin" => {
                let fill = name == "fill";
                let ty = self.array_range_mutation(context, base, args, loans, span, fill)?;
                expr.target = Some(if fill {
                    CallTarget::ArrayFill
                } else {
                    CallTarget::ArrayCopyWithin
                });
                ty
            }
            ExprKind::Field(base, name) if name == "toSpliced" => {
                let ty = self.array_to_spliced(context, base, args, loans, expected)?;
                expr.target = Some(CallTarget::ArrayToSpliced);
                ty
            }
            ExprKind::Field(base, name) if name == "entries" => {
                Self::arity(args.len(), 0, span)?;
                let (ty, pair) = self.array_entries(context, base, loans, expected, span)?;
                expr.target = Some(CallTarget::ArrayEntries(pair));
                ty
            }
            ExprKind::Field(base, name) if name == "keys" => {
                Self::arity(args.len(), 0, span)?;
                let ty = self.expression(context, base, Mode::Read, loans, None)?;
                if !matches!(ty, Type::Array(_)) {
                    return Err(Diagnostic::new("unsupported", "keys() is available on arrays.", span));
                }
                expr.target = Some(CallTarget::ArrayKeys);
                let result = Type::Array(ArrayElement::Int(0));
                self.require(result, expected, span)?;
                result
            }
            ExprKind::Field(base, name) if name == "toReversed" || name == "with" => {
                let replacement = name == "with";
                let ty =
                    self.array_copying(context, base, args, loans, span, replacement, expected)?;
                expr.target = Some(if replacement {
                    CallTarget::ArrayWith
                } else {
                    CallTarget::ArrayToReversed
                });
                ty
            }
            ExprKind::Field(base, name) if name == "sort" || name == "toSorted" => {
                let copying = name == "toSorted";
                let result =
                    self.array_sort(context, base, args, loans, span, (copying, expected))?;
                expr.target = Some(if copying {
                    CallTarget::ArrayToSorted
                } else {
                    CallTarget::ArraySort
                });
                result
            }
            ExprKind::Field(base, name) if name == "reverse" => {
                let ty = self.array_reverse(context, base, args, loans, span)?;
                expr.target = Some(CallTarget::ArrayReverse);
                ty
            }
            ExprKind::Field(base, name) if name == "at" => {
                Self::arity(args.len(), 1, span)?;
                let result = self.array_at(context, base, &mut args[0], loans, expected)?;
                expr.target = Some(CallTarget::ArrayAt);
                result
            }
            ExprKind::Field(base, name) if name == "pop" || name == "shift" => {
                let ty = self.array_optional_removal(context, base, args, loans, span, expected)?;
                expr.target = Some(if name == "pop" {
                    CallTarget::ArrayPop
                } else {
                    CallTarget::ArrayShift
                });
                ty
            }
            ExprKind::Field(base, name) if name == "splice" => {
                let ty = self.array_splice(context, base, args, loans, span, expected)?;
                expr.target = Some(CallTarget::ArraySplice);
                ty
            }
            ExprKind::Field(base, name) if name == "unshift" => {
                let ty = self.array_unshift(context, base, args, loans, span)?;
                expr.target = Some(CallTarget::ArrayUnshift);
                ty
            }
            ExprKind::Field(base, name) if name == "push" || name == "remove" => {
                let remove = name == "remove";
                let ty = self.array_mutation(context, base, args, loans, span, remove)?;
                expr.target = Some(if remove {
                    CallTarget::ArrayRemove
                } else {
                    CallTarget::ArrayPush
                });
                ty
            }
            ExprKind::Field(base, name)
                if matches!(
                    name.as_str(),
                    "startsWith" | "endsWith" | "includes" | "indexOf" | "lastIndexOf"
                ) =>
            {
                let ty = self.collection_search(context, base, args, loans, span, name)?;
                expr.target = Some(match (base.ty, name.as_str()) {
                    (Some(Type::String), "startsWith") => CallTarget::StringStartsWith,
                    (Some(Type::String), "endsWith") => CallTarget::StringEndsWith,
                    (Some(Type::String), "includes") => CallTarget::StringIncludes,
                    (Some(Type::String), "indexOf") => CallTarget::StringIndexOf,
                    (Some(Type::String), "lastIndexOf") => CallTarget::StringLastIndexOf,
                    (_, "includes") => CallTarget::ArrayIncludes,
                    (_, "indexOf") => CallTarget::ArrayIndexOf,
                    (_, "lastIndexOf") => CallTarget::ArrayLastIndexOf,
                    _ => unreachable!("checked search receiver"),
                });
                ty
            }
            ExprKind::Name(_) => unreachable!("named calls use their own frame"),
            _ => {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Unsupported call or method.",
                    callee.span,
                ));
            }
        };
        if let ExprKind::Field(base, _) = &callee.kind
            && crate::cloneability::copies_array_receiver(
                expr.target,
                Self::stable_at_receiver(base),
                args.len(),
            )
        {
            self.require_cloneable(base.ty.expect("checked array receiver"), base.span)?;
            if expr.target == Some(CallTarget::ArrayConcat) {
                for arg in args {
                    self.require_cloneable(arg.ty.expect("checked concat argument"), arg.span)?;
                }
            }
        }
        Ok(ty)
    }

    fn shared_callable_handle(&self, ty: Type) -> bool {
        let Type::StoredCallable(family)=ty else {return false;};
        self.callables.storage.declaration(family)
            .and_then(|declaration| self.callables.callbacks.get(declaration))
            .is_some_and(|contract| contract.kind == CallbackKind::SharedRead)
    }

    fn require_cloneable(&mut self, ty: Type, span: Span) -> Result<(), Diagnostic> {
        if self.shared_callable_handle(ty) { return Ok(()); }
        let payload = self.unions.payload(ty).unwrap_or(ty);
        if !matches!(
            payload,
            Type::Record(_) | Type::Map(_)
                | Type::Array(ArrayElement::Record(_, _) | ArrayElement::Map(_, _) | ArrayElement::StoredCallable(_, _))
                | Type::Closure(_)
                | Type::Callable(_)
                | Type::Callback(_)
                | Type::StoredCallable(_)
        ) {
            return Ok(());
        }
        if self
            .record_cloneability
            .as_ref()
            .is_none_or(|(count, _)| *count != self.program.records.len())
        {
            self.record_cloneability = Some((
                self.program.records.len(),
                crate::cloneability::Cloneability::new(&self.program.records, |id| {
                    self.unions.members(id)
                }),
            ));
        }
        if !self
            .record_cloneability
            .as_ref()
            .expect("clone eligibility")
            .1
            .allows(ty, |id| self.unions.members(id))
        {
            return Err(Diagnostic::new(
                "clone-type",
                "Values containing callable environments cannot be cloned.",
                span,
            ));
        }
        Ok(())
    }

    // Callable source provenance lives off the recursive operator frame.
    fn expression_name(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        if self.global_number_name(context, expr, false) {
            return self.expression(context, expr, mode, loans, expected);
        }
        let ExprKind::Name(name) = &expr.kind else {
            unreachable!("named expression dispatch")
        };
        if let Some((id, _)) = crate::native_bindings::by_symbol(name) {
            return self.native_function_value(id, span, expected);
        }
        if self.discovering_results && context.pending_names.contains(name) {
            return Err(Self::pending_result(span));
        }
        if !context.names.contains_key(name)
            && !self.function_names.contains_key(name)
            && crate::syntax::global_number_member(name, true).is_some()
        {
            return Err(Diagnostic::new(
                "unsupported",
                format!("The global function {name:?} cannot be used as a value; call it inside an arrow function, for example (text) => {name}(text)."),
                span,
            ));
        }
        if !context.names.contains_key(name) && self.generic_functions.template(name).is_some() {
            return self.contextual_generic_value(context, expr, mode, loans, expected);
        }
        let index = *context
            .names
            .get(name)
            .ok_or_else(|| Diagnostic::new("name", format!("Unknown value {name:?}."), span))?;
        if let Type::Closure(origin) = context.bindings[index].ty
            && self.deferred_generic_values.contains_key(&origin)
            && let Some(expected) = expected
            && self.checked_generic_callable_shape(expected).is_some() {
            return self.adapt_generic_value(context, expr, index, origin, expected, mode, loans);
        }
        let callback_argument = matches!(expected, Some(Type::Callback(_)));
        let observation =
            self.observing_values && mode == Mode::Read && !context.is_capture(index);
        if let Some(kind) = context.bindings[index].callback_kind {
            if !kind.is_owned()
                && (context.is_capture(index)
                    || (context.returning_closure && !callback_argument))
            {
                return Err(Diagnostic::new(
                    "callback-escape",
                    "Returns and captures require an owned callback contract.",
                    span,
                ));
            }
            if let Some(Type::Callback(wanted)) = expected
                && !kind.can_pass_to(self.callables.callbacks[wanted].kind)
            {
                return Err(Diagnostic::new(
                    "callback-contract",
                    "Callback ownership and invocation contracts cannot be interchanged.",
                    span,
                ));
            }
        }
        if let Type::Callback(id) = context.bindings[index].ty
            && (!(callback_argument
                || observation
                || (matches!(expected, Some(Type::Callable(_)))
                    && self.callables.callbacks[id].kind.is_owned()))
                || (context.is_capture(index)
                    && !self.callables.callbacks[id].kind.is_owned()))
        {
            return Err(Diagnostic::new(
                "callback-escape",
                "Callback values require a declaration move or matching callback contract; returns and captures require an owned contract.",
                span,
            ));
        }
        if matches!(context.bindings[index].ty, Type::Closure(_))
            && !(context.returning_closure
                || observation
                || callback_argument
                || matches!(expected, Some(Type::Closure(_))))
        {
            return Err(Diagnostic::new(
                "unsupported",
                "Callable value use requires a concrete declaration move, replacement, callback argument, or callable return; shared aliases are unsupported.",
                span,
            ));
        }
        expr.binding = Some(index);
        Self::access(context, index, mode, span, loans)?;
        let ty = context.bindings[index].ty;
        let ty = if matches!(
            expected,
            Some(Type::Callback(_) | Type::Callable(_) | Type::Closure(_))
        ) && let Type::Closure(origin) = ty
            && let Some(ClosureForward::Native(id)) = self.closures[origin].forward
        {
            let adapted = self.native_function_value(id, span, expected)?;
            if matches!(expected, Some(Type::Callable(_)))
                && context.bindings[index].callback_invocation == Some(Mode::Consume)
                && let Type::Closure(origin) = adapted
            {
                Type::Closure(self.once_return(origin, context, span))
            } else {
                adapted
            }
        } else if matches!(expected, Some(Type::Callable(_)))
            && let Type::Callback(id) = ty
        {
            Type::Closure(self.abstract_return(id, context, span))
        } else if matches!(expected, Some(Type::Callable(_)))
            && context.bindings[index].callback_invocation == Some(Mode::Consume)
            && let Type::Closure(id) = ty
            && self.closures[id].invocation != Mode::Consume
        {
            Type::Closure(self.once_return(id, context, span))
        } else {
            ty
        };
        expr.ty = Some(ty);
        Ok(ty)
    }

    fn expression_inner(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        _mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ty = match &mut expr.kind {
            ExprKind::EnumMember { enumeration, .. } => Type::Enum(*enumeration, self.program.records[*enumeration].enum_definition.as_ref().expect("enum member declaration").kind),
            ExprKind::TypeAssertion {..}
            | ExprKind::SpreadCall(_)
            | ExprKind::SpreadValue(_)
            | ExprKind::OptionalChain(_)
            | ExprKind::Optional { .. }
            | ExprKind::Selected { .. }
            | ExprKind::Conditional { .. }
            | ExprKind::Update { .. }
            | ExprKind::LogicalAssign { .. }
            | ExprKind::Assign { .. }
            | ExprKind::CoalesceAssign { .. }
            | ExprKind::CompoundAssign { .. } => {
                unreachable!("updates checked in their own frame")
            }
            ExprKind::Unary {
                op,
                operator,
                value,
            } => {
                if matches!(op, UnaryOp::Typeof | UnaryOp::Void) {
                    if *op == UnaryOp::Typeof
                        && let ExprKind::Name(name) = &value.kind
                        && !context.names.contains_key(name)
                        && context
                            .lexical_scopes
                            .iter()
                            .any(|scope| scope.contains(name))
                    {
                        return Err(Diagnostic::new(
                            "name",
                            format!("Cannot inspect {name:?} with typeof before its declaration."),
                            value.span,
                        ));
                    }
                    let unresolved = *op == UnaryOp::Typeof
                        && matches!(&value.kind, ExprKind::Name(name) if !context.names.contains_key(name) && !context.pending_names.contains(name) && !self.function_names.contains_key(name) && crate::syntax::global_number_member(name, false).is_none() && crate::syntax::global_number_member(name, true).is_none());
                    let first_capture = context.capture_loans.len();
                    if unresolved {
                        value.ty = Some(Type::Undefined);
                    } else {
                        let observing = self.observing_values;
                        self.observing_values = true;
                        let result = self.observed_expression(context, value, loans);
                        self.observing_values = observing;
                        result?;
                    }
                    context.capture_loans.truncate(first_capture);
                    let result = if *op == UnaryOp::Typeof {
                        Type::String
                    } else {
                        Type::Undefined
                    };
                    expr.ty = Some(result);
                    return Ok(result);
                }
                if *op == UnaryOp::Not
                    && matches!(
                        self.expression_type_hint(context, value),
                        Some(Type::Null | Type::Undefined)
                    )
                {
                    self.expression(context, value, Mode::Copy, loans, None)?;
                    expr.ty = Some(Type::Boolean);
                    return Ok(Type::Boolean);
                }
                let operand = if *op == UnaryOp::Not {
                    Type::Boolean
                } else if *op == UnaryOp::BitNot {
                    self.bitwise_or_power_type(context, value, None, expected)
                } else if expected == Some(Type::Int) {
                    Type::Int
                } else if expected == Some(Type::Number)
                    || self.expression_type_hint(context, value) == Some(Type::Number)
                {
                    Type::Number
                } else {
                    Type::Int
                };
                let actual = self.expression(context, value, Mode::Copy, loans, Some(operand))?;
                self.require(actual, Some(operand), *operator)?;
                if *op == UnaryOp::Not {
                    let (yes, no) = Self::condition_outcomes(context, value);
                    context
                        .condition_flows
                        .insert((span.start, span.end), (no, yes));
                }
                operand
            }
            ExprKind::Binary {
                op,
                operator,
                left,
                right,
            } => {
                let equality = matches!(op, BinaryOp::Equal | BinaryOp::NotEqual);
                if equality && Self::is_typeof_guard(context, left, right) {
                    return self.typeof_equality(context, span, *op, left, right, loans);
                }
                let hints = [
                    self.expression_type_hint(context, left),
                    self.expression_type_hint(context, right),
                ];
                if equality
                    && hints.iter().any(|ty| matches!(ty, Some(Type::Union(_) | Type::BorrowedUnion(_))))
                    && !hints
                        .iter()
                        .any(|ty| matches!(ty, Some(Type::Null | Type::Undefined)))
                {
                    return self.union_equality(context, (span, *operator), *op, left, right, loans);
                }
                let nullish_equality = equality
                    && [
                        self.expression_type_hint(context, left),
                        self.expression_type_hint(context, right),
                    ]
                    .iter()
                    .any(|ty| matches!(ty, Some(Type::Null | Type::Undefined)));
                if nullish_equality {
                    return self.nullish_equality(context, span, *op, left, right, loans);
                }

                let ordering = matches!(
                    op,
                    BinaryOp::Less
                        | BinaryOp::LessEqual
                        | BinaryOp::Greater
                        | BinaryOp::GreaterEqual
                );
                let comparison = equality || ordering;
                let logical = matches!(op, BinaryOp::And | BinaryOp::Or);
                if *op == BinaryOp::Add && (self.expression_type_hint(context, left) == Some(Type::String)
                    || self.expression_type_hint(context, right) == Some(Type::String)) {
                    self.string_concatenation_operand(context, left)?;
                    self.string_concatenation_operand(context, right)?;
                }
                let string_addition = *op == BinaryOp::Add
                    && (expected == Some(Type::String)
                        || self.expression_type_hint(context, left) == Some(Type::String)
                        || self.expression_type_hint(context, right) == Some(Type::String));
                let number = expected != Some(Type::Int)
                    && ((!comparison && expected == Some(Type::Number))
                        || self.expression_type_hint(context, left) == Some(Type::Number)
                        || self.expression_type_hint(context, right) == Some(Type::Number));
                let expected = if op.bitwise_or_power() {
                    Some(self.bitwise_or_power_type(context, left, Some(right), expected))
                } else if logical {
                    Some(Type::Boolean)
                } else if string_addition {
                    Some(Type::String)
                } else if number {
                    Some(Type::Number)
                } else if comparison {
                    None
                } else {
                    Some(Type::Int)
                };
                let left_type = self.expression(
                    context,
                    left,
                    if comparison || string_addition {
                        Mode::Read
                    } else {
                        Mode::Copy
                    },
                    loans,
                    expected,
                )?;
                if (equality
                    && !matches!(
                        left_type,
                        Type::Int | Type::Number | Type::Boolean | Type::String | Type::Enum(_, _)
                    ))
                    || (ordering && !matches!(left_type, Type::Int | Type::Number | Type::String | Type::Enum(_, _)))
                {
                    return Err(Diagnostic::new(
                        "type",
                        if equality {
                            "Strict equality requires two int, number, boolean, or string values."
                        } else {
                            "Ordering requires two int values, two number values, or two string values."
                        },
                        *operator,
                    ));
                }
                self.require(left_type, expected, *operator)?;
                // A borrowed string left operand remains live while evaluating
                // the right operand. Scalar accesses retain their prior release.
                let mut active = loans.clone();
                if left_type == Type::String {
                    for root in Self::loan_roots(left, Mode::Read) {
                        active
                            .entry(root)
                            .or_default()
                            .push((Mode::Read, left.span));
                    }
                }
                let right_type = self.expression(
                    context,
                    right,
                    if left_type == Type::String {
                        Mode::Read
                    } else {
                        Mode::Copy
                    },
                    &active,
                    Some(left_type),
                )?;
                self.require(right_type, Some(left_type), *operator)?;
                if equality && crate::flow::guard_pair(left, right, &context.names).is_some() {
                    self.remember_guard(context, span, *op, left, right);
                }
                match op {
                    op if op.bitwise_or_power() => left_type,
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Remainder => left_type,
                    _ => Type::Boolean,
                }
            }
            ExprKind::Closure {
                id,
                params,
                result,
                body,
            } => self.closure(context, (*id, params, result), expected, body, span, loans)?,
            ExprKind::String(_) => Type::String,
            ExprKind::Integer(spelling) => {
                if expected == Some(Type::Number) {
                    spelling.parse::<f64>().map_err(|_| {
                        Diagnostic::new("syntax", "Invalid contextual number literal.", span)
                    })?;
                    Type::Number
                } else {
                    spelling.parse::<i64>().map_err(|_|Diagnostic::new("integer-range", "Integer literal is outside the int range -9223372036854775808..=9223372036854775807.", span))?;
                    Type::Int
                }
            }
            ExprKind::Number(_) => Type::Number,
            ExprKind::Boolean(_) => Type::Boolean,
            ExprKind::Null => Type::Null,
            ExprKind::Undefined => Type::Undefined,
            ExprKind::Sequence(_) | ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) | ExprKind::Array(_) | ExprKind::ArrayOf(_, _) | ExprKind::Index(_, _) => {
                unreachable!("array expressions checked in the small dispatch frame")
            }
            ExprKind::Name(_) => unreachable!("names checked in their own frame"),
            ExprKind::Field(_, _) => unreachable!("fields checked in the small dispatch frame"),
            ExprKind::Record(_) => unreachable!("records checked in the small dispatch frame"),
            ExprKind::Template(_) => unreachable!("templates checked in their own frame"),
            ExprKind::Inject { .. } | ExprKind::Project { .. } => {
                unreachable!("injections checked in their own frame")
            }
            ExprKind::Call(_, _) => unreachable!("calls checked in their own frame"),
        };
        expr.ty = Some(ty);
        Ok(ty)
    }
}

fn native_type(ty: crate::native_bindings::NativeType) -> Type {
    use crate::native_bindings::NativeType;
    match ty {
        NativeType::String => Type::String,
        NativeType::Boolean => Type::Boolean,
        NativeType::Int => Type::Int,
        NativeType::Number => Type::Number,
        NativeType::Void => Type::Void,
        NativeType::OwnedVoidCallback | NativeType::SharedCallback | NativeType::ObservedValue => unreachable!("native callbacks resolve through callable contracts"),
    }
}

#[cfg(test)]
mod callable_shape_identity_tests {
    #[test]
    fn nested_equal_contracts_keep_distinct_source_ownership_paths() {
        let source = "function first(cb:Function<int, [next:Function<int, [value:int]>]>){}function second(cb:Function<int, [next:Function<int, [renamed:int]>]>){}function main(){}";
        let checked = crate::check(source).unwrap();
        let contracts = &checked.program.callback_contracts;
        assert_eq!(contracts.len(), 4);
        assert_eq!(contracts[0].structural, contracts[2].structural);
        assert_eq!(contracts[1].structural, contracts[3].structural);
        let functions = &checked.program.functions;
        assert_ne!(functions[0].bindings[0].ty, functions[1].bindings[0].ty);
        assert_eq!(contracts[0].parameter_names[0], "value");
        assert_eq!(contracts[2].parameter_names[0], "renamed");
    }
}

#[cfg(test)]
#[test]
fn stored_callable_owner_access_mutates_reusable_and_consumes_once() {
    let checked = crate::check(
        "function main() { const action: Function<int> = (): int => { return 7; }; }",
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let binding = checked.program.functions[0].bindings[0].clone();
    let mut reusable = Context::new("main".into());
    reusable.bindings.push(binding.clone());
    reusable.bindings[0].ty = Type::StoredCallable(0);
    let loans = Loans::new();
    for _ in 0..2 {
        Analyzer::access(&mut reusable, 0, Mode::Mutate, Span::new(2, 4), &loans).unwrap();
    }
    assert!(reusable.bindings[0].mutable);
    assert!(reusable.bindings[0].transferred.is_none());
    let mut once = Context::new("main".into());
    once.bindings.push(binding);
    once.bindings[0].ty = Type::StoredCallable(0);
    Analyzer::access(&mut once, 0, Mode::Consume, Span::new(3, 5), &loans).unwrap();
    let error = Analyzer::access(&mut once, 0, Mode::Consume, Span::new(7, 9), &loans).unwrap_err();
    assert_eq!(error.code, "closure-consumed");
    assert_eq!(error.span, Span::new(7, 9));
    assert_eq!(error.related[0].0, Span::new(3, 5));
}

#[cfg(test)]
#[test]
fn stored_named_calls_use_registered_signature_hints_and_consuming_access() {
    for once in [false, true] {
        let kind = if once { "Once<Function<int>>" } else { "Function<int>" };
        let source = format!(
            "function sink(action: {kind}): int {{ return action(); }} function main() {{}}"
        );
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse(&format!("{kind}")).unwrap();
        analyzer
            .resolve_callback(&crate::callable_annotations::callback(&annotation).unwrap())
            .unwrap();
        let mut context = Context::new("main".into());
        let name = Token {
            text: "action".into(),
            span: Span::new(2, 8),
            kind: TokenKind::Name,
        };
        let binding = Analyzer::bind(&mut context, &name, Type::StoredCallable(0), false).unwrap();
        let parsed = crate::syntax::parse("function main() { action(); }").unwrap();
        let StmtKind::Expr(mut call) = parsed
            .functions
            .into_iter()
            .next()
            .unwrap()
            .body
            .into_iter()
            .next()
            .unwrap()
            .kind
        else {
            panic!("parsed call");
        };
        assert_eq!(
            analyzer.expression_type_hint(&context, &call),
            Some(Type::Int)
        );
        assert!(context.bindings[binding].used.is_none());
        assert_eq!(
            analyzer
                .named_call(&mut context, &mut call, &Loans::new(), None)
                .unwrap(),
            Type::Int
        );
        assert_eq!(call.target, Some(CallTarget::StoredCallable(binding)));
        if once {
            assert_eq!(
                analyzer
                    .named_call(&mut context, &mut call, &Loans::new(), None)
                    .unwrap_err()
                    .code,
                "closure-consumed"
            );
        } else {
            analyzer
                .named_call(&mut context, &mut call, &Loans::new(), None)
                .unwrap();
            assert!(context.bindings[binding].mutable);
        }
    }
}

#[cfg(test)]
#[test]
fn stored_record_construction_checks_and_wraps_owned_environments() {
    for once in [false, true] {
        for transfer in [false, true] {
            let tail = if transfer {
                "const second:Holder={action:holder.action};"
            } else {
                ""
            };
            let owner = if transfer { "second" } else { "holder" };
            let calls = if once {
                format!("console.log({owner}.action());")
            } else {
                format!("console.log({owner}.action());console.log({owner}.action());")
            };
            let source = format!(
                "type Holder={{action:int;}};function main(){{const holder:Holder={{action:():int=>{{return 7;}}}};{tail}{calls}}}"
            );
            let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
            let annotation = crate::type_syntax::parse(if once {
                "Once<Function<int>>"
            } else {
                "Function<int>"
            })
            .unwrap();
            let (ty, declaration) = analyzer
                .callables
                .resolver(&analyzer.record_names, &analyzer.unions)
                .storage_declaration(&annotation)
                .unwrap()
                .unwrap();
            analyzer.fields[0].insert("action".into(), ty);
            analyzer.program.records[0].fields[0].ty = Some(ty);
            analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
            let checked = analyzer.run().unwrap();
            let StmtKind::Let { value, .. } = &checked.functions[0].body[0].kind else {
                panic!("holder binding");
            };
            let ExprKind::Record(fields) = &value.kind else {
                panic!("record initializer");
            };
            assert_eq!(fields[0].1.ty, Some(Type::StoredCallable(0)));
            assert!(matches!(
                fields[0].1.kind,
                ExprKind::Unary {
                    op: UnaryOp::CallableStore,
                    ..
                }
            ));
            let origins = checked.stored_contracts.populated().next().unwrap().2;
            assert_eq!(origins.len(), 1);
            let closure = &checked.closures[origins[0]];
            assert!(!closure.borrowed);
            assert_eq!(closure.invocation == Mode::Consume, once);
            let rust = crate::emit::emit(&checked);
            assert!(rust.contains("struct StoredCallable0("));
            assert!(rust.contains("field0: StoredCallable0("));
            crate::callable_dispatch::tests::execute(&rust, if once { "7\n" } else { "7\n7\n" });
        }
    }
}

#[cfg(test)]
#[test]
fn stored_record_construction_rejects_incompatible_and_borrowed_environments() {
    for (body, signature, code) in [
        (
            "const holder:Holder={action:():int=>{return 7;}};",
            "Function<string>",
            "type",
        ),
        (
            "const text=\"Pat\";const action=():int=>{return text.length;};const holder:Holder={action:action};",
            "Function<int>",
            "callback-escape",
        ),
        (
            "const holder:Holder={action:(text:Owned<string>):int=>{return text.length;}};",
            "Function<int, [text: ReadOnly<string>]>",
            "callback-access",
        ),
        (
            "const holder:Holder={action:():int=>{return 7;}};holder.action();holder.action();",
            "Once<Function<int>>",
            "partial-transfer",
        ),
    ] {
        let source = format!("type Holder={{action:int;}};function main(){{{body}}}");
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse(signature).unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        assert_eq!(analyzer.run().err().unwrap().code, code);
    }
}

#[cfg(test)]
#[test]
fn stored_callable_field_replacement_restores_consumed_storage() {
    for once in [false, true] {
        for consume_first in [false, true] {
            let first = if consume_first {
                "console.log(holder.action());"
            } else {
                ""
            };
            let source = format!(
                "type Holder={{action:int;}};function main(){{const holder:Holder={{action:():int=>{{return 7;}}}};{first}holder.action=():int=>{{return 8;}};console.log(holder.action());}}"
            );
            let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
            let annotation = crate::type_syntax::parse(if once {
                "Once<Function<int>>"
            } else {
                "Function<int>"
            })
            .unwrap();
            let (ty, declaration) = analyzer
                .callables
                .resolver(&analyzer.record_names, &analyzer.unions)
                .storage_declaration(&annotation)
                .unwrap()
                .unwrap();
            analyzer.fields[0].insert("action".into(), ty);
            analyzer.program.records[0].fields[0].ty = Some(ty);
            analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
            let checked = analyzer.run().unwrap();
            assert_eq!(
                checked.stored_contracts.populated().next().unwrap().2.len(),
                2
            );
            let rust = crate::emit::emit(&checked);
            assert!(rust.contains("enum StoredCallable0"));
            crate::callable_dispatch::tests::execute(
                &rust,
                if consume_first { "7\n8\n" } else { "8\n" },
            );
        }
    }
}

#[cfg(test)]
#[test]
fn stored_callable_replacement_does_not_restore_a_consumed_sibling() {
    let source = "type Holder={action:int;label:string;};function main(){const holder:Holder={action:():int=>{return 7;},label:\"Pat\"};const label=holder.label;holder.action();holder.action=():int=>{return 8;};holder.action();console.log(holder.label);}";
    let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
    let annotation = crate::type_syntax::parse("Once<Function<int>>").unwrap();
    let (ty, declaration) = analyzer
        .callables
        .resolver(&analyzer.record_names, &analyzer.unions)
        .storage_declaration(&annotation)
        .unwrap()
        .unwrap();
    analyzer.fields[0].insert("action".into(), ty);
    analyzer.program.records[0].fields[0].ty = Some(ty);
    analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
    let error = analyzer.run().err().unwrap();
    assert_eq!(error.code, "partial-transfer");
    assert!(error.message.contains("label"));
    assert_eq!(&source[error.span.start..error.span.end], "holder.label");
}

#[cfg(test)]
#[test]
fn unused_callable_record_fields_emit_uninhabited_native_families() {
    for once in [false, true] {
        let mut analyzer = Analyzer::new(
            crate::syntax::parse("type Holder={action:int;};function main(){}").unwrap(),
        )
        .unwrap();
        let annotation = crate::type_syntax::parse(if once {
            "Once<Function<int, [count:int]>>"
        } else {
            "Function<int, [count:int]>"
        })
        .unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        let checked = analyzer.run().unwrap();
        assert!(checked.stored_contracts.populated().next().is_none());
        let rust = crate::emit::emit(&checked);
        assert!(rust.contains("enum StoredCallable0 {\n}\n"));
        assert!(!rust.contains("struct StoredCallable0("));
        crate::callable_dispatch::tests::execute(&rust, "");
    }
}

#[cfg(test)]
#[test]
fn stored_callable_records_use_noncloneable_array_storage() {
    for once in [false, true] {
        let calls = if once {
            "const first=holders.remove(0);console.log(first.action());holders.push({action:():int=>{return 8;}});const second=holders.remove(0);console.log(second.action());"
        } else {
            "console.log(holders[0].action());holders.push({action:():int=>{return 8;}});console.log(holders[1].action());const first=holders.remove(0);console.log(first.action());"
        };
        let source = format!(
            "type Holder={{action:int;}};function main(){{const holders:Holder[]=[{{action:():int=>{{return 7;}}}}];{calls}}}"
        );
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse(if once {
            "Once<Function<int>>"
        } else {
            "Function<int>"
        })
        .unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        let checked = analyzer.run().unwrap();
        let rust = crate::emit::emit(&checked);
        assert!(rust.contains("type RecordArray0 = Vec<Record0>"));
        assert!(!rust.contains("impl Clone for Record0"));
        crate::callable_dispatch::tests::execute(&rust, if once { "7\n8\n" } else { "7\n8\n7\n" });
    }
}

#[cfg(test)]
#[test]
fn callable_record_arrays_reject_copying_operations_before_native_emission() {
    for operation in [
        "clone(holders)",
        "holders.slice()",
        "holders.toReversed()",
        "holders.concat([])",
        "holders.copyWithin(0,0)",
    ] {
        let source = format!(
            "type Holder={{action:int;}};function main(){{const holders:Holder[]=[{{action:():int=>{{return 7;}}}}];const copied={operation};}}"
        );
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse("Function<int>").unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        let error = analyzer.run().err().unwrap();
        assert_eq!(error.code, "clone-type", "{operation}: {error:?}");
    }
}

#[cfg(test)]
#[test]
fn definitive_compaction_keeps_stored_environment_dispatch_consistent() {
    for once in [false, true] {
        let source = "type Holder={action:int;};function main(){const holder:Holder={action:():int=>{return 7;}};console.log(holder.action());}";
        let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse(if once {
            "Once<Function<int>>"
        } else {
            "Function<int>"
        })
        .unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        analyzer
            .callables
            .results
            .push(crate::callable_resolution::ResultContract {
                structural: analyzer.callables.callbacks[declaration].structural,
                params: Vec::new(),
                result: Type::Int,
            });
        assert_eq!(analyzer.result_shape(0), Type::Closure(0));
        analyzer.assumed_modes = Some(vec![Vec::new()]);
        let checked = analyzer.run().unwrap();
        assert!(
            checked
                .closures
                .iter()
                .all(|closure| !closure.function.template)
        );
        let origins = checked.stored_contracts.origins(0).unwrap();
        assert_eq!(origins, &[usize::from(once)]);
        assert_eq!(
            checked.closures[origins[0]].invocation == Mode::Consume,
            once
        );
        let rust = crate::emit::emit(&checked);
        crate::callable_dispatch::tests::execute(&rust, "7\n");
    }
}

#[cfg(test)]
#[test]
fn discovery_and_effect_passes_do_not_publish_storage_origins() {
    for discovering in [false, true] {
        let source = "type Holder={action:int;};function main(){const holder:Holder={action:():int=>{return 7;}};}";
        let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let annotation = crate::type_syntax::parse("Function<int>").unwrap();
        let (ty, declaration) = analyzer
            .callables
            .resolver(&analyzer.record_names, &analyzer.unions)
            .storage_declaration(&annotation)
            .unwrap()
            .unwrap();
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(declaration);
        analyzer.discovering_results = discovering;
        analyzer.effects_only = !discovering;
        let StmtKind::Let { value, .. } = analyzer.program.functions[0].body[0].clone().kind else {
            panic!("holder binding");
        };
        let ExprKind::Record(mut fields) = value.kind else {
            panic!("record initializer");
        };
        let mut context = Context::new("main".into());
        context.effects_only = analyzer.effects_only;
        assert_eq!(
            analyzer
                .stored_callable_value(&mut context, &mut fields[0].1, 0, &Loans::new())
                .unwrap(),
            ty
        );
        assert!(!analyzer.closures.is_empty());
        assert_eq!(analyzer.callables.storage.origins(0), Some(&[][..]));
        assert_eq!(analyzer.callables.storage.declaration(0), Some(declaration));
    }
}

#[cfg(test)]
#[test]
fn shared_owned_callback_handle_clone_eligibility_does_not_enable_other_callable_clones() {
    let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
    let shared=CallbackContract{structural:analyzer.callables.shapes.intern([],Type::Int,Some(CallbackKind::SharedRead)),parameter_names:vec![],params:vec![],result:Type::Int,kind:CallbackKind::SharedRead};
    let owned=CallbackContract{structural:analyzer.callables.shapes.intern([],Type::Int,Some(CallbackKind::Owned)),parameter_names:vec![],params:vec![],result:Type::Int,kind:CallbackKind::Owned};
    let shared_type=analyzer.callables.storage.intern_shared(&analyzer.callables.shapes,&shared,0).unwrap();
    let owned_type=analyzer.callables.storage.intern(&analyzer.callables.shapes,&owned,1).unwrap();
    analyzer.callables.callbacks.extend([shared,owned]);
    analyzer.require_cloneable(shared_type,Span::default()).unwrap();
    assert_eq!(analyzer.require_cloneable(owned_type,Span::default()).unwrap_err().code,"clone-type");
    assert_eq!(analyzer.require_cloneable(Type::Callback(0),Span::default()).unwrap_err().code,"clone-type");
}

#[cfg(test)]
#[test]
fn shared_storage_checks_source_construction_and_clone_without_copying_captures() {
    let source = "type Holder={action:int;};function main(){const text=\"retained\";const holder:Holder={action:():int=>{return text.length;}};const other=clone(holder.action);console.log(holder.action());console.log(other());}";
    let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
    let callback = CallbackContract {
        structural: analyzer.callables.shapes.intern([], Type::Int, Some(CallbackKind::SharedRead)),
        parameter_names: vec![], params: vec![], result: Type::Int, kind: CallbackKind::SharedRead,
    };
    let ty = analyzer.callables.storage.intern_shared(&analyzer.callables.shapes, &callback, 0).unwrap();
    analyzer.callables.callbacks.push(callback);
    analyzer.fields[0].insert("action".into(), ty);
    analyzer.program.records[0].fields[0].ty = Some(ty);
    analyzer.program.records[0].fields[0].callable_declaration = Some(0);
    analyzer.assumed_modes = Some(vec![Vec::new()]);
    let checked = analyzer.run().unwrap();
    assert_eq!(checked.stored_contracts.origins(0), Some(&[0][..]));
    assert!(!checked.closures[0].borrowed);
    assert_eq!(checked.closures[0].invocation, Mode::Read);
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked), "8\n8\n");
}

#[cfg(test)]
#[test]
fn shared_source_storage_rejects_mutating_and_borrowed_environments() {
    for (body, code) in [
        ("const items=[1];const holder:Holder={action:():int=>{items.push(2);return items.length;}};", "callback-access"),
        ("const text=\"retained\";const action=():int=>{return text.length;};const holder:Holder={action:action};", "callback-escape"),
    ] {
        let source = format!("type Holder={{action:int;}};function main(){{{body}}}");
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let callback = CallbackContract {
            structural: analyzer.callables.shapes.intern([], Type::Int, Some(CallbackKind::SharedRead)),
            parameter_names: vec![], params: vec![], result: Type::Int, kind: CallbackKind::SharedRead,
        };
        let ty = analyzer.callables.storage.intern_shared(&analyzer.callables.shapes, &callback, 0).unwrap();
        analyzer.callables.callbacks.push(callback);
        analyzer.fields[0].insert("action".into(), ty);
        analyzer.program.records[0].fields[0].ty = Some(ty);
        analyzer.program.records[0].fields[0].callable_declaration = Some(0);
        analyzer.assumed_modes = Some(vec![Vec::new()]);
        let error = match analyzer.run() { Ok(_) => panic!("ineligible shared environment accepted"), Err(error) => error };
        assert_eq!(error.code, code, "{error:?}");
    }
}

#[cfg(test)]
#[test]
fn private_shared_callback_bridge_constructs_and_clones_owned_source_environment() {
    let source="function main(){const text=\"retained\";const shared=share(():int=>{return text.length;});const other=clone(shared);console.log(shared());console.log(other());}";
    let mut program=crate::syntax::parse(source).unwrap();
    let StmtKind::Let{value,..}=&mut program.functions[0].body[1].kind else {panic!("shared binding")};
    let ExprKind::Call(callee,..)=&mut value.kind else {panic!("sharing call")};
    callee.kind=ExprKind::Name("@native:std:testing-runtime#shareCallback".into());
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    assert_eq!(checked.callback_contracts[0].kind,CallbackKind::SharedRead);
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"8\n8\n");
}

#[cfg(test)]
#[test]
fn private_testing_numeric_extraction_observes_unions_without_coercing_or_consuming() {
    let source="function inspect(value:number|null|undefined):number{return numeric(value);}function main(){console.log(inspect(7.0));console.log(inspect(null));console.log(inspect(undefined));}";
    let mut program=crate::syntax::parse(source).unwrap();
    let StmtKind::Return(Some(value))=&mut program.functions[0].body[0].kind else {panic!("return")};
    let ExprKind::Call(callee,..)=&mut value.kind else {panic!("numeric call")};
    callee.kind=ExprKind::Name("@native:std:testing-runtime#numericValue".into());
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"7\nNaN\nNaN\n");
}

#[cfg(test)]
#[test]
fn private_testing_numeric_extraction_keeps_borrowed_data_and_does_not_coerce() {
    let source="type Row={value:int;};function mixed(value:string|number|null):number{return numeric(value);}function array(value:int[]):number{return numeric(value);}function record(value:Row):number{return numeric(value);}function boolean(value:boolean):number{return numeric(value);}function integer(value:int):number{return numeric(value);}function main(){console.log(mixed('42'));console.log(mixed(2.5));const values=[1];console.log(array(values));console.log(values.length);const row:Row={value:3};console.log(record(row));console.log(row.value);console.log(boolean(true));console.log(integer(7));}";
    let mut program=crate::syntax::parse(source).unwrap();
    for function in program.functions.iter_mut().take(5) {
        let StmtKind::Return(Some(value))=&mut function.body[0].kind else {panic!("return")};
        let ExprKind::Call(callee,..)=&mut value.kind else {panic!("numeric call")};
        callee.kind=ExprKind::Name("@native:std:testing-runtime#numericValue".into());
    }
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"NaN\n2.5\nNaN\n1\nNaN\n3\nNaN\n7\n");
}

#[cfg(test)]
#[test]
fn internal_containment_projection_materializes_generic_record_parameters() {
    let source="type Probe<T>={contains:Function<void,[expected:ReadOnly<ElementOf<T>>]>;};function make<T>(actual:T):Probe<T>{return {contains:(expected:int):void=>{console.log(expected);}};}function main(){const probe=make([1,2]);probe.contains(7);}";
    let mut program=crate::syntax::parse(source).unwrap();
    let crate::type_syntax::ValueAnnotationKind::Callable{parameters,..}=&mut program.records[0].fields[0].annotation.kind else {panic!("callback")};
    let crate::type_syntax::ValueAnnotationKind::Applied{name,..}=&mut parameters[0].annotation.kind else {panic!("projection")};
    name.text="@testing:element".into();
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"7\n");
}

#[cfg(test)]
#[test]
fn internal_containment_projection_preserves_borrowed_string_elements() {
    let source="type Probe<T>={contains:Function<void,[expected:ReadOnly<ElementOf<T>>]>;};function make<T>(actual:T):Probe<T>{return {contains:(expected:string):void=>{console.log(expected);}};}function main(){const probe=make(['first']);const expected='saved';probe.contains(expected);probe.contains(expected);console.log(expected);}";
    let mut program=crate::syntax::parse(source).unwrap();
    let crate::type_syntax::ValueAnnotationKind::Callable{parameters,..}=&mut program.records[0].fields[0].annotation.kind else {panic!("callback")};
    let crate::type_syntax::ValueAnnotationKind::Applied{name,..}=&mut parameters[0].annotation.kind else {panic!("projection")};name.text="@testing:element".into();
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"saved\nsaved\nsaved\n");
}

#[cfg(test)]
#[test]
fn private_containment_bridge_reuses_strict_search_without_consuming_values() {
    let source="function lookup(values:number[],expected:number):boolean{return contains(values,expected);}function text(value:string,expected:string):boolean{return contains(value,expected);}function main(){const values=[Number.NaN,-0.0,3.0];console.log(lookup(values,Number.NaN));console.log(lookup(values,0.0));console.log(lookup(values,4.0));console.log(values.length);const saved='a😀b';console.log(text(saved,'😀'));console.log(text(saved,''));console.log(saved);}";
    let mut program=crate::syntax::parse(source).unwrap();
    for function in program.functions.iter_mut().take(2) {
        let StmtKind::Return(Some(value))=&mut function.body[0].kind else {panic!("return")};
        let ExprKind::Call(callee,..)=&mut value.kind else {panic!("containment")};callee.kind=ExprKind::Name("@native:std:testing-runtime#containsValue".into());
    }
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"false\ntrue\nfalse\n3\ntrue\ntrue\na😀b\n");
}

#[cfg(test)]
#[test]
fn private_containment_searches_scalar_union_arrays() {
    let source="function lookup(values:(number|string|null)[],expected:number):boolean{return contains(values,expected);}function text(values:(number|string|null)[],expected:string):boolean{return contains(values,expected);}function main(){const values:(number|string|null)[]=[Number.NaN,-0.0,'saved',null];console.log(lookup(values,Number.NaN));console.log(lookup(values,0.0));console.log(lookup(values,4.0));const saved='saved';console.log(text(values,saved));console.log(text(values,'missing'));console.log(saved);console.log(values.length);}";
    let mut program=crate::syntax::parse(source).unwrap();
    for function in program.functions.iter_mut().take(2) {
        let StmtKind::Return(Some(value))=&mut function.body[0].kind else {panic!("return")};
        let ExprKind::Call(callee,..)=&mut value.kind else {panic!("containment")};callee.kind=ExprKind::Name("@native:std:testing-runtime#containsValue".into());
    }
    let checked=Analyzer::new(program).unwrap().run().unwrap();
    crate::callable_dispatch::tests::execute(&crate::emit::emit(&checked),"false\ntrue\nfalse\ntrue\nfalse\nsaved\n4\n");
}

#[cfg(test)]
mod catch_binding_tests {
    use super::*;
    #[test]
    fn catch_binding_shadows_outer_name_and_preserves_checked_local_slot() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".to_owned());
        let token=Token{text:"error".to_owned(),span:Span::new(0,5),kind:TokenKind::Name};
        let outer=Analyzer::bind(&mut context,&token,Type::Int,false).unwrap();
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){error.toUpperCase();}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let handler=region.catch.as_mut().unwrap();handler.error_type=Some(Type::String);
        assert!(!analyzer.check_catch_body(&mut context,handler,Type::Void).unwrap());
        let local=handler.binding.unwrap();
        assert_ne!(local,outer);assert_eq!(context.names["error"],outer);
        assert_eq!(context.bindings[local].ty,Type::String);
        assert_eq!(context.bindings[local].declaration_span,Some(handler.name.as_ref().unwrap().span));
        let StmtKind::Expr(value)=&handler.body[0].kind else {panic!("expression");};
        assert_eq!(value.target,Some(CallTarget::Upper));
        let declaration=handler.name.as_ref().unwrap().span;
        let mut checked=crate::check("function main(){}").unwrap();
        checked.program.functions[0].bindings=context.bindings;
        checked.program.functions[0].body=std::mem::take(&mut parsed.functions[0].body);
        assert!(checked.checked_type_facts().method_results.iter().any(|fact|fact.span==declaration && fact.ty=="string"));
        let StmtKind::Try(region)=&mut checked.program.functions[0].body[0].kind else {panic!("region");};
        region.catch.as_mut().unwrap().binding=None;
        assert!(!checked.checked_type_facts().method_results.iter().any(|fact|fact.span==declaration));
    }
    #[test]
    fn catch_assignment_keeps_its_type_and_restores_shadow_on_error() {
        for (source,valid) in [
            ("function main(){try{}catch(error){error=\"next\";error.toUpperCase();}}",true),
            ("function main(){try{}catch(error){error=4;}}",false),
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            let token=Token{text:"error".to_owned(),span:Span::new(0,5),kind:TokenKind::Name};
            let outer=Analyzer::bind(&mut context,&token,Type::Int,false).unwrap();
            let mut parsed=crate::syntax::parse(source).unwrap();
            let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
            let handler=region.catch.as_mut().unwrap();handler.error_type=Some(Type::String);
            let result=analyzer.check_catch_body(&mut context,handler,Type::Void);
            assert_eq!(result.is_ok(),valid,"{source}: {result:?}");
            assert_eq!(context.names["error"],outer);
            if !valid {assert_eq!(result.unwrap_err().code,"type");}
        }
    }
}

#[cfg(test)]
mod throw_value_tests {
    use super::*;
    #[test]
    fn throwing_an_owner_transfers_it_while_scalar_payloads_copy() {
        for ty in [Type::String,Type::Int] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
            let binding=Analyzer::bind(&mut context,&token,ty,false).unwrap();
            let mut parsed=crate::syntax::parse("function main(){throw payload;}").unwrap();
            let StmtKind::Throw(value)=&mut parsed.functions[0].body[0].kind else {panic!("throw");};
            assert_eq!(analyzer.check_throw_value(&mut context,value).unwrap(),ty);
            assert_eq!(context.bindings[binding].transferred.is_some(),!ty.is_copy());
            assert_eq!(value.binding,Some(binding));assert_eq!(value.ty,Some(ty));
        }
    }
    #[test]
    fn throwing_a_borrow_requires_an_explicit_clone() {
        for (source,valid) in [("function main(){throw payload;}",false),("function main(){throw clone(payload);}",true)] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            let token=Token{text:"payload".to_owned(),span:Span::new(0,7),kind:TokenKind::Name};
            let binding=Analyzer::bind(&mut context,&token,Type::String,false).unwrap();
            context.bindings[binding].borrowed_place=Some(token.span);
            let mut parsed=crate::syntax::parse(source).unwrap();
            let StmtKind::Throw(value)=&mut parsed.functions[0].body[0].kind else {panic!("throw");};
            let result=analyzer.check_throw_value(&mut context,value);
            if valid {assert_eq!(result.unwrap(),Type::String);}else{assert_eq!(result.unwrap_err().code,"borrowed-place");}
            assert!(context.bindings[binding].transferred.is_none());
        }
    }
    #[test]
    fn throw_undefined_is_a_value_but_void_expression_is_not() {
        for (source,valid) in [("function main(){throw undefined;}",true),("function main(){throw console.log(1);}",false)] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("main".to_owned());
            let mut parsed=crate::syntax::parse(source).unwrap();
            let StmtKind::Throw(value)=&mut parsed.functions[0].body[0].kind else {panic!("throw");};
            let result=analyzer.check_throw_value(&mut context,value);
            if valid {assert_eq!(result.unwrap(),Type::Undefined);}else{assert_eq!(result.unwrap_err().code,"type");}
        }
    }
}

#[cfg(test)]
mod exception_callable_boundary_tests {
    use super::*;
    #[test]
    fn callable_checker_materializes_runtime_error_for_named_handler() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("value".into());
        let mut parsed=crate::syntax::parse("function value():int{const values=[1];try{const item=values[2]!;}catch(error){return error.code.length;}return 0;}").unwrap();
        assert!(analyzer.check_exception_callable_body(&mut context,&mut parsed.functions[0].body,Type::Int).unwrap());
        let region=parsed.functions[0].body.iter().find_map(|stmt|if let StmtKind::Try(region)=&stmt.kind{Some(region)}else{None}).unwrap();
        let handler=region.catch.as_ref().unwrap();
        let Some(Type::Record(record))=handler.error_type else {panic!("runtime error type: {:?}",handler.error_type);};
        assert_eq!(analyzer.program.records[record].name.text,"#RuntimeError");
        assert!(handler.error_contract.as_ref().unwrap().runtime);
        assert_eq!(context.bindings[handler.binding.unwrap()].ty,Type::Record(record));
    }
    #[test]
    fn callable_checker_consumes_local_loop_and_block_completions() {
        for source in [
            "function value():int{let total=0;outer:for(let i=0;i<3;i++){try{continue outer;}finally{total+=1;}}return total;}",
            "function value():int{let total=0;outer:while(true){try{break outer;}finally{total=4;}}return total;}",
            "function value():int{let total=0;done:{try{break done;}finally{total=5;}}return total;}",
        ] {
            let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
            let mut context=Context::new("value".into());
            let mut parsed=crate::syntax::parse(source).unwrap();
            assert!(analyzer.check_exception_callable_body(&mut context,&mut parsed.functions[0].body,Type::Int).unwrap(),"{source}");
            assert!(context.loops.is_empty());assert!(context.completion_frame.is_none());
        }
    }
    #[test]
    fn callable_checker_restores_scopes_after_diagnostic() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("value".into());
        let mut parsed=crate::syntax::parse("function value():int{try{throw missing;}finally{return 6;}}").unwrap();
        assert!(analyzer.check_exception_callable_body(&mut context,&mut parsed.functions[0].body,Type::Int).is_err());
        assert!(context.exception_environment.is_none());assert!(context.completion_frame.is_none());assert!(context.error_frame.is_none());assert!(context.region_loop_depth.is_none());assert!(context.region_entry_loans.is_none());
    }
    #[test]
    fn callable_checker_collects_nested_finally_and_restores_context() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("value".into());
        let mut parsed=crate::syntax::parse("function value():int{try{try{throw 4;}catch(error){return error;}}finally{return 6;}}").unwrap();
        let returned=analyzer.check_exception_callable_body(&mut context,&mut parsed.functions[0].body,Type::Int).unwrap();
        assert!(returned);
        assert!(context.exception_environment.is_none());assert!(context.completion_frame.is_none());assert!(context.error_frame.is_none());assert!(context.region_loop_depth.is_none());
        let StmtKind::Try(region)=&parsed.functions[0].body[0].kind else {panic!("region");};
        let StmtKind::Try(inner)=&region.body[0].kind else {panic!("inner");};
        assert_eq!(inner.catch.as_ref().unwrap().error_type,Some(Type::Int));
    }
}

#[path = "sets.rs"]
mod sets;
#[path = "maps.rs"]
mod maps;

impl Analyzer {
    #[inline(never)]
    fn array_from_mapping(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let ExprKind::Call(callee,args)=&mut expression.kind else{unreachable!()};
        let ExprKind::Field(base,_)=&mut callee.kind else{unreachable!()};
        let result=self.array_transform(context,base,args,loans,expression.span,(false,false,Mode::Read,false,false,true,true)).map_err(|mut error|{if error.code=="arity"{error.message=format!("Array.from: {}",error.message);}error})?;
        self.require(result,expected,expression.span)?;expression.target=Some(CallTarget::ArrayMap);Ok(result)
    }
}

impl Analyzer {
    #[inline(never)]
    fn sequence_expression(&mut self,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>,discarded:bool)->Result<Type,Diagnostic>{
        let ExprKind::Sequence(values)=&mut expression.kind else{unreachable!()};
        let (last,prefixes)=values.split_last_mut().expect("nonempty sequence");
        for prefix in prefixes{
            let captures=context.capture_loans.len();
            let projection=context.projection.take();
            let checked=self.discarded_comma_operand(context,prefix,loans);
            context.projection=projection;
            context.capture_loans.truncate(captures);
            checked?;
        }
        let ty=if discarded {
            let captures=context.capture_loans.len();
            let checked=self.discarded_comma_operand(context,last,loans);
            context.capture_loans.truncate(captures);
            checked?;Type::Void
        }else{self.expression(context,last,mode,loans,expected)?};
        expression.ty=Some(ty);expression.binding=if discarded{None}else{last.binding};expression.callable_origin=if discarded{None}else{last.callable_origin};
        let outcomes=Self::condition_outcomes(context,last);
        context.condition_flows.insert((expression.span.start,expression.span.end),outcomes);
        Ok(ty)
    }
}

impl Analyzer {
    #[inline(never)]
    fn discarded_comma_operand(&mut self,context:&mut Context,operand:&mut Expr,loans:&Loans)->Result<Type,Diagnostic>{
        if let ExprKind::Sequence(_)=&operand.kind{return self.sequence_expression(context,operand,Mode::Read,loans,None,true);}
        if let ExprKind::Assign{target,value,..}=&mut operand.kind {
            if self.expression_type_hint(context,target).is_some_and(|ty|!ty.is_copy()){
                self.assign_statement_with_loans(context,target,value,None,loans)?;
                operand.ty=Some(Type::Void);return Ok(Type::Void);
            }
        }
        if let ExprKind::CompoundAssign{target,value,op,operator}=&mut operand.kind {
            if self.expression_type_hint(context,target)==Some(Type::String){
                self.compound_assignment_value(context,target,value,(*op,*operator),loans,true)?;
                operand.ty=Some(Type::Void);return Ok(Type::Void);
            }
        }
        self.expression(context,operand,Mode::Read,loans,None)
    }
}
