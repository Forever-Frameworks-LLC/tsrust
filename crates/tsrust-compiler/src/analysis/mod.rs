mod escape_work;
mod continuations;
use continuations::CallDispatch;
mod value_type_resolution;
mod function_scheduling;
mod function_parameters;
mod result_discovery;
mod discovery_work;
mod schema;
mod thread_transfer;
mod threads;
mod channels;
mod results;
mod json_typed;
mod record_union_fields;
mod json_refinement;
mod native_view_mutations;
mod native_view_expression_calls;
mod native_view_family_provenance;
#[cfg(test)]
mod native_view_callee_transport_tests;
mod native_view_scopes;
mod native_view_calls;
mod native_view_environments;
mod native_view_storage_epochs;
mod native_view_storage_contracts;
mod native_view_parameters;
mod native_view_returns;
mod native_view_places;
mod native_view_projections;
mod native_view_recursive;
pub(crate) mod native_view_loans;
mod native_capabilities;
mod retained_capture_projection;
mod result_construction;
mod result_matching;
mod installer_value_uses;
mod captured_record_borrows;
mod record_callable_identity;
mod map_cleanup_phase_plan;
mod map_entry_moves;
mod map_block_storage_plan;
mod capture_loan_binding;
mod array_capture_owners;
mod array_storage_epochs;
mod array_result_storage;
mod nullable_record_storage;
mod nullable_record_fields;
mod discarded_sequence;
mod map_entry_capture_owners;
mod map_entry_epochs;
mod map_epoch_work;
mod map_iteration_plan;
mod map_iteration_shape;
mod map_exception_storage_types;
mod map_completion_payloads;
mod map_completion_layouts;
mod map_entry_branch_transport;
mod map_entry_demand;
mod map_entry_completion_release;
mod loop_body_completion;
mod assignment_callee;
mod owned_array_patterns;
mod inferred_callable_arrays;
mod return_type_inference;
mod named_capture_layout;
mod named_capture_access;
mod checked_callable_storage;
mod conditional_callable_carriers;
mod borrowed_conditional_carriers;
mod record_call_capture_release;
mod record_parameter_carriers;
mod direct_callback_stores;
mod caller_error_origins;
pub(crate) mod record_parameter_borrows;
mod nullable_parameter_stores;
mod nullable_parameter_environments;
mod scoped_record_containers;
mod empty_scoped_maps;
mod scoped_map_carriers;
mod map_storage_edges;
mod storage_exit_availability;
pub(crate) use storage_exit_availability::MapEntryEpoch;
pub(crate) use map_completion_payloads::Phase as MapCompletionPhase;
mod record_array_carriers;
mod record_value_carriers;
mod carrier_type_join;
mod carrier_retarget;
mod carrier_literal_join;
mod record_storage_writes;
mod record_branch_carriers;
mod named_capture_context;
mod capture_context;
mod callable_expression;
mod scoped_record_construction;
mod exception_state;
mod labeled_control;
mod object_spread;
mod destructuring_rest;
mod object_keys;
mod object_values;
mod array_entries;
mod logical_values;
pub(crate) mod generic_function_values;
mod closure_methods;
mod capture_restoration;
mod generic_value_adapters;
mod checked_value_annotations;
mod generic_expression_adapters;
mod generic_value_metadata;
mod deferred_generic_values;
mod deferred_generic_call_work;
mod deferred_generic_preparation;
mod deferred_generic_methods;
mod generic_calls;
mod generic_call_work;
mod polymorphic_argument_hints;
mod optional_chain;
mod rest_calls;
mod spread_calls;
mod spread_call_work;
mod array_spread_work;
mod collection_constructors;
mod builtin_receivers;
mod program_testing;
pub(crate) use spread_calls::spread_position_range;
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
    /// Scoped storage borrow; mutable_place distinguishes exclusive match access.
    pub borrowed_place: Option<Span>,
    /// Exclusive scoped match alias; ownership remains with its source.
    pub mutable_place: bool,
    /// Receiver roots kept alive by a scoped optional-element view.
    pub view_roots: Vec<usize>,
    /// Retained native slice loans; a moved view transfers these with its value.
    pub(crate) native_view_loans: Option<Box<Vec<native_view_loans::Loan>>>,
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
    pub(crate) storage_exits: storage_exit_availability::Exits,
    pub(crate) native_returns: Option<Box<Vec<native_view_loans::Loan>>>,
    pub(crate) native_mutations: Option<Box<Vec<(usize, native_view_loans::Loan)>>>,
    pub template: bool,
    pub specialization: bool,
    pub name: String,
    pub bindings: Vec<Binding>,
    pub parameter_count: usize,
    pub result: Type,
    pub body: Vec<Stmt>,
}

pub struct Capture {
    /// Checked completion snapshots prove this moved capture is restored.
    pub restored: bool,
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
    /// Structural callback layout per parameter; origin IDs remain independent.
    pub(crate) parameter_callbacks:Vec<Option<crate::callable_contracts::ContractId>>,
    pub function: CheckedFunction,
}

#[derive(Clone)]
struct ClosureMethodSource {
    context: Context,
    parameters: Vec<usize>,
    result: Type,
    body: Vec<Stmt>,
}

pub struct ProgramCapture {
    pub declaration: Span,
    pub inner: usize,
    pub access: Mode,
    pub span: Span,
}

pub struct CheckedClosure {
    pub program_scoped: bool,
    /// Borrowed only during invocation through the private registry contract.
    pub program_captures: Vec<ProgramCapture>,
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
    pub(crate) program_callback_sites: BTreeMap<usize, CallbackKind>,
    pub(crate) module_owner_bindings: Vec<Span>,
    pub(crate) schema_choices: crate::stdlib::schema::plans::Choices,
    pub(crate) schema_pipelines: crate::stdlib::schema::pipelines::Pipelines,
    pub(crate) schema_factories: HashMap<Type,usize>,
    pub(crate) record_parameter_borrows: HashMap<usize,Vec<record_parameter_borrows::Borrow>>,
    pub(crate) closure_record_parameter_borrows: HashMap<usize,Vec<record_parameter_borrows::Borrow>>,
    pub(crate) closure_record_capture_borrows: HashMap<usize,Vec<record_parameter_borrows::CapturedBorrow>>,
    pub(crate) polymorphic_storage: crate::polymorphic_callable_storage::Registry,
    pub(crate) polymorphic_invocations: crate::polymorphic_invocation_plans::Registry,
    pub(crate) record_call_releases: HashMap<usize,Vec<(usize,Vec<String>)>>,
    pub(crate) closure_record_call_releases: HashMap<usize,Vec<(usize,Vec<String>)>>,
    pub(crate) record_parameter_replacements: HashMap<usize,Vec<(usize,Vec<String>,Type)>>,
    pub(crate) scoped_records: crate::scoped_record_layout::Registry,
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
    entry_loans: Vec<crate::capture_loan_identity::Loan>,
    before_owners: crate::capture_loan_owners::Owners,
    after_left_owners: crate::capture_loan_owners::Owners,
    left_context: Option<Type>,
    right_context: Option<Type>,
    after_left: Vec<Availability>,
    after_left_flow: crate::flow::Facts,
    left_loans: Vec<(usize, Mode, Span)>,
}

#[derive(Clone, Default)]
struct Availability {
    native_view_loans: Option<Box<Vec<native_view_loans::Loan>>>,
    transferred: Option<Span>,
    fields: BTreeMap<Vec<String>, Span>,
}
impl Availability {
    fn of(binding: &Binding) -> Self {
        Self {
            native_view_loans: binding.native_view_loans.clone(),
            transferred: binding.transferred,
            fields: binding.moved_fields.clone(),
        }
    }
    fn restore(self, binding: &mut Binding) {
        binding.native_view_loans = self.native_view_loans;
        binding.transferred = self.transferred;
        binding.moved_fields = self.fields;
    }
    fn join(mut self, other: Self) -> Self {
        self.native_view_loans = native_view_loans::join(self.native_view_loans, other.native_view_loans);
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
    break_owners: Option<crate::capture_loan_owners::Owners>,
    continue_owners: Option<crate::capture_loan_owners::Owners>,
    /// First binding declared inside the target. An exit leaves those
    /// bindings, so the loans they hold do not reach the code after it.
    scope_start: usize,
    // New loops execute a tail after continue, so ownership checks happen
    // after joining their fallthrough and continue states, not at continue.
    defer_continue: bool,
    continues: Option<Vec<Availability>>,
    span: Span,
}

type CallbackMetadata = (Option<Mode>, bool, Option<CallbackKind>);

#[derive(Clone)]
struct Context {
    block_result_type: Type,
    match_values: Vec<(usize, Option<Type>)>,
    match_loans: Vec<Loans>,
    native_recursive_shapes: Option<std::rc::Rc<native_view_recursive::Shapes>>,
    /// Locals stored inside a branch, loop or exception region, or replaced as
    /// a whole record. Their nullable callback records keep one carrier type
    /// and their capture loans until the scope ends.
    control_stored: HashSet<usize>,
    native_view_calls: bool,
    /// Lexical depth at which a local holding native views was declared.
    native_view_declaration_depths: HashMap<usize, usize>,
    native_view_environments: Option<Box<HashMap<usize,Vec<(usize,usize)>>>>,
    native_shadowed_bindings: Option<Box<Vec<usize>>>,
    native_view_results: Option<Box<HashMap<UnionId,(usize,Option<usize>)>>>,
    native_view_function_returns: Option<Box<HashMap<native_view_calls::Target,Vec<native_view_calls::Returned>>>>,
    storage_exits: storage_exit_availability::Exits,
    native_returns: Option<Box<Vec<native_view_loans::Loan>>>,
    native_mutations: Option<Box<Vec<(usize, native_view_loans::Loan)>>>,
    capture_restoration:Option<capture_restoration::Tracker>,
    region_loop_depth:Option<usize>,
    region_entry_loans:Option<std::rc::Rc<crate::capture_loan_identity::Boundary>>,
    exception_environment:Option<std::rc::Rc<exception_state::CheckEnvironment>>,
    caller_error_origins: Vec<(usize, Type)>,
    completion_frame:Option<Box<exception_state::CompletionFrame>>,
    error_frame:Option<Box<exception_state::ErrorFrame>>,
    selected_access: HashMap<usize, Mode>,
    flow: crate::flow::Facts,
    constant_absence: HashMap<usize,Type>,
    keep_flow: Vec<crate::flow::Place>,
    /// Singleton recursive payloads are guaranteed by type, not a flow guard.
    unconditional_projections: HashSet<(UnionId, Type)>,
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
    capture_loan_owners: crate::capture_loan_owners::Owners,
    capture_loan_ids: crate::capture_loan_identity::Registry,
    capture_loans: Vec<(usize, Mode, Span)>,
    iteration_loans: Vec<(usize, Span)>,
    capture_boundary: usize,
    named_capture_range: (usize, usize),
    closure_depth: usize,
    inferred_body_result: bool,
    returning_closure: bool,
    returned_callable: Option<Type>,
    /// The recorded return is a call in a template body; see `return_statement`.
    returned_symbolic_call: bool,
    function_name: String,
    /// Parameters declared ReadOnly. A const taken from one of their
    /// positions borrows it for reading.
    readonly_parameters: HashSet<usize>,
    mutable_parameters: HashSet<usize>,
    mutable_match_borrows: Vec<usize>,
}
impl Context {
    #[inline]
    fn is_capture(&self, slot: usize) -> bool {
        slot < self.capture_boundary || (slot >= self.named_capture_range.0 && slot < self.named_capture_range.1)
    }

    fn new(function_name: String) -> Self {
        Self {
            block_result_type: Type::Void,
            match_values: Vec::new(),
            match_loans: Vec::new(),
            control_stored: HashSet::new(),
            native_recursive_shapes: None,
            native_view_calls: false,
            native_view_declaration_depths: HashMap::new(),
            native_view_environments: None,
            native_shadowed_bindings: None,
            native_view_results: None,
            native_view_function_returns: None,
            storage_exits: Default::default(), native_returns: None, native_mutations: None,
            capture_restoration:None,
            region_loop_depth:None,
            region_entry_loans:None,
            exception_environment:None,
            caller_error_origins: Vec::new(),
            completion_frame:None,
            error_frame:None,
            selected_access: HashMap::new(),
            flow: HashMap::new(),
            constant_absence: HashMap::new(),
            keep_flow: Vec::new(),
            unconditional_projections: HashSet::new(),
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
            capture_loan_owners: Default::default(),
            capture_loan_ids: Default::default(),
            capture_loans: Vec::new(),
            iteration_loans: Vec::new(),
            capture_boundary: 0,
            named_capture_range: (0, 0),
            closure_depth: 0,
            inferred_body_result: false,
            returning_closure: false,
            returned_callable: None,
            returned_symbolic_call: false,
            function_name,
            readonly_parameters: HashSet::new(),
            mutable_parameters: HashSet::new(),
            mutable_match_borrows: Vec::new(),
        }
    }
}
use crate::argument_access_loans::Loans;

/// Inspect statement bodies without adding recursive analyzer frames.
pub(crate) fn has_exception_syntax(body:&[Stmt])->bool {
    let mut pending=body.iter().collect::<Vec<_>>();
    for expression in crate::error_effects::body_expressions(body) { if let ExprKind::Match { body, .. } = &expression.kind { pending.extend(body); } }
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

pub fn analyze(mut program: Program) -> Result<CheckedProgram, Diagnostic> {
    if crate::stdlib::testing::program::registry_reachable(&mut program)? {
        return analyze_recursive(program);
    }
    finish_analysis(Analyzer::new(program)?.run()?)
}

// Incomplete capture layouts stay private, even after typed flow checking.
fn finish_analysis(mut checked: CheckedProgram) -> Result<CheckedProgram, Diagnostic> {
    crate::callables::module_state::validate(&checked)?;
    native_view_calls::validate_closures(&checked.closures)?;
    checked.polymorphic_storage.validate(&checked.closures, Span::default())?;
    checked.polymorphic_invocations.validate(&checked.closures, Span::default())?;
    definitely_assigned(&mut checked)?;
    threads::validate(&checked)?;
    for (family, declaration, _) in checked.stored_contracts.populated() {
        if let Some(layout) = checked.stored_contracts.scoped_native_family(family) {
            let span = layout.alternatives.first().map(|alternative| checked.closures[alternative.origin].span).unwrap_or_default();
            crate::native_view_callable_storage::validate_abi(layout, &checked.callback_contracts[declaration], &checked.closures, &checked.error_effects, span)?;
        }
    }
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
    let mut program_access: BTreeMap<usize, Mode> = BTreeMap::new();
    let mut program_callback_sites = BTreeMap::new();
    // Every non-copy coordinate can rise from read to mutate to consume only.
    let mut remaining = parameter_count.saturating_mul(2) + 1;
    while remaining > 0 {
        remaining -= 1;
        let mut analyzer = Analyzer::new(program.clone())?;
        analyzer.assumed_modes = Some(modes.clone());
        analyzer.assumed_capture_access = capture_access.clone();
        analyzer.program_access = program_access.clone();
        analyzer.program_callback_sites = program_callback_sites.clone();
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
        for capture in inferred.closures.iter().flat_map(|closure| &closure.program_captures) {
            let prior = program_access.get(&capture.declaration.start).copied();
            let next = prior.map_or(capture.access, |mode| mode.max(capture.access));
            if prior != Some(next) { changed = true; if prior.is_none() { discovered += 1; } }
            program_access.insert(capture.declaration.start, next);
        }
        for (&site, &kind) in &inferred.program_callback_sites {
            if program_callback_sites.get(&site) != Some(&kind) {
                program_callback_sites.insert(site, kind);
                changed = true;
                discovered += 1;
            }
        }
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
            analyzer.program_access = program_access;
            analyzer.program_callback_sites = program_callback_sites;
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
    program_access: BTreeMap<usize, Mode>,
    program_callback_sites: BTreeMap<usize, CallbackKind>,
    schema_choices: crate::stdlib::schema::plans::Choices,
    schema_pipelines: crate::stdlib::schema::pipelines::Pipelines,
    schema_factories: HashMap<Type,usize>,
    native_borrow_types: bool,
    threads_enabled: bool,
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
    active_recursive_functions: HashSet<usize>,
    scheduled_functions: HashSet<usize>,
    assumed_modes: Option<Vec<Vec<Mode>>>,
    effects_only: bool,
    /// The whole analyzer is a discarded access-inference pass. Stored
    /// callable origins are noted for error-effect lookup without layout
    /// validation. A temporary effects-only scope in a definitive run never
    /// sets this.
    speculative_access_run: bool,
    builtin_literals: bool,
    /// Receivers of builtin chain links checked on the shared stack, by address.
    prechecked_receivers: Vec<(usize, Result<Type, Diagnostic>)>,
    /// Schema validations checked for their internal Result inside parse/safeParse adapters.
    raw_schema_results: HashSet<Span>,
    module_literal_scope: Option<Span>,
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
    polymorphic_storage: crate::polymorphic_callable_storage::Registry,
    polymorphic_invocations: crate::polymorphic_invocation_plans::Registry,
    closure_method_sources: HashMap<usize, ClosureMethodSource>,
    closure_specializations: HashMap<(usize, Vec<Type>, bool, bool), usize>,
    callables: crate::callable_resolution::Tables,
    absent_callback_payloads: HashMap<usize, usize>,
    record_cloneability: Option<(usize, crate::cloneability::Cloneability)>,
    record_parameter_replacements: HashMap<usize,Vec<(usize,Vec<String>,Type)>>,
    scoped_records: crate::scoped_record_layout::Registry,
    escaping: HashSet<usize>,
    discovering_results: bool,
    discovering_function: Option<usize>,
    result_dependents: HashMap<usize, HashSet<usize>>,
    known_results: HashMap<usize, Type>,
    /// Result types of recursive functions without a result annotation,
    /// taken from a return that does not depend on the recursive call.
    provisional_results: HashMap<usize, Type>,
    /// Handlers that no failure reached when their region was last checked,
    /// with the state at the region. Their bodies are checked once analysis is
    /// definitive; a later check that reaches the handler removes the entry.
    dead_handlers: BTreeMap<usize, Box<(Context, CatchClause, Type)>>,
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
    program_scoped: bool,
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
    value_type_resolution::resolve(annotation, allow_void, record_names, unions)
}


pub(crate) use crate::record_layout::append_checked_record_fields;

#[derive(Clone, Copy)]
enum CallbackOrigin {
    Parameter(usize),
    Closure(usize),
    Family(usize),
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
    invocations: &crate::polymorphic_invocation_plans::Registry,
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
                ExprKind::Match { body, .. } => nodes.extend(body.iter().map(Node::Statement)),
                ExprKind::SpreadCall(plan) => nodes.extend(plan.expressions().map(Node::Expression)),
                ExprKind::SpreadValue(_) => (),
                ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => nodes.push(Node::Expression(value)),
                ExprKind::Optional { base, body, .. } => {
                    nodes.push(Node::Expression(base));
                    nodes.push(Node::Expression(body));
                }
                ExprKind::Selected { .. } => (),
                ExprKind::Call(callee, args) => {
                    let selected = if matches!(expr.target, Some(CallTarget::ClosureMethod(_))) {
                        callee.generic_invocation.map(|id| invocations.checked(id, closures, callee.span)).transpose()?
                    } else { None };
                    let target = if selected.is_some() { None } else { match expr.target {
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
                    }};
                    let targets = target.into_iter().chain(selected.iter().flat_map(|plan| plan.cases.iter().map(|case| (
                        NativeCallbackTarget::Method(case.pair.origin, case.pair.method),
                        &case.body.function, case.body.parameters.clone(),
                    ))));
                    for (target, checked, parameter_ids) in targets {
                        let mut arguments = Vec::new();
                        for (&parameter, arg) in parameter_ids.iter().zip(args) {
                            if matches!(checked.bindings[parameter].ty, Type::Callback(_)) {
                                let origin = match arg.ty.expect("checked callback argument") {
                                    Type::Closure(id) => CallbackOrigin::Closure(id),
                                    Type::StoredCallable(id) | Type::BorrowedCallable(id) => CallbackOrigin::Family(id),
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
                | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _)
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
    invocations: &crate::polymorphic_invocation_plans::Registry,
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
            callback_calls(function, functions, closures, &parameters, invocations)?,
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
                invocations,
            )?);
        }
        for (method, checked) in closure.methods.iter().enumerate() {
            if checked.function.template {
                continue;
            }
            let node = NativeCallbackTarget::Method(origin, method);
            calls.insert(
                node,
                callback_calls(&checked.function, functions, closures, &checked.parameters, invocations)?,
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
                    // A family is one native environment type, regardless of
                    // its selected origin. Keep it distinct from every closure.
                    CallbackOrigin::Family(id) => closures.len() + id,
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
    crate::source_walk::body(&mut function.body,&mut |annotation|polymorphic_argument_hints::remap_annotation(annotation,ids),&mut |_|Ok(()))?;
    remap_closure_type(&mut function.result, ids)?;
    for loan in function.native_returns.as_deref_mut().into_iter().flatten() {crate::native_view_origin_remap::loan(loan,ids)?;}
    for (_,loan) in function.native_mutations.as_deref_mut().into_iter().flatten() {crate::native_view_origin_remap::loan(loan,ids)?;}
    for binding in &mut function.bindings {
        for loan in binding.native_view_loans.as_deref_mut().into_iter().flatten() {crate::native_view_origin_remap::loan(loan,ids)?;}
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
                // Constructors require their concrete payload identity. A call
                // with an unretained origin stays unresolved instead of using
                // another environment's error contract.
                if let Some(CallableOrigin::Closure(origin)) = expr.callable_origin {
                    if matches!(expr.kind, ExprKind::Closure { .. }) && ids.get(origin).copied().flatten().is_none() {
                        return Err(Diagnostic::new("callable-layout", "Constructed generic environment lost its origin during compaction.", expr.span));
                    }
                    expr.callable_origin = ids.get(origin).copied().flatten().map(CallableOrigin::Closure);
                }
                match &mut expr.kind {
                    ExprKind::Match { body, .. } => nodes.extend(body.iter_mut().map(Node::Statement)),
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
                    | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _)
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
        // Never silently drop preserved initializers while module ownership
        // lowering is under construction.
        if let Some(statement) = program.module_bindings.first() {
            return Err(Diagnostic::new("module-state", "Module storage requires program ownership lowering.", statement.span));
        }
        crate::stdlib::threads::prepare(&mut program)?;
        crate::stdlib::json::namespace::prepare(&mut program)?;
        crate::stdlib::schema::namespace::prepare(&mut program)?;
        if program.records.iter().any(|record| matches!(record.native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) {
            crate::native_type_resolution::intern(&mut program.records,crate::native_signatures::NativeTypeId::StringView,Span::default())?;
        }
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
        let mut callables = crate::callable_resolution::Tables::default();
        let mut generic_records = crate::generic_records::specialize_in(&mut program, &unions, &mut callables)?;
        let mut record_names = HashMap::new();
        let mut function_names = HashMap::new();
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
                &mut callables,
            )?;
            callables.resolver(&record_names, &unions).resolve_value_annotation(argument, false)?;
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
        let threads_enabled = program.records.iter().any(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::Thread));
        let native_borrow_types = program.records.iter().any(|record| record.native_type.is_some_and(|id| crate::native_types::declaration(id).borrow_access().is_some()));
        let mut analyzer = Self {
            schema_choices: Default::default(),
            schema_pipelines: Default::default(),
            schema_factories: HashMap::new(),
            native_borrow_types,
            threads_enabled,
            named_captures,
            program_access: BTreeMap::new(),
            program_callback_sites: BTreeMap::new(),
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
            active_recursive_functions: HashSet::new(),
            scheduled_functions: HashSet::new(),
            assumed_modes: None,
            effects_only: false,
            speculative_access_run: false,
            builtin_literals: false,
            prechecked_receivers: Vec::new(),
            raw_schema_results: HashSet::new(),
            module_literal_scope: None,
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
            polymorphic_storage: Default::default(),
            polymorphic_invocations: Default::default(),
            closure_method_sources: HashMap::new(),
            closure_specializations: HashMap::new(),
            callables,
            absent_callback_payloads: HashMap::new(),
            record_cloneability: None,
            record_parameter_replacements: HashMap::new(),
            scoped_records: Default::default(),
            escaping,
            discovering_results: false,
            discovering_function: None,
            result_dependents: HashMap::new(),
            known_results: HashMap::new(),
            provisional_results: HashMap::new(),
            dead_handlers: BTreeMap::new(),
            result_shapes: HashMap::new(),
            result_waits: HashMap::new(),
            pending_shapes: HashSet::new(),
            validated_closures: HashSet::new(),
        };
        let once_demands = if analyzer.program.functions.iter().any(|function|function.params.iter().any(|(_,ty,callback)|{
            callback.as_ref().cloned().or_else(||crate::callable_annotations::callback(ty)).is_some_and(|callback|callback.kind==CallbackKind::OwnedOnce)
        })) {
            Some(crate::generic_call_validation::once_parameter_demands(&analyzer.program)?)
        } else {None};
        for index in 0..count {
            if let Some(demands)=&once_demands {
                for (position,(_,ty,callback)) in analyzer.program.functions[index].params.iter_mut().enumerate() {
                    let annotation=callback.clone().or_else(||crate::callable_annotations::callback(ty));
                    if let Some(mut annotation)=annotation
                        && annotation.kind==CallbackKind::OwnedOnce
                        && demands[index][position]==crate::callable_environment_demand::Demand::Scoped {
                        annotation.kind=CallbackKind::Once;*callback=Some(annotation);
                    }
                }
            }
            let source_params = analyzer.program.functions[index].params.clone();
            let mut params = Vec::new();
            for (_, ty, callback) in &source_params {
                let resolved = if let Some(annotation) = callback {
                    analyzer.resolve_callback(annotation)?
                } else {
                    analyzer.callable_resolver().resolve_value_annotation(ty, false)?
                };
                params.push(resolved);
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
                            analyzer.callables.resolver(&analyzer.record_names, &analyzer.unions).resolve_value_annotation(argument, false)?,
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
            // Generic source constraints use nominal declarations. Bind each
            // nullable environment only after those constraints are resolved.
            for (parameter, (_, annotation, _)) in params.iter_mut().zip(&source_params) {
                *parameter = analyzer.nullable_parameter_environment(*parameter, annotation.span)?;
            }
            let body_params = params.clone();
            for (ty, initializer) in params.iter_mut().zip(&analyzer.program.functions[index].parameter_initializers) {
                if initializer.omission().is_omittable() { *ty = analyzer.unions.intern([*ty, Type::Undefined], analyzer.program.functions[index].name.span)?; }
            }
            analyzer.signatures.push(Signature { params, body_params, result, inferred_result: annotation.is_none() && analyzer.program.functions[index].name.text != "main" && return_type_inference::has_value_return(&analyzer.program.functions[index].body) });
        }
        Ok(analyzer)
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
            // The caller supplies the advertised parameter to the concrete
            // function. An owning Once input can enter a scoped Once consumer;
            // a borrowed input cannot enter an owning retainer.
            let once_parameter_input = matches!((actual.ty, expected.ty), (Type::Callback(a), Type::Callback(e))
                if self.callables.callbacks[a].kind.is_once()
                && self.callables.callbacks[e].kind.is_once());
            if once_parameter_input {
                self.require(expected.ty, Some(actual.ty), span)?;
            } else {
                self.require(actual.ty, Some(expected.ty), span)?;
            }
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
        if matches!(binding.module, "std:threads" | "std:result") { return Err(Diagnostic::new("thread-call", "Call thread operations and unwrap directly; polymorphic operation values are not supported.", span)); }
        let count = match expected {
            Some(Type::Callback(contract)) => self.callables.callbacks[contract].params.len(),
            Some(Type::Callable(contract)) => self.callables.results[contract].params.len(),
            Some(Type::Closure(origin)) => self.closures[origin].parameters.len(),
            _ => binding.parameters.len(),
        };
        let rest = binding.parameters.last().is_some_and(|parameter|parameter.rest) && match expected {
            Some(Type::Callback(contract)) => self.callables.callbacks[contract].params.last().is_some_and(|parameter|parameter.rest),
            Some(Type::Callable(contract)) => self.callables.results[contract].params.last().is_some_and(|parameter|parameter.rest),
            Some(Type::Closure(origin)) => self.closures[origin].parameters.last().is_some_and(|id|self.closures[origin].function.bindings[*id].rest),
            _ => true,
        };
        self.native_function_shape(id, span, if rest {binding.parameters.len()} else {count}, rest)
    }

    fn native_function_shape(
        &mut self,
        id: usize,
        span: Span,
        count: usize,
        rest: bool,
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
                && closure.parameters.last().is_some_and(|id|closure.function.bindings[*id].rest) == rest
        });
        let parameters=(0..count).map(|position|*binding.parameters.get(position).or_else(||binding.parameters.last().filter(|p|p.rest)).expect("checked native arity")).collect::<Vec<_>>();
        let parameter_types = parameters.iter().enumerate()
            .map(|(position,parameter)| { let ty=self.native_parameter_type(parameter.ty)?; Ok(if rest && position+1==count {Type::Array(ArrayElement::String(0))} else {ty}) })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let origin = existing.unwrap_or_else(|| {
            let bindings = parameters
                .iter()
                .zip(parameter_types)
                .enumerate()
                .map(|(position,(parameter, ty))| Binding { rest: rest && position+1==count,
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
                    mutable_place: false,
                    callback_invocation: None,
                    callback_reference: false,
                    callback_kind: None,
                    // Native scalar arguments borrow; an abstract language rest callable receives its existing owned pack ABI.
                    mode: if rest && position+1==count {Mode::Consume} else {parameter.access},
                    mutable: false,
                    transferred: None,
                    moved_fields: BTreeMap::new(),
                    used: None,
                    view_roots: Vec::new(),
                    native_view_loans: None,
                 omission: crate::ParameterOmission::Required, })
                .collect();
            let origin = self.closures.len();
            self.closures.push(CheckedClosure {
                program_scoped: false, program_captures: Vec::new(),
                polymorphic: false,
                methods: Vec::new(),
                forward: Some(ClosureForward::Native(id)),
                borrowed: false,
                lifetime_count: 0,
                depth: 0,
                parameters: (0..count).collect(),
                function: CheckedFunction {
                storage_exits: Default::default(), native_returns: None, native_mutations: None,
                    template: false,
                    specialization: false,
                    name: binding.name.into(),
                    bindings,
                    parameter_count: 0,
                    result: native_type(binding.result, &self.program.records),
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
                    active.argument(binding, contract.mode, arg);
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
                self.note_result_wait(index, span);
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
                    mutable_place: false,
                    view_roots: Vec::new(),
                    native_view_loans: None,
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
                program_scoped: false, program_captures: Vec::new(),
                polymorphic: false,
                methods: Vec::new(),
                forward: Some(ClosureForward::Function(index)),
                borrowed: false,
                lifetime_count: 0,
                depth: 0,
                parameters: (0..bindings.len()).collect(),
                function: CheckedFunction {
                storage_exits: Default::default(), native_returns: None, native_mutations: None,
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
        if let Some(target) = &self.checked[index] {
            self.closures[id].function.native_returns = target.native_returns.clone();
            self.closures[id].function.native_mutations = target.native_mutations.clone();
        }
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
            mutable_place: false,
            view_roots: Vec::new(),
            native_view_loans: None,
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
                program_scoped: false, program_captures: Vec::new(),
            polymorphic: false,
            methods: Vec::new(),
            forward: Some(ClosureForward::Abstract(contract)),
            borrowed: false,
            lifetime_count: 0,
            depth: context.closure_depth + 1,
            parameters: (1..bindings.len()).collect(),
            function: CheckedFunction {
                storage_exits: Default::default(), native_returns: None, native_mutations: None,
                template: true,
                specialization: false,
                name: context.function_name.clone(),
                bindings,
                parameter_count: 0,
                result: callback.result,
                body: Vec::new(),
            },
            captures: vec![Capture {
                restored:false,
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
            mutable_place: false,
            view_roots: Vec::new(),
            native_view_loans: None,
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
            program_scoped: false, program_captures: Vec::new(),
            polymorphic: inner.polymorphic,
            methods: Vec::new(),
            forward: Some(ClosureForward::Once(origin)),
            borrowed: inner.borrowed,
            lifetime_count: inner.lifetime_count,
            depth: context.closure_depth + 1,
            parameters: (1..bindings.len()).collect(),
            function: CheckedFunction {
                storage_exits: Default::default(), native_returns: None, native_mutations: None,
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
                restored:false,
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

    /// A native type shows the name that its module exports, not its linked identity.
    /// A generic application spells its checked arguments the same way.
    fn declaration_name(&self, id: usize) -> std::borrow::Cow<'_, str> {
        let record = &self.program.records[id];
        if let Some(native) = record.native_type { return crate::native_types::declaration(native).name.into(); }
        match &record.application {
            Some(application) if !application.resolved_arguments.is_empty() => format!("{}<{}>", application.declaration.text,
                application.resolved_arguments.iter().map(|argument| self.type_name(*argument).into_owned()).collect::<Vec<_>>().join(", ")).into(),
            _ => record.name.text.as_str().into(),
        }
    }

    fn type_name(&self, ty: Type) -> std::borrow::Cow<'_, str> {
        if let Some(name) = self.result_type_name(ty) { return name.into(); }
        let name = match ty {
            Type::Literal(id,_) => return self.program.records[id].literal_definition.as_ref().unwrap().text().into(),
            Type::Int => "int",
            Type::Number => "number",
            Type::Boolean => "boolean",
            Type::String => "string",
            Type::Void => "void",
            Type::Null => "null",
            Type::Undefined => "undefined",
            Type::Closure(_) | Type::Callable(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) => {
                "closure"
            }
            Type::Record(i) | Type::Enum(i, _) | Type::Set(i) | Type::Map(i) => return self.declaration_name(i),
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
                    ArrayElement::Literal(id,_,_)=>return format!("{}{}",self.program.records[id].literal_definition.as_ref().unwrap().text(),"[]".repeat(usize::from(element.depth())+1)).into(),
                    ArrayElement::Int(_) => "int",
                    ArrayElement::Number(_) => "number",
                    ArrayElement::Boolean(_) => "boolean",
                    ArrayElement::Null(_) => "null",
                    ArrayElement::Undefined(_) => "undefined",
                    ArrayElement::String(_) => "string",
                    ArrayElement::Record(id, _) | ArrayElement::Enum(id, _, _) | ArrayElement::Set(id, _) | ArrayElement::Map(id, _) => {
                        return format!("{}{}", self.declaration_name(id), "[]".repeat(usize::from(element.depth()) + 1)).into();
                    }
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
        if let Some(expected) = expected
            && self.scoped_native_storage_contract(actual, expected, span)? { return Ok(()); }
        if expected.is_some_and(|expected| self.scoped_map_contract(actual, expected)) { return Ok(()); }
        if expected.is_some_and(|expected| self.scoped_record_nullable_contract(actual, expected)) { return Ok(()); }
        if let (Type::Array(ArrayElement::Record(actual, depth)), Some(Type::Array(ArrayElement::Record(expected, expected_depth)))) = (actual, expected) {
            if depth == expected_depth && (self.scoped_records.declaration(actual) == expected || self.scoped_records.parameter_contract(expected) && self.scoped_records.declaration(actual) == self.scoped_records.declaration(expected)) { return Ok(()); }
        }
        if let (Type::Array(ArrayElement::Union(actual, depth)), Some(Type::Array(ArrayElement::Union(expected, expected_depth)))) = (actual, expected) {
            if depth == expected_depth
                && matches!(self.unions.payload(Type::Union(expected)),Some(Type::Record(record)) if self.scoped_records.parameter_contract(record))
                && self.scoped_record_nullable_contract(Type::Union(actual),Type::Union(expected)) { return Ok(()); }
        }
        if expected.is_some_and(|expected|self.callables.storage.equivalent_result(actual,expected)) {return Ok(());}
        if !self.deferred_generic_values.is_empty() {
            self.deferred_generic_requirement(actual, expected, span)?;
        }
        if let (Type::Record(actual), Some(Type::Record(expected))) = (actual, expected) {
            if self.scoped_records.declaration(actual) == expected || self.scoped_records.parameter_contract(expected) && self.scoped_records.declaration(actual) == self.scoped_records.declaration(expected) { return Ok(()); }
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
        if let Some(&main) = self.function_names.get("main") {
            if !self.signatures[main].params.is_empty() || self.signatures[main].result != Type::Void {
                return Err(Diagnostic::new(
                    "entry",
                    "main() must have no parameters or return value.",
                    self.program.functions[main].name.span,
                ));
            }
        }
        if self.assumed_modes.is_some() {
            self.discover_results()?;
        }
        self.discover_record_callbacks();
        let mut sequence = 0;
        let mut priority = self.program.diagnostic_first.and_then(|span|
            self.program.functions.iter().position(|function|function.name.span==span));
        // A function that catches the failure of a callback field of a record
        // parameter needs the records that later functions construct. Check
        // the rest first, then that function again.
        let mut postponed = Vec::new();
        // Checking consumes a body; keep the source of every function that may
        // need the second attempt, including one reached through a call.
        let sources = self.signatures.iter().zip(&self.program.functions)
            .map(|(signature, function)| (signature.params.iter().any(|ty| matches!(ty, Type::Record(_)))
                || return_type_inference::has_exception_region(&function.body)).then(|| function.body.clone()))
            .collect::<Vec<_>>();
        // The number of known record callbacks when each function was checked.
        let mut field_origins = vec![usize::MAX; sources.len()];
        while priority.is_some() || sequence < self.program.functions.len() {
            let index = if let Some(index)=priority.take() {index} else {
                let index=sequence;sequence+=1;index
            };
            if self.checked[index].is_none() && self.capturing_function_is_unused(index)? {
                continue;
            }
            if let Some(known) = field_origins.get_mut(index) {
                *known = 0;
            }
            if let Err(error) = self.function(index) {
                if !Self::unresolved_catch_contract(&error) || self.checked[index].is_some() || sources.get(index).is_none_or(Option::is_none) {
                    return Err(error);
                }
                for (unchecked, source) in sources.iter().enumerate() {
                    if let Some(source) = source && self.checked[unchecked].is_none() {
                        self.program.functions[unchecked].body = source.clone();
                    }
                }
                self.visiting.iter_mut().for_each(|visiting| *visiting = false);
                self.function_stack.clear();
                self.depth = 0;
                postponed.push(index);
            }
        }
        for index in postponed {
            if self.checked[index].is_none() {
                self.function(index)?;
            }
        }
        // A handler is typed from the callbacks that constructed records held
        // when its function was checked. A record constructed later adds
        // failures to the same field; check such a function again.
        {
            loop {
                // Schema plans hide readonly callback fields. Their finite source
                // origins can widen catch contracts just like public callback fields.
                // Rechecking may recreate an environment, so count source sites,
                // not transient closure IDs.
                let schema_origins=self.schema_factories.values().copied().chain(self.schema_pipelines.callbacks())
                    .flat_map(|family|self.callables.storage.origins(family).unwrap_or_default())
                    .filter_map(|origin|self.closures.get(*origin))
                    .map(|closure|(closure.span.start,closure.span.end)).collect::<HashSet<_>>();
                let known = crate::error_effects::record_field_origins(&self.unions,&self.program.records, &self.scoped_records).len()+schema_origins.len();
                let stale = (0..sources.len())
                    .filter(|&index| sources[index].is_some() && field_origins[index] < known && self.checked[index].as_ref().is_some_and(|function| !function.template))
                    .collect::<Vec<_>>();
                if known == 0 || stale.is_empty() {
                    break;
                }
                for &index in &stale {
                    self.checked[index] = None;
                    self.program.functions[index].body = sources[index].clone().expect("retained source");
                }
                for index in stale {
                    self.function(index)?;
                    field_origins[index] = known;
                }
            }
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
            for (index,function) in self.program.functions.iter_mut().enumerate() {
                if self.checked[index].as_ref().is_some_and(|checked|checked.template) {continue;}
                let mut remap=|annotation:&mut crate::type_syntax::ValueAnnotation|polymorphic_argument_hints::remap_annotation(annotation,&ids);
                crate::source_walk::function_annotations(function,&mut remap)?;
                crate::source_walk::function_body(function,&mut remap,&mut |_|Ok(()))?;
                if let Some(application)=&mut function.application {
                    for annotation in application.arguments.iter_mut().chain(&mut application.patterns) {remap(annotation)?;}
                }
                for parameter in &mut self.signatures[index].params {remap_closure_type(parameter,&ids)?;}
                remap_closure_type(&mut self.signatures[index].result,&ids)?;
            }
            self.deferred_generic_values=std::mem::take(&mut self.deferred_generic_values).into_iter().filter_map(|(origin,source)|ids[origin].map(|mapped|(mapped,source))).collect();
            self.polymorphic_storage = self.polymorphic_storage.remap(&ids, Span::default())?;
            self.polymorphic_invocations = self.polymorphic_invocations.remap(&ids, Span::default())?;
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
                &self.polymorphic_invocations,
            )?;
        }
        if !self.effects_only{self.check_dead_handlers()?;self.scoped_record_containers()?;self.record_parameter_carriers()?;}
        let record_call_releases=record_call_capture_release::summaries(&self.unions,&self.checked,&self.program.records,&self.closures);
        let closure_record_call_releases = record_call_capture_release::closure_summaries(&self.unions,&self.checked,&self.program.records,&self.closures);
        let deferred_generic_nominals = self.deferred_generic_nominal_uses();
        let record_parameter_borrows: HashMap<usize, Vec<record_parameter_borrows::Borrow>> = (0..self.checked.len()).filter_map(|id| {
            let borrows = record_parameter_borrows::summary(&self.checked, &self.closures, &self.callables.storage, id);
            (!borrows.is_empty()).then_some((id, borrows))
        }).collect();
        let mut closure_record_parameter_borrows = HashMap::new();
        let mut closure_record_capture_borrows = HashMap::new();
        for id in 0..self.closures.len() {
            let (borrows, captured) = record_parameter_borrows::summary_parts(&self.checked, &self.closures, &self.callables.storage, record_callable_identity::Callable::Closure(id));
            if !borrows.is_empty() { closure_record_parameter_borrows.insert(id, borrows); }
            if !captured.is_empty() { closure_record_capture_borrows.insert(id, captured); }
        }
        if !self.effects_only {
            installer_value_uses::check(&self.checked, &self.closures, &record_parameter_borrows, &closure_record_parameter_borrows, &closure_record_capture_borrows)?;
        }
        let checked = CheckedProgram {
            program_callback_sites: self.program_callback_sites,
            module_owner_bindings: self.program.module_owner_bindings,
            schema_choices: self.schema_choices,
            schema_pipelines: self.schema_pipelines,
            schema_factories: self.schema_factories,
            record_parameter_borrows,
            closure_record_parameter_borrows,
            closure_record_capture_borrows,
            polymorphic_storage: self.polymorphic_storage,
            polymorphic_invocations: self.polymorphic_invocations,
            record_call_releases,
            closure_record_call_releases,
            record_parameter_replacements:self.record_parameter_replacements,
            scoped_records: self.scoped_records,
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

    fn absent_callback(&mut self, mut id: usize) -> Type {
        let mut parents=Vec::new();
        let mut result=loop {
            if let Some(&empty) = self.absent_callback_payloads.get(&id) {
                break Type::Callback(empty);
            }
            let c = &self.callables.callbacks[id];
            let contract = CallbackContract {
                structural: c.structural,
                parameter_names: c.parameter_names.clone(),
                params: c.params.clone(),
                result: c.result,
                kind: c.kind,
            };
            if let Type::Callable(result) = contract.result {
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
                parents.push((id,contract));
                id=source;
            } else {
                let result=contract.result;parents.push((id,contract));break result;
            }
        };
        for (id,contract) in parents.into_iter().rev(){
            let empty = self.callables.callbacks.len();
            self.callables.callbacks.push(CallbackContract {
                structural: contract.structural,
                parameter_names: contract.parameter_names,
                params: contract.params,
                result,
                kind: contract.kind,
            });
            self.absent_callback_payloads.insert(id, empty);
            result=Type::Callback(empty);
        }
        result
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
                mutable_place: false,
                view_roots: Vec::new(),
                native_view_loans: None,
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
                program_scoped: false, program_captures: Vec::new(),
            polymorphic: false,
            methods: Vec::new(),
            forward: None,
            borrowed: false,
            lifetime_count: 0,
            depth: 1,
            parameters: (0..bindings.len()).collect(),
            function: CheckedFunction {
                storage_exits: Default::default(), native_returns: None, native_mutations: None,
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
        let found = self.discover_body_work(context, body, result);
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
        self.record_names.insert(record.name.text.clone(),id);
        self.fields.push(record.fields.iter().map(|field|(field.name.text.clone(),field.ty.expect("runtime error field type"))).collect());
        self.program.records.push(record);
        Type::Record(id)
    }

    #[inline(never)]
    fn prepare_discovery_exception(&mut self,context:&Context,region:&ExceptionRegion,result:Type,runtime:Option<Type>)->Result<ExceptionRegion,Diagnostic> {
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
        Ok(source)
    }

    #[cfg(test)]
    fn discover_checked_exception_result(&mut self,context:&Context,region:&ExceptionRegion,result:Type,runtime:Option<Type>)->Result<Option<Type>,Diagnostic>{
        let source=self.prepare_discovery_exception(context,region,result,runtime)?;
        self.discover_region_work(context,source,result)
    }

    #[inline(never)]
    fn discover_throw_value(&mut self,context:&mut Context,value:&Expr)->Result<(),Diagnostic> {
        let expected=self.expression_type_hint(context,value).filter(|ty|matches!(ty,Type::Closure(_)));
        if let Some((Type::Void,_))=self.discovery_expression(context,value,expected)? {
            return Err(Diagnostic::new("type","A thrown expression must produce a value.",value.span));
        }
        Ok(())
    }

    fn discover_leaf_results(
        &mut self,
        context: &mut Context,
        body: &[Stmt],
        result: Type,
    ) -> Result<Option<Type>, Diagnostic> {
        let mut absent = None;
        for stmt in body {
            match &stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Try(_) | StmtKind::If{..} | StmtKind::While{..} | StmtKind::Block(_) | StmtKind::Loop{..} | StmtKind::ForOf{..} => unreachable!("discovery composite is scheduled"),
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
            let ExprKind::Inject{value,..}=inner.into_kind() else {unreachable!()};inner=value;
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
    fn prepare_parameter_pattern(&mut self, context: &mut Context, parameter: &Token, body: &mut [Stmt], owned: bool) -> Option<std::ops::Range<usize>> {
        if !parameter.text.starts_with("pattern#") { return None; }
        let start=body.iter().position(|stmt| stmt.span.start>=parameter.span.start && stmt.span.end<=parameter.span.end);
        let start=start?;
        let count=body[start..].iter().take_while(|stmt| stmt.span.start>=parameter.span.start && stmt.span.end<=parameter.span.end).count();
        if owned && context.names.get(&parameter.text).is_some_and(|&binding| matches!(context.bindings[binding].ty, Type::Array(_) | Type::Record(_))) {
            owned_array_patterns::lower(&mut body[start..start+count], &parameter.text, context.bindings[context.names[&parameter.text]].ty, &self.fields);
        }
        Some(start..start+count)
    }

    fn initialize_parameter_pattern(&mut self,context:&mut Context,parameter:&Token,body:&mut[Stmt],owned:bool)->Result<(),Diagnostic>{
        if let Some(range)=self.prepare_parameter_pattern(context,parameter,body,owned){self.block_with_scope(context,&mut body[range],Type::Void,true)?;}
        Ok(())
    }

    fn uninitialized_body(body: &mut [Stmt]) -> &mut [Stmt] {
        let prefix=body.iter().take_while(|stmt| matches!(stmt.kind,StmtKind::Let{binding:Some(_),..})).count();
        &mut body[prefix..]
    }

    fn function_context(&mut self, index: usize, body: &mut Vec<Stmt>) -> Result<Context, Diagnostic> {
        let (mut context, parameters) = self.reserve_function_parameters(index)?;
        continuations::driver::check_function_parameters(self, &mut context, body, parameters)?;
        Ok(context)
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

    fn prepare_specialization(&mut self, index: usize, args: &[Expr], span: Span) -> Result<(usize, bool), Diagnostic> {
        if !self.templates.contains_key(&index) {
            return Ok((index, false));
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
                if !matches!(payload,Type::Closure(id) if !self.closures[id].function.template) && !self.absent_callable_type(payload) {return Ok((index, false));}
                *ty=self.unions.intern(self.unions.members(id).into_iter().map(|member|if matches!(member,Type::Callback(_)){payload}else{member}),arg.span)?;
                types.push(*ty);
            }
            if let Type::Callback(id) = *ty
                && self.callback_needs_context(id)
            {
                let actual = arg.ty.expect("checked argument");
                if self.discovering_results && self.has_pending_result(actual) {
                    return Ok((index, false));
                }
                // Template calls stay abstract until their caller is specialized.
                if !matches!(actual,Type::StoredCallable(_)) && !matches!(actual, Type::Closure(id) if !self.closures[id].function.template) {
                    return Ok((index, false));
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
            return Ok((id, false));
        }
        if self.callable_body_specialization_count()
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
        Ok((id, !self.discovering_results && self.callable_result_type(self.signatures[id].result)))
    }

    fn specialize(&mut self, index: usize, args: &[Expr], span: Span) -> Result<usize, Diagnostic> {
        let (index, immediate) = self.prepare_specialization(index, args, span)?;
        if immediate { self.function(index)?; }
        Ok(index)
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

    // Branch snapshots belong only to an if statement. Keep their temporary
    // maps outside block frames retained by deeply nested ordinary calls.
    // Availability joins carry large ownership records only after both arms finish.
    #[inline(never)]
    fn join_if_availability(context: &mut Context, before: Vec<(usize, Availability)>, after_then: Vec<Availability>, then_exits: bool, else_exits: bool) {
        for ((id, entry), then_state) in before.into_iter().zip(after_then) {
            let else_state = Availability::of(&context.bindings[id]);
            let joined = match (then_exits, else_exits) {
                (false, false) => then_state.join(else_state),
                (false, true) => then_state,
                (true, false) => else_state,
                // No branch reaches another statement, but capture ABI planning
                // still needs the incoming carried-view slots. Clearing their
                // provenance would make a valid owning view capture look untracked.
                (true, true) => Availability { native_view_loans:entry.native_view_loans, ..Availability::default() },
            };
            joined.restore(&mut context.bindings[id]);
        }
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
        crate::error_effects::annotate_partial_with_storage(&mut self.checked.iter_mut().map(Option::as_mut).collect::<Vec<_>>(),&mut self.closures,&families,&self.polymorphic_invocations)
    }

    #[inline(never)]
    fn checked_error_effects(&mut self)->Result<crate::error_effects::ErrorEffects,Diagnostic> {
        let mut functions=self.checked.iter_mut().map(Option::as_mut).collect::<Option<Vec<_>>>()
            .ok_or_else(||Diagnostic::new("error-effect","Error contracts require every function body to be checked first.",Span::default()))?;
        let families=self.callables.storage.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
        Ok(crate::error_effects::annotate_parts_with_storage(&mut functions,&mut self.closures,&families,&self.polymorphic_invocations))
    }

    #[inline(never)]
    fn check_source_callable_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<bool,Diagnostic> {
        self.enter_source_callable_body(context)?;
        let returned=if has_exception_syntax(body) {self.check_exception_callable_body(context,body,result)}else{self.block(context,body,result)}?;
        self.validate_native_returns(context)?;
        Ok(returned)
    }

    fn enter_source_callable_body(&mut self, context: &mut Context) -> Result<(), Diagnostic> {
        context.callable_scope_depth = context.lexical_scopes.len();
        context.native_returns=None;
        context.native_mutations=None;
        self.seed_native_parameters(context)?;
        Ok(())
    }

    #[inline(never)]
    fn check_exception_callable_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type)->Result<bool,Diagnostic> {
        self.exception_callable_work(context,body,result)
    }

    #[cfg(test)]
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

    #[cfg(test)]
    #[inline(never)]
    fn check_typed_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState))->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        self.region_work(context,region,result,continuations::RegionMode::Fixed(effects.clone()),runtime,unwind)
    }

    #[cfg(test)]
    #[inline(never)]
    fn check_current_exception_region(&mut self,context:&mut Context,region:&mut ExceptionRegion,result:Type,runtime:Option<Type>,unwind:&mut impl FnMut(Span,&mut exception_state::ExitState))->Result<(exception_state::CompletionStates,crate::error_effects::FailureSet),Diagnostic> {
        self.region_work(context,region,result,continuations::RegionMode::Current,runtime,unwind)
    }

    #[inline(never)]
    fn region_error_effects(&mut self,context:&Context)->Result<crate::error_effects::ErrorEffects,Diagnostic> {
        if context.exception_environment.as_ref().is_some_and(|environment|environment.partial_contracts) {
            Ok(self.discovery_error_effects())
        }else{self.checked_error_effects()}
    }

    #[cfg(test)]
    #[inline(never)]
    fn check_typed_catch_exits(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type,incoming:&crate::error_effects::FailureSet,effects:&crate::error_effects::ErrorEffects,runtime:Option<Type>)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),Diagnostic> {
        self.check_catch_contract_exits(context,handler,result,incoming,Some(effects),runtime)
    }

    #[cfg(test)]
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

    /// The failures of a borrowing callback field are those of every callback
    /// that a constructed record of its type holds. Records are constructed in
    /// any function, so construct them all before a handler is typed: check
    /// each body once without recording effects and without following calls.
    /// Recursive view parameters also need the concrete callback families that
    /// constructors select. Errors are ignored here; definitive checking reports
    /// them and replaces speculative callback return-lifetime metadata.
    #[inline(never)]
    fn discover_record_callbacks(&mut self) {
        let borrowing_field = |record: &Record| record.fields.iter().any(|field| {
            matches!(field.ty, Some(Type::Callback(id)) if !self.callables.callbacks[id].kind.is_owned())
        });
        let recursive_views = crate::recursive_view_shapes::Index::new(&self.program.records, &|id| self.unions.members(id));
        let recursive_nullable_callbacks = self.native_borrow_types && self.program.records.iter().enumerate().any(|(id, record)|
            recursive_views.recursive(id) && record.fields.iter().any(|field| field.ty.is_some_and(|ty|
                matches!(self.unions.payload(ty), Some(Type::StoredCallable(_))))));
        let recursive_callbacks = recursive_nullable_callbacks || self.native_borrow_types && self.program.records.iter().enumerate().any(|(id, record)|
            recursive_views.recursive(id) && record.fields.iter().any(|field| field.ty.is_some_and(|ty| matches!(self.unions.payload(ty).unwrap_or(ty), Type::StoredCallable(_)))))
            && self.signatures.iter().any(|signature| signature.params.iter().any(|ty|
                matches!(ty, Type::Record(record) if recursive_views.recursive(*record))));
        if !recursive_callbacks && (!self.program.records.iter().any(borrowing_field)
            || !self.program.functions.iter().any(|function| return_type_inference::has_exception_region(&function.body)))
        {
            return;
        }
        let discovering = self.discovering_results;
        let effects = self.effects_only;
        self.effects_only = true;
        self.discovering_results = true;
        // A call of a function that is not checked yet reads assumed access.
        let assumed = self.assumed_modes.is_none();
        if assumed {
            self.assumed_modes = Some(self.signatures.iter()
                .map(|signature| signature.params.iter().map(|ty| if ty.is_copy() { Mode::Copy } else { Mode::Read }).collect())
                .collect());
        }
        for index in 0..self.program.functions.len() {
            let mut body = self.program.functions[index].body.clone();
            if let Ok(mut context) = self.function_context(index, &mut body) {
                let result = self.signatures[index].result;
                context.inferred_body_result = self.signatures[index].inferred_result;
                let _ = self.check_source_callable_body(&mut context, Self::uninitialized_body(&mut body), result);
            }
            self.visiting.iter_mut().for_each(|visiting| *visiting = false);
            self.function_stack.clear();
            self.depth = 0;
        }
        if assumed {
            self.assumed_modes = None;
        }
        self.discovering_results = discovering;
        self.effects_only = effects;
    }

    fn unresolved_catch_contract(error: &Diagnostic) -> bool {
        error.code == "error-effect" && error.message.starts_with(crate::error_effects::UNRESOLVED_CATCH)
    }

    /// A handler whose try body cannot fail never runs, but its statements are
    /// still source: check them on a copy of the state at the region. The error
    /// name stands for a runtime error, the common case.
    #[inline(never)]
    fn check_dead_handlers(&mut self)->Result<(),Diagnostic> {
        let handlers=std::mem::take(&mut self.dead_handlers);
        if handlers.is_empty() {return Ok(());}
        let effects=self.effects_only;
        self.effects_only=true;
        let mut outcome=Ok(());
        for (_,entry) in handlers {
            let (mut context,mut handler,result)=*entry;
            context.effects_only=true;
            handler.binding=None;
            handler.error_type=handler.name.is_some().then(||self.materialize_discovery_runtime_error());
            if let Err(error)=self.check_catch_exits(&mut context,&mut handler,result) {outcome=Err(error);break;}
        }
        self.effects_only=effects;
        outcome
    }

    #[inline(never)]
    fn check_catch_exits(&mut self,context:&mut Context,handler:&mut CatchClause,result:Type)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame),Diagnostic> {
        self.check_with_exit_frames(context,|analyzer,context|analyzer.check_catch_body(context,handler,result))
    }

    #[cfg(test)]
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
        let caller=CheckedFunction{storage_exits:Default::default(),native_returns:None, native_mutations:None,template:false,specialization:false,name:"#region".into(),bindings:context.bindings.clone(),parameter_count:0,result,body:body.to_vec()};
        let families=self.callables.storage.families().map(|(id,_,origins)|(id,origins.iter().copied().collect())).collect::<Vec<_>>();
        let mut fields=crate::error_effects::record_field_origins(&self.unions,&self.program.records,&self.scoped_records);
        fields.extend(self.absent_callback_payloads.values().map(|id|(*id,Type::Undefined)));
        fields.extend(context.caller_error_origins.iter().copied());
        crate::error_effects::annotate_partial_with_callers(&mut self.checked.iter_mut().map(Option::as_mut).collect::<Vec<_>>(),&mut self.closures,&families,&caller,&self.unions,&self.polymorphic_invocations,&fields)
    }

    #[inline(never)]
    fn check_pending_region_functions(&mut self,body:&[Stmt],result:Type)->Result<(),Diagnostic> {
        let caller=CheckedFunction{storage_exits:Default::default(),native_returns:None, native_mutations:None,template:false,specialization:false,name:String::new(),bindings:vec![],parameter_count:0,result,body:body.to_vec()};
        let targets=crate::error_effects::checked_expressions(&caller).into_iter().filter_map(|expression|match expression.target {Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=>Some(id),_=>None}).collect::<HashSet<_>>();
        for id in targets {
            if self.checked[id].is_none() && !self.visiting[id] {self.function(id)?;}
        }
        Ok(())
    }

    #[cfg(test)]
    #[inline(never)]
    fn check_typed_region_body(&mut self,context:&mut Context,body:&mut [Stmt],result:Type,effects:&crate::error_effects::ErrorEffects)->Result<(Option<exception_state::ExitState>,exception_state::CompletionFrame,exception_state::ErrorFrame,crate::error_effects::FailureSet),Diagnostic> {
        let (normal,completions,mut errors)=self.check_region_body(context,body,result)?;
        let contract=crate::error_effects::annotate_region_body(body,effects);
        let points=crate::error_effects::OperationEffects::for_body(body);
        errors.refine(&mut |span|points.get(span));
        Ok((normal,completions,errors,contract))
    }

    #[cfg(test)]
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
        let boundary=Self::prepare_exit_frames(context);
        let checked=check(self,context);
        Self::finish_exit_frames(context,boundary,checked)
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
            && let Some(entry)=context.region_entry_loans.as_ref()
            && let Some(frame)=&mut context.completion_frame
            && let Some((_,_,state))=frame.exits.last_mut()
        {state.unwind_capture_loans(entry,&[]);}
    }

    #[cfg(test)]
    #[inline(never)]
    fn check_throw_value(&mut self,context:&mut Context,value:&mut Expr)->Result<Type,Diagnostic> {
        let expected=self.prepare_throw_value(context,value)?;
        let ty=self.expression(context,value,Mode::Consume,&Loans::new(),expected)?;
        self.finish_throw_value(context,value,ty)
    }

    #[inline(never)]
    fn skip_diagnostic_statement(&self, statement: &mut Stmt) -> bool {
        if !self.program.diagnostic_skips.contains(&statement.span.start) {return false;}
        statement.kind=StmtKind::Block(Vec::new());
        true
    }

    fn rebind_target(&self, context: &Context, target: &Expr) -> Result<usize, Diagnostic> {
        let ExprKind::Name(name) = &target.kind else {
            unreachable!("named assignment")
        };
        let id = *context.names.get(name).ok_or_else(|| {
            match self.program.local_declarations.iter().find(|declaration| declaration.emitted_name == *name) {
                Some(declaration) => Diagnostic::new("name", format!("Function {:?} cannot be assigned; a function name always refers to its declaration.", declaration.source_name.text), target.span),
                None => Diagnostic::new("name", format!("Unknown value {name:?}."), target.span),
            }
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
        Self::check_native_view_place(context, id, &[], Mode::Mutate, target.span)?;
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
        self.reassign_with_sequence(context,target,value,loans,false)
    }
    #[inline(never)]
    fn reassign_with_sequence(&mut self,context:&mut Context,target:&mut Expr,value:&mut Expr,loans:&Loans,precise_sequence:bool)->Result<(),Diagnostic>{
        self.store_work(context, target, value, loans, precise_sequence)
    }

    fn update_expression(&mut self, context: &mut Context, target: &mut Expr, loans: &Loans, operator: Span) -> Result<Type, Diagnostic> {
        let mut unused = Expr::new(ExprKind::Undefined, operator);
        self.assignment_work(context, target, &mut unused, continuations::AssignmentKind::Update, operator, loans)
    }

    /// `text[position]`: a new string, never a view of the text.
    #[inline(never)]
    pub(crate) fn string_position(&self, context: &Context, expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Index(base, _)
            if self.expression_type_hint(context, base).is_some_and(|ty| self.unions.payload(ty).unwrap_or(ty) == Type::String))
    }

    fn scalar_assignment(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, loans: &Loans, operator: Span) -> Result<Type, Diagnostic> {
        self.assignment_work(context, target, value, continuations::AssignmentKind::Scalar, operator, loans)
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

    fn logical_assignment(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, loans: &Loans, operator: Span) -> Result<Type, Diagnostic> {
        self.assignment_work(context, target, value, continuations::AssignmentKind::Logical, operator, loans)
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
        let scalar = |ty: Type| matches!(ty.scalar_base(), Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String);
        let supported = ty.union().map_or_else(|| scalar(ty), |id| self.unions.members(id).iter().copied().all(scalar));
        if ty.scalar_base() == Type::String { return Ok(()); }
        if !supported { return Err(Diagnostic::new("type", "String concatenation accepts only string or scalar data operands. Convert arrays or records explicitly with String() or a template literal; functions are unsupported.", value.span)); }
        let span = value.span;
        let original = std::mem::replace(value, Expr::new(ExprKind::Undefined, span));
        *value = Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("String".to_owned()), span)), vec![original]), span);
        Ok(())
    }

    fn compound_assignment_value(&mut self, context: &mut Context, target: &mut Expr, value: &mut Expr, operation: (BinaryOp, Span), loans: &Loans, allow_owned_statement: bool) -> Result<Type, Diagnostic> {
        self.assignment_work(context, target, value, continuations::AssignmentKind::Compound { op: operation.0, allow_owned: allow_owned_statement }, operation.1, loans)
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
            mutable_place: false,
            view_roots: Vec::new(),
            native_view_loans: None,
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
                Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) => "closure-consumed",
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
        if binding.name.starts_with(crate::data_lowering::ACCESS) {
            if mode == Mode::Mutate || (mode == Mode::Consume && !binding.ty.is_copy()) {
                return Err(Diagnostic::new("data-readonly", "Imported data is readonly; use clone() for an owned copy before mutation or ownership transfer.", span));
            }
            return Ok(());
        }

        if binding.mutable_place && mode == Mode::Consume && !binding.ty.is_copy() {
            return Err(Diagnostic::new("borrowed-place", "A mutable match payload borrows its source and cannot transfer ownership; use clone() for independent ownership.", span));
        }
        if matches!(mode, Mode::Mutate | Mode::Consume)
            && !binding.mutable_place
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
        Self::check_native_view_access(context, index, mode, span)?;
        Self::check_mutable_match_owner(context, index, span)?;
        if matches!(mode, Mode::Mutate | Mode::Consume) {
            if !context.effects_only {
                if matches!(context.bindings[index].ty, Type::BorrowedUnion(_)) {
                    return Err(Diagnostic::new(
                        "borrowed-view",
                        "An optional element view is readonly and cannot escape or transfer ownership; use clone() to own its value.",
                        span,
                    ));
                }
                // Name the earliest holder: map iteration order is not stable.
                let mut views = context.names.values().copied().collect::<Vec<_>>();
                views.sort_unstable();
                for view in views {
                    if context.bindings[view].view_roots.contains(&index) {
                        let holder = &context.bindings[view];
                        // A borrowed const names itself; other views keep the element wording.
                        let (message, note) = if holder.borrowed_place.is_some() {
                            (
                                format!("Cannot mutate, replace, or transfer this storage while {:?} borrows part of it. Use the borrow in an inner block, or clone() the part.", holder.name),
                                format!("{:?} borrows here.", holder.name),
                            )
                        } else if holder.callback_reference {
                            (
                                format!("Cannot call or transfer this callback while its alias {:?} borrows it. Call the alias instead, or keep the alias in an inner block.", holder.name),
                                format!("{:?} borrows the callback here.", holder.name),
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
        for loans in std::iter::once(loans).chain(context.match_loans.iter()) {
          if !context.effects_only && let Some(active) = loans.get(&index) {
            for (previous, previous_span) in active {
                if (mode != Mode::Read || *previous != Mode::Read)
                    && loans.overlaps(index, *previous, *previous_span, context.projection.as_ref().filter(|projection| projection.root == binding.name).map(|projection| projection.path.as_slice())) {
                    return Err(Diagnostic::new("conflicting-access", format!("Cannot access {:?} through overlapping shared, exclusive, or consuming access.", binding.name), span)
                        .related(*previous_span, "Earlier access starts here."));
                }
            }
        }
        }
        if binding.parameter && (!binding.ty.is_copy() || mode == Mode::Mutate) {
            // Moving a reusable callback transfers the local &mut reference,
            // not ownership of the caller's environment.
            let parameter_mode = if binding.callback_reference && mode == Mode::Consume {
                // A read-only callback is a shared reference; moving it stays shared.
                if binding.callback_kind == Some(CallbackKind::Read) { Mode::Read } else { Mode::Mutate }
            } else {
                mode
            };
            binding.mode = binding.mode.max(parameter_mode);
        }
        if mode == Mode::Mutate {
            binding.mutable = true;
        }
        if mode == Mode::Consume {
            context.capture_loan_owners.begin_consumption(
                &crate::capture_loan_owners::Place {
                    binding: index,
                    fields: projection.map(|value| value.path.clone()).unwrap_or_default(),
                },
                projection.map(|value| value.span).unwrap_or(span),
            );
            if let Some(projection) = projection {
                context.has_field_moves = true;
                if let Some(origins)=&mut binding.native_view_loans {
                    native_view_places::consume(origins,&projection.path);
                }
                binding
                    .moved_fields
                    .retain(|path, _| !path.starts_with(&projection.path));
                binding
                    .moved_fields
                    .insert(projection.path.clone(), projection.span);
            } else {
                binding.transferred = Some(span);
                if let Some(origins) = &mut binding.native_view_loans {
                    for loan in origins.iter_mut() { loan.retained = false; }
                }
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
        let mut roots = Vec::new();
        let mut work = vec![(expr, mode)];
        while let Some((expr, mode)) = work.pop() {
            if let ExprKind::Sequence(values)=&expr.kind {
                if let Some(last)=values.last(){work.push((last, mode));}
            } else if let ExprKind::Selected { source, .. } = &expr.kind {
                for root in &source.roots {
                    if !roots.contains(root) {
                        roots.push(*root);
                    }
                }
            } else if let ExprKind::Optional { body, .. } = &expr.kind {
                work.push((body, mode));
            } else if matches!(
                expr.target,
                Some(CallTarget::HeapMethod(HeapMethod::Peek) | CallTarget::DequeMethod(DequeMethod::Front | DequeMethod::Back | DequeMethod::At) | CallTarget::MapMethod(MapMethod::Get) | CallTarget::ArrayAt | CallTarget::ArrayFind | CallTarget::ArrayFindLast)
            ) && matches!(expr.ty, Some(Type::BorrowedUnion(_)))
            {
                if let ExprKind::Call(callee, _) = &expr.kind
                    && let ExprKind::Field(base, _) = &callee.kind
                {
                    work.push((base, Mode::Read));
                }
            } else if let ExprKind::CoalesceAssign { target, .. } = &expr.kind {
                if !expr.ty.is_some_and(|ty| ty.owning().is_copy()) {
                    work.push((target, Mode::Read));
                }
            } else if let ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } = &expr.kind {
                work.push((value, mode));
            } else if let ExprKind::Inject { value, .. } = &expr.kind {
                if matches!(expr.ty, Some(Type::BorrowedUnion(_)))
                    || value.ty.is_some_and(Type::is_copy)
                {
                    work.push((value, mode));
                }
            } else if let ExprKind::Unary {
                op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull | UnaryOp::ParameterMutableInput,
                value,
                ..
            } = &expr.kind
            {
                work.push((value, mode));
            } else if let ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                left,
                right,
                ..
            } = &expr.kind
            {
                work.push((right, mode));
                work.push((left, mode));
            } else if let ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } = &expr.kind
            {
                work.push((alternate, mode));
                work.push((consequent, mode));
            } else if let Some(root) = Analyzer::loan_root(expr, mode)
                && !roots.contains(&root)
            {
                roots.push(root);
            }
        }
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
                    if current.ty != Some(Type::Union(*union))
                        && !context.unconditional_projections.contains(&(*union, *narrowed)) {
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

    fn root_binding(mut expr: &Expr) -> Option<usize> {
        loop {
            match &expr.kind {
                ExprKind::Sequence(values)=>expr=values.last()?,
                ExprKind::SpreadValue(selected) => return selected.source.place.as_ref().map(|(root,_)|*root).or_else(||selected.source.roots.first().copied()),
                ExprKind::Selected { source, .. } => return source
                    .place
                    .as_ref()
                    .map(|(root, _)| *root)
                    .or_else(|| source.roots.first().copied()),
                ExprKind::Name(_) => return expr.binding,
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } | ExprKind::Inject {value,..}
                | ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull | UnaryOp::ParameterMutableInput,
                    value,
                    ..
                } => expr=value,
                ExprKind::Field(base, _) | ExprKind::Index(base, _) => expr=base,
                _ => return None,
            }
    }
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
    fn lower_scalar_unary_plus(&self, context: &Context, expr: &mut Expr) {
        let ExprKind::Unary { op: UnaryOp::Positive, value, .. } = &mut expr.kind else { return; };
        if self.expression_type_hint(context, value).is_none_or(|ty| matches!(ty, Type::Int | Type::Number)) { return; }
        let span = expr.span;
        let operand = std::mem::replace(value, Box::new(Expr::new(ExprKind::Undefined, span)));
        expr.kind = ExprKind::Call(Box::new(Expr::new(ExprKind::Name("Number".to_owned()), span)), vec![*operand]);
    }

    // Generic/spread staging is finished before ordinary recursive checking begins.
    #[inline(never)]
    fn prepare_other_expression(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, _loans: &Loans, expected: Option<Type>) -> Result<Option<continuations::adapters::Other>, Diagnostic> {
        if let Some(expected) = expected
            && self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).is_some()
        {
            if self.lower_selected_generic_factory(context, expr)? {
                return self.prepare_selected_generic_expression(context, expr, expected).map(|request|Some(continuations::adapters::Other::Adapter(request)));
            }
        }
        if let Some(expected) = expected
            && self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).is_some()
            && self.fresh_generic_origin(context, expr)?.is_some()
        {
            return self.prepare_generic_expression(context, expr, expected, mode).map(|request|Some(continuations::adapters::Other::Adapter(request)));
        }
        self.lower_enum_member(context, expr, mode)?;
        if matches!(&expr.kind, ExprKind::SpreadValue(_) | ExprKind::SpreadCall(_))
            || matches!(&expr.kind, ExprKind::Unary { op: UnaryOp::RestPack, value, .. } if matches!(value.kind, ExprKind::SpreadValue(_)))
        {
            return self.checked_spread_expression(context, expr, mode, expected).map(|ty|Some(continuations::adapters::Other::Ready(ty)));
        }
        Ok(None)
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

    #[inline(never)]
    fn checked_expression_prelude(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        mode: Mode,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<continuations::prelude::Prelude>, Diagnostic> {
        use continuations::prelude::{Prelude,Frame};
        if let Some(result)=self.json_typed_encode(context,expr,loans)?{return Ok(Some(Prelude::Ready(result)));}
        if let Some(result)=self.json_typed_decode(context,expr,loans,expected)?{return Ok(Some(Prelude::Ready(result)));}
        if let Some(result)=self.json_string_length(context,expr,mode,loans)?{return Ok(Some(Prelude::Ready(result)));}
        if let Some(result) = self.result_match_intrinsic(context, expr, loans)? {
            return Ok(Some(Prelude::Ready(result)));
        }
        if let Some(result) = self.result_constructor(context, expr, loans, expected)? {
            return Ok(Some(Prelude::Ready(result)));
        }
        if mode == Mode::Consume
            && let Some(Type::StoredCallable(family)) = expected
        {
            if !matches!(expr.kind, ExprKind::Conditional { .. })
                && self.expression_type_hint(context, expr) != expected
            {
                let frame=self.prepare_stored_value(context,expr,family)?;
                return Ok(Some(Prelude::Child{mode:Mode::Consume,expected:frame.expected,frame:Frame::Stored(frame)}));
            }
        }
        if mode == Mode::Mutate
            && context.mutation_place.is_none()
            && let Some(place) = crate::flow::mutation_footprint(expr, &context.names)
        {
            context.mutation_place=Some(place);
            return Ok(Some(Prelude::Child{mode:Mode::Mutate,expected,frame:Frame::Mutation}));
        }
        context
            .condition_flows
            .remove(&(expr.span.start, expr.span.end));
        if mode != Mode::Mutate && expected != crate::flow::place(expr,&context.names).and_then(|place|self.place_type(context,&place)) {
            if let Some(proof)=self.json_boolean_proof(context,expr) {return self.project_json_boolean(context,expr,loans,proof).map(|ty|Some(Prelude::Ready(ty)));}
            if let Some(ty)=self.json_string_projection_type(context,expr) {return self.project_json_string(context,expr,loans,ty).map(|ty|Some(Prelude::Ready(ty)));}
        }

        if !matches!(
            expr.kind,
            ExprKind::Project { .. } | ExprKind::Selected { .. }
        ) && let Some((place, union, narrowed)) = self.flow_view(context, expr)
        {
            return self.prepare_projection_work(context,expr,mode,expected,place,union,narrowed).map(Some);
        }
        if let Some(Type::BorrowedUnion(id)) = expected {
            if mode != Mode::Read {
                return (Err(Diagnostic::new(
                    "borrowed-view",
                    "Borrowed optional views cannot be consumed or escape; use clone() for independent ownership.",
                    expr.span,
                ))).map(|ty|Some(Prelude::Ready(ty)));
            }
            return Ok(Some(Prelude::Child{mode:Mode::Read,expected:None,frame:Frame::BorrowedExpected(id)}));
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
            return self.prepare_injection_work(context,expr,mode,loans,id).map(Some);
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
        expr.json_string_view=result.is_ok() && (self.json_string_proof(context,expr).is_some() || self.is_string_view(context,expr));
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
        let ExprKind::Unary { value, .. } = &mut expr.kind else { unreachable!() };
        let actual = self.expression(context, value, mode, loans, None)?;
        self.finish_non_null_work(expr, mode, expected, actual)
    }

    fn finish_non_null_work(&mut self, expr: &mut Expr, mode: Mode, expected: Option<Type>, actual: Type) -> Result<Type, Diagnostic> {
        let ExprKind::Unary { value, operator, .. } = &mut expr.kind else { unreachable!("non-null completion"); };
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
            ExprKind::Unary {op:UnaryOp::RecordCarrier,value,..}=>{
                let destination=expr.ty.expect("checked record carrier destination");
                self.expression(context,value,Mode::Consume,loans,None)?;
                self.require(destination,expected,expr.span)?;
                Ok(destination)
            }
            ExprKind::Unary {
                op: UnaryOp::CallableStore,
                value,
                ..
            } => {
                let (Type::StoredCallable(family)|Type::BorrowedCallable(family)) = expr.ty.expect("checked storage wrapper") else {
                    unreachable!("stored wrapper type");
                };
                let actual = self.expression(context, value, Mode::Consume, loans, None)?;
                if let Some(ty) = self.abstract_stored_callable(context, family, actual, value.span)? { return Ok(ty); }
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
                Ok(if self.callables.storage.is_borrowed(family){Type::BorrowedCallable(family)}else{Type::StoredCallable(family)})
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

    #[inline(never)]
    fn native_collection_constructor(&mut self,expr:&mut Expr,expected:Option<Type>)->Result<Type,Diagnostic> {
        let ExprKind::NativeConstructor(arguments,annotation)=&expr.kind else {unreachable!("native constructor")};
        if !arguments.is_empty() { return Err(Diagnostic::new("arity", "Native collection construction takes no arguments.", expr.span)); }
        let annotation = annotation.as_ref().expect("native constructor annotation");
        let ty = self.resolve_value_annotation(annotation, false)?;
        if !matches!(ty, Type::Record(id) if self.program.records[id].native_element().is_some())
            && !matches!(ty,Type::Map(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap)) {
            return Err(Diagnostic::new("native-constructor", "This type has no supported native constructor.", expr.span));
        }
        self.require(ty, expected, expr.span)?;
        Ok(ty)
    }

    fn expression_kind(&mut self, context: &mut Context, expr: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        // Aggregate nesting must not retain the larger general-expression frame.
        let previous_view = expr.ty;
        match &mut expr.kind {
            ExprKind::Match { .. } => unreachable!("match body is scheduled by the driver"),
            ExprKind::Unary { op: UnaryOp::PatternSelection, .. } => self.pattern_selection(context,expr,mode,loans,expected),
            ExprKind::Unary { op: UnaryOp::NonNull, .. } => self.non_null_assertion(context, expr, mode, loans, expected),
            ExprKind::OptionalChain(_) => self.optional_chain(context, expr, mode, loans, expected),
            ExprKind::Selected { .. } => {
                self.selected_expression(context, expr, mode, loans, expected)
            }
            ExprKind::CoalesceAssign { .. } => {
                unreachable!("nullish assignment is scheduled before expression dispatch")
            }
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                ..
            } => unreachable!("coalescing is scheduled before expression dispatch"),
            ExprKind::Binary { op: BinaryOp::And | BinaryOp::Or, .. } =>
                unreachable!("logical operands are scheduled before expression dispatch"),
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
            ExprKind::Record(fields) if fields.iter().any(|(key,_)|key.text=="record#spread") => unreachable!("object spread sources use the shared checker stack"),
            ExprKind::Record(fields) => {
                self.record_literal(context, fields, loans, expected, expr.span)
            }
            ExprKind::ArrayOf(_, _) => unreachable!("array constructors use the shared checker worklist"),
            ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) => unreachable!("collection constructors use the shared checker stack"),
            ExprKind::NativeConstructor(_, _) => {
                if let Some(ty) = self.channel_constructor(context, expr, loans, expected)? { Ok(ty) } else { self.native_collection_constructor(expr,expected) }
            },
            ExprKind::Array(values) if values.iter().any(|value| matches!(value.kind, ExprKind::Unary { op: UnaryOp::Spread, .. })) => unreachable!("array spread constructors use the shared checker worklist"),
            ExprKind::Array(_) => unreachable!("dense array literals use the shared checker worklist"),
            ExprKind::Template(parts) => self.template_literal(context, parts, loans),
            // Synthesized wrappers and other unary forms keep their
            // temporaries out of this recursively retained dispatcher.
            ExprKind::Unary {
                op: UnaryOp::CallableStore
                    | UnaryOp::RecordCarrier
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
                let valid = crate::flow::guard_pair(left, right, &context.names).is_some() || self.record_discriminator_guard(context,left,right).is_some();
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
        if let Some((place,equal,unequal))=self.record_discriminator_guard(context,left,right){
            let (a,b)=if op==BinaryOp::Equal{(equal,unequal)}else{(unequal,equal)};
            if !a.is_empty(){yes.insert(place.clone(),crate::flow::Fact{members:a,origin:Some((expr_span.start,expr_span.end))});}
            if !b.is_empty(){no.insert(place,crate::flow::Fact{members:b,origin:Some((expr_span.start,expr_span.end))});}
        }
        if let Some((place, guard)) = pair
            && let Some(union) = self.place_type(context, &place).and_then(Type::union)
        {
            let members = context
                .flow
                .get(&place)
                .map(|fact| fact.members.clone())
                .unwrap_or_else(|| self.unions.members(union));
            let dynamic_json = |ty: Type| matches!(ty, Type::Record(id) if matches!(self.program.records[id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView)));
            // Runtime JSON tags do not change the wrapper's storage type. Keep
            // its alternative on both feasible branches until scalar refinement.
            let static_match = |ty: Type| match &guard {
                crate::flow::Guard::Typeof(label) if matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::StringView)) => label == "string",
                _ => guard.matches(ty),
            };
            let may_match = |ty: Type| match &guard {
                crate::flow::Guard::Typeof(label) if dynamic_json(ty) => matches!(label.as_str(), "string" | "number" | "boolean" | "object"),
                crate::flow::Guard::Absent(Type::Null) if dynamic_json(ty) => true,
                _ => static_match(ty),
            };
            let may_differ = |ty: Type| matches!(&guard, crate::flow::Guard::Typeof(_) | crate::flow::Guard::Absent(Type::Null) if dynamic_json(ty)) || !static_match(ty);
            let equal = members
                .iter()
                .copied()
                .filter(|ty| may_match(*ty))
                .collect::<Vec<_>>();
            let unequal = members
                .iter()
                .copied()
                .filter(|ty| may_differ(*ty))
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
        self.remember_json_category(context,expr_span,op,left,right,&mut yes,&mut no);
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
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
        capture_boundary.unwind(&mut context.capture_loans);
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
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
        capture_boundary.unwind(&mut context.capture_loans);
        let (left_type, right_type, stable) = result?;
        self.finish_union_equality_work(context, (expr_span, operator), op, left, right, left_type, right_type, stable)
    }

    fn finish_union_equality_work(
        &mut self, context: &mut Context, spans: (Span, Span), op: BinaryOp,
        left: &Expr, right: &Expr, left_type: Type, right_type: Type, stable: bool,
    ) -> Result<Type, Diagnostic> {
        let (expr_span, operator) = spans;
        let primitive = |ty: Type| {
            matches!(ty.scalar_base(), Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Undefined | Type::Enum(_,_))
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
            (Some(union), None) if stable => crate::flow::place(left, &context.names).map(|place| (place, union, right_type, &*right)),
            (None, Some(union)) => crate::flow::place(right, &context.names).map(|place| (place, union, left_type, &*left)),
            _ => None,
        };
        if let Some((place, union, value, compared)) = guarded {
            let numeric = |ty: Type| matches!(ty.scalar_base(), Type::Int | Type::Number);
            let members = context.flow.get(&place).map(|fact| fact.members.clone())
                .unwrap_or_else(|| self.unions.members(union));
            let exact = |member: Type| -> Option<bool> {
                let Type::Literal(id, kind) = member else { return None; };
                let actual = if let Type::Literal(other, _) = value {
                    self.program.records[other].literal_definition.clone()
                } else {
                    crate::literal_types::LiteralValue::expression(compared, kind).and_then(Result::ok)
                }?;
                // Mixed numeric comparisons keep both paths conservatively: runtime
                // comparison can round an integer while its literal identity stays exact.
                if actual.kind() != kind { return None; }
                Some(self.program.records[id].literal_definition.as_ref() == Some(&actual))
            };
            let matching = members.iter().copied().filter(|member| {
                exact(*member).unwrap_or_else(|| member.scalar_base() == value.scalar_base()
                    || numeric(*member) && numeric(value))
            }).collect::<Vec<_>>();
            let remaining = members.into_iter().filter(|member| exact(*member) != Some(true)).collect::<Vec<_>>();
            let mut equal = context.flow.clone();
            let mut other = context.flow.clone();
            for (flow, members) in [(&mut equal, matching), (&mut other, remaining)] {
                if !members.is_empty() {
                    flow.insert(place.clone(), crate::flow::Fact {
                        members, origin: Some((expr_span.start, expr_span.end)),
                    });
                }
            }
            context.condition_flows.insert((expr_span.start, expr_span.end),
                if op == BinaryOp::Equal { (equal, other) } else { (other, equal) });
        }
        if stable && self.record_discriminator_guard(context,left,right).is_some(){self.remember_guard(context,expr_span,op,left,right);}
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



    // Appending ends each operand borrow before the next interpolation runs.
    // Incoming consumer loans remain active throughout the template.
    fn template_literal(&mut self, context: &mut Context, parts: &mut [Expr], loans: &Loans) -> Result<Type, Diagnostic> {
        for part in parts {
            let actual = self.expression(context, part, Mode::Read, loans, None)?;
            self.finish_template_part(part, actual)?;
        }
        Ok(Type::String)
    }

    fn finish_template_part(&self, part: &Expr, actual: Type) -> Result<(), Diagnostic> {
            self.require_native_capability(actual, crate::native_types::Capability::StringConversion, part.span, true)?;
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
        Ok(())
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
    fn pattern_selection_hint(&self,context:&Context,value:&Expr)->Option<Type>{self.type_hint_work(context,type_hints::Work::Pattern(value))}

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

    fn coalesce_hint(&self,context:&Context,left:&Expr,right:&Expr,span:Span,undefined_only:bool)->Option<Type>{
        self.type_hint_work(context,type_hints::Work::Coalesce{left,right,span,undefined_only})
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
        self.truthy_condition(context, predicate);
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
        let entry_loans = context.capture_loans.clone();
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
            entry_loans,
            before_owners: context.capture_loan_owners.clone(),
            after_left_owners: Default::default(),
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
        let (hint, assignment_alias, borrowed, infer_nullable_closure) = (plan.hint, plan.assignment_alias, plan.borrowed, plan.infer_nullable_closure);
        let else_flow = std::mem::take(&mut plan.else_flow);
        let before = &plan.before;
        self.require(left, hint, consequent.span)?;
        let after_left = before
            .iter()
            .map(|(id, _)| Availability::of(&context.bindings[*id]))
            .collect::<Vec<_>>();
        let after_left_flow = context.flow.clone();
        plan.after_left_owners = context.capture_loan_owners.clone();
        context.capture_loan_owners = plan.before_owners.clone();
        let left_loans = crate::capture_loan_identity::Boundary::capture(&plan.entry_loans)
            .added(&context.capture_loans);
        context.capture_loans.clone_from(&plan.entry_loans);
        Self::restore_live_capture_loans(context);
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
        let ConditionalPlan { assignment_alias, borrowed, infer_nullable_closure, before, after_left, after_left_flow, after_left_owners, left_loans, .. } = *plan;
        context.capture_loan_owners.join(&after_left_owners);
        Self::restore_live_capture_loans(context);
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
        if left!=right && !infer_nullable_closure
            && let Some(joined)=self.common_native_view_storage(left,right,consequent.span)? {
            consequent.wrap_record_carrier(joined);
            alternate.wrap_record_carrier(joined);
            left=joined;right=joined;
        }
        if left!=right && !infer_nullable_closure
            && let Some(joined)=self.nullable_native_view_alternatives(left,right,consequent.span)? {
            for (value,ty) in [(&mut *consequent,left),(&mut *alternate,right)] {
                if ty==joined {continue;}
                if matches!(ty,Type::Union(_)) {value.wrap_record_carrier(joined);} else {value.inject(joined.union().expect("nullable view storage"));}
            }
            left=joined;right=joined;
        }
        if left!=right && !infer_nullable_closure
            && let Some(family)=self.common_owned_conditional_family(left,right,consequent.span)? {
            left=self.checked_stored_callable_value(context,consequent,family,left)?;
            right=self.checked_stored_callable_value(context,alternate,family,right)?;
        }
        if left!=right && !infer_nullable_closure
            && let Some(family)=self.common_borrowed_conditional_family(left,right,consequent.span)? {
            consequent.wrap_storage(family);consequent.ty=Some(Type::BorrowedCallable(family));
            alternate.wrap_storage(family);alternate.ty=Some(Type::BorrowedCallable(family));
            left=Type::BorrowedCallable(family);right=left;
        }
        if left != right && let Some(joined) = self.conditional_record_join(consequent, alternate, left, right)? {
            left = joined;
            right = joined;
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

    fn conditional_hint(&self,context:&Context,consequent:&Expr,alternate:&Expr)->Option<Type>{
        self.type_hint_work(context,type_hints::Work::Conditional(consequent,alternate))
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

    fn record_literal(&mut self, context: &mut Context, fields: &mut Vec<(Token, Expr)>, loans: &Loans, expected: Option<Type>, span: Span) -> Result<Type, Diagnostic> {
        let mut frame = self.prepare_record_work(fields, expected, span)?;
        for (key, value) in fields.iter_mut() {
            let (expected, child) = self.prepare_record_child(context, key, value, &mut frame)?;
            let actual = self.expression(context, value, Mode::Consume, loans, expected)?;
            self.finish_record_child(context, key, value, &mut frame, child, actual)?;
        }
        self.finish_record_work(context, fields, &mut frame, span)
    }

    /// Convert a concrete owned environment once, after checking its contract.
    fn stored_callable_value(&mut self, context: &mut Context, value: &mut Expr, family: usize, loans: &Loans) -> Result<Type, Diagnostic> {
        let frame = self.prepare_stored_value(context, value, family)?;
        let actual = self.expression(context, value, Mode::Consume, loans, frame.expected)?;
        self.finish_stored_value(context, value, frame, actual)
    }

    fn prepare_stored_value(&mut self, context: &Context, value: &Expr, family: usize) -> Result<continuations::StoredValueFrame, Diagnostic> {
        if matches!(value.kind, ExprKind::Conditional { .. }) {
            return Ok(continuations::StoredValueFrame { family, expected: Some(Type::StoredCallable(family)), convert: false });
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
        let inferred_generic_factory=matches!(&value.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Name(name) if !context.names.contains_key(name) && self.generic_functions.template(name).is_some_and(|template|template.result.is_none())));
        Ok(continuations::StoredValueFrame {
            family,
            expected: if inferred_generic_factory { None } else { Some(if stored { Type::StoredCallable(family) } else { expected }) },
            convert: true,
        })
    }

    fn finish_stored_value(&mut self, context: &mut Context, value: &mut Expr, frame: continuations::StoredValueFrame, actual: Type) -> Result<Type, Diagnostic> {
        if frame.convert { self.checked_stored_callable_value(context, value, frame.family, actual) } else { Ok(actual) }
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

    #[inline(never)]
    fn native_collection_method_hint(&self,id:usize,name:&str,span:Span)->Option<Type> {
        match self.program.records[id].native_type {
            Some(crate::native_signatures::NativeTypeId::OrderedSet)=>self.ordered_set_method_type(name,span).ok(),
            Some(crate::native_signatures::NativeTypeId::Heap)=>{
                let result=self.heap_method_type(id,name,span).ok()?;
                Some(if name=="peek" && !result.is_copy(){Type::BorrowedUnion(result.union()?)}else{result})
            },
            _=>{
                let result=self.deque_method_type(id,name,span).ok()?;
                Some(if matches!(name,"front"|"back"|"at") && !result.is_copy(){Type::BorrowedUnion(result.union()?)}else{result})
            },
        }
    }

    fn expression_type_hint(&self,context:&Context,expr:&Expr)->Option<Type>{self.type_hint_work(context,type_hints::Work::Expression(expr))}
    fn type_hint_ready(&self,context:&Context,expr:&Expr)->Option<Option<Type>>{
        if matches!(expr.kind, ExprKind::Selected { .. }) {
            return Some(expr.ty);
        }
        if let ExprKind::Project { narrowed, .. } = &expr.kind {
            return Some(Some(*narrowed));
        }
        if self.json_boolean_proof(context,expr).is_some(){return Some(Some(Type::Boolean));}
        if let Some(ty)=self.json_string_projection_type(context,expr){return Some(Some(ty));}
        if let Some((_, _, narrowed)) = self.flow_view(context, expr) {
            return Some(Some(narrowed));
        }
        if let Some(ty) = expr.ty {
            return Some(Some(ty));
        }
        if let ExprKind::Field(base, _) = &expr.kind
            && let ExprKind::Name(name) = &base.kind
            && !context.names.contains_key(name)
            && let Some(&id) = self.record_names.get(name)
            && let Some(definition) = &self.program.records[id].enum_definition {
            return Some(Some(Type::Enum(id, definition.kind)));
        }
        None
    }
    fn type_hint_leaf(&self,context:&Context,expr:&Expr)->Option<Type>{
        match &expr.kind{
            ExprKind::EnumMember{enumeration,..}=>self.program.records[*enumeration].enum_definition.as_ref().map(|definition|Type::Enum(*enumeration,definition.kind)),
            ExprKind::Match { .. } | ExprKind::Optional{..}|ExprKind::Selected{..}|ExprKind::SpreadCall(_)|ExprKind::SpreadValue(_)=>expr.ty,
            ExprKind::TypeAssertion{annotation,..}=>self.resolve_value_annotation(annotation,false).ok(),
            ExprKind::Inject{union,..}=>Some(Type::Union(*union)),
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
                .or_else(|| name.strip_prefix(crate::data_lowering::ACCESS).and_then(|owner| self.function_names.get(owner)).map(|&function| self.signatures[function].result))
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
            ExprKind::SetConstructor(_, Some(annotation)) | ExprKind::MapConstructor(_, Some(annotation)) | ExprKind::NativeConstructor(_, Some(annotation)) => self.resolve_value_annotation(annotation, false).ok(),
            ExprKind::SetConstructor(_, None) | ExprKind::MapConstructor(_, None) | ExprKind::NativeConstructor(_, None) => None,
            ExprKind::ArrayOf(_, Some(element)) => self
                .resolve_value_annotation(element, false)
                .ok()
                .and_then(|ty| self.constructor_array_type(ty, expr.span).ok()),
            ExprKind::Closure { .. } | ExprKind::Record(_) => None,            _=>unreachable!("structural hint is scheduled by the worklist"),
        }
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

    #[inline(never)]
    fn array_literal_element_hint(
        &mut self,
        context: &mut Context,
        values: &mut [Expr],
        expected: Option<Type>,
        span: Span,
    ) -> Result<Option<ArrayElement>, Diagnostic> {
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
        Ok(element)
    }

    #[inline(never)]
    fn finish_array_literal_element(
        &mut self,
        context: &mut Context,
        value: &mut Expr,
        actual: Type,
        element: &mut Option<ArrayElement>,
    ) -> Result<(), Diagnostic> {
        let actual = if element.is_none() { self.inferred_callable_array_element(context, value, actual)? } else { actual };
        let Type::Array(inferred) = Self::transform_array_type(actual, value.span)? else {
            unreachable!("array element conversion returns an array");
        };
        if let Some(expected) = element.map(ArrayElement::ty)
            && let Some(Type::StoredCallable(joined)) = self.common_native_view_storage(expected, actual, value.span)? {
            // Different callbacks that hold byte views share one joined family.
            *element = Some(ArrayElement::StoredCallable(joined, 0));
            return Ok(());
        }
        self.require(actual, element.map(ArrayElement::ty), value.span)?;
        if let Some(expected) = element.map(ArrayElement::ty)
            && actual != expected && self.scoped_native_storage_contract(actual, expected, value.span)? {
            *element = Some(inferred);
        }
        element.get_or_insert(inferred);
        Ok(())
    }

    #[inline(never)]
    fn finish_array_literal(
        &mut self,
        values: &mut [Expr],
        element: Option<ArrayElement>,
        span: Span,
    ) -> Result<Type, Diagnostic> {
        let element = element.ok_or_else(|| {
            Diagnostic::new(
                "array-type",
                "An empty array needs an array annotation or call/return context.",
                span,
            )
        })?;
        Ok(Type::Array(self.scoped_record_array_element(values, element, span)?))
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
        let mut pending = values.iter().collect::<Vec<_>>();
        while let Some(value) = pending.pop() {
            if let ExprKind::Array(values) = &value.kind {
                if values.is_empty() { return Ok(None); }
                pending.extend(values);
            }
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

    fn field_access(&mut self, context: &mut Context, base: &mut Expr, name: &str, mode: Mode, loans: &Loans, span: Span) -> Result<Type, Diagnostic> {
        match self.prepare_field_work(context, base, name, mode, loans, span)? {
            continuations::FieldDispatch::Ready(ty) => Ok(ty),
            continuations::FieldDispatch::Receiver { mode: base_mode, frame } => {
                let result = self.expression(context, base, base_mode, loans, None);
                self.finish_field_work(context, base, name, mode, loans, span, frame, result)
            }
        }
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
        debug_assert_eq!(ty.scalar_base(),Type::String);
        let mut active=loans.clone();
        for root in Self::loan_roots(base,Mode::Read){active.entry(root).or_default().push((Mode::Read,base.span));}
        let actual=self.expression(context,index,Mode::Copy,&active,Some(Type::Int))?;
        self.require(actual,Some(Type::Int),index.span)?;
        Ok(Type::String)
    }

    fn index_expression(&mut self,context:&mut Context,expr:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic> {
        let ExprKind::Index(base,index)=&mut expr.kind else {unreachable!()};
        if let ExprKind::String(name) = &index.kind
            && matches!(self.expression_type_hint(context,base), Some(Type::Record(record)) if !self.program.records[record].is_tuple())
        {
            let result = self.field_access(context,base,name,mode,loans,expr.span)?;
            let name = name.clone();
            let span = base.span;
            let receiver = std::mem::replace(base,Box::new(Expr::new(ExprKind::Undefined,span)));
            expr.kind = ExprKind::Field(receiver,name);
            return Ok(result);
        }
        if !Self::optional_hint_boundary(base) && self.expression_type_hint(context,base).map(Type::scalar_base)==Some(Type::String) {
            return self.string_index(context,base,index,loans);
        }
        let result=if Self::named_storage(base) {self.array_index(context,base,index,mode,loans,expr.span)} else {self.temporary_array_index(context,base,index,mode,loans,expected)}?;
        if let Some((name,_))=self.checked_tuple_index(base.ty.expect("checked index receiver"),index)? {
            let span=base.span;let receiver=std::mem::replace(base,Box::new(Expr::new(ExprKind::Undefined,span)));
            expr.kind=ExprKind::Field(receiver,name);
        }
        Ok(result)
    }

    fn temporary_array_index(&mut self, context: &mut Context, base: &mut Expr, index: &mut Expr, mode: Mode, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let array_context = expected.and_then(ArrayElement::from_value_type).map(Type::Array);
        let borrowed = self.borrowed_place_hint(context, base) || self.borrowed_query_projection_hint(context, base);
        let ty = self.expression(context, base, if borrowed { Mode::Read } else { Mode::Consume }, loans, array_context)?;
        let element = match self.finish_temporary_index_receiver(context, base, index, mode, borrowed, ty)? {
            Ok(field) => return Ok(field), Err(element) => element,
        };
        let mut active = loans.clone();
        if borrowed {
            for root in Self::loan_roots(base, Mode::Read) { active.entry(root).or_default().push((Mode::Read, base.span)); }
        }
        let actual = self.expression(context, index, Mode::Copy, &active, Some(Type::Int))?;
        self.require(actual, Some(Type::Int), index.span)?;
        Ok(element)
    }

    fn finish_temporary_index_receiver(&mut self, context: &mut Context, base: &mut Expr, index: &Expr, mode: Mode, borrowed: bool, ty: Type) -> Result<Result<Type, Type>, Diagnostic> {
        let ty = self.recursive_array_receiver(context, base, ty);
        if let Some((_,field))=self.checked_tuple_index(ty,index)? {
            if borrowed && (mode==Mode::Mutate || mode==Mode::Consume && !field.is_copy()) {return Err(Diagnostic::new("borrowed-place","An owned tuple position of an assignment result is borrowed; use clone().",base.span));}
            return Ok(Ok(field));
        }
        let Type::Array(element) = ty else {
            return Err(Diagnostic::new(
                "type",
                "Indexing requires an array, a tuple or a string.",
                base.span,
            ));
        };
        if borrowed && (mode==Mode::Mutate || mode==Mode::Consume && !element.ty().is_copy()) {
            return Err(Diagnostic::new(
                "borrowed-place",
                "An owned element of an assignment result is borrowed; use clone().",
                base.span,
            ));
        }
        Ok(Err(element.ty()))
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
    fn array_index_receiver(&mut self, context: &mut Context, base: &mut Expr, index: &mut Expr, mode: Mode, loans: &Loans, span: Span) -> Result<Result<Type, Type>, Diagnostic> {
        Self::prepare_named_index(base)?;
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        self.finish_named_index_receiver(context, base, index, mode, loans, span, ty)
    }

    fn prepare_named_index(base: &Expr) -> Result<(), Diagnostic> {
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
        Ok(())
    }

    fn finish_named_index_receiver(&mut self, context: &mut Context, base: &mut Expr, index: &mut Expr, mode: Mode, loans: &Loans, span: Span, ty: Type) -> Result<Result<Type, Type>, Diagnostic> {
        let ty = self.recursive_array_receiver(context, base, ty);
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
        let mut roots = Vec::new();
        let mut work = vec![expr];
        while let Some(expr) = work.pop() {
            match &expr.kind {
                ExprKind::Sequence(values)=>{if let Some(last)=values.last(){work.push(last);}},
                ExprKind::Optional { body, .. } => work.push(body),
                ExprKind::Selected { source, .. } => roots.extend(source.roots.iter().copied()),
                ExprKind::CoalesceAssign { target, .. } => work.push(target),
                ExprKind::Name(_) => {
                    if let Some(id) = expr.binding {
                        if context.bindings[id].view_roots.is_empty() {
                            roots.push(id);
                        } else {
                            roots.extend(context.bindings[id].view_roots.iter().copied());
                        }
                    }
                }
                ExprKind::Inject { value, .. }
                    if matches!(expr.ty, Some(Type::BorrowedUnion(_))) =>
                {
                    work.push(value)
                }
                ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    value,
                    ..
                } => work.push(value),
                ExprKind::Binary {
                    op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                    left,
                    right,
                    ..
                } => {
                    work.push(right);
                    work.push(left);
                }
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. }
                | ExprKind::Field(value, _)
                | ExprKind::Index(value, _) => work.push(value),
                ExprKind::Call(callee, _)
                    if matches!(
                        expr.target,
                        Some(
                            CallTarget::HeapMethod(HeapMethod::Peek) | CallTarget::DequeMethod(DequeMethod::Front | DequeMethod::Back | DequeMethod::At) | CallTarget::MapMethod(MapMethod::Get) | CallTarget::ArrayAt | CallTarget::ArrayFind | CallTarget::ArrayFindLast
                        )
                    ) =>
                {
                    if let ExprKind::Field(base, _) = &callee.kind {
                        work.push(base);
                    }
                }
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    work.push(alternate);
                    work.push(consequent);
                }
                _ => {}
            }
        }
        roots.sort_unstable();
        roots.dedup();
        roots
    }

    fn stable_at_receiver(expr: &Expr) -> bool {
        let mut work = vec![expr];
        while let Some(expr) = work.pop() {
            match &expr.kind {
                ExprKind::Sequence(values) => {
                    let Some(last) = values.last() else { return false; };
                    work.push(last);
                }
                ExprKind::Conditional { consequent, alternate, .. } => {
                    work.push(alternate);
                    work.push(consequent);
                }
                _ if !Self::named_storage(expr) => return false,
                _ => {}
            }
        }
        true
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
        let contract = self.scoped_array_element_contract(base, element);
        for arg in args.iter_mut() {
            let actual = if let Type::StoredCallable(family) = element.ty() {
                self.stored_callable_value(context, arg, family, &active)?
            } else {
                self.expression(context, arg, Mode::Consume, &active, Some(contract))?
            };
            self.require(actual, Some(contract), arg.span)?;
            self.widen_scoped_array_storage(context, base, actual, arg.span)?;
            self.append_target_capture_loans(context, base, arg);
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
            let expected = if index < 2 { Type::Int } else { self.scoped_array_element_contract(base, element) };
            let mode = if index < 2 { Mode::Copy } else { Mode::Consume };
            let actual = if index >= 2
                && let Type::StoredCallable(family) = expected
            {
                self.stored_callable_value(context, arg, family, &active)?
            } else {
                self.expression(context, arg, mode, &active, Some(expected))?
            };
            self.require(actual, Some(expected), arg.span)?;
            if index >= 2 {
                self.widen_scoped_array_storage(context, base, actual, arg.span)?;
                self.append_target_capture_loans(context, base, arg);
            }
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
        Ok(if matches!(base.kind, ExprKind::Name(_)) { context.bindings[base.binding.expect("checked named array")].ty } else { Type::Array(element) })
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
        let widening = !remove && (Self::indexed_array_root(base).is_some() || Self::direct_fields(base).is_some()) && matches!(element, ArrayElement::Record(id, _) if self.aggregate_callable_facts(Type::Record(self.scoped_records.declaration(id))).scoped);
        let argument_type = if remove { Type::Int } else { self.scoped_array_element_contract(base, element) };
        // A new callback is checked against the written contract; the array then
        // takes the family that joins it with the callbacks it already holds.
        let argument_type = if remove { argument_type } else { self.native_view_element_contract(context, base, argument_type, &args[0]) };
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
        if widening { self.widen_scoped_array_storage(context, base, actual, span)?; }
        if !remove { self.widen_native_view_array_storage(context, base, actual, span)?; }
        if !remove {
            self.append_target_capture_loans(context, base, &args[0]);
            Self::append_native_view_element(context, base, &args[0])?;
        }
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
            Some(ty @ (Type::StoredCallable(_) | Type::BorrowedCallable(_))) => self.callables.storage.callback_declaration(ty).and_then(|id| self.callables.callbacks[id].params.iter().position(|parameter| parameter.rest)),
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

    // Reduction owns its accumulator independently from the borrowed source.
    // Keep callback and argument temporaries outside recursive expression frames.
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
        let string_proof=self.json_string_proof(context,base);
        if let Some(proof)=string_proof.clone(){Self::use_json_string_proof(context,proof);}
        let ty = self.expression(context, base, Mode::Read, loans, None)?;
        let string_receiver=ty.scalar_base() == Type::String || string_proof.is_some() || self.is_string_view(context,base);
        let element_type = if string_receiver {
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
        let needle_proof=if string_receiver{self.json_string_proof(context,&args[0])}else{None};
        if let Some(proof)=needle_proof.clone(){Self::use_json_string_proof(context,proof);}
        let borrowed_needle=string_receiver && (needle_proof.is_some() || self.is_string_view(context,&args[0]));
        let actual = self.expression(
            context,
            &mut args[0],
            Mode::Read,
            &active,
            if borrowed_needle{None}else{Some(element_type)},
        )?;
        if !borrowed_needle{self.require(actual, Some(element_type), args[0].span)?;}
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
            self.require_native_capability(ty, crate::native_types::Capability::Equality, argument.span, true)?;
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
    fn prepare_closure_method_call(
        &mut self,
        context: &mut Context,
        binding: usize,
        origin: usize,
        callee: &mut Expr,
        args: &mut Vec<Expr>,
        loans: &Loans,
        span: Span,
        expected: Option<Type>,
    ) -> Result<CallDispatch, Diagnostic> {
        let (origin, once) = match self.closures[origin].forward {
            Some(ClosureForward::Once(inner)) => (inner, true),
            _ => (origin, false),
        };
        if self.deferred_generic_values.contains_key(&origin) {
            return Ok(CallDispatch::Deferred(deferred_generic_call_work::Request { binding: Some(binding), origin, once, expected, checked_receiver: false }));
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
        Ok(CallDispatch::Method { binding, origin, params, invocation })
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
        self.resolved_prepared_call(context, index, args, span)
    }

    fn resolved_prepared_call(&mut self, context: &Context, index: usize, args: &[Expr], span: Span) -> Result<(usize, Type), Diagnostic> {
        let result = if let Some(target) = self.checked[index].as_ref() {
            target.result
        } else if let Some(&result) = self.known_results.get(&index) {
            result
        } else if self.callable_result_type(self.signatures[index].result) {
            if self.discovering_results {
                self.note_result_wait(index, span);
                self.callable_result_shape(self.signatures[index].result)
            } else if context.template {
                self.callable_result_shape(self.signatures[index].result)
            } else {
                return Err(Self::pending_result(span));
            }
        } else {
            self.signatures[index].result
        };
        // Recursive factories share their component even while result discovery
        // still sees the written nominal return annotation.
        let result = self.scoped_records.recursive_parameter(result);
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
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
        capture_boundary.unwind(&mut context.capture_loans);
        let (function, result) = self.resolved_call(context, function, args, span)?;
        self.release_record_call_captures(context, function, args);
        self.retain_record_parameter_borrows(context, function, args)?;
        Ok((function, result))
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn console_inspectable(&self, ty: Type) -> bool {
        let mut pending = vec![ty];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::Literal(_,_) | Type::String | Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined => {},
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
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
            if string_conversion {
                self.require_native_capability(actual, crate::native_types::Capability::StringConversion, argument.span, true)?;
            }
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
        capture_boundary.unwind(&mut context.capture_loans);
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
            let ExprKind::Unary { value, .. } = codes.pop().expect("one spread argument").into_kind() else {
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
    fn prepare_call_work(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
        receiver_prepared: bool,
    ) -> Result<CallDispatch, Diagnostic> {
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
            return Ok(CallDispatch::Declared { resolve_completed: false });
        }
        let result = self.expression_call_other(context, expr, loans, expected, receiver_prepared)?;
        if let CallDispatch::Ready(ty) = result { self.resolve_completed_capture_invocation(context, expr, ty); }
        Ok(result)
    }

    pub(super) fn call_preparation_receiver(expr: &mut Expr) -> &mut Expr {
        let ExprKind::Call(callee, _) = &mut expr.kind else { unreachable!("call receiver preparation") };
        let mut receiver = callee.as_mut();
        while matches!(receiver.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _)) {
            receiver = match &mut receiver.kind { ExprKind::Field(base, _) | ExprKind::Index(base, _) => base, _ => unreachable!() };
        }
        receiver
    }

    fn expression_call_other(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
        receiver_prepared: bool,
    ) -> Result<CallDispatch, Diagnostic> {
        if !receiver_prepared {
            self.stage_assignment_callee(context, expr)?;
            if let ExprKind::Call(callee, _) = &mut expr.kind { self.global_number_name(context, callee, true); }
            let receiver = Self::call_preparation_receiver(expr);
            if matches!(receiver.kind, ExprKind::Call(..) | ExprKind::OptionalChain(_) | ExprKind::Unary { op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected, .. }) {
                return Ok(CallDispatch::PrepareReceiver);
            }
        }
        if Self::map_group_by_callee(context,expr) {return self.prepare_group_by_transform(context,expr,expected).map(CallDispatch::Transform);}
        if Self::array_from_callee(context, expr) {
            self.array_from(context, expr)?;
        }
        if matches!(&expr.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(_,name) if name=="map#arrayFrom")) {
            return Ok(CallDispatch::Transform(continuations::transform::Request { options:(false,false,Mode::Read,false,false,true,true),target:CallTarget::ArrayMap,kind:continuations::transform::ResultKind::ArrayFrom,expected }));
        }
        if matches!(&expr.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(base,_) if self.ordered_set_receiver(context,base).is_some())){return self.ordered_set_method_call(context,expr,loans,expected).map(CallDispatch::Ready);}
        if matches!(&expr.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Field(base, _) if self.heap_receiver(context, base).is_some())) { return self.heap_method_call(context, expr, loans, expected).map(CallDispatch::Ready); }
        if matches!(&expr.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Field(base, _) if self.deque_receiver(context, base).is_some())) { return self.deque_method_call(context, expr, loans, expected).map(CallDispatch::Ready); }
        if let Some(ty) = self.result_call(context, expr, loans, expected)? { return Ok(CallDispatch::Ready(ty)); }
        if let Some(ty) = self.thread_call(context, expr, loans, expected)? { return Ok(CallDispatch::Ready(ty)); }
        if let Some(ty) = self.schema_call(context, expr, loans, expected)? { return Ok(CallDispatch::Ready(ty)); }
        if self.map_method_receiver(context, expr) || self.set_method_receiver(context, expr) {
            if let Some(request) = self.transform_request(context, expr, expected, true) { return Ok(CallDispatch::Transform(request)); }
        }
        if self.map_method_receiver(context, expr) { return self.map_method_call(context, expr, loans, expected).map(CallDispatch::Ready); }
        if self.set_method_receiver(context, expr) { return self.set_method_call(context, expr, loans, expected).map(CallDispatch::Ready); }
        self.array_values(context, expr)?;
        self.string_at(context, expr);
        self.string_raw(context, expr)?;
        self.string_from_codes(context, expr);
        self.string_code_point_at(context, expr)?;
        self.string_locale_case(context, expr)?;
        if matches!(&expr.kind, ExprKind::Call(_, args) if args.iter().any(|arg| matches!(arg.kind, ExprKind::Unary {op:UnaryOp::Spread,..}))) {
            return Ok(CallDispatch::Spread);
        }
        if let ExprKind::Call(callee, _) = &expr.kind
            && matches!(callee.kind, ExprKind::Field(_, _))
            && self
                .expression_type_hint(context, callee)
                .is_some_and(|ty| matches!(ty, Type::StoredCallable(_)))
        {
            return self.prepare_builtin_call_work(context, expr, loans, expected);
        }
        if let ExprKind::Call(callee, _) = &expr.kind
            && self.flow_view(context, callee).is_some_and(|(_, _, payload)| matches!(payload, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_))) {
            return Ok(CallDispatch::Conditional { publish_type: false });
        }
        // Concat receiver and arguments suspend on the shared expression stack.
        if matches!(&expr.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Field(_, name) if name == "concat"))
        {
            return Ok(CallDispatch::Concat);
        }
        let ordinary_name = matches!(&expr.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Name(name)
                if !matches!(name.as_str(), "String" | "Boolean" | "Number" | "clone")));
        if ordinary_name {
            self.prepare_generic_call(context, expr, expected)?;
            self.prepare_named_call(context, expr, loans, expected)
        } else {
            self.prepare_builtin_call_work(context, expr, loans, expected)
        }
    }

    // Schema dispatch temporaries must not stay live across ordinary nested calls.
    #[inline(never)]
    fn schema_call(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        if let Some(ty)=self.schema_safe_parse_outcome(context, expr, loans, expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_context_issue(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_recursive_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_text_decode(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_export(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_parse_call(context, expr, loans, expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_constraint_options(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_message_constraint(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_constructor_options(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_pipeline_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_pipeline_validation(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_default_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_presence_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_object_policy(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_union_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_enum_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_literal_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_object_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_collection_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_array_constructor(context,expr,loans,expected)? {return Ok(Some(ty));}
        if let Some(ty)=self.schema_composite_validation(context,expr,loans,expected)? {return Ok(Some(ty));}
        self.schema_method_call(context, expr)?;
        Ok(None)
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

    fn prepare_named_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<CallDispatch, Diagnostic> {
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
        Ok(CallDispatch::Declared { resolve_completed: true })
    }

    fn bound_named_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<CallDispatch, Diagnostic> {
        if let Some(result) = self.program_callback_call(context, expr, loans) {
            return result.map(CallDispatch::Ready);
        }
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
                expr.ty = Some(native_type(binding.result, &self.program.records));
                return Ok(CallDispatch::Ready(native_type(binding.result, &self.program.records)));
            }
            if binding.result == crate::native_bindings::NativeType::SharedCallback {
                return self.shared_callback_call(context, expr, loans).map(CallDispatch::Ready);
            }
            if let Some((local, _)) = stored_native {
                let invocation = context.bindings[local]
                    .callback_invocation
                    .unwrap_or(Mode::Read);
                Self::access(context, local, invocation, callee.span, loans)?;
                callee.binding = Some(local);
                callee.ty = Some(context.bindings[local].ty);
            }
            if binding.module == "std:testing-runtime" && crate::stdlib::testing::program::operation(name).is_some()
                && self.program.records.iter().any(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry)) {
                return self.program_testing_call(context, expr, id, loans).map(CallDispatch::Ready);
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
            let contracts = (0..args.len()).map(|position| {
                let parameter = binding.parameters.get(position).or_else(|| binding.parameters.last().filter(|p| p.rest)).expect("checked native arity");
                // A parameter with typed alternatives takes the argument's own native type.
                let alternative = self.expression_type_hint(context, &args[position]).filter(|ty| matches!(ty, Type::Record(record)
                    if self.program.records[*record].native_type.is_some_and(|id| binding.argument_alternatives(position).contains(&id))));
                if let Some(ty) = alternative {
                    return Ok(ArgumentContract { ty, mode: parameter.access, omission: crate::ParameterOmission::Required, rest: false });
                }
                Ok(ArgumentContract { ty: self.native_parameter_type(parameter.ty)?,
                    mode: parameter.access, omission: crate::ParameterOmission::Required, rest: false })
            }).collect::<Result<Vec<_>, Diagnostic>>()?;
            self.check_call_arguments(context, args, &contracts, loans)?;
            if let Some(contract) = binding.returned_borrow {
                Self::native_view_source_loans(context, &args[contract.source_parameter], contract)?;
                context.native_view_calls = true;
            }
            let result = native_type(binding.result, &self.program.records);
            if binding.returned_borrow.is_some_and(|contract| matches!(contract.slot, crate::native_return_borrows::ReturnSlot::ResultOk | crate::native_return_borrows::ReturnSlot::ResultOkOptional)) {
                let (union,variants) = self.result_variant_records(result).expect("validated native Result borrow");
                let contract = binding.returned_borrow.expect("validated native Result borrow");
                let optional_payload = (contract.slot == crate::native_return_borrows::ReturnSlot::ResultOkOptional).then(|| {
                    self.program.records.iter().position(|record| record.native_type == Some(contract.payload)).expect("prepared optional native view")
                });
                context.native_view_results.get_or_insert_with(Default::default).insert(union,(variants[0],optional_payload));
            }
            expr.target = Some(CallTarget::NativeBinding(id));
            expr.ty = Some(result);
            return Ok(CallDispatch::Ready(result));
        }
        if self.discovering_results && context.pending_names.contains(name) {
            return Err(Self::pending_result(span));
        }
        if let Some(&binding) = context.names.get(name) {
            if let Type::Closure(origin) = context.bindings[binding].ty
                && let Some(ClosureForward::Function(function)) = self.closures[origin].forward
            {
                let (function, result) =
                    self.forward_call(context, binding, function, callee, args, loans, span)?;
                self.remember_native_function_returns(context,function,span)?;
                expr.target = Some(CallTarget::ForwardFunction(function));
                expr.ty = Some(result);
                return Ok(CallDispatch::Ready(result));
            }
            if let Type::Closure(origin) = context.bindings[binding].ty
                && let Some(ClosureForward::Once(inner)) = self.closures[origin].forward
                && let Some(ClosureForward::Function(function)) = self.closures[inner].forward
            {
                let (function, result) =
                    self.forward_call(context, binding, function, callee, args, loans, span)?;
                self.remember_native_function_returns(context,function,span)?;
                expr.target = Some(CallTarget::ForwardFunction(function));
                expr.ty = Some(result);
                return Ok(CallDispatch::Ready(result));
            }
            if let Type::Closure(origin) = context.bindings[binding].ty
                && (self.closures[origin].polymorphic
                    || matches!(self.closures[origin].forward, Some(ClosureForward::Once(inner)) if self.closures[inner].polymorphic))
            {
                return self.prepare_closure_method_call(context, binding, origin, callee, args, loans, span, expected);
            }
            let (params, result, invocation, target) = match context.bindings[binding].ty {
                Type::StoredCallable(family) | Type::BorrowedCallable(family) => {
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
            return Ok(CallDispatch::Bound { binding, params, result, invocation, target });

        }
        unreachable!("bound named-call dispatch");
    }

    fn finish_bound_call(&mut self, context: &mut Context, expr: &mut Expr, binding: usize, result: Type, target: CallTarget) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("bound call completion") };
            let result = if let Type::Callable(contract) = result {
                if context.template || self.discovering_results {
                    self.result_shape(contract)
                } else {
                    return Err(Self::pending_result(span));
                }
            } else {
                result
            };
            if let Type::Closure(origin) = context.bindings[binding].ty {
                self.release_record_closure_captures(context, origin, args);
                self.retain_record_closure_borrows(context, origin, binding, args, callee.span)?;
            }
            // Preserve origin without treating the callee name as an evaluated value.
            if let Type::Closure(origin)=context.bindings[binding].ty {
                self.remember_native_closure_returns(context,origin,None,span)?;
                expr.callable_origin=Some(CallableOrigin::Closure(origin));
            }
            if let Type::StoredCallable(family)|Type::BorrowedCallable(family)=context.bindings[binding].ty {
                self.remember_native_family_returns(context,family,span)?;
                expr.callable_origin=Some(CallableOrigin::Storage(family));
            }
            if let Type::Callback(contract)=context.bindings[binding].ty {
                expr.callable_origin=Some(CallableOrigin::Callback(contract));
            }
            expr.target = Some(target);
            expr.ty = Some(result);
            Ok(result)
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
        if matches!(ty, crate::native_bindings::NativeType::OwnedVoidCallback | crate::native_bindings::NativeType::ProgramVoidCallback | crate::native_bindings::NativeType::ProgramReusableVoidCallback) {
            let annotation = crate::type_syntax::parse("Owned<Function<void>>")?;
            let mut callback = crate::callable_annotations::callback(&annotation).expect("native callback annotation");
            if ty == crate::native_bindings::NativeType::ProgramVoidCallback { callback.kind = CallbackKind::Program; }
            if ty == crate::native_bindings::NativeType::ProgramReusableVoidCallback { callback.kind = CallbackKind::ProgramReusable; }
            self.resolve_callback(&callback)
        } else {
            Ok(native_type(ty, &self.program.records))
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
        let selected_capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
                    active.argument(binding, mode, arg);
                }
            }
        }
        selected_capture_boundary.unwind(&mut context.capture_loans);
        Ok(())
    }

    #[inline(never)]
    #[allow(clippy::type_complexity)]
    fn prepare_declared_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
    ) -> Result<usize, Diagnostic> {
        let ExprKind::Call(callee, _) = &mut expr.kind else {
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
        Ok(index)
    }

    fn declared_call_contract(&mut self, context: &mut Context, expr: &mut Expr) -> Result<(usize, Vec<(usize, Mode)>, Vec<ArgumentContract>), Diagnostic> {
        let index = self.prepare_declared_call(context, expr)?;
        self.function(index)?;
        self.completed_declared_call_contract(context, expr, index)
    }

    fn completed_declared_call_contract(&mut self, context: &mut Context, expr: &mut Expr, index: usize) -> Result<(usize, Vec<(usize, Mode)>, Vec<ArgumentContract>), Diagnostic> {
        let ExprKind::Call(callee, _) = &mut expr.kind else { unreachable!("declared call dispatch") };
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
        Ok((index, capture_slots, params))
    }

    #[inline(never)]
    fn finish_declared_call(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        index: usize,
        capture_slots: &[(usize, Mode)],
        params: &[ArgumentContract],
        loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("declared call dispatch");
        };
        // A function still being checked is on a recursive path. A new closure
        // around a borrowed callback would give every level a larger callback
        // type, which has no finite native instantiation. Owned callbacks have
        // their own specialization limit check.
        if self.visiting.get(index).copied().unwrap_or(false) || self.active_recursive_functions.contains(&index) {
            for argument in args.iter() {
                let mut value = argument;
                while let ExprKind::Unary { value: inner, .. } | ExprKind::Inject { value: inner, .. } = &value.kind { value = inner; }
                if matches!(value.kind, ExprKind::Closure { .. })
                    && let Some(Type::Closure(origin)) = value.ty
                    && self.closures.get(origin).is_some_and(|closure| closure.captures.iter().any(|capture| matches!(closure.function.bindings[capture.inner].ty, Type::Callback(id) if !self.callables.callbacks[id].kind.is_owned())))
                {
                    return Err(Diagnostic::new(
                        "callback-escape",
                        "A recursive call cannot wrap a borrowed callback in a new closure: every level would need a larger callback type. Pass the callback itself, or build the closure before the recursion starts.",
                        value.span,
                    ));
                }
            }
        }
        if !capture_slots.is_empty() {
            self.named_capture_arguments(context, capture_slots, args, params, callee.span, loans)?;
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
        self.release_record_call_captures(context,index,args);
        self.retain_record_parameter_borrows(context,index,args)?;
        self.retain_caller_error_origins(context, index, args);
        self.remember_native_function_returns(context,index,span)?;
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
                    let scalar = |ty: Type| matches!(ty.scalar_base(), Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined | Type::String | Type::Enum(_, _));
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
                if !matches!(ty.scalar_base(), Type::String | Type::Record(_) | Type::Set(_) | Type::Map(_) | Type::Array(_))
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
                let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
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
                    self.require_native_capability(actual, crate::native_types::Capability::Inspection, argument.span, true)?;
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
                capture_boundary.unwind(&mut context.capture_loans);
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
                let json_string=self.json_string_proof(context,base).is_some() || self.is_string_view(context,base);
                let ty = self.collection_search(context, base, args, loans, span, name)?;
                expr.target = Some(match (if json_string{Some(Type::String)}else{base.ty.map(Type::scalar_base)}, name.as_str()) {
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
        self.finish_builtin_result(context, expr, ty)
    }

    pub(super) fn finish_builtin_result(&mut self, context: &mut Context, expr: &mut Expr, ty: Type) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!("builtin completion") };
        if let ExprKind::Field(base, _) = &callee.kind
            && matches!(expr.target, Some(CallTarget::ArrayPop | CallTarget::ArrayShift
                | CallTarget::ArraySplice | CallTarget::ArrayUnshift | CallTarget::ArrayRemove
                | CallTarget::ArraySort | CallTarget::ArrayReverse | CallTarget::ArrayFill
                | CallTarget::ArrayCopyWithin))
        {
            Self::collapse_array_capture_slots(context, base);
        }
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
            if self.native_borrow_types && self.native_shape_contains_view(ty) {
                return Err(Diagnostic::new(
                    "clone-type",
                    "Borrowed views cannot be cloned. For byte views, use copy(view, start, end) to make independent Bytes.",
                    span,
                ));
            }
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
        if !context.names.contains_key(name)
            && let Some(owner) = name.strip_prefix(crate::data_lowering::ACCESS)
            && let Some(&function) = self.function_names.get(owner)
        {
            let ty = self.signatures[function].result;
            let token = Token { text: name.clone(), span, kind: crate::syntax::TokenKind::Symbol };
            let binding = Self::bind(context, &token, ty, false)?;
            context.bindings[binding].borrowed_place = Some(span);
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
            return self.deferred_generic_value(context, expr, loans);
        }
        let index = *context
            .names
            .get(name)
            .ok_or_else(|| Diagnostic::new("name", format!("Unknown value {name:?}."), span))?;
        let callback_argument = matches!(expected, Some(Type::Callback(_)));
        let observation =
            self.observing_values && mode == Mode::Read && !context.is_capture(index);
        if let Some(kind) = context.bindings[index].callback_kind {
            if !kind.is_owned()
                && context.returning_closure && !callback_argument
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
            ExprKind::Match { .. } | ExprKind::TypeAssertion {..}
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
            ExprKind::Unary { .. } => {
                let (ty, finish) = self.expression_unary(context, expr, loans, expected)?;
                if !finish {
                    return Ok(ty);
                }
                ty
            }
            ExprKind::Binary { .. } => {
                let (ty, finish) = self.expression_binary(context, expr, loans, expected)?;
                if !finish {
                    return Ok(ty);
                }
                ty
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
            ExprKind::Sequence(_) | ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) | ExprKind::NativeConstructor(_, _) | ExprKind::Array(_) | ExprKind::ArrayOf(_, _) | ExprKind::Index(_, _) => {
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

    /// Checked outside `expression_inner` so that nested operands recurse through a
    /// small frame. The flag is false when the result type is already recorded.
    #[inline(never)]
    fn expression_unary(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, bool), Diagnostic> {
        let span = expr.span;
        let ExprKind::Unary { op, operator, value } = &mut expr.kind else {
            unreachable!("unary dispatch");
        };
        if let UnaryOp::VariantIs(variant) = *op {
            let ty = self.expression(context, value, Mode::Read, loans, None)?;
            if !ty.union().is_some_and(|union| self.unions.members(union).contains(&variant)) {
                return Err(Diagnostic::new("match-pattern", "The selected variant is not part of this value's type.", *operator));
            }
            self.require(Type::Boolean, expected, span)?;
            return Ok((Type::Boolean, false));
        }
        {
            if matches!(op, UnaryOp::Typeof | UnaryOp::Void) {
                return self.unary_inspection(context, expr, loans, expected);
            }
            if *op == UnaryOp::Not
                && self
                    .expression_type_hint(context, value)
                    .is_some_and(|ty| !matches!(ty, Type::Boolean | Type::Void))
            {
                return self.unary_truth_test(context, expr, loans, expected);
            }
            let _ = (span, operator);
            self.unary_operator(context, expr, loans, expected)
        }
    }

    #[inline(never)]
    #[allow(unused_variables)]
    fn unary_inspection(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, bool), Diagnostic> {
        let span = expr.span;
        let ExprKind::Unary { op, operator, value } = &mut expr.kind else {
            unreachable!("unary dispatch");
        };
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
                    let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
                    if unresolved {
                        value.ty = Some(Type::Undefined);
                    } else {
                        let observing = self.observing_values;
                        self.observing_values = true;
                        let result = self.observed_expression(context, value, loans);
                        self.observing_values = observing;
                        result?;
                    }
                    capture_boundary.unwind(&mut context.capture_loans);
                    let result = if *op == UnaryOp::Typeof {
                        Type::String
                    } else {
                        Type::Undefined
                    };
                    expr.ty = Some(result);
                    return Ok((result, false));
                    }

    #[inline(never)]
    #[allow(unused_variables, unreachable_code)]
    fn unary_truth_test(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, bool), Diagnostic> {
        let span = expr.span;
        let ExprKind::Unary { op, operator, value } = &mut expr.kind else {
            unreachable!("unary dispatch");
        };
                if *op == UnaryOp::Not
                    && matches!(
                        self.expression_type_hint(context, value),
                        Some(Type::Null | Type::Undefined)
                    )
                {
                    self.expression(context, value, Mode::Copy, loans, None)?;
                    expr.ty = Some(Type::Boolean);
                    return Ok((Type::Boolean, false));
                }
                if *op == UnaryOp::Not
                    && let Some(ty) = self.expression_type_hint(context, value)
                    && !matches!(ty, Type::Boolean | Type::Void)
                {
                    // A nullable variable or field becomes presence tests, which
                    // the ordinary boolean path below narrows.
                    if self.nullable_place(ty, value) {
                        self.truthy_condition(context, value);
                        let actual = self.expression(context, value, Mode::Copy, loans, Some(Type::Boolean))?;
                        self.require(actual, Some(Type::Boolean), *operator)?;
                        let (yes, no) = Self::condition_outcomes(context, value);
                        context.condition_flows.insert((span.start, span.end), (no, yes));
                        expr.ty = Some(Type::Boolean);
                        return Ok((Type::Boolean, false));
                    }
                    // `!value` tests truthiness; the value is only read.
                    let mode = if ty.is_copy() { Mode::Copy } else { Mode::Read };
                    let actual = self.expression(context, value, mode, loans, None)?;
                    if actual == Type::Void {
                        return Err(Diagnostic::new("type", "A function call without a return value has no truth value.", value.span));
                    }
                    expr.ty = Some(Type::Boolean);
                    return Ok((Type::Boolean, false));
                }
        unreachable!("the dispatch selects a non-boolean operand")
    }

    #[inline(never)]
    #[allow(unused_variables)]
    fn unary_operator(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, bool), Diagnostic> {
        let span = expr.span;
        let ExprKind::Unary { op, operator, value } = &mut expr.kind else {
            unreachable!("unary dispatch");
        };
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
                Ok((operand, true))
    }

    /// Checked outside `expression_inner` so that nested operands recurse through a
    /// small frame. The flag is false when the result type is already recorded.
    #[inline(never)]
    fn expression_binary(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<(Type, bool), Diagnostic> {
        let span = expr.span;
        let ExprKind::Binary { op, operator, left, right } = &mut expr.kind else {
            unreachable!("binary dispatch");
        };
        {
                let equality = matches!(op, BinaryOp::Equal | BinaryOp::NotEqual);
                if equality && Self::is_typeof_guard(context, left, right) {
                    return self.typeof_equality(context, span, *op, left, right, loans).map(|ty| (ty, false));
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
                    return self.union_equality(context, (span, *operator), *op, left, right, loans).map(|ty| (ty, false));
                }
                let nullish_equality = equality
                    && [
                        self.expression_type_hint(context, left),
                        self.expression_type_hint(context, right),
                    ]
                    .iter()
                    .any(|ty| matches!(ty, Some(Type::Null | Type::Undefined)));
                if nullish_equality {
                    return self.nullish_equality(context, span, *op, left, right, loans).map(|ty| (ty, false));
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
                if *op == BinaryOp::Add && (self.expression_type_hint(context, left).map(Type::scalar_base) == Some(Type::String)
                    || self.expression_type_hint(context, right).map(Type::scalar_base) == Some(Type::String)) {
                    self.string_concatenation_operand(context, left)?;
                    self.string_concatenation_operand(context, right)?;
                }
                let string_addition = *op == BinaryOp::Add
                    && (expected == Some(Type::String)
                        || self.expression_type_hint(context, left).map(Type::scalar_base) == Some(Type::String)
                        || self.expression_type_hint(context, right).map(Type::scalar_base) == Some(Type::String));
                let number = expected != Some(Type::Int)
                    && ((!comparison && expected == Some(Type::Number))
                        || self.expression_type_hint(context, left).map(Type::scalar_base) == Some(Type::Number)
                        || self.expression_type_hint(context, right).map(Type::scalar_base) == Some(Type::Number));
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
                )?.scalar_base();
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
                if equality && (crate::flow::guard_pair(left, right, &context.names).is_some() || self.record_discriminator_guard(context,left,right).is_some()) {
                    self.remember_guard(context, span, *op, left, right);
                }
                let ty = match op {
                    op if op.bitwise_or_power() => left_type,
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Remainder => left_type,
                    _ => Type::Boolean,
                };
                Ok((ty, true))
        }
    }
}

fn native_type(ty: crate::native_bindings::NativeType, records: &[Record]) -> Type {
    use crate::native_bindings::NativeType;
    match ty {
        NativeType::Structured(signature) => crate::native_binding_types::resolve(records, signature).expect("prepared checked native signature"),
        NativeType::String => Type::String,
        NativeType::Boolean => Type::Boolean,
        NativeType::Int => Type::Int,
        NativeType::Number => Type::Number,
        NativeType::Void => Type::Void,
        NativeType::ProgramReusableVoidCallback | NativeType::ProgramVoidCallback | NativeType::OwnedVoidCallback | NativeType::SharedCallback | NativeType::ObservedValue => unreachable!("native callbacks resolve through callable contracts"),
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
            .into_kind()
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
                .expression(&mut context, &mut call, Mode::Consume, &Loans::new(), None)
                .unwrap(),
            Type::Int
        );
        assert_eq!(call.target, Some(CallTarget::StoredCallable(binding)));
        if once {
            assert_eq!(
                analyzer
                    .expression(&mut context, &mut call, Mode::Consume, &Loans::new(), None)
                    .unwrap_err()
                    .code,
                "closure-consumed"
            );
        } else {
            analyzer
                .expression(&mut context, &mut call, Mode::Consume, &Loans::new(), None)
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
        let StmtKind::Let { value, .. } = analyzer.program.functions[0].body[0].clone().into_kind() else {
            panic!("holder binding");
        };
        let ExprKind::Record(mut fields) = value.into_kind() else {
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

mod sets;
mod maps;
#[path = "../stdlib/collections/check.rs"]
mod deques;
#[path = "../stdlib/collections/heap_check.rs"]
mod heaps;

impl Analyzer {
    #[inline(never)]
    fn sequence_expression(&mut self,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>,discarded:bool)->Result<Type,Diagnostic>{
        self.sequence_expression_with_removal(context,expression,mode,loans,expected,discarded,false)
    }
    #[inline(never)]
    pub(super) fn sequence_expression_with_removal(&mut self,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>,discarded:bool,precise:bool)->Result<Type,Diagnostic>{
        self.sequence_expression_removal_snapshot(context,expression,mode,loans,expected,discarded,precise,None)
    }
    #[inline(never)]
    pub(super) fn sequence_expression_removal_snapshot(&mut self,context:&mut Context,expression:&mut Expr,mode:Mode,loans:&Loans,expected:Option<Type>,discarded:bool,precise:bool,final_snapshot:Option<&mut Option<Box<crate::capture_loan_owners::Owners>>>)->Result<Type,Diagnostic>{
        self.sequence_work(context, expression, mode, loans, expected, discarded, precise, final_snapshot)

    }
}

impl Analyzer {
    /// `(target = value)` with a string, array or record target has the target
    /// itself as its value: store first, then read the place. That is the comma
    /// form `(target = value, target)`, so no copy is made. Only a target with
    /// a fixed path qualifies; an index expression would be evaluated twice.
    #[inline(never)]
    fn owned_assignment_value(&self,context:&Context,expr:&mut Expr){
        let ExprKind::Assign{target,..}=&expr.kind else{return;};
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        if !fixed_path(target) || !self.expression_type_hint(context,target).is_some_and(|ty|!ty.is_copy()){return;}
        let place=(**target).clone();
        let span=expr.span;
        let store=std::mem::replace(expr,Expr::new(ExprKind::Null,span));
        *expr=Expr::new(ExprKind::Sequence(vec![store,place]),span);
    }

    /// `(target = value).field` reads the field of the target after the store:
    /// `(target = value, target.field)`. Projecting inside the comma form keeps
    /// the record in place instead of moving it into a temporary.
    #[inline(never)]
    fn owned_assignment_projection(&self,context:&Context,expr:&mut Expr){
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        let mut base=&*expr;
        while let ExprKind::Field(inner,_)=&base.kind{base=inner;}
        let ExprKind::Assign{target,..}=&base.kind else{return;};
        if !fixed_path(target) || !matches!(self.expression_type_hint(context,target),Some(Type::Record(_))){return;}
        let place=(**target).clone();
        let span=expr.span;
        let mut projection=std::mem::replace(expr,Expr::new(ExprKind::Null,span));
        let mut slot=&mut projection;
        while let ExprKind::Field(inner,_)=&mut slot.kind{
            if matches!(inner.kind,ExprKind::Assign{..}){
                let store=std::mem::replace(&mut **inner,place);
                *expr=Expr::new(ExprKind::Sequence(vec![store,projection]),span);
                return;
            }
            slot=inner;
        }
        unreachable!("assignment base located above");
    }

    /// `target ||= value` and `target &&= value` on a target that is not a
    /// boolean test the target by borrowing it, then store only on the selected
    /// branch. The resulting value reads the target in place, like assignment.
    /// Fixed paths keep the additional reads free of observable evaluation.
    #[inline(never)]
    fn truthy_logical_assignment(&self,context:&Context,expr:&mut Expr){
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        let ExprKind::LogicalAssign{target,..}=&expr.kind else{return;};
        if !fixed_path(target) || !self.expression_type_hint(context,target).is_some_and(|ty|!matches!(ty,Type::Boolean | Type::Void)){return;}
        let span=expr.span;
        let ExprKind::LogicalAssign{target,value,and,operator}=std::mem::replace(&mut expr.kind,ExprKind::Null) else{unreachable!()};
        let place=(*target).clone();
        let predicate=Expr::new(ExprKind::Unary{op:UnaryOp::Not,operator,value:target.clone()},span);
        let predicate=if and {Expr::new(ExprKind::Unary{op:UnaryOp::Not,operator,value:Box::new(predicate)},span)} else {predicate};
        let store=Expr::new(ExprKind::Assign{target,value,operator},span);
        let consequent=Expr::new(ExprKind::Sequence(vec![store,Expr::new(ExprKind::Boolean(true),span)]),span);
        let selection=Expr::new(ExprKind::Conditional{predicate:Box::new(predicate),consequent:Box::new(consequent),alternate:Box::new(Expr::new(ExprKind::Boolean(false),span))},span);
        expr.kind=ExprKind::Sequence(vec![selection,place]);
    }

    fn nullable_place(&self,ty:Type,value:&Expr)->bool{
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        fixed_path(value) && ty.union().is_some_and(|id|self.unions.members(id).iter().any(|member|matches!(member,Type::Null | Type::Undefined)))
    }

    /// A condition of another type than boolean tests truthiness: false, zero,
    /// NaN, the empty string, null and undefined are false. For a nullable
    /// variable or field the test is written as presence checks followed by
    /// the truth of the value, so the branch sees the value as present.
    #[inline(never)]
    /// A fixed field path whose every record alternative declares a boolean literal.
    fn boolean_discriminant(&self,context:&Context,value:&Expr)->bool{
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        let ExprKind::Field(base,name)=&value.kind else{return false};
        if !fixed_path(base){return false;}
        let Some(union)=self.expression_type_hint(context,base).and_then(Type::union) else{return false};
        let members=self.unions.members(union);
        members.len()>1 && members.iter().all(|member|matches!(member,Type::Record(id)
            if matches!(self.fields.get(*id).and_then(|fields|fields.get(name)),Some(Type::Literal(_,crate::literal_types::LiteralKind::Boolean)))))
    }

    fn truthy_condition(&self,context:&Context,condition:&mut Expr){
        fn fixed_path(target:&Expr)->bool{match &target.kind{ExprKind::Name(_)=>true,ExprKind::Field(base,_)=>fixed_path(base),_=>false}}
        let span=condition.span;
        let not=|value:Expr|Expr::new(ExprKind::Unary{op:UnaryOp::Not,operator:span,value:Box::new(value)},span);
        // A boolean literal discriminant narrows like TypeScript: `if(result.success)`
        // means `=== true` and `if(!result.success)` means `=== false`.
        let negated=matches!(&condition.kind,ExprKind::Unary{op:UnaryOp::Not,value,..} if self.boolean_discriminant(context,value));
        if negated || self.boolean_discriminant(context,condition){
            let place=std::mem::replace(condition,Expr::new(ExprKind::Null,span));
            let place=if negated{let ExprKind::Unary{value,..}=place.into_kind() else{unreachable!("negated discriminant")};*value}else{place};
            *condition=Expr::new(ExprKind::Binary{op:BinaryOp::Equal,operator:span,left:Box::new(place),right:Box::new(Expr::new(ExprKind::Boolean(!negated),span))},span);
            return;
        }
        // `!value` converts its own operand when it is checked.
        if matches!(condition.kind,ExprKind::Unary{op:UnaryOp::Not,..}){return;}
        // Each side of `&&` and `||` is a condition. The right side is typed
        // after the left side has narrowed, so it converts itself through `!!`.
        if let ExprKind::Binary{op:BinaryOp::And | BinaryOp::Or,left,right,..}=&mut condition.kind{
            if self.expression_type_hint(context,left)==Some(Type::Boolean) && self.expression_type_hint(context,right)==Some(Type::Boolean){return;}
            self.truthy_condition(context,left);
            if self.expression_type_hint(context,right)!=Some(Type::Boolean){
                let value=std::mem::replace(&mut **right,Expr::new(ExprKind::Null,span));
                **right=not(not(value));
            }
            return;
        }
        let Some(ty)=self.expression_type_hint(context,condition) else{return;};
        if matches!(ty,Type::Boolean | Type::Void){return;}
        let members=ty.union().map(|id|self.unions.members(id)).unwrap_or_else(||vec![ty]);
        let absent=[(Type::Null,ExprKind::Null),(Type::Undefined,ExprKind::Undefined)].into_iter().filter(|(member,_)|members.contains(member)).collect::<Vec<_>>();
        if absent.is_empty() || !fixed_path(condition){
            let value=std::mem::replace(condition,Expr::new(ExprKind::Null,span));
            *condition=not(not(value));
            return;
        }
        let place=std::mem::replace(condition,Expr::new(ExprKind::Null,span));
        let mut tests=absent.into_iter().map(|(_,literal)|Expr::new(ExprKind::Binary{op:BinaryOp::NotEqual,operator:span,left:Box::new(place.clone()),right:Box::new(Expr::new(literal,span))},span)).collect::<Vec<_>>();
        if members.iter().any(|member|matches!(member,Type::Boolean | Type::Int | Type::Number | Type::String)){
            tests.push(not(not(place)));
        }
        let mut tests=tests.into_iter();
        let first=tests.next().expect("presence test");
        *condition=tests.fold(first,|left,right|Expr::new(ExprKind::Binary{op:BinaryOp::And,operator:span,left:Box::new(left),right:Box::new(right)},span));
    }

}

#[cfg(test)]
#[test]
fn native_callback_graph_retains_selected_generic_method_identity() {
    let source=crate::format("function first<T>(value:T,f:ReadOnly<Function<int,[n:int]>>):int{return f(1);}function second<T>(value:T,f:ReadOnly<Function<int,[n:int]>>):int{return f(2);}function main(){const one=first;const two=second;one<int>(7,(n:int):int=>n);two<int>(9,(n:int):int=>n);}").unwrap();
    assert_eq!(crate::format(&source).unwrap(),source);
    let mut program=crate::check(&source).unwrap().program;
    let main=program.functions.iter().position(|function|function.name=="main").unwrap();
    let selected=program.functions[main].body.iter().filter_map(|statement|match &statement.kind {StmtKind::Expr(call)=>Some(call),_=>None}).nth(1).unwrap();
    let ExprKind::Call(callee,_)=&selected.kind else {panic!("method call");};
    let Some(Type::Closure(origin))=callee.ty else {panic!("environment");};
    let Some(CallTarget::ClosureMethod(method))=selected.target else {panic!("method");};
    let plan=program.polymorphic_invocations.publish([crate::polymorphic_invocation_plans::Pair{origin,method}],&program.closures,Span::default()).unwrap();
    let mut seen=false;
    program.functions[main].body.retain_mut(|statement| {
        let StmtKind::Expr(call)=&mut statement.kind else {return true;};
        if seen {return false;}seen=true;
        let ExprKind::Call(callee,_)=&mut call.kind else {panic!("call");};
        callee.generic_invocation=Some(plan);true
    });
    let functions=program.functions.iter().collect::<Vec<_>>();
    let calls=callback_calls(&program.functions[main],&functions,&program.closures,&[],&program.polymorphic_invocations).unwrap();
    assert!(calls.iter().any(|call|matches!(call.target,NativeCallbackTarget::Method(found,ordinal) if found==origin && ordinal==method)),"selected method must retain a native callback edge");
    assert_eq!(calls.len(),1,"inactive receiver method must not remain a call edge");
}

#[path = "../stdlib/collections/ordered_set_check.rs"]
mod ordered_sets;

#[cfg(test)]
mod ownership_walk_tests {
    use super::*;

    fn selected(roots: Vec<usize>) -> Expr {
        Expr::new(ExprKind::Selected {
            source: Box::new(crate::syntax::SelectedSource {
                readonly: false, roots, place: None, span: Span::default(),
            }),
            slot: 0, owned: false, access: Mode::Read,
        }, Span::default())
    }

    #[test]
    fn ownership_walks_preserve_root_order_and_ignore_sequence_prefixes() {
        let context = Context::new("walk".into());
        let branch = Expr::new(ExprKind::Conditional {
            predicate: Box::new(selected(vec![99])),
            consequent: Box::new(selected(vec![7, 2, 7])),
            alternate: Box::new(selected(vec![2, 4])),
        }, Span::default());
        let value = Expr::new(ExprKind::Sequence(vec![selected(vec![98]), branch]), Span::default());
        assert_eq!(Analyzer::loan_roots(&value, Mode::Read), vec![7, 2, 4]);
        assert_eq!(Analyzer::view_sources(&context, &value), vec![2, 4, 7]);
        assert!(Analyzer::stable_at_receiver(&value));
        assert_eq!(Analyzer::root_binding(&value), None);
    }

    #[test]
    fn ownership_walks_use_normal_stack_for_deep_checked_expressions() {
        let context = Context::new("walk".into());
        let mut value = selected(vec![7, 2]);
        // Phase-isolated witness: parser admission is irrelevant to these walks.
        for _ in 0..16384 {
            value = Expr::new(ExprKind::Sequence(vec![value]), Span::default());
        }
        assert_eq!(Analyzer::root_binding(&value), Some(7));
        assert_eq!(Analyzer::loan_roots(&value, Mode::Read), vec![7, 2]);
        assert_eq!(Analyzer::view_sources(&context, &value), vec![2, 7]);
        assert!(Analyzer::stable_at_receiver(&value));
        drop(value);
    }
}

mod borrow_hints;

mod type_hints;
