mod declaration_scope;
#[cfg(test)]
mod statement_work_tests;
use declaration_scope::Declarations;
mod coalesce_work;
mod expression_work;
mod program_testing;
mod threads;
mod futures;
mod http;
mod sqlite;
mod sql;
mod channels;
#[path="../stdlib/cache/emit.rs"] mod cache;
mod rust_bridge;
mod results;
mod schema;
mod json_encoding;
mod json_typed;
mod json_refinement;
mod nullable_link_emit;
mod recursive_union_payload_emit;
mod native_view_record_parameters;
mod native_view_family_emit;
mod native_view_lifetime_emit;
mod native_view_environment_emit;
mod closure_owner_uses;
mod closure_record_signature_emit;
mod forwarded_owner_uses;
mod forwarded_record_signature_emit;
mod native_function_signature_emit;
mod local_borrowed_family_emit;
mod record_store_branches_emit;
mod borrowed_owner_provenance;
mod borrowed_owner_emit;
mod record_point_call_emit;
mod direct_record_store_emit;
mod record_callback_reification_emit;
mod nullable_callback_clear_emit;
mod nullable_callback_replacement_emit;
mod native_capture_storage;
mod borrowed_payload_emit;
mod callback_capture_emit;
mod record_parameter_borrow_emit;
mod nullable_record_parameter_emit;
mod map_error_storage_emit;
mod statement_work;
mod completion_work;
mod map_completion_work;
mod owned_work;
mod storage_statement_work;
mod restored_capture_slots;
mod finally_move_slots;
mod field_completion_transport;
mod field_control_transport;
mod polymorphic_storage_emit;
mod callback_family_result_adapters;
mod borrowed_callable_emit;
mod scoped_record_emit;
mod record_parameter_replacement_emit;
mod nullable_parameter_store_emit;
mod scoped_record_parameter_views;
mod record_join_emit;
mod scoped_record_copy_emit;
mod scoped_record_owned_places;
mod record_carrier_emit;
mod scoped_record_capture_views;
mod scoped_record_console_emit;
mod scoped_record_support_emit;
mod exception_contract_emit;
mod map_exception_storage_plan;
mod native_storage_availability;
mod exception_emit;
mod spread_emit;
mod spread_builtin_emit;
mod logical_values_emit;
use crate::analysis::{ArgumentContract, CheckedFunction, CheckedProgram, ClosureForward, Mode};
use crate::storage::{inline_array, numeric_array};
use crate::syntax::*;
mod native_view_emit;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

pub fn emit(program: &CheckedProgram) -> String {
    emit_with_native_features(program, "").0
}

fn emitter_for<'a>(program:&'a CheckedProgram,entry_setup:&'a str)->Emitter<'a> {
    let fields = program
        .records
        .iter()
        .map(|record| {
            record
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| (field.name.text.as_str(), index))
                .collect()
        })
        .collect();
    let mut record_runtime =
        crate::record_runtime::RecordRuntime::new(&program.records, &program.unions);
    let mut inspect_keys = |expr: &Expr| {
        if matches!(expr.target, Some(CallTarget::ObjectKeys | CallTarget::ObjectValues | CallTarget::ObjectEntries(_))) {
            if let ExprKind::Call(_, args) = &expr.kind {
                if let Some(Type::Record(id)) = args.first().and_then(|value| value.ty) {
                    if !program.records[id].is_tuple() { record_runtime.ordered.insert(id); }
                }
            }
        }
        if let Some(CallTarget::PatternRest(result,start))=expr.target && start==usize::MAX {
            record_runtime.ordered.insert(result);
            if let ExprKind::Call(callee,_) = &expr.kind && let ExprKind::Field(base,_) = &callee.kind && let Some(Type::Record(source))=base.ty {record_runtime.ordered.insert(source);}
        }
        if let Some(CallTarget::RecordSpread(result))=expr.target {
            record_runtime.ordered.insert(result);
            if let ExprKind::Call(_,args)=&expr.kind {for arg in args {if let Some(Type::Record(id))=arg.ty {record_runtime.ordered.insert(id);}}}
        }
        // Console display lists record fields in each value's constructor
        // order, so every record type that display can reach keeps that order.
        if expr.target == Some(CallTarget::Log) {
            if let ExprKind::Call(_, args) = &expr.kind {
                let mut pending = args
                    .iter()
                    .filter_map(|value| match &value.kind {
                        ExprKind::Unary { op: UnaryOp::Spread, value, .. } => value.ty,
                        _ => value.ty,
                    })
                    .collect::<Vec<_>>();
                let mut seen = std::collections::HashSet::new();
                while let Some(ty) = pending.pop() {
                    if !seen.insert(ty) {
                        continue;
                    }
                    match ty {
                        Type::Record(id) => {
                            if !program.records[id].is_tuple() { record_runtime.ordered.insert(id); }
                            pending.extend(program.records[id].fields.iter().filter_map(|field| field.ty));
                        }
                        Type::Array(element) => pending.push(element.with_depth(0).ty()),
                        Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(program.unions[id.index()].members.iter().copied()),
                        _ => {}
                    }
                }
            }
        }
    };
    for function in program.functions.iter().chain(program.closures.iter().map(|value| &value.function)).chain(program.closures.iter().flat_map(|value| value.methods.iter().map(|method| &method.function))) {
        crate::fixed_storage::visit_function_expressions(function, &mut inspect_keys);
    }
    let ordered_declarations=record_runtime.ordered.iter().map(|&id|program.scoped_records.declaration(id)).collect::<HashSet<_>>();
    for id in 0..program.records.len(){if ordered_declarations.contains(&program.scoped_records.declaration(id)){record_runtime.ordered.insert(id);}}
    let fixed = crate::fixed_storage::FixedStorage::new(program, &record_runtime);
    let callback_traits = !program.callback_contracts.is_empty()
        || program
            .closures
            .iter()
            .any(|closure| matches!(closure.forward, Some(ClosureForward::Once(_))));
    let counted_calls = program.closures.iter().any(|closure| matches!(closure.forward, Some(ClosureForward::RestAdapter { counted: true, .. })));
    let borrowed_owners = borrowed_owner_emit::marker_plan(program);
    let borrowed_capture_alternatives = borrowed_payload_emit::capture_alternatives(program);
    let borrowed_lifetime_alternatives = borrowed_owner_emit::lifetime_alternatives(program,&borrowed_capture_alternatives,&borrowed_owners);
    let native_captures = native_capture_storage::Storage::new(program, &borrowed_lifetime_alternatives);
    let native_views=native_view_emit::Shapes::new(program);
    let native_record_fields=native_view_record_parameters::Index::new(program,&native_views);
    let mut emitter = Emitter {
        work_functions:owned_work::Arena::default(),
        work_regions:owned_work::Arena::default(),
        borrowed_capture_alternatives,
        borrowed_owners,
        borrowed_lifetime_alternatives,
        native_captures,
        native_views,
        native_record_fields,
        recursive_links:crate::recursive_links::Index::new(&program.records,|id|program.unions[id.index()].members.clone()),
        callback_traits,
        counted_calls,
        program,
        restored_slots:restored_capture_slots::Index::new(program),
        finally_moves:finally_move_slots::Index::new(program),
        entry_setup,
        source_segments: &[],
        source_graph: None,
        fields,
        record_runtime,
        borrowing_records: HashSet::new(),
        fixed,
        active_plan: Cell::new(None),
        exception_regions: exception_contract_emit::RegionStack::default(),
        control_targets: exception_contract_emit::ControlStack::default(),
        completion_next: Cell::new(0),
        return_passes_through: Cell::new(false),
        inline_return: RefCell::new(None),
        inline_abrupt: RefCell::new(None),
        inline_map_return: RefCell::new(None),
        cleanup_replaces_exits: Cell::new(false),
        result_functions: RefCell::new(HashMap::new()),
        result_closures: RefCell::new(HashMap::new()),
        result_methods: RefCell::new(HashMap::new()),
        result_storage: RefCell::new(HashMap::new()),
        result_callbacks: RefCell::new(HashMap::new()),
        conditional_temporaries: RefCell::new(Vec::new()),
        rendered_receivers: RefCell::new(Vec::new()),
        conditional_next: Cell::new(0),
        expression_depth: Cell::new(0),
        native_bindings: RefCell::new(std::collections::BTreeSet::new()),
    };
    emitter.borrowing_records=emitter.record_borrowing_family();
    emitter
}

pub(crate) fn needs_exception_transport(program:&CheckedProgram)->bool {
    // Native emission includes every checked non-template definition, even when
    // main never invokes it. Its throwing body still requires a saved Result ABI.
    program.functions.iter().chain(program.closures.iter().flat_map(|closure|
        std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function))))
        .filter(|function|!function.template)
        .any(|function|crate::analysis::has_exception_syntax(&function.body)
            || crate::error_effects::checked_expressions(function).iter().any(|expression|(matches!(expression.target,Some(CallTarget::ResultUnwrap(_))) || matches!(expression.target,Some(CallTarget::NativeBinding(id)) if crate::native_bindings::BINDINGS[id].module=="std:cache" && crate::native_bindings::BINDINGS[id].name=="__create"))))
}

pub(crate) fn validate_exception_program(program:&CheckedProgram)->Result<(),crate::Diagnostic> {
    emitter_for(program,"").prepare_result_declarations().map(|_|())
}

pub(crate) fn emit_with_native_features(program:&CheckedProgram,entry_setup:&str)->(String,Vec<&'static str>) {
    emit_with_source_segments(program,entry_setup,&[],None)
}

pub(crate) fn emit_with_source_segments(program:&CheckedProgram,entry_setup:&str,segments:&[(crate::project::SourceId,usize,usize)],graph:Option<&crate::project::ProjectGraph>)->(String,Vec<&'static str>) {
    let mut emitter=emitter_for(program,entry_setup);
    emitter.source_segments=segments;
    emitter.source_graph=graph;
    let declarations=if needs_exception_transport(program) {emitter.prepare_result_declarations().expect("checked source exception transports")}else{String::new()};
    let mut rust = emitter.program();
    if needs_exception_transport(program) {rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declarations);}
    let features = emitter
        .native_bindings
        .borrow()
        .iter()
        .flat_map(|&id| {
            crate::native_bindings::BINDINGS[id]
                .features
                .iter()
                .copied()
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    (rust, features)
}

struct Emitter<'a> {
    work_functions:owned_work::Arena<CheckedFunction>,
    work_regions:owned_work::Arena<ExceptionRegion>,
    borrowed_lifetime_alternatives: borrowed_payload_emit::Alternatives,
    borrowed_owners: borrowed_owner_provenance::Plan,
    native_captures: native_capture_storage::Storage,
    native_views: native_view_emit::Shapes,
    native_record_fields: native_view_record_parameters::Index,
    recursive_links:crate::recursive_links::Index,
    borrowed_capture_alternatives: borrowed_payload_emit::Alternatives,
    callback_traits: bool,
    counted_calls: bool,
    program: &'a CheckedProgram,
    restored_slots:restored_capture_slots::Index,
    finally_moves:finally_move_slots::Index,
    entry_setup: &'a str,
    source_segments: &'a [(crate::project::SourceId,usize,usize)],
    source_graph: Option<&'a crate::project::ProjectGraph>,
    fields: Vec<HashMap<&'a str, usize>>,
    record_runtime: crate::record_runtime::RecordRuntime,
    borrowing_records: HashSet<usize>,
    fixed: crate::fixed_storage::FixedStorage,
    active_plan: Cell<Option<usize>>,
    exception_regions: exception_contract_emit::RegionStack,
    control_targets: exception_contract_emit::ControlStack,
    completion_next: Cell<usize>,
    /// Set for the next completion body: its returns reach the enclosing region unchanged.
    return_passes_through: Cell<bool>,
    inline_return: RefCell<Option<exception_contract_emit::InlineExit>>,
    inline_abrupt: RefCell<Option<exception_contract_emit::InlineExit>>,
    inline_map_return: RefCell<Option<exception_contract_emit::InlineExit>>,
    /// Inside a `try` whose abrupt `finally` replaces every pending exit.
    cleanup_replaces_exits: Cell<bool>,
    result_functions: RefCell<HashMap<usize,(String,String)>>,
    result_closures: RefCell<HashMap<usize,String>>,
    result_methods: RefCell<HashMap<(usize,usize),String>>,
    result_storage: RefCell<HashMap<usize,(String,crate::error_effects::FailureSet)>>,
    result_callbacks: RefCell<HashMap<usize,(String,crate::error_effects::FailureSet)>>,
    conditional_temporaries: RefCell<Vec<String>>,
    /// Receivers of builtin chain links rendered on the expression work stack.
    rendered_receivers: RefCell<Vec<(usize, String)>>,
    conditional_next: Cell<usize>,
    expression_depth: Cell<usize>,
    native_bindings: RefCell<std::collections::BTreeSet<usize>>,
}

enum StorageStep<'a> {
    Discard(&'a Expr),
    Field(usize),
    Union(UnionId, usize, Option<crate::diagnostic::Span>),
    Index(&'a Expr, crate::diagnostic::Span),
}

struct StorageSelection {
    reselected_shared: String,
    place: String,
    setup: String,
    shared: String,
    exclusive: String,
    final_conversion: String,
}

fn float_literal(value: f64) -> String {
    if value == f64::INFINITY {
        "f64::INFINITY".into()
    } else if value == f64::NEG_INFINITY {
        "f64::NEG_INFINITY".into()
    } else {
        format!("{value:?}f64")
    }
}

pub(crate) fn ty(value: Type) -> String {
    match value {
        Type::Literal(_,kind)=>ty(kind.base()),
        Type::String => "String".into(),
        Type::Int => "i64".into(),
        Type::Number => "f64".into(),
        Type::Boolean => "bool".into(),
        Type::Void => "()".into(),
        Type::Union(id) => format!("Union{}", id.index()),
        Type::BorrowedUnion(id) => format!("ViewUnion{}<'_>", id.index()),
        Type::Null => "TsrNull".into(),
        Type::Undefined => "TsrUndefined".into(),
        Type::Record(i) => format!("Record{i}"),
        Type::Enum(i, _) => format!("Enum{i}"),
        Type::Set(i) => format!("Set{i}"),
        Type::Map(i) => format!("Map{i}"),
        Type::Array(element) if numeric_array(value) => {
            format!("NumericArray<{}>", ty(element.ty()))
        }
        Type::Array(ArrayElement::Record(index, 0)) => format!("RecordArray{index}"),
        Type::Array(element) => format!(
            "{}<{}>",
            if inline_array(value) { "Array" } else { "Vec" },
            ty(element.ty())
        ),
        Type::Closure(i) => format!("Closure{i}"),
        Type::StoredCallable(i) => format!("StoredCallable{i}"),
        Type::BorrowedCallable(i) => format!("BorrowedCallable{i}"),
        Type::Callable(_) => unreachable!("unresolved callable result"),
        Type::Callback(_) => unreachable!("callback types only occur on generic parameters"),
    }
}

fn tuple(values: impl Iterator<Item = String>) -> String {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        "()".into()
    } else {
        format!("({},)", values.join(", "))
    }
}

fn lifetime_parameters(lifetimes: &[String]) -> String {
    if lifetimes.is_empty() {
        String::new()
    } else {
        format!("<{}>", lifetimes.join(", "))
    }
}

impl Emitter<'_> {
    fn callback_payload(&self, value: Type) -> Option<Type> {
        value.union().and_then(|id| {
            self.program.unions[id.index()]
                .members
                .iter()
                .copied()
                .find(|member| {
                    matches!(
                        member,
                        Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)
                    )
                })
        })
    }

    fn schema_inherited_factories(&self,output:Type)->Vec<(Type,usize)> {
        crate::stdlib::schema::defaults::inherited_factories(output,&self.program.schema_factories,|id|self.program.unions[id.index()].members.clone())
    }
    fn schema_factory_declaration(&self,id:usize)->String {
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return String::new();}
        let output=record.application.as_ref().unwrap().resolved_arguments[0];
        let mut fields=self.program.schema_factories.get(&output).map(|family|format!("factory:Option<Box<StoredCallable{family}>>,")).unwrap_or_default();
        if self.program.schema_choices.for_output(output).next().is_some(){fields.push_str(&format!("choice:Option<Box<SchemaChoice{}>>,",self.native_record_declaration(id)));if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")){fields.push_str("choice_policy:Option<bool>,");}}
        for (source,family) in self.schema_inherited_factories(output){let Type::Union(source)=source else{unreachable!()};fields.push_str(&format!("factory_from{}:Option<Box<StoredCallable{family}>>,",source.index()));}
        fields
    }
    fn schema_factory_initializer(&self,id:usize)->String {
        let record=&self.program.records[id];
        if record.native_type!=Some(crate::native_signatures::NativeTypeId::Schema){return String::new();}
        let output=record.application.as_ref().unwrap().resolved_arguments[0];
        let mut fields=if self.program.schema_factories.contains_key(&output){String::from("factory:None,")}else{String::new()};
        if self.program.schema_choices.for_output(output).next().is_some(){fields.push_str("choice:None,");if matches!(output,Type::Record(object) if !self.program.records[object].name.text.starts_with("@schema:retained:")){fields.push_str("choice_policy:None,");}}
        for (source,_) in self.schema_inherited_factories(output){let Type::Union(source)=source else{unreachable!()};fields.push_str(&format!("factory_from{}:None,",source.index()));}
        fields
    }

    fn native_type(&self, value: Type) -> String {
        if let Type::Array(ArrayElement::StoredCallable(family,depth))=value
            && self.program.stored_contracts.scoped_native_family(family).is_some() {
            let mut native=self.native_type(Type::StoredCallable(family));
            for _ in 0..=depth {native=format!("Vec<{native}>");}
            return native;
        }
        if let Type::StoredCallable(id)=value
            && let Some(layout)=self.program.stored_contracts.scoped_native_family(id) {
            let slots=layout.alternatives.iter().map(|alternative|alternative.slots.len()).sum::<usize>();
            return format!("NativeViewCallable{id}<{}>",vec!["'_";slots].join(", "));
        }
        if let Type::Record(id) = value && (native_view_emit::access(self.program, id).is_some() || self.native_views.recursive(id)) {
            return format!("Record{id}<'_>");
        }
        if let Type::Record(id) = value && let Some(element) = self.program.records[id].native_element()
            && self.native_collection_payload_parameter(element) {
            return format!("Record{}<{}>", self.native_record_declaration(id), self.native_type(element));
        }
        if let Type::Map(id) = value {
            let payload = self.program.records[id].map_definition.as_ref().unwrap().value;
            if self.scoped_record_native_parameter(payload) { return format!("Map{id}<{}>", self.native_type(payload)); }
        }
        if let Type::Array(ArrayElement::Record(id, depth)) = value
            && depth > 0
        {
            let mut native = self.native_type(Type::Array(ArrayElement::Record(id, 0)));
            for _ in 0..depth { native = format!("Vec<{native}>"); }
            return native;
        }
        if let Type::Array(ArrayElement::Record(id, 0)) = value {
            let record = self.native_type(Type::Record(id));
            if let Some((_, arguments)) = record.split_once('<') { return format!("RecordArray{id}<{arguments}"); }
        }
        if let Some(id) = value.union() {
            let payloads = self.program.unions[id.index()].members.iter().copied().filter(|ty| !ty.is_copy()).collect::<Vec<_>>();
            if payloads.iter().any(|ty| self.scoped_record_native_parameter(*ty)) && (payloads.len() == 1 || self.native_views.contains(value)) {
                let payload = payloads.iter().map(|&ty| {let native=self.native_type(ty);if matches!(value,Type::Union(_)){self.union_stored_type(id,ty,native)}else{native}}).collect::<Vec<_>>().join(", ");
                return if matches!(value, Type::BorrowedUnion(_)) { format!("ViewUnion{}<'_, {payload}>", id.index()) } else { format!("Union{}<{payload}>", id.index()) };
            }
        }
        // Elements that are nullable callback records name their environment.
        if let Type::Array(ArrayElement::Union(id, depth)) = value
            && self.program.unions[id.index()].members.iter().any(|&member| matches!(member, Type::Record(_)) && self.scoped_record_native_parameter(member))
        {
            let mut native = self.native_type(Type::Union(id));
            for _ in 0..=depth { native = format!("Vec<{native}>"); }
            return native;
        }
        if let Type::Record(id) = value {
            // The declaration decides which fields are generic. An instance
            // whose nullable field owns all captures still fills that position.
            let base = self.native_record_declaration(id);
            let parameters = self.program.records[id].fields.iter().enumerate().filter_map(|(position,field)| {
                let ty = field.ty.expect("checked field");
                self.scoped_record_native_field(id,position,ty).then(|| self.native_type(ty))
            }).collect::<Vec<_>>();
            if !parameters.is_empty() { return format!("Record{base}<{}>", parameters.join(", ")); }
        }
        if let Type::BorrowedCallable(id)=value {return self.borrowed_family_type(id);}
        if let Type::Closure(id) = value {
            if self.program.polymorphic_storage.contains(id) {
                return self.generic_storage_type(id);
            }
            return self.native_environment_type(id);
        }
        if let Some(payload) = self.callback_payload(value) {
            let argument = self.native_type(payload);
            let id = value.union().expect("nullable payload");
            return if matches!(value, Type::BorrowedUnion(_)) {
                format!("ViewUnion{}<'_, {argument}>", id.index())
            } else {
                format!("Union{}<{argument}>", id.index())
            };
        }
        if let Type::Callback(id) = value {
            return if self.program.absent_callback_payloads.contains(&id) {
                "TsrAbsentCallable".into()
            } else {
                format!("C{id}")
            };
        }
        ty(value)
    }

    fn multiple_data_union(&self,ty:Type)->bool {ty.union().is_some_and(|id|!id.copy && self.program.unions[id.index()].members.iter().filter(|ty|!matches!(ty,Type::Null|Type::Undefined)).count()>1)}

    fn native_argument_type(&self, contract: ArgumentContract, lifetime: &str) -> String {
        if (contract.omission.is_omittable() || self.multiple_data_union(contract.ty)) && contract.mode == Mode::Mutate
            && let Type::Union(id) = contract.ty && !id.copy {
            if let Some(payload)=self.callback_payload(contract.ty) {
                return format!("Union{}<&{lifetime} mut {}>",id.index(),self.native_type(payload));
            }
            let payloads = self.program.unions[id.index()].members.iter().copied().filter(|ty|!ty.is_copy()).map(|payload| format!("&{lifetime} mut {}",self.native_type(payload))).collect::<Vec<_>>().join(", ");
            return format!("Union{}<{payloads}>",id.index());
        }
        if let Type::Union(id) = contract.ty
            && !id.copy
            && contract.mode == Mode::Read
        {
            let parameters = if self.native_views.contains(contract.ty) {
                self.program.unions[id.index()].members.iter().filter(|ty| !ty.is_copy())
                    .map(|&ty| format!(", {}", self.native_type(ty))).collect::<String>()
            } else {
                self.callback_payload(contract.ty).map_or_else(String::new, |ty| format!(", {}", self.native_type(ty)))
            };
            return format!(
                "ViewUnion{}<{}{parameters}>",
                id.index(),
                if lifetime.is_empty() { "'_" } else { lifetime }
            );
        }
        let reference = match contract.mode {
            Mode::Read => format!("&{lifetime} "),
            Mode::Mutate => format!("&{lifetime} mut "),
            _ => String::new(),
        };
        format!("{reference}{}", self.native_type(contract.ty))
    }

    fn native_argument_tuple(&self, params: &[ArgumentContract]) -> (String, Vec<String>) {
        let mut lifetimes = Vec::new();
        let types = params
            .iter()
            .enumerate()
            .map(|(index, &parameter)| {
                let lifetime = if matches!(parameter.mode, Mode::Read | Mode::Mutate) {
                    let name = format!("'p{index}");
                    lifetimes.push(name.clone());
                    name
                } else {
                    String::new()
                };
                self.native_argument_type(parameter, &lifetime)
            })
            .collect::<Vec<_>>();
        (tuple(types.into_iter()), lifetimes)
    }

    /// The earlier union whose enum a nullable callback-record union reuses.
    fn nominal_union_alias(&self, index: usize) -> Option<usize> {
        let nominal = |members: &[Type]| {
            let mut scoped = 0;
            let shape = members.iter().map(|&member| match member {
                Type::Record(record) if self.scoped_record_native_parameter(member) => { scoped += 1; Type::Record(self.native_record_declaration(record)) }
                other => other,
            }).collect::<Vec<_>>();
            (scoped == 1 && members.iter().filter(|member| !member.is_copy()).count() == 1).then_some(shape)
        };
        let shape = nominal(&self.program.unions[index].members)?;
        self.program.unions[..index].iter().position(|prior| nominal(&prior.members).as_ref() == Some(&shape))
    }

    fn nullable_callback_bounds(&self, types: impl IntoIterator<Item = Type>) -> Vec<String> {
        self.callback_bounds(types, false)
    }

    fn callback_bounds(&self, types: impl IntoIterator<Item = Type>, direct: bool) -> Vec<String> {
        let mut pending = types.into_iter().map(|ty| (ty, direct)).collect::<Vec<_>>();
        let mut seen = std::collections::BTreeSet::new();
        let mut walked = std::collections::BTreeSet::new();
        let mut bounds = Vec::new();
        let mut captured = std::collections::BTreeSet::new();
        let mut records_seen = std::collections::BTreeSet::new();
        while let Some((ty, nested)) = pending.pop() {
            if let Type::Record(id) = ty {
                if records_seen.insert(id) {
                    pending.extend(self.program.records[id].fields.iter().filter_map(|field| field.ty.map(|ty| (ty, true))));
                }
            }
            // An array of records names the callbacks of its elements.
            if let Type::Array(ArrayElement::Record(id, _)) = ty {
                pending.push((Type::Record(id), true));
            }
            if let Type::Array(ArrayElement::Union(id, _)) = ty {
                pending.push((Type::Union(id), true));
            }
            if let Type::Map(id) = ty && let Some(map) = self.program.records[id].map_definition.as_ref() && matches!(map.value, Type::Record(_)) {
                pending.push((map.value, true));
            }
            // A nullable callback record names its callback through the record member.
            if let Type::Union(id) | Type::BorrowedUnion(id) = ty {
                pending.extend(self.program.unions[id.index()].members.iter().filter(|member| matches!(member, Type::Record(_))).map(|&member| (member, true)));
            }
            if let Type::Closure(id) = ty {
                let closure = &self.program.closures[id];
                if matches!(closure.forward, Some(ClosureForward::RestAdapter { .. })) && captured.insert(id) {
                    pending.extend(closure.captures.iter().map(|capture| (closure.function.bindings[capture.inner].ty, true)));
                }
            }
            let selected = self.callback_payload(ty);
            let Some(Type::Callback(id)) =
                selected.or_else(|| matches!(ty, Type::Callback(_)).then_some(ty))
            else {
                continue;
            };
            if self.program.absent_callback_payloads.contains(&id) {
                continue;
            }
            let contract = &self.program.callback_contracts[id];
            if (selected.is_some() || nested) && seen.insert(id) {
                let (args, lifetimes) = self.native_argument_tuple(&contract.params);
                let borrow = if lifetimes.is_empty() {
                    String::new()
                } else {
                    format!("for{} ", lifetime_parameters(&lifetimes))
                };
                let trait_name = if matches!(contract.kind, crate::syntax::CallbackKind::Program | crate::syntax::CallbackKind::ProgramReusable) { "ProgramCallback" } else if matches!(contract.kind, crate::syntax::CallbackKind::Read | crate::syntax::CallbackKind::SharedRead) {
                    "ReadCallback"
                } else if contract.kind==CallbackKind::Task {
                    "TaskCallback"
                } else if contract.kind.is_once() {
                    "OnceCallback"
                } else {
                    "Callback"
                };
                bounds.push(format!(
                    "C{id}: {borrow}{trait_name}<{args}, {}>{}",
                    self.callback_result_type(id,contract.result),
                    if contract.kind.is_owned() { " + 'static" } else { "" }
                ));
            }
            if walked.insert(id) {
                pending.extend(contract.params.iter().map(|parameter| (parameter.ty, true)));
                pending.push((contract.result, true));
            }
        }
        bounds
    }

    fn emit_nullable_types(&self, output: &mut String) {
        if !self.program.absent_callback_payloads.is_empty() || self.program.unions.iter().any(|union| {
            union
                .members
                .iter()
                .any(|member| matches!(member, Type::Callback(_)))
        }) {
            output.push_str("enum TsrAbsentCallable {}\nimpl<A,R> ReadCallback<A,R> for TsrAbsentCallable {fn invoke_read(&self,_:A)->R {match *self {}}}\nimpl<A,R> Callback<A,R> for TsrAbsentCallable {fn invoke(&mut self,_:A)->R {match *self {}}}\nimpl<A,R> OnceCallback<A,R> for TsrAbsentCallable {fn invoke_once(self,_:A)->R {match self {}}}\n");
        }
        let cloneability = crate::cloneability::Cloneability::new(&self.program.records, |id| {
            self.program.unions[id.index()].members.clone()
        });
        for (index, union) in self.program.unions.iter().enumerate() {
            let callable = union.members.iter().any(|member| {
                matches!(
                    member,
                    Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_)
                )
            });
            if callable {
                let absent = union
                    .members
                    .iter()
                    .copied()
                    .filter(|ty| matches!(ty, Type::Null | Type::Undefined))
                    .collect::<Vec<_>>();
                if let Some(prior) = self.program.unions[..index].iter().position(|prior| {
                    prior.members.iter().any(|ty| {
                        matches!(
                            ty,
                            Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_)
                        )
                    }) && prior
                        .members
                        .iter()
                        .copied()
                        .filter(|ty| matches!(ty, Type::Null | Type::Undefined))
                        .collect::<Vec<_>>()
                        == absent
                }) {
                    let _ = writeln!(
                        output,
                        "type Union{index}<F> = Union{prior}<F>;\ntype ViewUnion{index}<'a,F> = ViewUnion{prior}<'a,F>;"
                    );
                    continue;
                }
            }
            // Every concrete environment of one nullable callback record uses
            // the enum of the first such union, with its own payload argument.
            if !callable && let Some(prior) = self.nominal_union_alias(index) {
                let _ = writeln!(output, "type Union{index}<P> = Union{prior}<P>;\ntype ViewUnion{index}<'a,P> = ViewUnion{prior}<'a,P>;");
                continue;
            }
            let copy = union.members.iter().all(|member| member.is_copy());
            let id = crate::syntax::UnionId {
                index: index as u32,
                copy,
            };
            let cloneable = cloneability.allows(Type::Union(id), |id| {
                self.program.unions[id.index()].members.clone()
            });
            // A named payload breaks Rust's recursive generic-default query
            // without changing ownership or adding another allocation.
            for &member in &union.members {
                self.emit_recursive_payload(output, id, member, cloneable);
            }
            // An exclusive input can borrow the payload without owning its
            // caller's storage. Keep the usual owned spelling as the default.
            let data_payloads = if !copy && !callable { union.members.iter().copied().filter(|member| !member.is_copy()).collect::<Vec<_>>() } else { Vec::new() };
            let parameter_name = |position: usize| if data_payloads.len() == 1 { "P".to_owned() } else { format!("P{position}") };
            let scoped_data = data_payloads.iter().any(|ty| self.scoped_record_native_parameter(*ty));
            let view_parameters = data_payloads.iter().enumerate().map(|(position, payload)| if scoped_data {parameter_name(position)} else {format!("{} = {}",parameter_name(position),self.native_type(*payload))}).collect::<Vec<_>>().join(", ");
            let data_parameters = data_payloads.iter().enumerate().map(|(position, payload)| if scoped_data {parameter_name(position)} else {format!("{} = {}",parameter_name(position),self.union_stored_type(id,*payload,self.native_type(*payload)))}).collect::<Vec<_>>().join(", ");
            let data_arguments = (0..data_payloads.len()).map(parameter_name).collect::<Vec<_>>().join(", ");
            let generic_payload = callable || !data_payloads.is_empty();
            let generic = if callable { "<F>".to_owned() } else if generic_payload { format!("<{data_parameters}>") } else { String::new() };
            let type_arguments = if callable { "<F>".to_owned() } else if generic_payload { format!("<{data_arguments}>") } else { String::new() };
            let derive = if copy {
                "#[derive(Clone,Copy)] "
            } else if cloneable {
                "#[derive(Clone)] "
            } else {
                ""
            };
            let member_type = |member: Type| {
                if matches!(
                    member,
                    Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_)
                ) {
                    "F".into()
                } else if let Some(position) = data_payloads.iter().position(|payload| *payload == member) {
                    parameter_name(position)
                } else {
                    self.native_type(member)
                }
            };
            let variants = union
                .members
                .iter()
                .enumerate()
                .map(|(v, &member)| format!("V{v}({})", member_type(member)))
                .collect::<Vec<_>>()
                .join(", ");
            let _ = writeln!(
                output,
                "{derive}enum Union{index}{generic} {{ {variants} }}"
            );
            if copy {
                continue;
            }
            let view_generic = if callable { "<'a,F>".to_owned() } else if generic_payload { format!("<'a,{view_parameters}>") } else { "<'a>".to_owned() };
            let view_type = if callable { format!("ViewUnion{index}<'_,F>") } else if generic_payload { format!("ViewUnion{index}<'_,{data_arguments}>") } else { format!("ViewUnion{index}<'_>") };
            let variants = union
                .members
                .iter()
                .enumerate()
                .map(|(v, &member)| format!("V{v}(&'a {})", member_type(member)))
                .collect::<Vec<_>>()
                .join(", ");
            let view = union
                .members
                .iter()
                .enumerate()
                .map(|(v, _)| {
                    format!("Union{index}::V{v}(payload)=>ViewUnion{index}::V{v}(payload)")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let impl_generic = if callable { "<F>".to_owned() } else if generic_payload { format!("<{data_arguments}>") } else { String::new() };
            let derive = if generic_payload {
                ""
            } else {
                "#[derive(Clone,Copy)] "
            };
            let _ = writeln!(
                output,
                "{derive}enum ViewUnion{index}{view_generic} {{ {variants} }}\nimpl{impl_generic} Union{index}{type_arguments} {{ fn tsr_view(&self)->{view_type} {{ match self {{ {view} }} }} }}"
            );
            if scoped_data && data_payloads.iter().any(|&member|self.boxed_union_member(id,member)) {
                let mut lifetimes=Vec::new();
                let logical=data_payloads.iter().enumerate().map(|(position,&payload)| {
                    let count=self.native_captures.type_lifetimes(self.program,&self.borrowed_lifetime_alternatives,payload);
                    let names=(0..count).map(|slot|format!("'link{position}_{slot}")).collect::<Vec<_>>();
                    let native=self.named_record_environment_type(payload,&names);lifetimes.extend(names);native
                }).collect::<Vec<_>>();
                let bounds=logical.iter().enumerate().map(|(position,native)|format!("{}:std::borrow::Borrow<{native}>",parameter_name(position))).collect::<Vec<_>>().join(",");
                let arms=union.members.iter().enumerate().map(|(variant,member)|{
                    let payload=if let Some(position)=data_payloads.iter().position(|ty|ty==member){format!("std::borrow::Borrow::<{}>::borrow(payload)",logical[position])}else{"payload".into()};
                    format!("Union{index}::V{variant}(payload)=>ViewUnion{index}::V{variant}({payload})")
                }).collect::<Vec<_>>().join(",");
                let _=writeln!(output,"impl<{data_arguments}> Union{index}<{data_arguments}> {{ fn tsr_data_view{}(&self)->ViewUnion{index}<'_,{}> where {bounds} {{ match self {{{arms}}} }} }}",lifetime_parameters(&lifetimes),logical.join(","));
            } else if scoped_data {
                let _ = writeln!(output,"impl<{data_arguments}> Union{index}<{data_arguments}> {{ fn tsr_data_view(&self)->{view_type} {{ self.tsr_view() }} }}");
            } else if !data_payloads.is_empty() {
                let bounds = data_payloads.iter().enumerate().map(|(position, payload)| format!("{}: std::borrow::Borrow<{}>",parameter_name(position),self.native_type(*payload))).collect::<Vec<_>>().join(", ");
                let native_view = union.members.iter().enumerate().map(|(variant, member)| {
                    let payload = if data_payloads.contains(member) { format!("std::borrow::Borrow::<{}>::borrow(payload)",self.native_type(*member)) } else { "payload".to_owned() };
                    format!("Union{index}::V{variant}(payload)=>ViewUnion{index}::V{variant}({payload})")
                }).collect::<Vec<_>>().join(", ");
                let _ = writeln!(output,"impl<{bounds}> Union{index}<{data_arguments}> {{ fn tsr_data_view(&self)->ViewUnion{index}<'_> {{ match self {{ {native_view} }} }} }}");
            }
            if generic_payload {
                let parameters = if callable { "F".to_owned() } else { data_arguments.clone() };
                let _ = writeln!(output, "impl<'a,{parameters}> Copy for ViewUnion{index}<'a,{parameters}> {{}}\nimpl<'a,{parameters}> Clone for ViewUnion{index}<'a,{parameters}> {{ fn clone(&self)->Self {{ *self }} }}");
            }
            if cloneable {
                let bounds = data_payloads.iter().enumerate().map(|(position, payload)| format!("{}: std::borrow::Borrow<{}>",parameter_name(position),self.native_type(*payload))).collect::<Vec<_>>().join(", ");
                let generics = if generic_payload { format!("<'a,{bounds}>") } else { "<'a>".to_owned() };
                let arguments = if generic_payload { format!("<'a,{data_arguments}>") } else { "<'a>".to_owned() };
                let owned=union.members.iter().enumerate().map(|(v,member)| {
                    let value=if data_payloads.contains(member) { format!("std::borrow::Borrow::<{}>::borrow(payload)",self.native_type(*member)) } else { "payload".to_owned() };
                    format!("ViewUnion{index}::V{v}(payload)=>Union{index}::V{v}({})",self.union_store_payload(id,*member,&format!("Clone::clone({value})")))
                }).collect::<Vec<_>>().join(", ");
                let _ = writeln!(output, "impl{generics} ViewUnion{index}{arguments} {{ fn tsr_owned(self)->Union{index} {{ match self {{ {owned} }} }} }}");
            }
        }
    }

    // Materialized methods borrow or move the same ordinary captured fields.
    fn closure_method(
        &self,
        output: &mut String,
        index: usize,
        closure: &crate::analysis::CheckedClosure,
        method: usize,
    ) {
        let checked = &closure.methods[method];
        let error_name=self.result_methods.borrow().get(&(index,method)).cloned();
        let native_result=error_name.as_ref().map_or_else(||self.native_type(checked.function.result),|name|format!("Result<{},{}>",self.native_type(checked.function.result),name));
        let (capture_lifetimes, _) = self.capture_layout(closure);
        let lifetime = lifetime_parameters(&capture_lifetimes);
        let receiver = match checked.invocation {
            Mode::Read => "&self",
            Mode::Mutate => "&mut self",
            Mode::Consume => "self",
            Mode::Copy => unreachable!(),
        };
        let mut generics = Vec::new();
        let parameters = checked
            .parameters
            .iter()
            .map(|&id| {
                let binding = &checked.function.bindings[id];
                let mutable = if binding.mutable && (binding.mode==Mode::Consume || ((binding.omission.is_omittable() || self.multiple_data_union(binding.ty))&&binding.mode==Mode::Mutate)){"mut "}else{""};
                let native=self.native_argument_type(ArgumentContract::from(binding),"");
                format!(", {mutable}value{id}: {native}")
            })
            .collect::<String>();
        generics.extend(
            self.callback_bounds(
                checked
                    .parameters
                    .iter()
                    .map(|&id| checked.function.bindings[id].input_ty.unwrap_or(checked.function.bindings[id].ty))
                    .chain(std::iter::once(checked.function.result)),
                true,
            ),
        );
        let generics = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };
        let view_signature=self.native_view_closure_signature(closure,Some(method),&checked.function,&checked.parameters);
        let receiver=view_signature.as_ref().and_then(|signature|signature.receiver).map_or_else(||receiver.to_owned(),|mode|format!("&'native_receiver {}self",if mode==Mode::Mutate {"mut "}else{""}));
        let parameters=view_signature.as_ref().map_or(parameters,|signature|format!(", {}",signature.params));
        let generics=view_signature.as_ref().map_or(generics,|signature|signature.generics.clone());
        let native_result=view_signature.as_ref().map_or(native_result,|signature|error_name.as_ref().map_or_else(||signature.result.clone(),|name|format!("Result<{},{}>",signature.result,name)));
        let view_where=view_signature.as_ref().map_or_else(String::new,|signature|signature.replacement_where.clone());
        let _ = writeln!(
            output,
            "impl{lifetime} Closure{index}{lifetime} {{\n    fn call{method}{generics}({receiver}{parameters}) -> {}{view_where} {{",
            native_result
        );
        for (capture, &slot) in closure.captures.iter().zip(&checked.captures) {
            let binding = &checked.function.bindings[slot];
            let reference = if self.record_capture_is_view(capture,closure){if checked.invocation==Mode::Mutate{"&mut "}else{"&"}}else{match (capture.storage, checked.invocation) {
                (Mode::Copy, Mode::Mutate) if binding.capture_reference => "&mut ",
                (Mode::Consume, Mode::Read) => "&",
                (Mode::Consume, Mode::Mutate) => "&mut ",
                (Mode::Mutate, Mode::Read) => "&*",
                (Mode::Mutate, Mode::Mutate) => "&mut *",
                _ => "",
            }};
            let mutable = if (binding.mutable || self.finally_moves.partial_root(&checked.function,slot)) && !binding.capture_reference {
                "mut "
            } else {
                ""
            };
            let _ = writeln!(
                output,
                "        let {mutable}value{} = {reference}self.capture{};",
                slot, capture.inner
            );
            if self.record_capture_is_view(capture,closure){if let Some(entry)=self.record_parameter_entry_view(&checked.function,slot){let _=writeln!(output,"        {entry}");}}
        }
        if let Some(name)=&error_name {
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let id=self.completion_next.get();
            let body=self.callable_result_body(&checked.function,&checked.function.body,id,name,self.program.error_effects.method_errors[&(index,method)].clone(),runtime);
            let _=writeln!(output,"        {body}");
        }else if crate::analysis::has_exception_syntax(&checked.function.body) {
            let body=self.pure_callable_completion_body(&checked.function);
            let _=writeln!(output,"        {body}");
        }else{
            self.parameter_initialization(output, &checked.function);
            self.initialized_function_body(output, &checked.function, 2);
        }
        if matches!(closure.forward, Some(ClosureForward::RestAdapter { counted: true, .. })) {
            output.push_str("    }\n");
            let _ = writeln!(output, "    fn call{method}_with_count{generics}({receiver}, supplied_count: usize{parameters}) -> {native_result}{view_where} {{");
            self.counted_closure_function_body(output, closure, &checked.function);
            output.push_str("    }\n}\n\n");
        } else {
            output.push_str("    }\n}\n\n");
        }
    }

    fn native_call_name(&self, function: usize, args: &[Expr]) -> String {
        let registered=self.result_functions.borrow().get(&function).map(|(name,_)|name.clone());
        if self.active_plan.get().is_some() {
            let shapes=args.iter().map(|arg|self.expression_shape(arg)).collect::<Vec<_>>();
            if shapes.iter().any(Option::is_some) {
                let plan=self.fixed.call(function,&shapes);
                return registered.map_or_else(||format!("fixed_function{plan}"),|name|format!("{name}_fixed{plan}"));
            }
        }
        registered.unwrap_or_else(||format!("function{function}"))
    }

    fn conditional_invocation(&self, fun: &CheckedFunction, callee: &Expr, fallback: Mode) -> Mode {
        let mut pending = vec![callee];
        let mut result = None;
        while let Some(callee) = pending.pop() {
            match &callee.kind {
                ExprKind::Conditional { consequent, alternate, .. } => {
                    pending.push(alternate);
                    pending.push(consequent);
                }
                ExprKind::Unary {
                    op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull,
                    value, ..
                } => pending.push(value),
                ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, right, .. } => pending.push(right),
                _ => {
                    let mode = callee.binding.and_then(|binding| fun.bindings[binding].callback_invocation).unwrap_or(fallback);
                    result = Some(result.map_or(mode, |prior: Mode| prior.max(mode)));
                }
            }
        }
        result.expect("invocation leaf")
    }

    fn capture_layout(
        &self,
        closure: &crate::analysis::CheckedClosure,
    ) -> (Vec<String>, Vec<String>) {
        let (lifetimes,mut fields)=crate::callable_layout::capture_layout_with_environments(
            self.program, closure, |ty| self.native_type(ty),
            |ty, lifetimes| Some(self.named_record_environment_type(ty, lifetimes)),
            |binding| self.native_captures.ty(&closure.function, binding),
            |ty| self.native_captures.type_lifetimes(self.program, &self.borrowed_lifetime_alternatives, ty),
        );
        for (capture,field) in closure.captures.iter().zip(&mut fields){
            if !matches!(closure.forward, Some(ClosureForward::RestAdapter { .. } | ClosureForward::Abstract(_)))
                && let Type::Callback(id) = closure.function.bindings[capture.inner].ty {
                *field = field.replace(&format!("C{id}"), &format!("K{}", capture.inner));
            }
            if let Type::Closure(origin) = closure.function.bindings[capture.inner].ty
                && self.program.polymorphic_storage.contains(origin) {
                *field = field.replace(&format!("Closure{origin}"), &self.generic_storage_type(origin));
            }
            if capture.restored {*field=format!("Option<{field}>");continue;}
            if !self.record_capture_is_view(capture,closure){continue;}
            let lifetime=field.strip_prefix('&').unwrap().split_whitespace().next().unwrap();
            let start=lifetime.strip_prefix("'c").unwrap().parse::<usize>().unwrap()+1;
            *field=self.record_capture_view_type(self.native_captures.ty(&closure.function,capture.inner),capture.storage==Mode::Mutate,lifetime,&lifetimes[start..]);
        }
        (lifetimes,fields)
    }

    fn record_inspectable(&self, record: usize) -> bool {
        let mut pending = vec![Type::Record(record)];
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            match ty {
                Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callable(_) | Type::Closure(_) | Type::Callback(_) | Type::Void => return false,
                Type::Union(id) | Type::BorrowedUnion(id) => pending.extend(self.program.unions[id.index()].members.iter().copied()),
                Type::Array(element) => pending.push(element.ty()),
                Type::Set(id) => pending.push(self.program.records[id].set_definition.as_ref().unwrap().key),
                Type::Map(id) => {
                    let map=self.program.records[id].map_definition.as_ref().unwrap();
                    pending.extend([map.key,map.value]);
                },
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    if let Some(id) = record.native_type
                        && !crate::native_types::declaration(id).source_constructible()
                        && !crate::native_types::declaration(id).capabilities.contains(&crate::native_types::Capability::Inspection) {
                        return false;
                    }
                    pending.extend(record.fields.iter().map(|field| field.ty.expect("checked field")));
                },
                _ => {}
            }
        }
        true
    }

    fn console_value(&self, value: &str, kind: Type, depth: &str, nested: bool) -> String {
        let _ = nested;
        match kind {
            Type::Literal(_,kind)=>self.console_value(value,kind.base(),depth,nested),
            Type::String => format!("TsrConsoleValue::Text(({value}).as_ref().to_owned())"),
            Type::Enum(_, crate::syntax::EnumKind::Number) => format!("TsrConsoleValue::Number(({value}).tsr_value())"),
            Type::Enum(_, crate::syntax::EnumKind::String) => format!("TsrConsoleValue::Text(({value}).tsr_value().to_owned())"),
            Type::Int => format!("TsrConsoleValue::Int(*({value}))"),
            Type::Number => format!("TsrConsoleValue::Number(*({value}))"),
            Type::Boolean => format!("TsrConsoleValue::Boolean(*({value}))"),
            Type::Null => format!("{{ let _ = {value}; TsrConsoleValue::Null }}"),
            Type::Undefined => format!("{{ let _ = {value}; TsrConsoleValue::Undefined }}"),
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let observed = if nested && !id.copy && matches!(kind, Type::Union(_)) {
                    format!("({value}).tsr_view()")
                } else { value.to_owned() };
                let arms = self.program.unions[id.index()].members.iter().enumerate().map(|(index, member)| {
                    let result = self.console_value("console_payload", *member, depth, false);
                    format!("{}{}::V{index}(console_payload) => {result}", if id.copy { "Union" } else { "ViewUnion" }, id.index())
                }).collect::<Vec<_>>().join(", ");
                format!("match {observed} {{ {arms} }}")
            }
            Type::Map(id) => {
                let map = self.program.records[id].map_definition.as_ref().unwrap();
                let next = format!("({depth}) + 1");
                let key = self.console_value("console_key",map.key,&next,true);
                let cell = self.console_value("console_value",map.value,&next,true);
                format!("if ({depth}) >= 128 {{ TsrConsoleValue::Truncated }} else {{ TsrConsoleValue::Map(({value}).iter().map(|(console_key,console_value)|({key},{cell})).collect()) }}")
            }
            Type::Set(id) => {
                let key = self.program.records[id].set_definition.as_ref().unwrap().key;
                let item = self.console_value("console_item", key, &format!("({depth}) + 1"), true);
                format!("TsrConsoleValue::Set(({value}).iter().map(|console_item| {item}).collect())")
            }
            Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::StringView)=>format!("TsrConsoleValue::Text(({value}).text.to_owned())"),
            Type::Record(id) => format!("tsr_console_record_{id}({value}, {depth})"),
            Type::Array(element) => {
                let item = self.console_value(
                    "console_item",
                    element.ty(),
                    &format!("({depth}) + 1"),
                    true,
                );
                format!(
                    "if ({depth}) >= 128 {{ TsrConsoleValue::Truncated }} else {{ TsrConsoleValue::Array(({value}).iter().map(|console_item| {item}).collect()) }}"
                )
            }
            _ => unreachable!("checked console value"),
        }
    }
    fn union_observation(&self, union: UnionId, value: &str, target: CallTarget) -> String {
        let arms = self.program.unions[union.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let scalar = member.scalar_base();
                let member = &scalar;
                let result = match target {
                    CallTarget::Number => match member {
                        Type::Null => "0.0f64".into(),
                        Type::Undefined => "f64::NAN".into(),
                        Type::Int => "*payload as f64".into(),
                        Type::Number => "*payload".into(),
                        Type::Boolean => "if *payload {1.0f64} else {0.0f64}".into(),
                        Type::String => "tsr_number_from_text(payload.as_ref())".into(),
                        Type::Enum(_, crate::syntax::EnumKind::Number) => "payload.tsr_value()".into(),
                        Type::Enum(_, crate::syntax::EnumKind::String) => "tsr_number_from_text(payload.tsr_value())".into(),
                        _ => unreachable!("checked scalar Number union"),
                    },
                    CallTarget::ArrayIsArray if matches!(member, Type::Record(id) if matches!(self.program.records[*id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) => "payload.value.is_array()".into(),
                    CallTarget::ArrayIsArray => (matches!(member, Type::Array(_)) || matches!(member,Type::Record(id) if self.program.records[*id].is_tuple())).to_string(),
                    CallTarget::BooleanConversion => match member {
                        Type::Null | Type::Undefined => "false".into(),
                        Type::Boolean => "*payload".into(),
                        Type::Int => "*payload != 0".into(),
                        Type::Number => "*payload != 0.0 && !payload.is_nan()".into(),
                        Type::Enum(_, crate::syntax::EnumKind::Number) => "payload.tsr_value() != 0.0 && !payload.tsr_value().is_nan()".into(),
                        Type::Enum(_, crate::syntax::EnumKind::String) => "!payload.tsr_value().is_empty()".into(),
                        Type::String => "!payload.is_empty()".into(),
                        Type::Record(id) if self.program.records[*id].native_type==Some(crate::native_signatures::NativeTypeId::StringView) => "!payload.text.is_empty()".into(),
                        _ => "true".into(),
                    },
                    CallTarget::StringConversion | CallTarget::ObjectHasOwn => match member {
                        Type::Null => "String::Borrowed(\"null\")".into(),
                        Type::Undefined => "String::Borrowed(\"undefined\")".into(),
                        Type::String if target==CallTarget::ObjectHasOwn => "String::Borrowed(payload.as_ref())".into(),
                        Type::String => "{let text: &String = payload; text.clone()}".into(),
                        Type::Record(id) if self.program.records[*id].native_type==Some(crate::native_signatures::NativeTypeId::StringView) => "String::Owned(payload.text.to_owned())".into(),
                        Type::Record(id) if self.program.records[*id].is_tuple() => "{ let mut text = std::string::String::new(); TsrJoinText::append_join_text(payload, &mut text); String::Owned(text) }".into(),
                        Type::Map(_) => "String::Borrowed(\"[object Map]\")".into(),
                        Type::Record(_) => "String::Borrowed(\"[object Object]\")".into(),
                        Type::Array(_) => "tsr_array_join(payload.iter(), \",\")".into(),
                        other => format!(
                            "String::Owned(({}).label())",
                            self.console_value("payload", *other, "0", false)
                        ),
                    },
                    _ => unreachable!("nullable observation"),
                };
                format!(
                    "{}{}::V{index}(payload)=>{result}",
                    if union.copy { "Union" } else { "ViewUnion" },
                    union.index()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("match {value} {{ {arms} }}")
    }

    fn union_typeof(&self, union: UnionId, value: &str) -> String {
        let arms = self.program.unions[union.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                if matches!(member, Type::Record(id) if matches!(self.program.records[*id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) {
                    return format!("{}{}::V{index}(value)=>String::Borrowed(tsrust_native_json::type_of(&value.value))",if union.copy {"Union"} else {"ViewUnion"},union.index());
                }
                let scalar = member.scalar_base();
                let label = match &scalar {
                    Type::Record(id) if self.program.records[*id].native_type==Some(crate::native_signatures::NativeTypeId::StringView) => "string",
                    Type::Null | Type::Set(_) | Type::Map(_) | Type::Record(_) | Type::Array(_) => "object",
                    Type::Undefined => "undefined",
                    Type::String | Type::Enum(_, crate::syntax::EnumKind::String) => "string",
                    Type::Boolean => "boolean",
                    Type::Int | Type::Number | Type::Enum(_, crate::syntax::EnumKind::Number) => "number",
                    Type::Closure(_)
                    | Type::Callback(_)
                    | Type::StoredCallable(_)
                    | Type::BorrowedCallable(_)
                    | Type::Callable(_) => "function",
                    _ => unreachable!("nullable data member"),
                };
                format!(
                    "{}{}::V{index}(_)=>String::Borrowed({label:?})",
                    if union.copy { "Union" } else { "ViewUnion" },
                    union.index()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("match {value} {{ {arms} }}")
    }

    fn data_view_payload(&self, member: Type, payload: &str, source_view: bool) -> String {
        if matches!(member, Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_)) { return payload.to_owned(); }
        if source_view {
            format!("{{ let selected: &{} = {payload}; selected }}", self.native_type(member))
        } else {
            let slot = self.conditional_next.get();
            self.conditional_next.set(slot + 1);
            self.conditional_temporaries.borrow_mut().push(format!("let conditional_temporary_{slot};"));
            format!("{{ conditional_temporary_{slot} = {payload}; &conditional_temporary_{slot} }}")
        }
    }

    /// Assertion diagnostics own their message text; user values remain borrowed.
    #[inline(never)]
    fn typed_native_assertion(&self,arguments:&[String],span:crate::Span,closure:Option<usize>)->Option<String> {
        let (name,contract,runtime)=if let Some(id)=closure {
            let name=self.result_closures.borrow().get(&id).cloned()?;
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            (name,self.program.error_effects.closure_errors[id].clone(),runtime)
        }else{self.exception_regions.current_transport()?};
        if !contract.runtime {return None;}
        let Some(Type::Record(record))=runtime else{return None;};
        let error=self.runtime_error_value(record,"AssertionError","assertion","",span);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let failed=if closure.is_some() {
            let payload=exception_contract_emit::checked_throw_value("assertion_error",Type::Record(record),&name,&contract,&unions,runtime,span).expect("checked native assertion contract");
            format!("Err({payload})")
        }else{
            self.exception_regions.throw_value("assertion_error",Type::Record(record),&unions,span).expect("checked native assertion region")
        };
        let succeeded=if closure.is_some(){"Ok(())"}else{"()"};
        Some(format!("{{let assertion_condition={};let assertion_message={};if assertion_condition{{{succeeded}}}else{{let mut assertion_error={error};assertion_error.field2=String::Owned(assertion_message.as_ref().to_owned());{failed}}}}}",arguments[0],arguments[1]))
    }

    #[inline(never)]
    fn typed_runner_execution(&self,span:crate::Span,closure:Option<usize>)->Option<String> {
        self.typed_runner_result("tsrust_testing_run_tests_result()",span,closure)
    }

    #[inline(never)]
    fn typed_runner_result(&self,call:&str,span:crate::Span,closure:Option<usize>)->Option<String> {
        self.typed_native_result(call,span,closure,"Error","testing")
    }

    #[inline(never)]
    fn typed_native_result(&self,call:&str,span:crate::Span,closure:Option<usize>,error_name:&str,error_code:&str)->Option<String> {
        let (name,contract,runtime)=if let Some(id)=closure {
            let name=self.result_closures.borrow().get(&id).cloned()?;
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            (name,self.program.error_effects.closure_errors[id].clone(),runtime)
        }else{self.exception_regions.current_transport()?};
        let Some(Type::Record(record))=runtime else{return None;};
        let value=self.runtime_error_value(record,error_name,error_code,"",span);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let failed=if closure.is_some() {
            let payload=exception_contract_emit::checked_throw_value("runner_error",Type::Record(record),&name,&contract,&unions,runtime,span).expect("checked runner contract");
            format!("Err({payload})")
        }else{self.exception_regions.throw_value("runner_error",Type::Record(record),&unions,span).expect("checked runner region")};
        let success=if closure.is_some(){"Ok(value)"}else{"value"};
        Some(format!("match {call}{{Ok(value)=>{success},Err(message)=>{{let mut runner_error={value};runner_error.field2=String::Owned(message);{failed}}}}}"))
    }

    #[inline(never)]
    fn typed_test_registration(&self,arguments:&[String],callback:Type,span:crate::Span,closure:Option<usize>,hook:bool)->Option<String> {
        let registered=match callback {
            Type::Callback(id)=>self.result_callbacks.borrow().get(&id).cloned(),
            Type::StoredCallable(id)=>self.result_storage.borrow().get(&id).cloned(),
            Type::Closure(id)=>self.result_closures.borrow().get(&id).cloned().map(|name|(name,self.program.error_effects.closure_errors[id].clone())),
            _=>return None,
        };
        let invocation="ProgramCallback::invoke_program(&mut registered_callback,(),program,registry)";
        let result=if let Some((name,contract))=registered {
            let invocation=format!("{{let result:Result<(),{name}>={invocation};result}}");
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let payloads=contract.native_payloads(&unions,runtime,span).expect("checked runner callback contract");
            let mut arms=String::new();
            for (tag,ty) in payloads.into_iter().enumerate() {
                let text=if Some(ty)==runtime {"std::format!(\"{}: {}\",payload.field0.as_ref(),payload.field2.as_ref())".to_owned()}else{
                    let value=self.console_value("payload",ty,"0",false);
                    format!("tsr_console_format(&[{value}])")
                };
                arms.push_str(&format!("{}::V{tag}(payload)=>{text},",exception_emit::constructor_path(&name)));
            }
            if arms.is_empty(){format!("{invocation}.map_err(|error|match error{{}})")}else{format!("{invocation}.map_err(|error|match error{{{arms}}})")}
        }else{format!("{{let result:()={invocation};Ok(result)}}")};
        let helper=if hook{"tsrust_testing_register_hook_result"}else{"tsrust_testing_register_test_result"};
        let call=format!("{{let registered_name={};let mut registered_callback={};{helper}({},registered_name,Box::new(move |program,registry|{result}))}}",arguments[0],arguments[1],arguments[2]);
        self.typed_runner_result(&call,span,closure)
    }

    fn ui_press_adapter(&self,arguments:&[String],callback:Type,span:crate::Span)->String {
        let registered=match callback {
            Type::Callback(id)=>self.result_callbacks.borrow().get(&id).cloned(),
            Type::StoredCallable(id)=>self.result_storage.borrow().get(&id).cloned(),
            Type::Closure(id)=>self.result_closures.borrow().get(&id).cloned().map(|name|(name,self.program.error_effects.closure_errors[id].clone())),
            _=>None,
        };
        let invocation="Callback::invoke(&mut registered_callback,())";
        let result=if let Some((name,contract))=registered {
            let invocation=format!("{{let result:Result<(),{name}>={invocation};result}}");
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let payloads=contract.native_payloads(&unions,runtime,span).expect("checked runner callback contract");
            let mut arms=String::new();
            for (tag,ty) in payloads.into_iter().enumerate() {
                let text=if Some(ty)==runtime {"std::format!(\"{}: {}\",payload.field0.as_ref(),payload.field2.as_ref())".to_owned()}else{
                    let value=self.console_value("payload",ty,"0",false);
                    format!("tsr_console_format(&[{value}])")
                };
                arms.push_str(&format!("{}::V{tag}(payload)=>{text},",exception_emit::constructor_path(&name)));
            }
            if arms.is_empty(){format!("{invocation}.map_err(|error|match error{{}})")}else{format!("{invocation}.map_err(|error|match &error{{{arms}}})")}
        }else{format!("{{let result:()={invocation};Ok(result)}}")};
        format!("{{let label={};let class_name={};let disabled={};let mut registered_callback={};tsrust_ui_press_button(label,class_name,disabled,move ||{result})}}",arguments[0],arguments[1],arguments[2],arguments[3])
    }
    #[inline(never)]
    fn runtime_error_value(&self,record:usize,name:&str,code:&str,message:&str,span:crate::Span)->String {
        let location=self.source_segments.iter().find(|(_,base,len)|span.start>=*base && span.end>=span.start && span.end<=base+len);
        let (source,start,end)=location.map_or(("<source>",span.start,span.end),|(source,base,_)|(source.0.as_str(),span.start-base,span.end-base));
        exception_emit::runtime_error_value_with_order(record,self.record_runtime.ordered.contains(&record),name,code,message,source,start,end)
    }

    #[inline(never)]
    fn runtime_failure_exit(&self, name: &str, code: &str, message: &str, span: crate::Span) -> Option<String> {
        let (_, contract, runtime) = self.exception_regions.current_transport()?;
        if !contract.runtime { return None; }
        let Some(Type::Record(record)) = runtime else { return None; };
        let failure = self.runtime_error_value(record,name,code,message,span);
        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        Some(format!("{{ {} }}", self.exception_regions.throw_value(&failure, Type::Record(record), &unions, span).expect("checked runtime failure transport")))
    }

    #[inline(never)]
    fn non_null_projection(&self,fun:&CheckedFunction,expr:&Expr,value:&Expr,operator:crate::Span,mode:Mode)->String {
        if let ExprKind::Project{value:source,union,narrowed}=&value.kind {
            let message=format!("Non-null assertion failed: value is null or undefined at source bytes {}..{}",operator.start,operator.end);
            let recovery = if expr.own_may_fail {
                self.runtime_failure_exit("TypeError", "non-null", &message, operator)
            } else { None };
            self.union_projection_with_failure(fun,value,source,*union,*narrowed,mode,&message,recovery.as_deref())
        }else{self.expr(fun,value,mode)}
    }

    fn union_projection(&self,fun:&CheckedFunction,expr:&Expr,value:&Expr,union:UnionId,narrowed:Type,mode:Mode,failure:&str)->String {
        self.union_projection_with_failure(fun,expr,value,union,narrowed,mode,failure,None)
    }

    fn union_projection_with_failure(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        value: &Expr,
        union: UnionId,
        narrowed: Type,
        mode: Mode,
        failure: &str,
        recovery: Option<&str>,
    ) -> String {
        if expr.ty.and_then(Type::union) == Some(union) {
            return self.expr(fun, value, mode);
        }
        let failed=recovery.map(str::to_owned).unwrap_or_else(||format!("panic!({failure:?})"));
        let members = &self.program.unions[union.index()].members;
        if let Type::Union(target) = narrowed {
            let read_view = !union.copy && (mode == Mode::Read || target.copy);
            let actual_mode = if union.copy { Mode::Copy } else if read_view { Mode::Read } else { Mode::Consume };
            let source = self.expr(fun, value, actual_mode);
            let from_kind = if read_view { "ViewUnion" } else { "Union" };
            let to_kind = if read_view && !target.copy { "ViewUnion" } else { "Union" };
            let arms = self.program.unions[target.index()].members.iter().enumerate().map(|(to, ty)| {
                let from = members.iter().position(|member| member == ty).expect("subset narrowing");
                let payload = if read_view && target.copy { "*payload".to_owned() } else if read_view { self.data_view_payload(*ty,"payload",true) } else { self.union_transfer_payload(union,target,*ty,"payload") };
                format!("{from_kind}{}::V{from}(payload) => {to_kind}{}::V{to}({payload})",union.index(),target.index())
            }).collect::<Vec<_>>().join(", ");
            return format!("match {source} {{ {arms}, _ => {failed} }}");
        }
        let variant = members
            .iter()
            .position(|ty| *ty == narrowed)
            .expect("proven payload alternative");
        let access = if union.copy {
            Mode::Copy
        } else if narrowed.is_copy() || matches!(value.ty,Some(Type::BorrowedUnion(_))) {
            // A temporary observation may request an owning carrier, but a
            // borrowed optional payload still uses its readonly union form.
            Mode::Read
        } else {
            mode
        };
        let source = if failure != "checked nullable guard" && Self::temporary_storage(value) {
            self.conditional_arm(fun, value, access)
        } else { self.expr(fun, value, access) };
        let result = if narrowed.is_copy() && !union.copy {
            "*payload".to_owned()
        } else if access==Mode::Consume {
            self.union_move_payload(union,narrowed,"payload")
        } else if access==Mode::Mutate && self.boxed_union_member(union,narrowed) {
            "&mut **payload".to_owned()
        } else { "payload".to_owned() };
        format!(
            "match {source} {{ {}{}::V{variant}(payload) => {result}, _ => {failed} }}",
            if !union.copy && access == Mode::Read {
                "ViewUnion"
            } else {
                "Union"
            },
            union.index()
        )
    }

    fn absent_callable_initializer(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Inject { value, .. } | ExprKind::Unary { value, .. } => {
                Self::absent_callable_initializer(value)
            }
            ExprKind::Null | ExprKind::Undefined => true,
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                Self::absent_callable_initializer(consequent)
                    && Self::absent_callable_initializer(alternate)
            }
            _ => false,
        }
    }

    fn union_injection(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        value: &Expr,
        union: UnionId,
        mode: Mode,
        infer_callable_carrier: bool,
    ) -> String {
        if matches!(expr.ty, Some(Type::BorrowedUnion(_))) {
            let actual = value.ty.expect("view injection");
            let source = if infer_callable_carrier && !actual.is_copy() {
                self.conditional_arm(fun, value, Mode::Read)
            } else {
                self.expr(fun, value, Mode::Read)
            };
            return self.finish_borrowed_union_injection(value, union, infer_callable_carrier, source);
        }
        if matches!(mode, Mode::Read | Mode::Mutate) {
            return self.conditional_arm(fun, expr, mode);
        }
        let target = &self.program.unions[union.index()].members;
        let actual = value.ty.expect("checked injection");
        let code = self.expr(
            fun,
            value,
            if actual.is_copy() {
                Mode::Copy
            } else {
                Mode::Consume
            },
        );
        if let Type::Union(source) = actual {
            let arms = self.program.unions[source.index()]
                .members
                .iter()
                .enumerate()
                .map(|(index, member)| {
                    let next = target
                        .iter()
                        .position(|ty| {
                            ty.accepts_scalar_value(*member)
                                || matches!(ty, Type::Callback(_))
                                    && matches!(member, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))
                                // A concrete environment of the same nominal record.
                                || matches!((ty, member), (Type::Record(wanted), Type::Record(found)) if self.program.scoped_records.declaration(*wanted) == self.program.scoped_records.declaration(*found))
                        })
                        .expect("subset union widening");
                    format!(
                        "Union{}::V{index}(payload)=>Union{}::V{next}({})",
                        source.index(),union.index(),self.union_transfer_payload(source,union,*member,"payload")
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("match {code} {{ {arms} }}")
        } else {
            let variant = target
                .iter()
                .position(|ty| ty.accepts_scalar_value(actual))
                .or_else(|| {
                    target.iter().position(|ty| {
                        matches!(ty, Type::Callback(_))
                            && matches!(actual, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))
                    })
                })
                .expect("union member injection");
            if matches!(self.callback_payload(Type::Union(union)),Some(Type::Callback(id)) if !self.program.absent_callback_payloads.contains(&id))
                && matches!(actual, Type::Null | Type::Undefined)
            {
                format!(
                    "Union{}::<TsrAbsentCallable>::V{variant}({code})",
                    union.index()
                )
            } else {
                format!("Union{}::V{variant}({})", union.index(),self.union_store_payload(union,actual,&code))
            }
        }
    }

    fn finish_borrowed_union_injection(&self, value: &Expr, union: UnionId, infer_callable_carrier: bool, source: String) -> String {
        let target = &self.program.unions[union.index()].members;
        let actual = value.ty.expect("view injection");
            if let Some(from) = actual.union() {
                let arms = self.program.unions[from.index()]
                    .members
                    .iter()
                    .enumerate()
                    .map(|(i, member)| {
                        let to = target
                            .iter()
                            .position(|ty| {
                                ty.accepts_scalar_value(*member)
                                    || matches!((ty,member),(Type::Record(left),Type::Record(right)) if self.program.scoped_records.declaration(*left)==self.program.scoped_records.declaration(*right))
                                    || matches!(ty, Type::Callback(_))
                                        && matches!(member, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))
                            })
                            .expect("view subset");
                        format!(
                            "ViewUnion{}::V{i}(payload)=>ViewUnion{}::V{to}(payload)",
                            from.index(),
                            union.index()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                return format!("match {source} {{ {arms} }}");
            }
            let to = target
                .iter()
                .position(|ty| {
                    ty.accepts_scalar_value(actual)
                        || matches!(ty, Type::Callback(_))
                            && matches!(actual, Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_))
                })
                .expect("view member");
            let source = if actual.is_copy() {
                format!("&({source})")
            } else {
                source
            };
            if !infer_callable_carrier
                && matches!(self.callback_payload(Type::Union(union)), Some(Type::Callback(id)) if !self.program.absent_callback_payloads.contains(&id))
                && matches!(actual, Type::Null | Type::Undefined)
            {
                return format!(
                    "ViewUnion{}::<TsrAbsentCallable>::V{to}({source})",
                    union.index()
                );
            }
            if matches!(actual, Type::Null | Type::Undefined)
                && let Some(payload) = self.program.unions[union.index()].members.iter().copied().find(|ty| matches!(ty, Type::Record(_)))
                && (self.scoped_record_native_parameter(payload) || matches!(payload,Type::Record(id) if self.borrowing_records.contains(&self.program.scoped_records.declaration(id))))
            {
                return format!("ViewUnion{}::<{}>::V{to}({source})", union.index(), self.native_type(payload));
            }
            return format!("ViewUnion{}::V{to}({source})", union.index());
    }

    fn value_builtin_bindings(&self, fun: &CheckedFunction, args: &[Expr]) -> String {
        let mut code = String::from("{ ");
        for (index, argument) in args.iter().enumerate() {
            let mode = if argument.ty.unwrap().is_copy() {
                Mode::Copy
            } else {
                Mode::Read
            };
            let _ = write!(
                code,
                "let builtin_argument_{index} = {}; ",
                self.conditional_arm(fun, argument, mode)
            );
        }
        code
    }

    fn value_builtin_call(
        &self,
        fun: &CheckedFunction,
        target: CallTarget,
        args: &[Expr],
    ) -> String {
        let mut code = self.value_builtin_bindings(fun, args);
        let first = args.first();
        let result = if let Some(union) = first.and_then(|arg| arg.ty).and_then(Type::union) {
            self.union_observation(
                union,
                if Type::Union(union).is_copy() {
                    "&builtin_argument_0"
                } else {
                    "builtin_argument_0"
                },
                target,
            )
        } else {
            match target {
                CallTarget::ArrayIsArray if first.is_some_and(|arg| matches!(arg.ty, Some(Type::Record(id)) if matches!(self.program.records[id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView)))) => "builtin_argument_0.value.is_array()".into(),
                CallTarget::ArrayIsArray => first
                    .is_some_and(|a| matches!(a.ty, Some(Type::Array(_))) || matches!(a.ty,Some(Type::Record(id)) if self.program.records[id].is_tuple()))
                    .to_string(),
                CallTarget::BooleanConversion => match first.map(|a| a.ty.unwrap().scalar_base()) {
                    None | Some(Type::Null | Type::Undefined | Type::Void) => "false".into(),
                    Some(Type::Boolean) => "builtin_argument_0".into(),
                    Some(Type::Int) => "builtin_argument_0 != 0".into(),
                    Some(Type::Number) => {
                        "builtin_argument_0 != 0.0 && !builtin_argument_0.is_nan()".into()
                    }
                    Some(Type::Enum(_, crate::syntax::EnumKind::Number)) => "builtin_argument_0.tsr_value() != 0.0 && !builtin_argument_0.tsr_value().is_nan()".into(),
                    Some(Type::Enum(_, crate::syntax::EnumKind::String)) => "!builtin_argument_0.tsr_value().is_empty()".into(),
                    Some(Type::String) => "!builtin_argument_0.is_empty()".into(),
                    Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::StringView) => "!builtin_argument_0.text.is_empty()".into(),
                    _ => "true".into(),
                },
                CallTarget::StringConversion => match first.map(|a| a.ty.unwrap().scalar_base()) {
                    None => "String::Borrowed(\"\")".into(),
                    Some(Type::Null) => "String::Borrowed(\"null\")".into(),
                    Some(Type::Undefined | Type::Void) => "String::Borrowed(\"undefined\")".into(),
                    Some(Type::String) => "(*builtin_argument_0).clone()".into(),
                    Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::StringView)=>"String::Owned(builtin_argument_0.text.to_owned())".into(),
                    Some(Type::Record(id)) if self.program.records[id].is_tuple() => "{ let mut text = std::string::String::new(); TsrJoinText::append_join_text(builtin_argument_0, &mut text); String::Owned(text) }".into(),
                    Some(Type::Map(_)) => "String::Borrowed(\"[object Map]\")".into(),
                    Some(Type::Record(_)) => "String::Borrowed(\"[object Object]\")".into(),
                    Some(Type::Array(_)) => {
                        let iterator = if self.expression_shape(first.unwrap())
                            == Some(crate::fixed_storage::Shape::LiteralStrings)
                        {
                            "builtin_argument_0.literal_iter()"
                        } else {
                            "builtin_argument_0.iter()"
                        };
                        format!("tsr_array_join({iterator}, \",\")")
                    }
                    Some(kind) => {
                        let reference = if kind.is_copy() {
                            "&builtin_argument_0"
                        } else {
                            "builtin_argument_0"
                        };
                        format!(
                            "String::Owned(({}).label())",
                            self.console_value(reference, kind, "0", false)
                        )
                    }
                },
                _ => unreachable!("checked value builtin"),
            }
        };
        let _ = write!(code, "{result} }}");
        code
    }

    fn console_call(&self, fun: &CheckedFunction, callee: &Expr, args: &[Expr]) -> String {
        let ExprKind::Field(_, method) = &callee.kind else {
            unreachable!("console method")
        };
        let mut code = String::from("{ ");
        // Borrowed temporaries survive all argument evaluations. Primitive values
        // are copied immediately so later effects cannot change earlier values.
        for (index, argument) in args.iter().enumerate() {
            if matches!(argument.ty, Some(Type::Union(id)) if !id.copy)
                && matches!(argument.kind, ExprKind::Call(_, _)) {
                let owner = self.expr(fun, argument, Mode::Consume);
                let _ = write!(code, "let console_owner_{index} = {owner}; let console_argument_{index} = console_owner_{index}.tsr_view(); ");
                continue;
            }
            let value = self.expr(
                fun,
                argument,
                if argument.ty.unwrap().is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
            );
            let _ = write!(code, "let console_argument_{index} = {value}; ");
        }
        if let Some(graph)=self.source_graph {
            if let Some((source,base,_))=self.source_segments.iter().find(|(_,base,len)|callee.span.start>=*base && callee.span.end<=base+len) {
                let text=&graph.modules[source].source.text;
                let offset=callee.span.start-base;
                if let Some(prefix)=text.get(..offset) {
                    let line=prefix.bytes().filter(|byte|*byte==b'\n').count()+1;
                    let column=prefix.rsplit('\n').next().unwrap_or("").encode_utf16().count()+1;
                    let _=write!(code,"let _console_location = TsrConsoleLocation::new({:?}, {line}, {column}); ",source.0);
                }
            }
        }
        let values = args
            .iter()
            .enumerate()
            .map(|(index, argument)| {
                let value = if argument.ty.unwrap().is_copy() {
                    format!("&console_argument_{index}")
                } else {
                    format!("console_argument_{index}")
                };
                self.console_value(&value, argument.ty.unwrap(), "0", false)
            })
            .collect::<Vec<_>>();
        if matches!(
            method.as_str(),
            "log" | "info" | "debug" | "warn" | "error" | "dirxml"
        ) && args.len() == 1
        {
            let error = matches!(method.as_str(), "warn" | "error");
            match args[0].ty.unwrap() {
                Type::Int | Type::Boolean => {
                    let _ = write!(code, "tsr_console_scalar({error}, console_argument_0); }}");
                    return code;
                }
                Type::String => {
                    let _ = write!(
                        code,
                        "tsr_console_text({error}, console_argument_0.as_ref()); }}"
                    );
                    return code;
                }
                _ => {}
            }
        }
        if method == "table" {
            let columns = if args.len() == 2 && args[1].ty != Some(Type::Undefined) {
                "Some(console_argument_1.iter().map(|column| column.to_string()).collect())"
            } else {
                "None"
            };
            if values.is_empty() {
                code.push_str("tsr_console_write(false, \"undefined\\n\"); ");
            } else {
                let _ = write!(
                    code,
                    "let console_data = {}; tsr_console_write(false, &tsr_console_table(&console_data, {columns})); ",
                    values[0]
                );
            }
        } else {
            let _ = write!(
                code,
                "tsr_console_call({method:?}, &[{}]); ",
                values.join(", ")
            );
        }
        code.push('}');
        code
    }
    fn parameter_initialization(&self, output: &mut String, fun: &CheckedFunction) {
        for (slot, binding) in fun.bindings.iter().enumerate() {
            if let Some(owner) = binding.name.strip_prefix(crate::data_lowering::ACCESS) {
                let index = self.program.functions.iter().position(|function| function.name == owner).expect("checked data owner");
                let _ = writeln!(output, "let value{slot} = &*TSR_DATA{index};");
            }
        }

        for id in 0..fun.parameter_count {
            if self.finally_moves.partial_root(fun,id) && !self.restored_capture(fun,id)
                && fun.bindings[id].mode==Mode::Consume && self.owned_record_layout(fun,id).is_none() {
                let _=writeln!(output,"let mut value{id}=value{id};");
            }
        }
        for id in 0..fun.parameter_count {
            if fun.bindings[id].input_ty.is_none(){if let Some(entry)=self.owned_record_declaration(fun,id,&format!("value{id}")){let _=writeln!(output,"{entry}");}}
        }
        for id in 0..fun.bindings.len() {
            if fun.bindings[id].borrowed_element.is_none() && let Some(entry)=self.record_parameter_entry_view(fun,id){let _=writeln!(output,"{entry}");}
        }
        for (id,binding) in fun.bindings.iter().enumerate().filter(|(_,binding)|binding.input_ty.is_some()) {
            self.parameter_default_initialization(output,fun,id);
            if id<fun.parameter_count{if let Some(entry)=self.owned_record_declaration(fun,id,&format!("value{id}")){let _=writeln!(output,"{entry}");}}
            if !binding.name.starts_with("pattern#") { continue; }
            let Some(span)=binding.declaration_span else {continue;};
            let patterns=fun.body.iter().take_while(|stmt| matches!(stmt.kind,StmtKind::Let{..})).filter(|stmt|stmt.span.start>=span.start && stmt.span.end<=span.end).cloned().collect::<Vec<_>>();
            self.block(output,fun,&patterns,0);
        }
    }

    fn initialized_function_body(&self, output: &mut String, fun: &CheckedFunction, indent: usize) {
        let _callable_scope=self.exception_regions.callable_boundary();
        let _control_boundary=self.control_targets.callable_boundary();
        let count=fun.body.iter().take_while(|stmt|fun.bindings.iter().any(|binding|binding.input_ty.is_some() && binding.name.starts_with("pattern#") && binding.declaration_span.is_some_and(|span|stmt.span.start>=span.start && stmt.span.end<=span.end))).count();
        self.block(output,fun,&fun.body[count..],indent);
    }

    fn parameter_default_initialization(&self, output: &mut String, fun: &CheckedFunction, id: usize) {
            let binding=&fun.bindings[id];
            let Some(value) = &binding.initializer else { return; };
            let Some(Type::Union(input)) = binding.input_ty else { return; };
            let borrowed = matches!(binding.mode,Mode::Read|Mode::Mutate) && !input.copy;
            let source = format!("value{id}");
            let source_name = if borrowed && binding.mode == Mode::Read { "ViewUnion" } else { "Union" };
            let callback = matches!(binding.ty,Type::Callback(_));
            let mut default_source=value;
            while let ExprKind::Inject{value,..}=&default_source.kind {default_source=value;}
            let borrowed_default = borrowed && (default_source.binding.is_some_and(|id|fun.bindings[id].parameter) || matches!(value.kind,ExprKind::Unary{op:UnaryOp::ParameterMutableInput,..}));
            let fallback = self.expr(fun, value, if binding.ty.is_copy() {Mode::Copy} else if borrowed_default {binding.mode} else {Mode::Consume});
            let mut absent_value = value;
            while let ExprKind::Inject {value,..} = &absent_value.kind {absent_value=value;}
            let absent = match absent_value.kind {ExprKind::Null=>Some(Type::Null),ExprKind::Undefined=>Some(Type::Undefined),_=>None};
            let absent_body = absent.and_then(|absent|binding.ty.union().map(|body|(absent,body)));
            if borrowed && absent_body.is_none() && let Some(body)=binding.ty.union()
                && self.callback_payload(binding.ty).is_some() {
                // The callable origin selected by the supplied/default branch is
                // a local native enum; borrowed origins stay references within it.
                let default_source=if borrowed_default {self.expr(fun,value,binding.mode)}else{self.expr(fun,value,Mode::Consume)};
                let default_union=value.ty.unwrap().union().expect("nullable checked default");
                let default_tag=if matches!(value.ty,Some(Type::BorrowedUnion(_))) || borrowed_default && binding.mode==Mode::Read {"ViewUnion"}else{"Union"};
                let map=|from:crate::syntax::UnionId,tag:&str,selection:&str|self.program.unions[from.index()].members.iter().enumerate().map(|(variant,member)|{
                    let to=self.program.unions[body.index()].members.iter().position(|ty|ty==member || !matches!(member,Type::Null|Type::Undefined)&&matches!(ty,Type::Callback(_)|Type::Closure(_))).expect("nullable default member");
                    let payload=if matches!(member,Type::Null|Type::Undefined){if tag=="ViewUnion"{"*payload".into()}else{"payload".into()}}else{format!("TsrDefault::{selection}(payload)")};
                    format!("{tag}{}::V{variant}(payload)=>Union{}::V{to}({payload})",from.index(),body.index())
                }).collect::<Vec<_>>().join(",");
                let default=format!("match {default_source} {{{}}}",map(default_union,default_tag,"Default"));
                let arms=self.program.unions[input.index()].members.iter().enumerate().map(|(variant,member)|{
                    if *member==Type::Undefined {format!("{source_name}{}::V{variant}(payload)=>{default}",input.index())}
                    else {
                        let to=self.program.unions[body.index()].members.iter().position(|ty|ty==member || !matches!(member,Type::Null|Type::Undefined)&&matches!(ty,Type::Callback(_)|Type::Closure(_))).expect("supplied callable member");
                        let payload:String=if matches!(member,Type::Null|Type::Undefined){if source_name=="ViewUnion"{"*payload".into()}else{"payload".into()}}else{"TsrDefault::Supplied(payload)".into()};
                        format!("{source_name}{}::V{variant}(payload)=>Union{}::V{to}({payload})",input.index(),body.index())
                    }
                }).collect::<Vec<_>>().join(",");
                let _=writeln!(output,"let mut value{id}=match {source} {{{arms}}};");
                if binding.mode==Mode::Read {let _=writeln!(output,"let value{id}=value{id}.tsr_view();");}
                return;
            }
            let fresh_borrow = borrowed && !borrowed_default && !binding.ty.is_copy() && absent_body.is_none();
            if fresh_borrow { let _ = writeln!(output,"let mut default_owner{id};"); }
            let default = if let Some((absent,body))=absent_body {
                    let to=self.program.unions[body.index()].members.iter().position(|ty|*ty==absent).expect("default absence belongs to body");
                    let tag=if borrowed && binding.mode==Mode::Read {"ViewUnion"}else{"Union"};
                    let payload=if absent==Type::Null {"TsrNull"}else{"TsrUndefined"};
                    let reference=if borrowed && binding.mode==Mode::Read {"&"}else{""};
                    format!("{{let _={};{tag}{}::V{to}({reference}{payload})}}",self.expr(fun,absent_value,Mode::Copy),body.index())
                } else if callback && !fresh_borrow { format!("TsrDefault::Default({fallback})") }
                else if fresh_borrow {
                    let reference = if binding.mode == Mode::Mutate { "&mut " } else { "&" };
                    let view = if let Some(body)=binding.ty.union() {
                        if binding.mode==Mode::Mutate {
                            let arms=self.program.unions[body.index()].members.iter().enumerate().map(|(variant,member)|format!("Union{}::V{variant}(payload)=>Union{}::V{variant}({})",body.index(),body.index(),if member.is_copy(){"*payload"}else{"payload"})).collect::<Vec<_>>().join(",");
                            format!("match &mut default_owner{id} {{{arms}}}")
                        }else{format!("default_owner{id}.tsr_view()")}
                    } else {format!("{reference}default_owner{id}")};
                    let view = if callback {format!("TsrDefault::Default({view})")} else {view};
                    format!("{{ default_owner{id}={fallback}; {view} }}")
                } else {fallback};
            let arms = self.program.unions[input.index()].members.iter().enumerate().map(|(variant,member)| {
                let result = if *member == Type::Undefined {default.clone()}
                    else if callback {"TsrDefault::Supplied(payload)".into()}
                    else if let Type::StoredCallable(family)=binding.ty {
                        if let Type::Closure(origin)=member {
                            self.stored_callable_constructor(family, *origin, "payload")
                        } else if *member==binding.ty {"payload".into()} else {"match payload {}".into()}
                    }
                    else if let Some(body) = binding.ty.union() {
                        let to = self.program.unions[body.index()].members.iter().position(|ty|ty==member || matches!(ty,Type::StoredCallable(_)|Type::Callback(_))&&!matches!(member,Type::Null|Type::Undefined)).expect("accepted supplied member");
                        let payload=if let Type::StoredCallable(family)=self.program.unions[body.index()].members[to] {
                            if let Type::Closure(origin)=member {
                                self.stored_callable_constructor(family, *origin, "payload")
                            }else if matches!(member,Type::Callback(_)){"match payload {}".into()}else{"payload".into()}
                        }else{"payload".into()};
                        let tag = if borrowed && binding.mode == Mode::Read {"ViewUnion"} else {"Union"};
                        format!("{tag}{}::V{to}({payload})",body.index())
                    } else if borrowed && binding.ty.is_copy() {"*payload".into()} else {"payload".into()};
                format!("{source_name}{}::V{variant}(payload)=>{result}", input.index())
            }).collect::<Vec<_>>().join(",");
            let _ = writeln!(output,"let mut value{id}=match {source} {{{arms}}};");
    }

    fn counted_invocation(
        &self,
        method: &str,
        receiver: &str,
        arguments: &str,
        count: impl std::fmt::Display,
    ) -> String {
        if self.counted_calls {
            format!("{method}_with_count({receiver}, {arguments}, {count})")
        } else {
            format!("{method}({receiver}, {arguments})")
        }
    }

    fn counted_call(&self, receiver: &str, arguments: &str, count: impl std::fmt::Display) -> String {
        if self.counted_calls {
            let arguments = if arguments.is_empty() {
                String::new()
            } else {
                format!(", {arguments}")
            };
            format!("{receiver}.call_with_count({count}{arguments})")
        } else {
            format!("{receiver}.call({arguments})")
        }
    }

    fn callback_result_type(&self,id:usize,result:Type)->String {
        let registered=self.result_callbacks.borrow().get(&id).map(|(name,_)|name.clone());
        registered.map_or_else(||self.native_type(result),|name|format!("Result<{},{}>",self.native_type(result),name))
    }

    #[inline(never)]
    fn registered_callback_call(&self,call:String,id:usize,span:crate::Span)->String {
        let call=if self.program.callback_contracts[id].kind==CallbackKind::Task {format!("({}).await",call.replace("OnceCallback::","TaskCallback::"))}else{call};
        let registered=self.result_callbacks.borrow().get(&id).cloned();
        if let Some((name,contract))=registered {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            self.exception_regions.result_call(&call,&name,&contract,&unions,span).expect("checked registered callback Result call")
        } else {
            call
        }
    }

    #[inline(never)]
    fn registered_traversal_call(&self,call:String,callback:&Expr)->String {
        // A concrete family may implement multiple checked Result adapters.
        // Traversal uses its canonical storage error ABI, not an inferred bridge.
        let call = if let Some(Type::StoredCallable(family) | Type::BorrowedCallable(family)) = callback.ty
            && let Some((error, _)) = self.result_storage.borrow().get(&family) {
            let declaration = self.program.stored_contracts.declaration(family).expect("checked traversal family");
            let result = format!("Result<{},{}>", self.native_type(self.program.callback_contracts[declaration].result), error);
            if let Some(method) = call.strip_prefix("ReadCallback::") {
                format!("ReadCallback::<_,{result}>::{method}")
            } else if let Some(method) = call.strip_prefix("Callback::") {
                format!("Callback::<_,{result}>::{method}")
            } else { call }
        } else { call };
        match callback.ty.expect("checked traversal callback") {
            Type::Callback(id)=>self.registered_callback_call(call,id,callback.span),
            Type::StoredCallable(family)|Type::BorrowedCallable(family)=>self.registered_storage_call(call,family,callback.span),
            Type::Closure(origin)=>{
                if let Some(method)=callback.traversal_method {
                    if let Some(name)=self.result_methods.borrow().get(&(origin,method.id)).cloned() {
                        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(),&self.program.records);
                        return self.exception_regions.result_call(&call,&name,&self.program.error_effects.method_errors[&(origin,method.id)],&unions,callback.span).expect("checked traversal method Result call");
                    }
                    call
                }else{self.registered_closure_call(call,origin,callback.span)}
            },
            _=>unreachable!("checked reusable traversal callback"),
        }
    }

    /// Prepare transport declarations without partially activating an invalid ABI.
    #[inline(never)]
    fn prepare_result_declarations(&self)->Result<String,crate::Diagnostic> {
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let mut declarations=String::new();let mut functions=HashMap::new();let mut closures=HashMap::new();let mut methods=HashMap::new();let mut storage=HashMap::new();let mut callbacks=HashMap::new();
        for (index,function) in self.program.functions.iter().enumerate() {
            if function.template {continue;}
            let span=function.body.first().map_or(crate::Span::default(),|statement|statement.span);
            let contract=self.program.error_effects.function_errors.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result preparation requires saved function contracts.",span))?;
            if function.name!="main" && !contract.runtime && !contract.opaque && contract.explicit.is_empty() {continue;}
            contract.native_payloads(&unions,runtime,span)?;
            let (declaration,name)=self.completion_error_layout(&format!("TsrFunctionError{index}"),contract,runtime,span);
            declarations.push_str(&declaration);functions.insert(index,(format!("tsr_result_function{index}"),name));
        }
        let mut active_callbacks=HashSet::new();
        let mut callback_types=Vec::new();
        for function in self.program.functions.iter().chain(self.program.closures.iter().flat_map(|closure|std::iter::once(&closure.function).chain(closure.methods.iter().map(|method|&method.function)))).filter(|function|!function.template) {
            callback_types.extend(function.bindings.iter().flat_map(|binding|std::iter::once(binding.ty).chain(binding.input_ty)));
            callback_types.push(function.result);
            for expression in crate::error_effects::checked_expressions(function) {
                callback_types.extend(expression.ty);
                active_callbacks.extend(expression.callback_contract());
                if let Some(CallTarget::ConditionalCallback(id))=expression.target {active_callbacks.insert(id);}
            }
        }
        for closure in self.program.closures.iter().filter(|closure|!closure.function.template) {
            if let Some(ClosureForward::Abstract(id))=closure.forward {active_callbacks.insert(id);}
        }
        while let Some(ty)=callback_types.pop() {
            match ty {
                Type::Callback(id)=>{active_callbacks.insert(id);},
                Type::Union(id)|Type::BorrowedUnion(id)=>callback_types.extend(self.program.unions[id.index()].members.iter().copied()),
                _=>{},
            }
        }
        let mut callback_contracts=self.program.error_effects.callback_errors.iter().collect::<Vec<_>>();
        callback_contracts.sort_by_key(|(id,_)|**id);
        let mut callback_layouts:Vec<(crate::error_effects::FailureSet,String)>=vec![];
        for (id,contract) in callback_contracts {
            if !contract.runtime && !contract.opaque && contract.explicit.is_empty() {continue;}
            let span=crate::Span::default();
            if *id>=self.program.callback_contracts.len() {return Err(crate::Diagnostic::new("error-effect","Result preparation requires a known callback declaration.",span));}
            if contract.opaque && !active_callbacks.contains(id) {continue;}
            contract.native_payloads(&unions,runtime,span)?;
            let name=if let Some((_,name))=callback_layouts.iter().find(|(saved,_)|saved==contract) {name.clone()} else {
                let (declaration,name)=self.completion_error_layout(&format!("TsrCallbackError{id}"),contract,runtime,span);
                declarations.push_str(&declaration);callback_layouts.push((contract.clone(),name.clone()));name
            };
            callbacks.insert(*id,(name,contract.clone()));
        }
        for (index,closure) in self.program.closures.iter().enumerate() {
            if closure.function.template {continue;}
            if closure.polymorphic {
                for (method,body) in closure.methods.iter().enumerate().filter(|(_,body)|!body.function.template) {
                    let contract=self.program.error_effects.method_errors.get(&(index,method)).ok_or_else(||crate::Diagnostic::new("error-effect","Result preparation requires saved method contracts.",closure.span))?;
                    if !contract.runtime && !contract.opaque && contract.explicit.is_empty() {continue;}
                    contract.native_payloads(&unions,runtime,closure.span)?;
                    let (declaration,name)=self.completion_error_layout(&format!("TsrMethodError{index}_{method}"),contract,runtime,closure.span);
                    let _=body;
                    declarations.push_str(&declaration);methods.insert((index,method),name);
                }
                continue;
            }
            let contract=self.program.error_effects.closure_errors.get(index).ok_or_else(||crate::Diagnostic::new("error-effect","Result preparation requires saved closure contracts.",closure.span))?;
            if !contract.runtime && !contract.opaque && contract.explicit.is_empty() {continue;}
            contract.native_payloads(&unions,runtime,closure.span)?;
            if closure.polymorphic {
                return Err(crate::Diagnostic::new("error-effect","Polymorphic Result closures require method-specific registration.",closure.span));
            }
            if let Some((_,name))=callback_layouts.iter().find(|(saved,_)|saved==contract) {
                // Equal checked payload sets share their native callback ABI.
                closures.insert(index,name.clone());
            } else {
                let (declaration,name)=self.completion_error_layout(&format!("TsrClosureError{index}"),contract,runtime,closure.span);
                declarations.push_str(&declaration);closures.insert(index,name);
            }
        }
        let declared_families=crate::callable_layout::declared_storage_families(self.program,|record,position|self.scoped_record_native_field(record,position,self.program.records[record].fields[position].ty.expect("checked field")));
        for (family,_,origins) in self.program.stored_contracts.families() {
            if origins.is_empty() && !declared_families.contains(&family) {continue;}
            // An emitted empty owning family has no value and no callable error
            // ABI. Actual invocations still require resolved function effects.
            if origins.is_empty() && !self.program.stored_contracts.is_borrowed(family)
                && self.program.stored_contracts.scoped_native_family(family).is_none() { continue; }
            let span=origins.iter().next().map_or(crate::Span::default(),|origin|self.program.closures[*origin].span);
            let contract=self.program.error_effects.storage_errors.get(&family).ok_or_else(||crate::Diagnostic::new("error-effect","Result preparation requires saved storage contracts.",span))?;
            if !contract.runtime && !contract.opaque && contract.explicit.is_empty() {continue;}
            contract.native_payloads(&unions,runtime,span)?;
            for origin in origins {
                let source=self.program.error_effects.closure_errors.get(*origin).ok_or_else(||crate::Diagnostic::new("error-effect","Result preparation requires saved member contracts.",span))?;
                source.native_injection(contract,&unions,runtime,span)?;
            }
            let (declaration,name)=self.completion_error_layout(&format!("TsrStorageError{family}"),contract,runtime,span);
            declarations.push_str(&declaration);storage.insert(family,(name,contract.clone()));
        }
        *self.result_functions.borrow_mut()=functions;
        *self.result_closures.borrow_mut()=closures;
        *self.result_methods.borrow_mut()=methods;
        *self.result_storage.borrow_mut()=storage;
        *self.result_callbacks.borrow_mut()=callbacks;
        Ok(declarations)
    }

    #[inline(never)]
    fn abstract_result_forward(&self,call:String,contract:usize,outer:usize)->String {
        let Some(destination)=self.result_closures.borrow().get(&outer).cloned() else {return call;};
        let source=self.program.error_effects.callback_errors.get(&contract).expect("saved abstract forwarding contract");
        if let Some((source_name,_))=self.result_callbacks.borrow().get(&contract) {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let mapping=source.native_injection(&self.program.error_effects.closure_errors[outer],&unions,runtime,self.program.closures[outer].span).expect("checked abstract forwarding transport");
            let conversion=exception_emit::error_conversion("error",source_name,&destination,&mapping);
            format!("{call}.map_err(|error|{conversion})")
        } else {
            assert!(!source.runtime && !source.opaque && source.explicit.is_empty(),"fallible abstract target requires Result registration");
            format!("Ok({call})")
        }
    }

    /// Add callback-specific Result bridges without widening the direct closure ABI.
    #[inline(never)]
    fn callback_result_adapters(&self,output:&mut String,index:usize,params:&[ArgumentContract],arguments:&str,forwarded:&str,impl_lifetimes:&str,lifetime:&str,native_result:&str) {
        if !self.callback_traits || self.result_callbacks.borrow().is_empty() {return;}
        let closure=&self.program.closures[index];
        let forwarded_where = self.forwarded_record_signature(closure).map_or_else(||self.closure_record_signature(index).map_or_else(String::new, |signature|signature.trait_where), |(target,_)| self.forwarded_record_trait_where(target));
        let Some(source)=self.program.error_effects.closure_errors.get(index) else {return;};
        let source_name=self.result_closures.borrow().get(&index).cloned();
        if source_name.is_none() && (source.runtime || source.opaque || !source.explicit.is_empty()) {return;}
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let registry=self.result_callbacks.borrow();
        let mut contracts=registry.iter().collect::<Vec<_>>();contracts.sort_by_key(|(id,_)|**id);
        let mut emitted=std::collections::HashSet::new();
        for (id,(destination_name,destination)) in contracts {
            let checked=&self.program.callback_contracts[*id];
            if (checked.kind==CallbackKind::Task)!=self.program.task_origins.contains(&index){continue;}
            if checked.result!=closure.function.result || checked.params.len()!=params.len() || !checked.params.iter().zip(params).all(|(expected,actual)|expected.ty==actual.ty && expected.mode==actual.mode && expected.rest==actual.rest) {continue;}
            let result=format!("Result<{},{}>",self.native_type(checked.result),destination_name);
            if result==native_result || !emitted.insert(result.clone()) {continue;}
            let Ok(mapping)=source.native_injection(destination,&unions,runtime,closure.span) else {continue;};
            let convert=|call:String|if let Some(source_name)=&source_name {
                let conversion=exception_emit::error_conversion("error",source_name,destination_name,&mapping);
                format!("{call}.map_err(|error|{conversion})")
            }else{format!("Ok({call})")};
            if closure.program_scoped {
                if !matches!(checked.kind, CallbackKind::Program | CallbackKind::ProgramReusable) { continue; }
                let separator = if forwarded.is_empty() { "" } else { ", " };
                let body = convert(format!("self.call({forwarded}{separator}program,registry)"));
                let _ = writeln!(output, "impl{impl_lifetimes} ProgramCallback<{arguments},{result}> for Closure{index}{lifetime}{forwarded_where} {{ fn invoke_program(&mut self,args:{arguments},program:&mut TsrTestContext<'_>,registry:&mut TsrTestRegistry)->{result}{{{body}}} }}");
                continue;
            }
            for (trait_name,method,receiver,allowed) in [
                ("OnceCallback","invoke_once","mut self",true),
                ("ReadCallback","invoke_read","&self",closure.invocation==Mode::Read),
                ("Callback","invoke","&mut self",closure.invocation!=Mode::Consume),
            ] {
                if !allowed || self.program.task_origins.contains(&index) && trait_name!="OnceCallback" {continue;}
                let task=self.program.task_origins.contains(&index);
                let asynchronous=if task{"async "}else{""};
                let task_await=if task{".await"}else{""};
                let trait_name=if task{"TaskCallback"}else{trait_name};
                let body=convert(format!("self.call({forwarded}){task_await}"));
                let counted=if self.counted_calls {
                    let body=convert(format!("self.call_with_count(count, {forwarded}){task_await}"));
                    format!("    {asynchronous}fn {method}_with_count({receiver}, args:{arguments}, count:usize)->{result}{{{body}}}\n")
                }else{String::new()};
                let _=writeln!(output,"impl{impl_lifetimes} {trait_name}<{arguments},{result}> for Closure{index}{lifetime}{forwarded_where}{{\n    {asynchronous}fn {method}({receiver},args:{arguments})->{result}{{{body}}}\n{counted}}}\n");
            }
        }
    }

    /// Register a finite family's Result ABI only after its members are ready.
    #[inline(never)]
    #[cfg(test)]
    fn prepare_storage_result(&self,family:usize,enum_name:&str,span:crate::Span)->Result<(String,String),crate::Diagnostic> {
        let contract=self.program.error_effects.storage_errors.get(&family).ok_or_else(||crate::Diagnostic::new("error-effect","Stored Result registration requires a saved family contract.",span))?;
        let (_,_,origins)=self.program.stored_contracts.families().find(|(id,_,_)|*id==family).ok_or_else(||crate::Diagnostic::new("error-effect","Stored Result registration requires a known family.",span))?;
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        contract.native_payloads(&unions,runtime,span)?;
        for origin in origins {
            let source=self.program.error_effects.closure_errors.get(*origin).ok_or_else(||crate::Diagnostic::new("error-effect","Stored Result registration requires saved member contracts.",span))?;
            source.native_injection(contract,&unions,runtime,span)?;
            if (source.runtime || source.opaque || !source.explicit.is_empty()) && !self.result_closures.borrow().contains_key(origin) {
                return Err(crate::Diagnostic::new("error-effect","Fallible stored member requires Result registration before its family.",span));
            }
        }
        let (declaration,name)=self.completion_error_layout(enum_name,contract,runtime,span);
        self.result_storage.borrow_mut().insert(family,(name.clone(),contract.clone()));
        Ok((declaration,name))
    }

    #[inline(never)]
    fn registered_storage_call(&self,call:String,family:usize,span:crate::Span)->String {
        let task=self.program.stored_contracts.declaration(family).is_some_and(|id|self.program.callback_contracts[id].kind==CallbackKind::Task);
        let call=if task {format!("({call}).await")}else{call};
        let result_layout=self.result_storage.borrow().get(&family).cloned();
        if let Some((source_name,source))=result_layout {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            self.exception_regions.result_call(&call,&source_name,&source,&unions,span).expect("checked registered storage Result call")
        } else {
            call
        }
    }

    #[inline(never)]
    fn stored_origin_result(&self,family:usize,origin:usize,call:String)->String {
        let call=if self.program.task_origins.contains(&origin){format!("({call}).await")}else{call};
        let destination=self.result_storage.borrow().get(&family).cloned();
        let Some((destination_name,destination))=destination else {return call;};
        let source=&self.program.error_effects.closure_errors[origin];
        let source_name=self.result_closures.borrow().get(&origin).cloned();
        if let Some(source_name)=source_name {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let mapping=source.native_injection(&destination,&unions,runtime,self.program.closures[origin].span).expect("checked stored family Result contract");
            let conversion=exception_emit::error_conversion("error",&source_name,&destination_name,&mapping);
            format!("({call}).map_err(|error|{conversion})")
        } else {
            assert!(!source.runtime && !source.opaque && source.explicit.is_empty(),"fallible stored origin requires Result registration");
            format!("Ok({call})")
        }
    }

    #[inline(never)]
    fn registered_closure_call(&self,call:String,origin:usize,span:crate::Span)->String {
        let call=if self.program.task_origins.contains(&origin){format!("({call}).await")}else{call};
        let result_layout=self.result_closures.borrow().get(&origin).cloned();
        if let Some(source_name)=result_layout {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            self.exception_regions.closure_result_call(&call,origin,&source_name,&self.program.error_effects,&unions,span).expect("checked registered closure Result call")
        } else {
            call
        }
    }

    #[inline(never)]
    fn storage_result_forward(&self,call:String,family:usize,outer:usize)->String {
        let Some(destination)=self.result_closures.borrow().get(&outer).cloned() else {return call;};
        let source=self.program.error_effects.storage_errors.get(&family).expect("saved rest storage contract");
        if let Some((source_name,_))=self.result_storage.borrow().get(&family) {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let mapping=source.native_injection(&self.program.error_effects.closure_errors[outer],&unions,runtime,self.program.closures[outer].span).expect("checked rest storage transport");
            let conversion=exception_emit::error_conversion("error",source_name,&destination,&mapping);
            format!("({call}).map_err(|error|{conversion})")
        } else {
            assert!(!source.runtime && !source.opaque && source.explicit.is_empty(),"fallible stored source requires Result registration");
            format!("Ok({call})")
        }
    }

    #[inline(never)]
    fn closure_result_forward(&self,call:String,inner:usize,outer:usize)->String {
        let destination=self.result_closures.borrow().get(&outer).cloned();
        let Some(destination)=destination else {return call;};
        let source=&self.program.error_effects.closure_errors[inner];
        let source_name=self.result_closures.borrow().get(&inner).cloned();
        if let Some(source_name)=source_name {
            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
            let mapping=source.native_injection(&self.program.error_effects.closure_errors[outer],&unions,runtime,self.program.closures[outer].span).expect("checked adapter Result contract");
            let conversion=exception_emit::error_conversion("error",&source_name,&destination,&mapping);
            format!("({call}).map_err(|error|{conversion})")
        } else {
            assert!(!source.runtime && !source.opaque && source.explicit.is_empty(),"fallible adapter source requires Result registration");
            format!("Ok({call})")
        }
    }

    fn counted_closure_body(&self, output: &mut String, closure: &crate::analysis::CheckedClosure) {
        self.counted_closure_function_body(output, closure, &closure.function);
    }

    fn counted_closure_function_body(&self, output: &mut String, closure: &crate::analysis::CheckedClosure, function: &CheckedFunction) {
        let _callable_scope=self.exception_regions.callable_boundary();
        let _control_boundary=self.control_targets.callable_boundary();
        match closure.forward {
            Some(ClosureForward::RestAdapter {
                fixed,
                counted: true,
            }) => {
                let capture = &closure.captures[0];
                let reference = if self.record_capture_is_view(capture,closure){if closure.invocation==Mode::Mutate{"&mut "}else{"&"}}else{match (capture.storage, closure.invocation) {
                    (Mode::Consume, Mode::Read) => "&",
                    (Mode::Consume, Mode::Mutate) => "&mut ",
                    (Mode::Mutate, Mode::Mutate) => "&mut *",
                    _ => "",
                }};
                let _ = writeln!(output, "        let value0 = {reference}self.capture0;");
                let StmtKind::Return(Some(call)) = &function.body[0].kind else {
                    unreachable!("rest adapter return")
                };
                let ExprKind::Call(_, arguments) = &call.kind else {
                    unreachable!("rest adapter call")
                };
                let ExprKind::Unary {
                    op: UnaryOp::RestPack,
                    value,
                    ..
                } = &arguments[fixed].kind
                else {
                    unreachable!("rest adapter pack")
                };
                let ExprKind::Array(tail) = &value.kind else {
                    unreachable!("rest adapter tail")
                };
                let array = self.native_type(value.ty.expect("checked rest pack"));
                let _ = writeln!(output, "        let mut rest_values = <{array}>::from([]);");
                for (position, value) in tail.iter().enumerate() {
                    let supplied = fixed + position;
                    let value = self.expr(function, value, Mode::Consume);
                    let _ = writeln!(
                        output,
                        "        if supplied_count > {supplied} {{ rest_values.push({value}); }}"
                    );
                }
                let source = function.bindings[0].ty;
                let parameters = match source {
                    Type::Closure(id) => {
                        let source = &self.program.closures[id];
                        source
                            .parameters
                            .iter()
                            .map(|&binding| {
                                ArgumentContract::from(&source.function.bindings[binding])
                            })
                            .collect::<Vec<_>>()
                    }
                    Type::Callback(id) => self.program.callback_contracts[id].params.clone(),
                    Type::StoredCallable(id) => self.program.callback_contracts
                        [self.program.stored_contracts.declaration(id).unwrap()]
                    .params
                    .clone(),
                    _ => unreachable!("rest source"),
                };
                let mut values = arguments[..fixed]
                    .iter()
                    .zip(&parameters)
                    .map(|(value, parameter)| self.expr(function, value, parameter.mode))
                    .collect::<Vec<_>>();
                values.push("rest_values".into());
                let code = if let Type::Callback(id) = source {
                    let kind = self.program.callback_contracts[id].kind;
                    let method = if matches!(kind, CallbackKind::Read | CallbackKind::SharedRead) {
                        "ReadCallback::invoke_read"
                    } else if kind.is_once() {
                        "OnceCallback::invoke_once"
                    } else {
                        "Callback::invoke"
                    };
                    format!("{method}(value0, {})", tuple(values.into_iter()))
                } else if let Some(CallTarget::ClosureMethod(method)) = call.target {
                    format!("value0.call{method}({})", values.join(", "))
                } else {
                    format!("value0.call({})", values.join(", "))
                };
                let outer=self.program.closures.iter().position(|candidate|std::ptr::eq(candidate,closure)).expect("saved rest adapter");
                let outer_method=closure.methods.iter().position(|method|std::ptr::eq(&method.function,function));
                let code=if let (Type::Closure(inner),Some(CallTarget::ClosureMethod(method)),Some(outer_method))=(source,call.target,outer_method) {
                    let destination=self.result_methods.borrow().get(&(outer,outer_method)).cloned();
                    let source_name=self.result_methods.borrow().get(&(inner,method)).cloned();
                    match (destination,source_name) {
                        (Some(destination),Some(source_name))=>{
                            let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                            let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
                            let mapping=self.program.error_effects.method_errors[&(inner,method)].native_injection(&self.program.error_effects.method_errors[&(outer,outer_method)],&unions,runtime,call.span).expect("checked counted method transport");
                            let conversion=exception_emit::error_conversion("error",&source_name,&destination,&mapping);
                            format!("({code}).map_err(|error|{conversion})")
                        }
                        (Some(_),None)=>format!("Ok({code})"),
                        (None,None)=>code,
                        (None,Some(_))=>unreachable!("fallible counted method requires destination"),
                    }
                }else{match source {
                    Type::Closure(inner)=>self.closure_result_forward(code,inner,outer),
                    Type::Callback(contract)=>self.abstract_result_forward(code,contract,outer),
                    Type::StoredCallable(family)=>self.storage_result_forward(code,family,outer),
                    _=>code,
                }};
                let _ = writeln!(output, "        {code}");
            }
            Some(ClosureForward::Abstract(contract)) => {
                let kind=self.program.callback_contracts[contract].kind;
                let task_method=if kind==CallbackKind::Task{format!("TaskCallback::<(),{}>::invoke_once_with_count",self.callback_result_type(contract,self.program.callback_contracts[contract].result))}else{String::new()};
                let (method,receiver)=if kind==CallbackKind::Task{(task_method.as_str(),"self.capture0")}else if matches!(kind,CallbackKind::Read | CallbackKind::SharedRead){("ReadCallback::invoke_read_with_count","&self.capture0")}else if kind.is_once(){("OnceCallback::invoke_once_with_count","self.capture0")}else{("Callback::invoke_with_count","&mut self.capture0")};
                let arguments=tuple(closure.parameters.iter().map(|id|format!("value{id}")));
                let outer=self.program.closures.iter().position(|candidate|std::ptr::eq(candidate,closure)).expect("saved abstract forwarder");
                let call=format!("{method}({receiver},{arguments},supplied_count){}",if kind==CallbackKind::Task{".await"}else{""});
                let call=self.abstract_result_forward(call,contract,outer);
                let _=writeln!(output,"        {call}");
            }
            Some(ClosureForward::Once(inner)) => {
                let arguments = tuple(closure.parameters.iter().map(|id| format!("value{id}")));
                let call=if self.program.task_origins.contains(&inner){format!("{{ let {}task=self.capture0; task.call_with_count(supplied_count).await }}",if self.program.closures[inner].invocation==Mode::Mutate{"mut "}else{""})}else{format!("OnceCallback::invoke_once_with_count(self.capture0, {arguments}, supplied_count)")};
                let outer=self.program.closures.iter().position(|candidate|std::ptr::eq(candidate,closure)).expect("saved Once adapter");
                let call=self.closure_result_forward(call,inner,outer);
                let _=writeln!(output,"        {call}");
            }
            _ => {
                let mut arguments = closure.parameters.iter().map(|id| format!("value{id}")).collect::<Vec<_>>();
                if let Some(ClosureForward::Function(target)) = closure.forward {
                    arguments.extend(self.owner_inputs(target).iter().map(|source| format!("std::marker::PhantomData::<O{source}>")));
                }
                let arguments=arguments.join(", ");
                let _ = writeln!(output, "        self.call({arguments})");
            }
        }
    }

    #[inline(never)]
    fn result_entry_adapter(&self,index:usize,symbol:&str,error_name:&str)->String {
        let function=&self.program.functions[index];
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let payloads=self.program.error_effects.function_errors[index].native_payloads(&unions,runtime,function.body.first().map_or(crate::Span::default(),|statement|statement.span)).expect("checked entry error contract");
        let matched=if payloads.is_empty(){"*error"}else{"error"};
        let mut arms=String::new();
        for (tag,ty) in payloads.into_iter().enumerate() {
            let report=if Some(ty)==runtime {
                "tsr_console_write(true,&std::format!(\"Uncaught {}: {} ({}:{}..{})\\n\",payload.field0.as_ref(),payload.field2.as_ref(),payload.field3.as_ref(),payload.field4,payload.field5));".to_owned()
            } else if matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::CacheInsertError)) {
                // Rejected inputs may contain opaque resources; report the stable code without inspecting them.
                "tsr_console_write(true,&std::format!(\"Uncaught CacheInsertError: {}\\n\",payload.field0.as_ref()));".to_owned()
            } else {
                let value=self.console_value("payload",ty,"0",false);
                format!("let text=tsr_console_format(&[{value}]);tsr_console_write(true,&std::format!(\"Uncaught {{text}}\\n\"));")
            };
            arms.push_str(&format!("{}::V{tag}(payload)=>{{{report}}},",exception_emit::constructor_path(error_name)));
        }
        format!("fn main()->std::process::ExitCode{{ {} tsr_entry_result({symbol}(),|error|{{match {matched}{{{arms}}}}}) }}\n",self.entry_setup)
    }

    fn program(&self) -> String {
        let mut output = String::from(
            "// Generated by the experimental Rust-based tsrust compiler.\n#![allow(dead_code, unused_variables, unused_mut)]\n#![recursion_limit = \"512\"]\n\n",
        );
        for (index, function) in self.program.functions.iter().enumerate() {
            if function.name.ends_with(crate::data_lowering::OWNER) {
                let ty = self.native_type(function.result);
                let _ = writeln!(output, "static TSR_DATA{index}: std::sync::LazyLock<{ty}> = std::sync::LazyLock::new(function{index});");
            }
        }
        self.rust_modules(&mut output);
        self.emit_owner_markers(&mut output);
        self.emit_nullable_types(&mut output);
        self.emit_generic_storage(&mut output);
        if self.callback_traits && !self.counted_calls {
            output.push_str("trait Callback<A, R> { fn invoke(&mut self, args: A) -> R; }\n\n");
            output
                .push_str("trait ReadCallback<A, R> { fn invoke_read(&self, args: A) -> R; }\n\n");
            output.push_str("trait OnceCallback<A, R> { fn invoke_once(self, args: A) -> R; }\n\n");
            output.push_str("enum TsrDefault<S,D> { Supplied(S), Default(D) }\nimpl<S:ReadCallback<A,R>,D:ReadCallback<A,R>,A,R> ReadCallback<A,R> for TsrDefault<S,D>{ fn invoke_read(&self,args:A)->R { match self {Self::Supplied(value)=>value.invoke_read(args),Self::Default(value)=>value.invoke_read(args)} } }\nimpl<S:Callback<A,R>,D:Callback<A,R>,A,R> Callback<A,R> for TsrDefault<S,D>{ fn invoke(&mut self,args:A)->R { match self {Self::Supplied(value)=>value.invoke(args),Self::Default(value)=>value.invoke(args)} } }\nimpl<S:OnceCallback<A,R>,D:OnceCallback<A,R>,A,R> OnceCallback<A,R> for TsrDefault<S,D>{ fn invoke_once(self,args:A)->R { match self {Self::Supplied(value)=>value.invoke_once(args),Self::Default(value)=>value.invoke_once(args)} } }\nimpl<F:ReadCallback<A,R>+?Sized,A,R> ReadCallback<A,R> for &F { fn invoke_read(&self,args:A)->R { (**self).invoke_read(args) } }\nimpl<F:Callback<A,R>+?Sized,A,R> Callback<A,R> for &mut F { fn invoke(&mut self,args:A)->R { (**self).invoke(args) } }\n");
        }
        if self.callback_traits && self.counted_calls {
            output.push_str("trait Callback<A,R> { fn invoke(&mut self,args:A)->R; fn invoke_with_count(&mut self,args:A,_count:usize)->R { self.invoke(args) } }\n");
            output.push_str("trait ReadCallback<A,R> { fn invoke_read(&self,args:A)->R; fn invoke_read_with_count(&self,args:A,_count:usize)->R { self.invoke_read(args) } }\n");
            output.push_str("trait OnceCallback<A,R> { fn invoke_once(self,args:A)->R; fn invoke_once_with_count(self,args:A,_count:usize)->R where Self:Sized { self.invoke_once(args) } }\n");
            output.push_str("enum TsrDefault<S,D> { Supplied(S), Default(D) }\n");
            output.push_str("impl<S:ReadCallback<A,R>,D:ReadCallback<A,R>,A,R> ReadCallback<A,R> for TsrDefault<S,D>{ fn invoke_read(&self,args:A)->R { match self {Self::Supplied(value)=>value.invoke_read(args),Self::Default(value)=>value.invoke_read(args)} } fn invoke_read_with_count(&self,args:A,count:usize)->R { match self {Self::Supplied(value)=>value.invoke_read_with_count(args,count),Self::Default(value)=>value.invoke_read_with_count(args,count)} } }\n");
            output.push_str("impl<S:Callback<A,R>,D:Callback<A,R>,A,R> Callback<A,R> for TsrDefault<S,D>{ fn invoke(&mut self,args:A)->R { match self {Self::Supplied(value)=>value.invoke(args),Self::Default(value)=>value.invoke(args)} } fn invoke_with_count(&mut self,args:A,count:usize)->R { match self {Self::Supplied(value)=>value.invoke_with_count(args,count),Self::Default(value)=>value.invoke_with_count(args,count)} } }\n");
            output.push_str("impl<S:OnceCallback<A,R>,D:OnceCallback<A,R>,A,R> OnceCallback<A,R> for TsrDefault<S,D>{ fn invoke_once(self,args:A)->R { match self {Self::Supplied(value)=>value.invoke_once(args),Self::Default(value)=>value.invoke_once(args)} } fn invoke_once_with_count(self,args:A,count:usize)->R { match self {Self::Supplied(value)=>value.invoke_once_with_count(args,count),Self::Default(value)=>value.invoke_once_with_count(args,count)} } }\n");
            output.push_str("impl<F:ReadCallback<A,R>+?Sized,A,R> ReadCallback<A,R> for &F { fn invoke_read(&self,args:A)->R { (**self).invoke_read(args) } fn invoke_read_with_count(&self,args:A,count:usize)->R { (**self).invoke_read_with_count(args,count) } }\n");
            output.push_str("impl<F:Callback<A,R>+?Sized,A,R> Callback<A,R> for &mut F { fn invoke(&mut self,args:A)->R { (**self).invoke(args) } fn invoke_with_count(&mut self,args:A,count:usize)->R { (**self).invoke_with_count(args,count) } }\n");
        }
        self.test_program_declarations(&mut output);
        if self.program.records.iter().any(|record| record.native_element().is_some()) {
            output.push_str(include_str!("../stdlib/collections/deque_runtime.rs"));
        }
        let has_heap=self.program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet | crate::native_signatures::NativeTypeId::OrderedMap)));
        if has_heap { output.push_str(include_str!("../stdlib/collections/ordered_runtime.rs")); output.push_str(include_str!("../stdlib/collections/ordered_key_runtime.rs")); }
        if !self.program.task_origins.is_empty() || self.program.callback_contracts.iter().any(|contract|contract.kind==CallbackKind::Task) || self.program.records.iter().any(|record|record.native_type.is_some_and(|id|matches!(crate::native_types::declaration(id).module,"std:http"|"std:sql"))) { output.push_str(include_str!("async_runtime.rs")); output.push_str(crate::stdlib::futures::RUNTIME); }
        self.set_types(&mut output);
        self.map_types(&mut output);
        if self.program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::Schema)) {
            output.push_str("mod tsrust_schema_plan {\n");
            output.push_str(include_str!("../stdlib/schema/runtime/plan.rs").trim_start_matches("//! Scalar schema plan storage shared by generated programs and backend tests.\n"));
            output.push_str("\n}\n");
        }
        output.push_str(self.schema_choice_storage());
        if self.program.records.iter().any(|record|record.native_type==Some(crate::native_signatures::NativeTypeId::SchemaExportError)) {
            output.push_str("mod tsrust_schema_export {\n");
            output.push_str(include_str!("../stdlib/schema/runtime/export_runtime.rs").trim_start_matches("//! Shared JSON Schema assembly. Generated visitors borrow plans and own output.\n"));
            output.push_str("\n}\n");
        }
        if self.program.records.iter().any(|record|record.native_type.is_some_and(|id|crate::native_types::declaration(id).module=="std:sql")){output.push_str(crate::stdlib::sql::RUNTIME);}
        if self.program.records.iter().any(|record|record.native_type.is_some_and(|id|crate::native_types::declaration(id).module=="std:sqlite")){output.push_str(crate::stdlib::sqlite::RUNTIME);}
        if self.program.records.iter().any(|record|record.native_type.is_some_and(|id|crate::native_types::declaration(id).module=="std:http")){output.push_str(crate::stdlib::http::RUNTIME);}
        let mut record_support_cache=scoped_record_support_emit::SupportCache::default();
        for (index, record) in self.program.records.iter().enumerate() {
            if self.sql_record(&mut output,index) || self.http_record(&mut output,index) || self.sqlite_record(&mut output,index){continue;}
            if self.future_record(&mut output, index) { continue; }
            if self.cache_record(&mut output,index) {continue;}
            if self.thread_record(&mut output, index) { continue; }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::TestRegistry) {
                let _ = writeln!(output, "type Record{index} = TsrTestRegistry;");
                continue;
            }
            if record.literal_definition.is_some(){continue;}
            if self.native_record_declaration(index) != index { continue; }
            if record.native_type==Some(crate::native_signatures::NativeTypeId::ValidationContext) {
                let _=writeln!(output,"struct Record{index} {{ issues: tsrust_schema_issues::Issues }}");continue;
            }
            if let Some(declaration)=self.schema_plan_declaration(index){output.push_str(&declaration);continue;}
            if let Some(definition) = &record.enum_definition {
                let variants = (0..definition.members.len()).map(|member| format!("V{member}")).collect::<Vec<_>>().join(",");
                let arms = definition.members.iter().enumerate().map(|(member, entry)| {
                    let value = match &entry.value { crate::syntax::EnumValue::Number(value) => float_literal(*value), crate::syntax::EnumValue::String(value) => format!("{value:?}") };
                    format!("Self::V{member} => {value}")
                }).collect::<Vec<_>>().join(",");
                let backing = match definition.kind { crate::syntax::EnumKind::Number => "f64", crate::syntax::EnumKind::String => "&'static str" };
                if !definition.members.is_empty() { output.push_str("#[repr(u32)]\n"); }
                let _ = writeln!(output, "#[derive(Clone, Copy, Debug)] enum Enum{index} {{ {variants} }} impl Enum{index} {{ fn tsr_value(self) -> {backing} {{ match self {{ {arms} }} }} }} impl PartialEq for Enum{index} {{ fn eq(&self, other: &Self) -> bool {{ self.tsr_value() == other.tsr_value() }} }} impl PartialOrd for Enum{index} {{ fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {{ self.tsr_value().partial_cmp(&other.tsr_value()) }} }}");
            }
            if has_heap && let Some(definition)=&record.enum_definition {
                let comparison=match definition.kind { crate::syntax::EnumKind::Number=>"TsrOrderedNumber::new(self.tsr_value()).cmp(&TsrOrderedNumber::new(other.tsr_value()))",crate::syntax::EnumKind::String=>"self.tsr_value().cmp(other.tsr_value())" };
                let _=writeln!(output,"impl TsrOrderedKey for Enum{index} {{ fn tsr_compare(&self,other:&Self)->std::cmp::Ordering {{ {comparison} }} }}");
            }
            if matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::JsonlWriter | crate::native_signatures::NativeTypeId::JsonlWriterView)) {
                let borrowed=record.native_type==Some(crate::native_signatures::NativeTypeId::JsonlWriterView);
                let lifetime=if borrowed{"<'a>"}else{""};let borrow=if borrowed{"&'a mut "}else{""};let implementation=if borrowed{"<'_>"}else{""};
                let _=writeln!(output,"struct Record{index}{lifetime} {{ handle: tsrust_native_json::writer::Writer<{borrow}tsrust_native_files::buffered_fs::BufferedWriter> }} impl tsrust_native_json::writer::Sink for Record{index}{implementation} {{fn write_jsonl(&mut self,value:&tsrust_native_json::Value)->Result<(),tsrust_native_json::writer::WriteError>{{self.handle.write(value)}}fn flush_jsonl(&mut self)->Result<(),tsrust_native_json::writer::WriteError>{{self.handle.flush()}}fn close_jsonl(self)->Result<(),tsrust_native_json::writer::WriteError>{{self.handle.close()}}}}");continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::JsonlReaderView) {
                let _=writeln!(output,"struct Record{index}<'a> {{ handle: tsrust_native_json::lines::Lines<&'a mut tsrust_native_files::buffered_fs::BufferedReader> }} impl tsrust_native_json::lines::Source for Record{index}<'_> {{ fn read_jsonl(&mut self)->Result<Option<tsrust_native_json::Value>,tsrust_native_json::lines::LineError>{{self.handle.next()}} fn close_jsonl(self)->Result<(),tsrust_native_json::lines::CloseError>{{self.handle.close_with(|_|Ok(()))}} }}");continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::JsonlReader) {
                let _=writeln!(output,"struct Record{index} {{ handle: tsrust_native_json::lines::Lines<tsrust_native_files::buffered_fs::BufferedReader> }}");
                let _=writeln!(output,"impl tsrust_native_json::lines::Source for Record{index} {{ fn read_jsonl(&mut self)->Result<Option<tsrust_native_json::Value>,tsrust_native_json::lines::LineError>{{self.handle.next()}} fn close_jsonl(self)->Result<(),tsrust_native_json::lines::CloseError>{{self.handle.close_with(|input|input.close()).map_err(|error|tsrust_native_json::lines::CloseError{{code:error.code,os_code:error.os_code}})}} }}");
                continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::StringView) {
                let _=writeln!(output,"struct Record{index}<'a>{{text:&'a str}}");continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::JsonRaw) {
                let _=writeln!(output,"struct Record{index}<'a>{{fragment:tsrust_native_json::raw::Fragment<'a>}}");continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::JsonView) {
                let _ = writeln!(output, "struct Record{index}<'a> {{ value: &'a tsrust_native_json::Value }} impl tsrust_native_json::Source for Record{index}<'_> {{ fn json_value(&self)->&tsrust_native_json::Value {{ self.value }} }} impl<'a> tsrust_native_json::ViewSource<'a> for &Record{index}<'a> {{ fn json_view(self)->&'a tsrust_native_json::Value {{ self.value }} }}");
                continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::JsonValue) {
                let _ = writeln!(output, "#[derive(Clone)] struct Record{index} {{ value: tsrust_native_json::Value }} impl tsrust_native_json::Source for Record{index} {{ fn json_value(&self)->&tsrust_native_json::Value {{ &self.value }} }} impl<'a> tsrust_native_json::ViewSource<'a> for &'a Record{index} {{ fn json_view(self)->&'a tsrust_native_json::Value {{ &self.value }} }}");
                continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::Bytes) {
                let _ = writeln!(output, "#[derive(Clone)] struct Record{index} {{ bytes: Vec<u8> }}");
                continue;
            }
            if let Some(id @ (crate::native_signatures::NativeTypeId::BufferedReader | crate::native_signatures::NativeTypeId::BufferedWriter)) = record.native_type {
                let backend=crate::native_types::declaration(id).name;
                let _=writeln!(output,"struct Record{index} {{ handle: tsrust_native_files::buffered_fs::{backend} }}");
                continue;
            }
            if let Some(name)=record.native_type.and_then(crate::stdlib::date::runtime_type) {let _=writeln!(output,"type Record{index} = tsrust_native_date::{name};");
                continue;
            }
            if let Some(name)=match record.native_type {Some(crate::native_signatures::NativeTypeId::UiSignal)=>Some("Signal"),_=>None} {let _=writeln!(output,"type Record{index} = tsrust_native_ui::{name};");continue;}
            if record.native_type == Some(crate::native_signatures::NativeTypeId::UiNode) {
                let _ = writeln!(output, "type Record{index} = tsrust_native_ui::Node;");continue;}
            if record.native_type == Some(crate::native_signatures::NativeTypeId::SignalListener) { let _=writeln!(output,"type Record{index} = tsrust_native_signals::Listener;");continue;}
            if record.native_type == Some(crate::native_signatures::NativeTypeId::Uuid) { let _=writeln!(output,"type Record{index} = tsrust_native_uuid::Uuid;");continue;}
            if record.native_type == Some(crate::native_signatures::NativeTypeId::Decimal) { let _=writeln!(output,"type Record{index} = tsrust_native_decimal::Decimal;");continue;}
            if record.native_type == Some(crate::native_signatures::NativeTypeId::TrackedResource) {
                let _ = writeln!(output, "type Record{index} = TsrTrackedResource;");
                continue;
            }
            if record.native_type == Some(crate::native_signatures::NativeTypeId::File) {
                let _ = writeln!(output, "struct Record{index} {{ handle: tsrust_native_files::fs::FileHandle }}");
                continue;
            }
            if matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::Heap | crate::native_signatures::NativeTypeId::OrderedSet)) {
                let element=self.native_type(record.native_element().unwrap());
                let backend=if record.native_type==Some(crate::native_signatures::NativeTypeId::OrderedSet){"TsrOrderedSet"}else{"TsrHeap"};
                let _=writeln!(output,"#[derive(Clone)] struct Record{index} {{ values: {backend}<TsrOrdered<{element}>> }}");
                continue;
            }
            if let Some(element) = record.native_element() {
                if self.record_runtime.cloneable(index) { output.push_str("#[derive(Clone)]\n"); }
                let (parameters, payload) = if self.native_views.recursive(index) {
                    ("<'view>", self.recursive_view_native_type(element, "'view"))
                } else if self.native_collection_payload_parameter(element) {
                    ("<P>", "P".to_owned())
                } else { ("", self.native_type(element)) };
                let _ = writeln!(output, "struct Record{index}{parameters} {{ values: TsrDeque<{payload}> }}");
                continue;
            }
            if let Some(access) = native_view_emit::access(self.program, index) {
                let mutable = if access == crate::native_return_borrows::BorrowAccess::Exclusive { "mut " } else { "" };
                let _ = writeln!(output, "struct Record{index}<'a> {{ bytes: &'a {mutable}[u8] }}");
                continue;
            }
            if self.native_views.recursive(index) {
                let fields = record.fields.iter().enumerate().map(|(position, field)| {
                    format!("field{position}: {}", self.recursive_view_native_type(field.ty.expect("checked field"), "'view"))
                }).collect::<Vec<_>>().join(", ");
                let fields = self.order_fields(index, fields);
                let _ = writeln!(output, "struct Record{index}<'view> {{ {fields} }}");
                continue;
            }
            let mut generic_fields = Vec::new();
            let fields = record
                .fields
                .iter()
                .enumerate()
                .map(|(position, field)| {
                    let ty = field.ty.expect("checked record field");
                    let native = if self.scoped_record_native_field(index,position,ty) {
                        let parameter = format!("P{position}");
                        generic_fields.push(parameter.clone());
                        parameter
                    } else { self.native_type(ty) };
                    format!("field{position}: {native}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let derive =
                if self.record_runtime.recursive(index) || !self.record_runtime.cloneable(index) {
                    ""
                } else {
                    "#[derive(Clone)]\n"
                };
            let fields = self.order_fields(index, fields);
            let parameters = lifetime_parameters(&generic_fields);
            let _ = writeln!(output, "{derive}struct Record{index}{parameters} {{ {fields} }}\n");
            if self.borrowing_records.contains(&index)
                && let Some(support)=self.record_borrowing_support_cached(index,&mut record_support_cache) {
                output.push_str(&support);
            }
        }
        for (index, closure) in self.program.closures.iter().enumerate() {
            let asynchronous=if self.program.task_origins.contains(&index){"async "}else{""};
            let task_await=if asynchronous.is_empty(){""}else{".await"};
            if closure.function.template {
                continue;
            }
            let result_layout=self.result_closures.borrow().get(&index).cloned();
            let (capture_lifetimes, capture_types) = self.capture_layout(closure);
            let capture_bounds = self.captured_callback_bounds(closure);
            let mut capture_parameters = capture_lifetimes.clone();
            capture_parameters.extend(capture_bounds.iter().map(|bound| bound.split(':').next().expect("capture generic").trim().to_owned()));
            let lifetime = lifetime_parameters(&capture_parameters);
            let mut capture_declarations = capture_lifetimes.clone();
            capture_declarations.extend(capture_bounds.clone());
            let capture_impl = lifetime_parameters(&capture_declarations);
            let _ = writeln!(output, "struct Closure{index}{lifetime} {{");
            for (capture, field_type) in closure.captures.iter().zip(capture_types) {
                let _ = writeln!(output, "    capture{}: {field_type},", capture.inner);
            }
            let descriptor = match closure.forward {
                Some(ClosureForward::Once(inner)) => &self.program.closures[inner],
                _ => closure,
            };
            if closure.polymorphic
                || matches!(descriptor.forward, Some(ClosureForward::Function(_)))
                    && descriptor
                        .parameters
                        .iter()
                        .any(|&p| matches!(descriptor.function.bindings[p].ty, Type::Callback(_)))
            {
                output.push_str("}\n\n");
                for (method, body) in closure.methods.iter().enumerate() {
                    if !body.function.template {
                        self.closure_method(&mut output, index, closure, method);
                    }
                }
                continue;
            }
            let native_result=match &result_layout {
                Some(error)=>format!("Result<{},{}>",self.native_type(closure.function.result),error),
                None=>self.native_type(closure.function.result),
            };
            let _ = writeln!(output, "}}\nimpl{capture_impl} Closure{index}{lifetime} {{");
            let receiver = match closure.invocation {
                Mode::Read => "&self",
                Mode::Mutate => "&mut self",
                Mode::Consume => "self",
                Mode::Copy => unreachable!(),
            };
            let parameters = closure
                .parameters
                .iter()
                .map(|&id| {
                    let binding = &closure.function.bindings[id];
                    let mutable = if binding.mutable && (binding.mode == Mode::Consume || (binding.mode == Mode::Mutate && (binding.omission.is_omittable() || self.multiple_data_union(binding.ty)))) {
                        "mut "
                    } else {
                        ""
                    };
                    format!(
                        ", {mutable}value{id}: {}",
                        self.native_argument_type(ArgumentContract::from(binding), "")
                    )
                })
                .collect::<String>();
            let generic_bounds = self.nullable_callback_bounds(
                closure
                    .parameters
                    .iter()
                    .map(|&id| closure.function.bindings[id].input_ty.unwrap_or(closure.function.bindings[id].ty))
                    .chain(std::iter::once(closure.function.result)),
            );
            let call_generics = if generic_bounds.is_empty() {
                String::new()
            } else {
                format!("<{}>", generic_bounds.join(", "))
            };
            let forwarded_signature = self.forwarded_record_signature(closure);
            let closure_signature = self.closure_record_signature(index);
            let native_view_signature=self.native_view_closure_signature(closure,None,&closure.function,&closure.parameters);
            let signature = forwarded_signature.as_ref().map(|(_, signature)| signature).or_else(|| closure_signature.as_ref().map(|signature| &signature.signature)).or(native_view_signature.as_ref());
            let receiver = closure_signature.as_ref().and_then(|signature|signature.receiver_lifetime)
                .map_or_else(||receiver.to_owned(),|lifetime|format!("&{lifetime} mut self"));
            let receiver=signature.and_then(|signature|signature.receiver).map_or(receiver,|mode|format!("&'native_receiver {}self",if mode==Mode::Mutate {"mut "}else{""}));
            let mut parameters = signature.map_or(parameters, |signature| format!(", {}", signature.params));
            if closure.program_scoped { parameters.push_str(", test_program: &mut TsrTestContext<'_>, test_registry: &mut TsrTestRegistry"); }
            let call_generics = signature.map_or(call_generics, |signature| signature.generics.clone());
            let replacement_where = signature.map_or_else(String::new, |signature| signature.replacement_where.clone());
            let native_result = signature.map_or(native_result, |signature| match &result_layout {
                Some(error)=>format!("Result<{},{}>",signature.result,error),
                None=>signature.result.clone(),
            });
            // A task returned by a callback borrows its argument, not the
            // implicit &self lifetime chosen by Rust method elision.
            let task_argument_loan = signature.is_none()
                && self.program.task_type(closure.function.result)
                && native_result.contains("'_")
                && closure.parameters.iter().any(|&id| matches!(closure.function.bindings[id].mode,Mode::Read|Mode::Mutate));
            let (receiver,parameters,call_generics,native_result) = if task_argument_loan {
                let parameters=parameters.replace(": &", ": &'task ");
                let call_generics=if call_generics.is_empty(){"<'task>".into()}else{format!("<'task,{}",&call_generics[1..])};
                let receiver=if closure.captures.iter().any(|capture|capture.storage==Mode::Consume){receiver.replace('&',"&'task ")}else{receiver};
                (receiver,parameters,call_generics,native_result.replace("'_","'task"))
            } else {(receiver,parameters,call_generics,native_result)};
            let _ = writeln!(
                output,
                "    {asynchronous}fn call{call_generics}({receiver}{parameters}) -> {}{replacement_where} {{",
                native_result.clone()
            );
            self.test_program_capture_bindings(&mut output, closure);
            for capture in &closure.captures {
                let binding = &closure.function.bindings[capture.inner];
                let reference = if self.record_capture_is_view(capture,closure){if closure.invocation==Mode::Mutate{"&mut "}else{"&"}}else{match (capture.storage, closure.invocation) {
                    (Mode::Copy, Mode::Mutate) if capture.access == Mode::Mutate => "&mut ",
                    (Mode::Consume, Mode::Read) => "&",
                    (Mode::Consume, Mode::Mutate) => "&mut ",
                    (Mode::Mutate, Mode::Mutate) => "&mut *",
                    _ => "",
                }};
                let mutable = if (binding.mutable || self.finally_moves.partial_root(&closure.function,capture.inner)) && !binding.capture_reference {
                    "mut "
                } else {
                    ""
                };
                let _ = writeln!(
                    output,
                    "        let {mutable}value{} = {reference}self.capture{};",
                    capture.inner, capture.inner
                );
                if self.record_capture_is_view(capture,closure){if let Some(entry)=self.record_parameter_entry_view(&closure.function,capture.inner){let _=writeln!(output,"        {entry}");}}
            }
            if let Some(ClosureForward::Function(target)) = closure.forward {
                let mut arguments = closure.parameters.iter().map(|id| format!("value{id}")).collect::<Vec<_>>();
                arguments.extend(self.owner_inputs(target).iter().map(|source| format!("std::marker::PhantomData::<O{source}>")));
                let arguments=arguments.join(", ");
                let call=if let Some(destination)=&result_layout {
                    let source=&self.program.error_effects.function_errors[target];
                    if let Some((symbol,source_name))=self.result_functions.borrow().get(&target).cloned() {
                        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
                        let mapping=source.native_injection(&self.program.error_effects.closure_errors[index],&unions,runtime,closure.span).expect("checked forwarded Result contract");
                        let conversion=exception_emit::error_conversion("error",&source_name,destination,&mapping);
                        format!("{symbol}({arguments}).map_err(|error|{conversion})")
                    } else {
                        assert!(!source.runtime && !source.opaque && source.explicit.is_empty(),"fallible forwarded target requires Result registration");
                        format!("Ok(function{target}({arguments}))")
                    }
                } else {
                    format!("function{target}({arguments})")
                };
                let _ = writeln!(output, "        {call}");
            } else if let Some(ClosureForward::Native(id)) = closure.forward {
                self.native_bindings.borrow_mut().insert(id);
                let arguments = closure.parameters.iter().map(|id|format!("value{id}")).collect::<Vec<_>>();
                let binding=&crate::native_bindings::BINDINGS[id];
                let call=if closure.parameters.last().is_some_and(|parameter|closure.function.bindings[*parameter].rest) && binding.parameters.last().is_some_and(|parameter|parameter.rest) {
                    let crate::native_bindings::NativeBackend::Bundled{symbol}=binding.backend;
                    format!("{symbol}(&value0.iter().map(|value|value.as_ref()).collect::<Vec<_>>())")
                }else if binding.module=="std:testing-runtime" && binding.name=="checkAssertion" {
                    self.typed_native_assertion(&arguments,closure.span,Some(index)).unwrap_or_else(||crate::native_emit::call(binding,arguments))
                }else if binding.module=="std:testing-runtime" && binding.name=="finishTests" {
                    self.typed_runner_result("tsrust_testing_finish_tests_result()",closure.span,Some(index)).unwrap_or_else(||crate::native_emit::call(binding,arguments))
                }else if binding.module=="std:testing-runtime" && binding.name=="runTests" {
                    self.typed_runner_execution(closure.span,Some(index)).unwrap_or_else(||crate::native_emit::call(binding,arguments))
                }else if binding.module=="std:testing-runtime" && matches!(binding.name,"registerTest"|"registerHook") {
                    self.typed_test_registration(&arguments,closure.function.bindings[closure.parameters[1]].ty,closure.span,Some(index),binding.name=="registerHook").unwrap_or_else(||crate::native_emit::call(binding,arguments))
                }else if binding.module=="std:testing-runtime" && matches!(binding.name,"beginSuite"|"beginFile"|"endSuite") {
                    let call=if binding.name=="beginSuite"{format!("tsrust_testing_begin_suite_result({})",arguments[0])}else if binding.name=="beginFile"{format!("tsrust_testing_begin_file_result({})",arguments[0])}else{"tsrust_testing_end_suite_result()".into()};
                    self.typed_runner_result(&call,closure.span,Some(index)).unwrap_or_else(||crate::native_emit::call(binding,arguments))
                }else if binding.module=="std:sqlite" && binding.name=="open" {
                    let timeout=arguments.get(1).map(|value|format!("Some({value})")).unwrap_or("None".into());
                    self.http_result(closure.function.result,&format!("tsrust_native_sqlite::Connection::open(({}).as_ref(),{timeout})",arguments[0]))
                }else if binding.module=="std:path" && binding.name=="relative" {
                    let call=crate::native_emit::call(binding,arguments);
                    self.typed_native_result(&call,closure.span,Some(index),"Error","path-relative").unwrap_or_else(||format!("{call}.unwrap_or_else(|message|panic!(\"path.relative: {{message}}\"))"))
                }else{crate::native_emit::call(binding,arguments)};
                let _ = writeln!(output, "        {call}");
            } else if let Some(ClosureForward::Abstract(contract))=closure.forward {
                let kind=self.program.callback_contracts[contract].kind;
                let task_method=if kind==CallbackKind::Task{format!("TaskCallback::<(),{}>::invoke_once",self.callback_result_type(contract,self.program.callback_contracts[contract].result))}else{String::new()};
                let method=if kind==CallbackKind::Task{task_method.as_str()}else if matches!(kind,CallbackKind::Read | CallbackKind::SharedRead){"ReadCallback::invoke_read"}else if kind.is_once(){"OnceCallback::invoke_once"}else{"Callback::invoke"};
                let arguments=tuple(closure.parameters.iter().map(|id|format!("value{id}")));
                let call=format!("{method}(value0,{arguments}){}",if kind==CallbackKind::Task{".await"}else{""});
                let call=self.abstract_result_forward(call,contract,index);
                let _=writeln!(output,"        {call}");
            } else if let Some(ClosureForward::Once(inner))=closure.forward {
                let arguments = tuple(closure.parameters.iter().map(|id| format!("value{id}")));
                let call=if self.program.task_origins.contains(&inner){format!("{{ let {}task=value0; task.call().await }}",if self.program.closures[inner].invocation==Mode::Mutate{"mut "}else{""})}else{format!("OnceCallback::invoke_once(value0, {arguments})")};
                let call=self.closure_result_forward(call,inner,index);
                let _=writeln!(output,"        {call}");
            } else {
                if let Some(error_name)=&result_layout {
                    let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
                    let id=self.completion_next.get();
                    let body=self.callable_result_body(&closure.function,&closure.function.body,id,error_name,self.program.error_effects.closure_errors[index].clone(),runtime);
                    let _=writeln!(output,"        {body}");
                } else if crate::analysis::has_exception_syntax(&closure.function.body) {
                    let body=self.pure_callable_completion_body(&closure.function);
                    let _=writeln!(output,"        {body}");
                } else {
                    self.parameter_initialization(&mut output, &closure.function);
                    self.initialized_function_body(&mut output, &closure.function, 2);
                }
            }
            output.push_str("    }\n");
            if self.counted_calls {
                let _ = writeln!(output, "    {asynchronous}fn call_with_count{call_generics}({receiver}, supplied_count: usize{parameters}) -> {}{replacement_where} {{", native_result.clone());
                if closure.program_scoped { self.test_program_capture_bindings(&mut output, closure); }
                self.counted_closure_body(&mut output, closure);
                output.push_str("    }\n");
            }
            output.push_str("}\n\n");
            let params = closure
                .parameters
                .iter()
                .map(|&id| ArgumentContract::from(&closure.function.bindings[id]))
                .collect::<Vec<_>>();
            let (arguments, mut lifetimes) = self.native_argument_tuple(&params);
            lifetimes.splice(0..0, capture_lifetimes);
            // Trait adapters retain the lifetime of a borrowed owner argument.
            // Carried view arguments share one adapter lifetime; the direct call
            // keeps its more precise per-slot lifetimes and result intersections.
            let argument_views = (self.native_views.contains(closure.function.result) || closure.function.native_mutations.is_some())
                && (params.iter().any(|parameter| self.native_views.contains(parameter.ty))
                    || native_view_signature.as_ref().is_some_and(|signature|!signature.storage_lifetimes.is_empty())
                    // Empty recursive results quantify their output lifetime in
                    // the trait implementation as well as the direct method.
                    || self.native_view_empty_result(&closure.function));
            let mut argument_trait_bounds = String::new();
            let (arguments, native_result) = if argument_views {
                lifetimes.push("'native_arguments".into());
                let mut result = native_result.replace("'_", "'native_arguments");
                if let Some(signature) = &native_view_signature {
                    argument_trait_bounds = signature.replacement_where.clone();
                    let mut names=signature.generics.trim_matches(|c| c == '<' || c == '>').split(',').map(str::trim).filter(|name| name.starts_with('\'')).collect::<Vec<_>>();
                    // Replace longer identifiers first, so slot 1 cannot rewrite slot 10.
                    names.sort_by_key(|name|std::cmp::Reverse(name.len()));
                    for name in names {
                        let replacement=closure.parameters.iter().enumerate().find(|(_,root)|signature.storage_lifetimes.get(root).is_some_and(|storage|storage==name)).map_or_else(||"'native_arguments".to_owned(),|(position,_)|format!("'p{position}"));
                        result = result.replace(name, &replacement);
                        argument_trait_bounds = argument_trait_bounds.replace(name, &replacement);
                    }
                }
                (arguments.replace("'_", "'native_arguments"), result)
            } else {
                (arguments, native_result)
            };
            let arguments=if task_argument_loan {
                // The returned task retains the intersection of input loans.
                let mut arguments=arguments;
                for position in 0..params.len(){arguments=arguments.replace(&format!("'p{position} "),"'task ");}
                lifetimes.retain(|name|!name.starts_with("'p"));
                lifetimes.push("'task".into());arguments
            }else{arguments};
            let impl_lifetimes = {
                let mut declarations = lifetimes.clone();
                declarations.extend(generic_bounds);
                declarations.extend(capture_bounds);
                if declarations.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", declarations.join(", "))
                }
            };
            let forwarded = (0..closure.parameters.len())
                .map(|id| format!("args.{id}"))
                .collect::<Vec<_>>()
                .join(", ");
            let forwarded_trait_where = forwarded_signature.as_ref().map_or_else(||closure_signature.as_ref().map_or_else(String::new, |signature|signature.trait_where.clone()), |(target,_)| self.forwarded_record_trait_where(*target));
            let forwarded_trait_where = if argument_trait_bounds.is_empty() {forwarded_trait_where} else if forwarded_trait_where.is_empty() {argument_trait_bounds} else {format!("{forwarded_trait_where}, {}",argument_trait_bounds.trim_start_matches(" where "))};
            let forwarded = forwarded_signature.as_ref().map_or_else(|| {
                let mut arguments = vec![forwarded.clone()];
                arguments.extend(self.unnamed_owner_markers(self.closure_owner_inputs(index).len()).into_iter().map(|marker|format!("std::marker::PhantomData::<{marker}>")));
                arguments.into_iter().filter(|value|!value.is_empty()).collect::<Vec<_>>().join(", ")
            }, |(target,_)|self.forwarded_record_trait_arguments(*target,&forwarded));
            if closure.program_scoped {
                let separator = if forwarded.is_empty() { "" } else { ", " };
                let _ = writeln!(output, "impl{impl_lifetimes} ProgramCallback<{arguments}, {native_result}> for Closure{index}{lifetime}{forwarded_trait_where} {{ fn invoke_program(&mut self, args:{arguments}, program:&mut TsrTestContext<'_>, registry:&mut TsrTestRegistry)->{native_result} {{self.call({forwarded}{separator}program,registry)}} }}");
                self.callback_result_adapters(&mut output,index,&params,&arguments,&forwarded,&impl_lifetimes,&lifetime,&native_result);
                continue;
            }
            // HTTP retains this receiver while polling its borrowed task. The
            // ordinary callback traits cannot express a result borrowing self.
            if let Type::Closure(result)=closure.function.result
                && self.program.task_origins.contains(&result)
                && self.program.closures[result].borrowed
                && closure.captures.iter().any(|capture|capture.storage==Mode::Consume) {continue;}
            if self.callback_traits || self.program.task_origins.contains(&index) {
                let result = native_result.clone();
                let task_trait=if asynchronous.is_empty(){"OnceCallback"}else{"TaskCallback"};
                let counted_method = if self.counted_calls { format!("    {asynchronous}fn invoke_once_with_count(mut self, args: {arguments}, count: usize) -> {result} {{ self.call_with_count(count, {forwarded}){task_await} }}\n") } else { String::new() };
                let _ = writeln!(
                    output,
                    "impl{impl_lifetimes} {task_trait}<{arguments}, {result}> for Closure{index}{lifetime}{forwarded_trait_where} {{\n    {asynchronous}fn invoke_once(mut self, args: {arguments}) -> {result} {{ self.call({forwarded}){task_await} }}\n{counted_method}}}\n"
                );
            }
            if self.callback_traits && !self.program.task_origins.contains(&index) && closure.invocation == Mode::Read {
                let result = native_result.clone();
                let counted_method = if self.counted_calls { format!("    fn invoke_read_with_count(&self, args: {arguments}, count: usize) -> {result} {{ self.call_with_count(count, {forwarded}) }}\n") } else { String::new() };
                let _ = writeln!(
                    output,
                    "impl{impl_lifetimes} ReadCallback<{arguments}, {result}> for Closure{index}{lifetime}{forwarded_trait_where} {{\n    fn invoke_read(&self, args: {arguments}) -> {result} {{ self.call({forwarded}) }}\n{counted_method}}}\n"
                );
            }
            if self.callback_traits && !self.program.task_origins.contains(&index) && closure.invocation != Mode::Consume {
                let result = native_result.clone();
                let counted_method = if self.counted_calls { format!("    fn invoke_with_count(&mut self, args: {arguments}, count: usize) -> {result} {{ self.call_with_count(count, {forwarded}) }}\n") } else { String::new() };
                let _ = writeln!(
                    output,
                    "impl{impl_lifetimes} Callback<{arguments}, {result}> for Closure{index}{lifetime}{forwarded_trait_where} {{\n    fn invoke(&mut self, args: {arguments}) -> {result} {{ self.call({forwarded}) }}\n{counted_method}}}\n"
                );
            }
            self.callback_result_adapters(&mut output,index,&params,&arguments,&forwarded,&impl_lifetimes,&lifetime,&native_result);
        }
        let declared_families = crate::callable_layout::declared_storage_families(self.program,|record,position|self.scoped_record_native_field(record,position,self.program.records[record].fields[position].ty.expect("checked field")));
        if self.program.stored_contracts.families().any(|(_, declaration, _)| self.program.callback_contracts[declaration].kind == crate::syntax::CallbackKind::SharedRead) {
            output.push_str(include_str!("../stdlib/testing/runtime/shared_callback.rs"));
        }
        // A lifetime-free family without environments cannot hold a callback whose
        // contract mentions a byte view; such callbacks live in scoped families. The
        // type can still be named by an absent nullable value. It has no variant, so
        // its call can never run and needs no result lifetime.
        let mut uninhabited_view_families = HashSet::new();
        for (id, declaration, origins) in self.program.stored_contracts.families() {
            let contract = &self.program.callback_contracts[declaration];
            if !origins.is_empty()
                || self.program.stored_contracts.is_borrowed(id)
                || self.program.stored_contracts.scoped_native_family(id).is_some()
                || !(self.native_views.contains(contract.result) || contract.params.iter().any(|parameter| self.native_views.contains(parameter.ty)))
            {
                continue;
            }
            let receiver = match contract.invocation(&self.program.callable_shapes) { Mode::Consume => "self", Mode::Read => "&self", _ => "&mut self" };
            let parameters = contract.params.iter().enumerate().map(|(position, &parameter)| format!(",_argument{position}:{}", self.native_argument_type(parameter, ""))).collect::<String>();
            let body = if receiver == "self" { "match self {}" } else { "match *self {}" };
            writeln!(output, "#[allow(dead_code)] enum StoredCallable{id} {{}}\n#[allow(dead_code)] impl StoredCallable{id} {{ fn call<R>({receiver}{parameters})->R {{ {body} }} }}").unwrap();
            uninhabited_view_families.insert(id);
        }
        for (id, declaration, origins) in self
            .program
            .stored_contracts
            .families()
            .filter(|(id, _, origins)| !origins.is_empty() || declared_families.contains(id))
            .filter(|(id, _, _)| !uninhabited_view_families.contains(id))
        {
            let task_family_start=output.len();
            let contract = &self.program.callback_contracts[declaration];
            if self.program.stored_contracts.is_borrowed(id) {self.emit_borrowed_family(&mut output,id,declaration,origins);continue;}
            if let Some(layout)=self.program.stored_contracts.scoped_native_family(id) {
                self.emit_native_view_family(&mut output,id,layout,contract).expect("checked scoped owning native callable family");
                continue;
            }
            let family = if self.program.stored_contracts.is_owned_read(id) {
                crate::callable_dispatch::Family::checked_owned_read(contract,&self.program.callable_shapes,origins.iter().copied(),&self.program.closures,crate::Span::default()).expect("checked owned readonly family")
            } else {
                crate::callable_dispatch::Family::checked_results(
                contract, &self.program.callable_shapes, origins.iter().copied(),
                &self.program.closures, crate::Span::default(),
                |actual,expected|self.program.stored_contracts.equivalent_result(actual,expected),
            )
            .expect("checked native callable storage family")
            };
            let result_layout=self.result_storage.borrow().get(&id).cloned();
            let native_result=match &result_layout {Some((name,_))=>format!("Result<{},{}>",self.native_type(contract.result),name),None=>self.native_type(contract.result)};
            // A lifetime-free family whose result is a byte view can only return a
            // view that it received: its view arguments and its result share one
            // named lifetime.
            let argument_views=self.native_views.contains(contract.result) && contract.params.iter().any(|parameter|self.native_views.contains(parameter.ty));
            let native_result=if argument_views {native_result.replace("'_","'native_arguments")} else {native_result};
            let mut family_text=String::new();
            family.emit_counted_mapped(
                &mut family_text,
                id,
                self.counted_calls,
                |parameter| {let ty=self.native_argument_type(parameter, "");if argument_views {ty.replace("'_","'native_arguments")} else {ty}},
                |_| native_result.clone(),
                |origin,call| self.stored_origin_result(id,origin,call),
            );
            if argument_views {
                family_text=family_text.replace("fn call(","fn call<'native_arguments>(").replace("fn call_with_count(","fn call_with_count<'native_arguments>(");
            }
            output.push_str(&family_text);
            if self.callback_traits {
                let contract = &self.program.callback_contracts[declaration];
                let (arguments, mut lifetimes) = self.native_argument_tuple(&contract.params);
                let arguments=if argument_views {lifetimes.push("'native_arguments".into());arguments.replace("'_","'native_arguments")} else {arguments};
                family.emit_invocation_adapters_counted(
                    &mut output,
                    id,
                    self.counted_calls,
                    &lifetime_parameters(&lifetimes),
                    &arguments,
                    &native_result,
                );
                let forwarded = (0..contract.params.len()).map(|i| format!("args.{i}")).collect::<Vec<_>>().join(", ");
                let traits = match contract.invocation(&self.program.callable_shapes) {
                    Mode::Consume => vec![("OnceCallback", "invoke_once", "self")],
                    Mode::Read => vec![("ReadCallback", "invoke_read", "&self"), ("Callback", "invoke", "&mut self")],
                    _ => vec![("Callback", "invoke", "&mut self")],
                };
                self.callback_family_result_adapters(&mut output, id, contract, &format!("StoredCallable{id}"), &lifetime_parameters(&lifetimes), &arguments, &forwarded, &native_result, &traits);
            }
            if contract.kind==CallbackKind::Task {
                let text=output.split_off(task_family_start).replace("fn call", "async fn call").replace("OnceCallback<","TaskCallback<").replace("fn invoke_once","async fn invoke_once").replace("self.call() }","self.call().await }").replace("self.call()}","self.call().await}");
                output.push_str(&text);
            }
        }
        for (plan_id, plan) in self.fixed.plans.iter().enumerate() {
            let index = plan.function;
            let fun = &self.program.functions[index];
            self.active_plan.set(Some(plan_id));
            let native_function_signature_emit::NativeSignature {params,generics,replacement_where,result: view_result,..} = self.native_function_signature(index,&plan.parameters);
            let result_target=self.result_functions.borrow().get(&index).cloned();
            if result_target.is_some() {
            }
            let name = if let Some((symbol,_))=&result_target {
                if plan.parameters.iter().any(Option::is_some) {format!("{symbol}_fixed{plan_id}")} else {symbol.clone()}
            } else if plan.parameters.iter().any(Option::is_some) {
                format!("fixed_function{plan_id}")
            } else if fun.name == "main" {
                "main".to_owned()
            } else {
                format!("function{index}")
            };
            let native_result=match &result_target {Some((_,error))=>format!("Result<{},{}>",view_result,error),None=>view_result};
            let _ = writeln!(output,"fn {name}{generics}({params}) -> {native_result}{replacement_where} {{");
            let async_entry=fun.body.iter().any(|stmt|matches!(stmt.kind,StmtKind::Expr(Expr{kind:ExprKind::Unary{op:UnaryOp::Await,..},..})));
            if async_entry {output.push_str("tsrust_async_block_on(async move {\n");}
            if name == "main" {
                output.push_str(self.entry_setup);
            }
            if let Some(body)=self.rust_function_body(index,&plan.parameters) {
                let _=writeln!(output,"    {body}");
            } else if let Some((_,error_name))=&result_target {
                let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
                let id=self.completion_next.get();
                let body=self.callable_result_body(fun,&fun.body,id,error_name,self.program.error_effects.function_errors[index].clone(),runtime);
                let _=writeln!(output,"    {body}");
            } else if crate::analysis::has_exception_syntax(&fun.body) {
                let body=self.pure_callable_completion_body(fun);
                let _=writeln!(output,"    {body}");
            } else {
                self.parameter_initialization(&mut output, fun);
                self.initialized_function_body(&mut output, fun, 1);
            }
            if async_entry {output.push_str("})\n");}
            output.push_str("}\n\n");
        }
        self.active_plan.set(None);
        if let Some(index)=self.program.functions.iter().position(|function|function.name=="main") {
            if let Some((symbol,error_name))=self.result_functions.borrow().get(&index).cloned() {
                output.push_str(&self.result_entry_adapter(index,&symbol,&error_name));
            }
        }
        if !self.program.functions.iter().any(|function| function.name == "main") {
            let _ = writeln!(output, "fn main() {{ {} }}", self.entry_setup);
        }
        self.fixed_records(&mut output);
        output.push_str(&self.record_runtime.emit_with_storage(&self.program.records, &self.program.stored_contracts));
        if output.contains("tsr_console_record_") {
            for (id, record) in self.program.records.iter().enumerate() {
                if record.literal_definition.is_some(){continue;}
                if !self.record_inspectable(id) || record.native_type == Some(crate::native_signatures::NativeTypeId::StringView) {
                    continue;
                }
                let borrowed_head=self.borrowed_record_console_head(id);
                let borrowed_layout=borrowed_head.is_some();
                if let Some(head)=borrowed_head{output.push_str(&head);}else{
                let _ = writeln!(
                    output,
                    "fn tsr_console_record_{id}(value: &{}, depth: usize) -> TsrConsoleValue {{ if depth >= 128 {{ return TsrConsoleValue::Truncated; }} let mut fields = Vec::new();", self.native_type(Type::Record(id))
                );
                }
                let ordered = self.record_runtime.ordered.contains(&id);
                if ordered {
                    output.push_str("for display_index in value.key_order.iter() { match *display_index {");
                }
                for (field, info) in record.fields.iter().enumerate() {
                    let field_value=if borrowed_layout && !matches!(info.ty,Some(Type::Record(child)) if !self.program.records[child].fields.is_empty()){format!("value.field{field}")}else{format!("&value.field{field}")};
                    let cell = self.console_value(
                        &field_value,
                        info.ty.unwrap(),
                        "depth + 1",
                        true,
                    );
                    if record.is_tuple() {
                        let _ = writeln!(output,"fields.push({cell});");
                    } else if ordered {
                        let _ = writeln!(output,"{field} => fields.push((std::string::String::from({:?}), {cell})),",info.name.text);
                    } else {
                        let _ = writeln!(output,"fields.push((std::string::String::from({:?}), {cell}));",info.name.text);
                    }
                }
                if ordered {
                    output.push_str("_ => unreachable!(\"record field index\") } }");
                }
                output.push_str(if record.is_tuple() { "TsrConsoleValue::Array(fields) }\n" } else { "TsrConsoleValue::Object(fields) }\n" });
            }
        }
        // Record arrays inline only small records whose entire field tree has no
        // arrays. This preserves recursive indirection and bounds inline size.
        for index in 0..self.program.records.len() {
            if !self.record_runtime.recursive(index)
                && (output.contains(&format!("RecordArray{index}")) || native_view_emit::access(self.program, index).is_some())
            {
                if native_view_emit::access(self.program, index).is_some() {
                    let _ = writeln!(output, "type RecordArray{index}<'a> = Vec<Record{index}<'a>>;");
                    continue;
                }
                let storage = if self.small_record(index) {
                    "Array"
                } else {
                    "Vec"
                };
                // The declaration decides which fields are generic, as for the record itself.
                let declaration = self.native_record_declaration(index);
                let parameters = self.program.records[index].fields.iter().enumerate().filter_map(|(field,value)| self.scoped_record_native_field(index,field,value.ty.expect("checked field")).then(||format!("P{field}"))).collect::<Vec<_>>();
                let arguments = if parameters.is_empty() {String::new()} else {format!("<{}>",parameters.join(", "))};
                let _ = writeln!(
                    output,
                    "type RecordArray{index}{arguments} = {storage}<Record{declaration}{arguments}>;"
                );
            }
        }
        if output.contains("TsrNull") || output.contains("TsrUndefined") {
            output.push_str(include_str!("../runtime/values.rs"));
        }
        // Support is emitted once per program, only when its representation is used.
        if output.contains("String")
            || output.contains("tsr_boolean_string(")
            || output.contains("tsr_int_locale_string(")
            || output.contains("tsr_number_locale_string(")
            || output.contains("tsr_int_to_string(")
            || output.contains("tsr_number_to_string(")
            || output.contains("tsr_number_fixed(")
            || output.contains("tsr_number_exponential(")
            || output.contains("tsr_number_precision(")
            || output.contains("tsr_number_parse_float(")
            || output.contains("tsr_number_parse_int(")
            || output.contains("tsr_number_from_text(")
            || output.contains("tsr_string_normalize(")
            || output.contains("tsr_array_locale(")
            || output.contains("tsr_array_join(")
        {
            output.push_str(crate::runtime::STRINGS);
        }
        if self.program.records.iter().any(|record|crate::stdlib::cache::is_cache(record.native_type)){output.push_str(&crate::stdlib::cache::runtime());}
        if self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].module == "std:threads")
            || self.program.records.iter().any(|record| matches!(record.native_type, Some(crate::native_signatures::NativeTypeId::Thread | crate::native_signatures::NativeTypeId::ScopedThread | crate::native_signatures::NativeTypeId::ThreadPool | crate::native_signatures::NativeTypeId::Channel | crate::native_signatures::NativeTypeId::Sender | crate::native_signatures::NativeTypeId::TaskContext))) {
            output.push_str(crate::stdlib::threads::RUNTIME);
        }
        if self.native_bindings.borrow().iter().any(|&id| {
            crate::native_bindings::BINDINGS[id]
                .features
                .contains(&"path")
        }) {
            output.push_str(&crate::native_runtime::path_module());
            output.push_str("fn tsrust_std_path_basename(path: &String, suffix: Option<&String>) -> String { String::Owned(tsrust_native_path::basename(path.as_ref(), suffix.map(|value| value.as_ref()), cfg!(windows))) }\n");
        }
        if self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].features.contains(&"path")) {
            output.push_str("fn tsrust_std_path_dirname(path: &String) -> String { String::Owned(tsrust_native_path::dirname(path.as_ref(), cfg!(windows))) }\nfn tsrust_std_path_extname(path: &String) -> String { String::Owned(tsrust_native_path::extname(path.as_ref(), cfg!(windows))) }\n");
        }
        if self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].features.contains(&"path")) {
            output.push_str("fn tsrust_std_path_is_absolute(path: &String) -> bool { tsrust_native_path::is_absolute(path.as_ref(), cfg!(windows)) }\nfn tsrust_std_path_normalize(path: &String) -> String { String::Owned(tsrust_native_path::normalize(path.as_ref(), cfg!(windows))) }\n");
        }
        if self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:path" && crate::native_bindings::BINDINGS[id].name=="relative") {
            output.push_str("fn tsrust_std_path_relative(from:&String,to:&String)->Result<String,std::string::String>{tsrust_native_path::relative_host(from.as_ref(),to.as_ref()).map(String::Owned)}\n");
        }
        if self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:path" && crate::native_bindings::BINDINGS[id].name=="join") {
            output.push_str("fn tsrust_std_path_join(paths: &[&str]) -> String { String::Owned(tsrust_native_path::join(paths, cfg!(windows))) }\n");
        }
        if self.program.records.iter().any(|record| record.native_type == Some(crate::native_signatures::NativeTypeId::TrackedResource)) || self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].features.contains(&"testing-resources")) {
            output.push_str(include_str!("../stdlib/testing/runtime/resources.rs"));
        }
        if self.native_bindings.borrow().iter().any(|&id| {
            crate::native_bindings::BINDINGS[id].features.contains(&"testing")
        }) {
            output.push_str(include_str!("../stdlib/testing/runtime/assertion.rs"));
        }
        if self.native_bindings.borrow().iter().any(|&id| {
            crate::native_bindings::BINDINGS[id].features.contains(&"testing-runner")
        }) {
            output.push_str(include_str!("../stdlib/testing/runtime/mod.rs"));
            if self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].name == "registerHook") {
                output.push_str(include_str!("../stdlib/testing/runtime/hooks.rs"));
            }
            if self.native_bindings.borrow().iter().any(|&id| crate::native_bindings::BINDINGS[id].name == "registerTest") {
                output.push_str(include_str!("../stdlib/testing/runtime/registration.rs"));
            }
        }
        if self.program.records.iter().any(|r|r.native_type.and_then(crate::stdlib::date::runtime_type).is_some()){output.push_str(crate::stdlib::date::RUNTIME);}
        if self.program.records.iter().any(|r|r.native_type==Some(crate::native_signatures::NativeTypeId::SignalListener)){output.push_str(crate::stdlib::signals::RUNTIME);}
        if self.program.records.iter().any(|r|r.native_type==Some(crate::native_signatures::NativeTypeId::Uuid)){output.push_str(crate::stdlib::uuid::RUNTIME);}
        if self.program.records.iter().any(|r|r.native_type==Some(crate::native_signatures::NativeTypeId::Decimal)){output.push_str(crate::stdlib::decimal::RUNTIME);}
        if self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:numeric"){output.push_str(&format!("mod tsrust_numeric {{ {} }}",crate::stdlib::numeric::emit::RUNTIME));}
        if self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:crypto"){output.push_str(crate::stdlib::crypto::RUNTIME);}
        if self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:env"){output.push_str(&format!("mod tsrust_native_env {{ {} }}",include_str!("../stdlib/env/runtime.rs")));}
        if self.program.records.iter().any(|r|r.native_type==Some(crate::native_signatures::NativeTypeId::UiNode)){output.push_str(&crate::stdlib::ui::runtime(self.native_bindings.borrow().iter().any(|&id|crate::native_bindings::BINDINGS[id].module=="std:ui-runtime" && crate::native_bindings::BINDINGS[id].name=="run")));}
        for &id in self.native_bindings.borrow().iter() {
            let binding = &crate::native_bindings::BINDINGS[id];
            if binding.module == "std:date" {output.push_str(&crate::stdlib::date::emit::bridge(self.program,binding));}
            if binding.module == "std:crypto" {output.push_str(&crate::stdlib::crypto::emit::bridge(self.program,binding));}
            if binding.module == "std:uuid" {output.push_str(&crate::stdlib::uuid::emit::bridge(self.program,binding));}
            if binding.module == "std:decimal" {output.push_str(&crate::stdlib::decimal::emit::bridge(self.program,binding));}
            if binding.module == "std:numeric" { output.push_str(&crate::stdlib::numeric::emit::bridge(self.program, binding));
            }
            if binding.module == "std:ui-runtime" {
                output.push_str(&crate::stdlib::ui::emit::bridge(self.program,binding)); }
            if binding.module == "std:env" { output.push_str(&crate::stdlib::env::emit::bridge(self.program,binding)); }
            if binding.module == "std:time" { output.push_str(crate::stdlib::time::RUNTIME); }
            if binding.module == "std:os" {
                output.push_str(&crate::stdlib::os::bridge(binding.name));
            }
        }
        let issue_bridge = crate::stdlib::schema::emit::issue_bridge(self.program, &self.record_runtime.ordered);
        output.push_str(&issue_bridge);
        // Constructing a plan without validating it need not materialize public
        // validation error records or their dependent trial runtime.
        if !issue_bridge.is_empty() && (!self.program.schema_choices.is_empty()||self.program.schema_pipelines.has_recursive()||self.program.schema_pipelines.has_collections()||self.program.schema_pipelines.has_messages()||self.program.schema_pipelines.callbacks().next().is_some()){
            output.push_str("mod tsrust_schema_union {\n");
            output.push_str(&include_str!("../stdlib/schema/runtime/union.rs").trim_start_matches("//! Borrowed branch preparation precedes a single consuming commitment.\n").replace("use super::issues::{Code, Issues, Kind, Segment};","use super::tsrust_schema_issues::{Code, Issues, Kind, Segment};"));
            output.push_str("\n}\n");
            output.push_str("mod tsrust_schema_trial {\n");
            output.push_str(&include_str!("../stdlib/schema/runtime/trial.rs")
                .lines().filter(|line| !line.starts_with("//!")).collect::<Vec<_>>().join("\n")
                .replace("use super::issues::{Code, Issues, Kind, Segment};", "use super::tsrust_schema_issues::{Code, Issues, Kind, Segment};")
                .replace("use super::union::Preparation;", "use super::tsrust_schema_union::Preparation;"));
            output.push_str("\n}\n");
        }
        for &binding_id in self.native_bindings.borrow().iter() {
            let binding = &crate::native_bindings::BINDINGS[binding_id];
            if binding.module != "std:schema" { continue; }
            let constructor = binding.parameters.is_empty();
            let crate::native_bindings::NativeType::Structured(signature) = (if constructor { binding.result } else { binding.parameters[0].ty }) else { unreachable!() };
            let Some(Type::Record(id)) = crate::native_binding_types::resolve(&self.program.records, signature) else { unreachable!("checked schema type") };
            let id = self.native_record_declaration(id);
            if constructor {
                let crate::native_bindings::NativeBackend::Bundled { symbol } = binding.backend;
                let fields=if binding.name=="__string" { "bounds: tsrust_schema_plan::StringBounds::default()" } else if binding.name=="__int" {"bounds: tsrust_schema_plan::IntBounds::default()"} else if binding.name=="__number" {"bounds: tsrust_schema_plan::NumberBounds::default()"} else { "" };
                let factory=self.schema_factory_initializer(id);
                let _ = writeln!(output, "fn {symbol}() -> Record{id} {{ Record{id} {{{factory}{fields}}} }}");
            } else if matches!(binding.name,"__minString"|"__maxString"|"__minInt"|"__maxInt"|"__gtInt"|"__ltInt"|"__minNumber"|"__maxNumber"|"__gtNumber"|"__ltNumber") {
                output.push_str(&crate::stdlib::schema::emit::constraint(self.program,id,binding));
            } else {
                output.push_str(&crate::stdlib::schema::emit::safe_parse(self.program, id, binding));
            }
        }
        if self.program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::JsonRaw | crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView | crate::native_signatures::NativeTypeId::JsonlReader | crate::native_signatures::NativeTypeId::JsonlReaderView | crate::native_signatures::NativeTypeId::JsonlWriter | crate::native_signatures::NativeTypeId::JsonlWriterView))) {output.push_str(&crate::native_runtime::json_module_with_writer(self.program.records.iter().any(|record|matches!(record.native_type,Some(crate::native_signatures::NativeTypeId::JsonlWriter | crate::native_signatures::NativeTypeId::JsonlWriterView)))));}
        for &id in self.native_bindings.borrow().iter() {let binding=&crate::native_bindings::BINDINGS[id];if binding.module=="std:json" {output.push_str(&crate::stdlib::json::emit::bridge(self.program,binding,&self.record_runtime.ordered));}}
        let byte_bindings = crate::native_bindings::BINDINGS.iter().enumerate().filter_map(|(id, binding)| (binding.module == "std:bytes" && self.native_bindings.borrow().contains(&id)).then_some(binding)).collect::<Vec<_>>();
        if !byte_bindings.is_empty() || self.native_bindings.borrow().iter().any(|&id| { let binding = &crate::native_bindings::BINDINGS[id]; binding.module == "std:fs" && matches!(binding.name, "write" | "writeAll" | "writeFile") }) { output.push_str(&crate::stdlib::bytes::emit::bridges(self.program, &byte_bindings)); }
        let file_bindings = crate::native_bindings::BINDINGS.iter().enumerate().filter_map(|(id, binding)| (binding.module == "std:fs" && self.native_bindings.borrow().contains(&id)).then_some(binding)).collect::<Vec<_>>();
        output.push_str(&crate::stdlib::fs::emit::bridges(self.program, &file_bindings));
        let numeric_support = output.contains("NumericArray<") || output.contains("NumericArray::");
        let literal_support = output.contains("LiteralStringArray");
        let split_support = output.contains("tsr_string_split(");
        let owned_support = split_support
            || literal_support
            || ["Array<", "Array::"].iter().any(|pattern| {
                output
                    .match_indices(pattern)
                    .any(|(index, _)| !output[..index].ends_with("Numeric"))
            });
        if owned_support {
            output.push_str(crate::runtime::ARRAYS);
        }
        if output.contains("tsr_console_test_")
            || output.contains("TsrConsoleValue")
            || output.contains("tsr_console_call(")
            || output.contains("tsr_console_write(")
            || output.contains("tsr_console_scalar(")
            || output.contains("tsr_console_text(")
        {
            output.push_str(include_str!("../runtime/console.rs"));
        }
        if numeric_support {
            output.push_str(crate::runtime::NUMERIC_ARRAYS);
        }
        self.data_spread_declarations(&mut output);
        if output.contains("TsrSpreadSlot") { output.push_str(spread_emit::SPREAD_SLOTS); }
        if output.contains(".tsr_spread_mut_items()") {
            output.push_str(crate::runtime::SPREAD_MUT_ITEMS_VEC);
            if owned_support {output.push_str(crate::runtime::SPREAD_MUT_ITEMS_ARRAY);}
            if numeric_support {output.push_str(crate::runtime::SPREAD_MUT_ITEMS_NUMERIC);}
        }

        if output.contains("tsr_flat_capacity(") || output.contains(".tsr_owned_items()") {
            output.push_str(crate::runtime::OWNED_ITEMS_VEC);
            if owned_support {
                output.push_str(crate::runtime::OWNED_ITEMS_ARRAY);
            }
            if numeric_support {
                output.push_str(crate::runtime::OWNED_ITEMS_NUMERIC);
            }
            for (id, _) in self.program.records.iter().enumerate() {
                if output.contains(&format!("struct RecordArray{id}(")) {
                    let _ = writeln!(
                        output,
                        "impl TsrOwnedItems<Record{id}> for RecordArray{id} {{ fn tsr_owned_items(mut self) -> impl Iterator<Item=Record{id}> {{ std::mem::take(&mut self.0).into_iter() }} }}"
                    );
                }
            }
        }
        if output.contains(".tsr_sort(") || output.contains(".tsr_try_sort(") {
            output.push_str(crate::runtime::SORT_VEC);
            if owned_support {
                output.push_str(crate::runtime::SORT_ARRAY);
            }
            if numeric_support {
                output.push_str(crate::runtime::SORT_NUMERIC);
            }
        }
        if output.contains(".tsr_splice(") || output.contains(".tsr_splice_iter(") {
            output.push_str(crate::runtime::SPLICE_VEC);
            if owned_support {
                output.push_str(crate::runtime::SPLICE_ARRAY);
            }
            if numeric_support {
                output.push_str(crate::runtime::SPLICE_NUMERIC);
            }
        }
        if output.contains(".tsr_unshift(") || output.contains(".tsr_unshift_iter(") {
            output.push_str(crate::runtime::UNSHIFT_VEC);
            if owned_support {
                output.push_str(crate::runtime::UNSHIFT_ARRAY);
            }
            if numeric_support {
                output.push_str(crate::runtime::UNSHIFT_NUMERIC);
            }
        }
        if output.contains("tsr_string_normalize(") { output.push_str(crate::runtime::STRING_NORMALIZATION); }
        if output.contains("tsr_math_") {
            output.push_str(crate::runtime::MATH_EXACT);
        }
        if split_support {
            output.push_str(crate::runtime::STRING_SPLIT);
        }
        if literal_support {
            output.push_str(crate::runtime::LITERAL_STRING_ARRAYS);
        }
        if output.contains("tsr_number_tie(") && !output.contains("fn tsr_number_tie(") {
            output.push_str(crate::runtime::NUMBER_TIE);
        }
        if output.contains("tsr_array_join(") || output.contains("tsr_array_locale(") || output.contains("TsrJoinText::append_join_text(") {
            if self.program.records.iter().any(|record|record.map_definition.is_some()) {
                output.push_str("impl<K: TsrSetKey,V,P:TsrMapEntry<K,V>> TsrJoinText for TsrMap<K,V,P> { fn append_join_text(&self, output: &mut std::string::String) { output.push_str(\"[object Map]\"); } }\n");
            }
            if self.program.records.iter().any(|record| record.set_definition.is_some()) {
                output.push_str("impl<T: TsrSetKey> TsrJoinText for TsrSet<T> { fn append_join_text(&self, output: &mut std::string::String) { output.push_str(\"[object Set]\"); } }\n");
            }
            output.push_str(crate::runtime::ARRAY_JOIN);
            if owned_support {
                output.push_str("impl<T: TsrJoinText> TsrJoinText for Array<T> { fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.iter()); } fn append_join_text(&self, output: &mut std::string::String) { tsr_append_join(output, self.iter(), \",\"); } }\n");
            }
            if numeric_support {
                output.push_str("impl<T: Copy + Default + TsrJoinText> TsrJoinText for NumericArray<T> { fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.iter()); } fn append_join_text(&self, output: &mut std::string::String) { tsr_append_join(output, self.iter(), \",\"); } }\n");
            }
            if literal_support {
                output.push_str("impl TsrJoinText for LiteralStringArray { fn append_locale_text(&self, output: &mut std::string::String) { tsr_append_locale(output, self.literal_iter()); } fn append_join_text(&self, output: &mut std::string::String) { tsr_append_join(output, self.literal_iter(), \",\"); } }\n");
            }
            for name in ["TsrNull", "TsrUndefined"] {
                if output.contains(name) {
                    let _ = writeln!(
                        output,
                        "impl TsrJoinText for {name} {{ fn append_join_text(&self, _: &mut std::string::String) {{}} }}"
                    );
                }
            }
            // A data union writes the text of its active alternative; null
            // and undefined write nothing, as array text requires.
            for (index, union) in self.program.unions.iter().enumerate() {
                if union.members.iter().any(|member| matches!(member, Type::Callback(_) | Type::Closure(_) | Type::StoredCallable(_) | Type::Callable(_) | Type::Void)) {
                    continue;
                }
                if self.nominal_union_alias(index).is_some() { continue; }
                let payloads = union.members.iter().filter(|member| !member.is_copy()).count();
                let parameter = |position: usize| if payloads == 1 { "P".to_owned() } else { format!("P{position}") };
                let (bounds, arguments) = if payloads == 0 {
                    (String::new(), String::new())
                } else {
                    (
                        format!("<{}>", (0..payloads).map(|position| format!("{}: TsrJoinText", parameter(position))).collect::<Vec<_>>().join(", ")),
                        format!("<{}>", (0..payloads).map(parameter).collect::<Vec<_>>().join(", ")),
                    )
                };
                let _ = write!(output, "impl{bounds} TsrJoinText for Union{index}{arguments} {{");
                for method in ["append_join_text", "append_locale_text"] {
                    let _ = write!(output, "fn {method}(&self, output: &mut std::string::String) {{ match self {{");
                    for (variant, member) in union.members.iter().enumerate() {
                        let body = if matches!(member, Type::Null | Type::Undefined) { "{ let _ = (payload, &output); }".to_owned() } else { format!("payload.{method}(output)") };
                        let _ = write!(output, "Union{index}::V{variant}(payload)=>{body},");
                    }
                    output.push_str("} }");
                }
                output.push_str("}\n");
            }
            for index in 0..self.program.records.len() {
                if let Some(registration)=self.record_join_registration(index){output.push_str(&registration);}
                if output.contains(&format!("struct RecordArray{index}(")) {
                    let _ = writeln!(
                        output,
                        "impl TsrJoinText for RecordArray{index} {{ fn append_locale_text(&self, output: &mut std::string::String) {{ tsr_append_locale(output, self.iter()); }} fn append_join_text(&self, output: &mut std::string::String) {{ tsr_append_join(output, self.iter(), \",\"); }} }}"
                    );
                }
            }
        }
        if literal_support
            || output.contains(".tsr_includes(")
            || output.contains(".tsr_index_of(")
            || output.contains("tsr_search_start(")
        {
            output.push_str(crate::runtime::SEARCH_START);
            if numeric_support {
                output.push_str(crate::runtime::SEARCH_NUMERIC);
            }
            if owned_support {
                output.push_str(crate::runtime::SEARCH_OWNED);
            }
        }
        output
    }

    fn string_literal(&self, value: &str) -> String {
        format!("String::Borrowed({value:?})")
    }

    fn small_record(&self, index: usize) -> bool {
        if self.native_views.contains(Type::Record(index)) { return false; }
        let mut pending = vec![(Type::Record(index), 0_usize)];
        let mut bytes = 0_usize;
        while let Some((value, depth)) = pending.pop() {
            if depth > 32 {
                return false;
            }
            match value {
                Type::Record(index) => {
                    // Charge padding conservatively for every field and record.
                    bytes += 8;
                    for field in &self.program.records[index].fields {
                        pending.push((field.ty.expect("checked field type"), depth + 1));
                    }
                }
                Type::String => bytes += crate::storage::string_storage_bound(),
                Type::Null | Type::Undefined => {}
                Type::Int | Type::Number | Type::Boolean => bytes += 8,
                _ => return false,
            }
            if bytes > crate::storage::MAX_INLINE_RECORD_BYTES {
                return false;
            }
        }
        true
    }

    fn binding_shape(&self, id: usize) -> Option<crate::fixed_storage::Shape> {
        self.active_plan
            .get()
            .and_then(|p| self.fixed.plans[p].bindings.get(&id).cloned())
    }

    fn expression_shape(&self, expr: &Expr) -> Option<crate::fixed_storage::Shape> {
        let plan = self.active_plan.get()?;
        crate::fixed_storage::shape_of(self.program, &self.fixed.plans[plan].bindings, expr)
    }

    fn shape_type(&self, shape: &crate::fixed_storage::Shape) -> String {
        match shape {
            crate::fixed_storage::Shape::LiteralStrings => "LiteralStringArray".into(),
            crate::fixed_storage::Shape::ScalarArray { scalar, width } => {
                format!(
                    "[{}; {width}]",
                    match scalar {
                        crate::fixed_storage::Scalar::Int => "i64",
                        crate::fixed_storage::Scalar::Number => "f64",
                        crate::fixed_storage::Scalar::Boolean => "bool",
                    }
                )
            }
            crate::fixed_storage::Shape::Record { .. } => {
                format!("FixedRecord{}", self.fixed.record_id(shape))
            }
            crate::fixed_storage::Shape::Array { record, width } => {
                format!("[Record{record}; {width}]")
            }
        }
    }

    fn fixed_literal(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        shape: &crate::fixed_storage::Shape,
    ) -> String {
        if let ExprKind::Unary {
            op: UnaryOp::OptionalSelected,
            value,
            ..
        } = &expr.kind
        {
            return self.fixed_literal(fun, value, shape);
        }
        if matches!(
            shape,
            crate::fixed_storage::Shape::ScalarArray { .. }
                | crate::fixed_storage::Shape::Array { .. }
        ) {
            if expr.target == Some(CallTarget::Clone) {
                let ExprKind::Call(_, args) = &expr.kind else {
                    unreachable!()
                };
                return format!("Clone::clone({})", self.expr(fun, &args[0], Mode::Read));
            }
            let ExprKind::Array(items) = &expr.kind else {
                unreachable!()
            };
            return format!(
                "[{}]",
                items
                    .iter()
                    .map(|item| self.expr(fun, item, Mode::Consume))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        if matches!(shape, crate::fixed_storage::Shape::LiteralStrings) {
            let ExprKind::Array(items) = &expr.kind else {
                unreachable!()
            };
            let values = items
                .iter()
                .map(|item| {
                    let ExprKind::String(text) = &item.kind else {
                        unreachable!()
                    };
                    format!("{text:?}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            return format!(
                "{{ static LITERAL_VALUES: [&str; {}] = [{values}]; LiteralStringArray::new(&LITERAL_VALUES) }}",
                items.len()
            );
        }
        let crate::fixed_storage::Shape::Record { record, widths } = shape else {
            unreachable!()
        };
        let ExprKind::Record(fields) = &expr.kind else {
            unreachable!()
        };
        let values = fields
            .iter()
            .map(|(key, value)| {
                let field = self.fields[*record][key.text.as_str()];
                let code = if widths[field].is_some() {
                    let ExprKind::Array(items) = &value.kind else {
                        unreachable!()
                    };
                    format!(
                        "[{}]",
                        items
                            .iter()
                            .map(|item| self.expr(fun, item, Mode::Consume))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    self.expr(fun, value, Mode::Consume)
                };
                format!("field{field}: {code}")
            })
            .collect::<Vec<_>>()
            .join(", ");
        let values = self.order_values(*record, fields, values);
        format!("{} {{ {values} }}", self.shape_type(shape))
    }

    fn fixed_records(&self, output: &mut String) {
        for (id, shape) in self.fixed.records.iter().enumerate() {
            let crate::fixed_storage::Shape::Record { record, widths } = shape else {
                unreachable!()
            };
            let fields = self.program.records[*record]
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let field_type = if let Some(width) = widths[index] {
                        let Type::Array(ArrayElement::Record(child, 0)) =
                            field.ty.expect("checked fixed field")
                        else {
                            unreachable!()
                        };
                        format!("[Record{child}; {width}]")
                    } else {
                        self.native_type(field.ty.expect("checked field"))
                    };
                    format!("field{index}: {field_type}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let fields = self.order_fields(*record, fields);
            let _ = writeln!(output, "struct FixedRecord{id} {{ {fields} }}");
            if !self.record_runtime.cloneable(*record) {
                continue;
            }
            let values = self.program.records[*record]
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let value = if widths[index].is_some() {
                        let Type::Array(ArrayElement::Record(child, 0)) =
                            field.ty.expect("checked fixed field")
                        else {
                            unreachable!()
                        };
                        format!("RecordArray{child}::from(self.field{index}.clone())")
                    } else {
                        format!("self.field{index}.clone()")
                    };
                    format!("field{index}: {value}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            let values = if self.record_runtime.ordered.contains(record) { format!("{values}, key_order: self.key_order") } else { values };
            let _ = writeln!(
                output,
                "impl FixedRecord{id} {{ fn clone_owned(&self) -> Record{record} {{ Record{record} {{ {values} }} }} }}"
            );
        }
    }

    #[inline(never)]
    fn completion_error_layout(&self,name:&str,contract:&crate::error_effects::FailureSet,runtime:Option<Type>,span:crate::Span)->(String,String) {
        let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
        let payloads=contract.native_payloads(&unions,runtime,span).expect("checked native error layout");
        if payloads.is_empty() {return (String::new(),"std::convert::Infallible".into());}
        let parameters=(0..payloads.len()).map(|index|format!("P{index}")).collect::<Vec<_>>();
        let parameter_types=parameters.iter().map(String::as_str).collect::<Vec<_>>();
        let declaration=exception_emit::typed_error_enum(&format!("{name}<{}>",parameters.join(",")),&parameter_types);
        let arguments=payloads.into_iter().map(|ty|self.native_type(ty)).collect::<Vec<_>>();
        (declaration,format!("{name}<{}>",arguments.join(",")))
    }

    #[inline(never)]
    #[cfg(test)]
    fn completion_handlers(&self,fun:&CheckedFunction,region:&ExceptionRegion,ids:(usize,usize,usize),names:(&str,&str),outgoing:crate::error_effects::FailureSet,runtime:Option<Type>)->String {
        let mut output=String::new();let mut pending=Vec::new();
        self.schedule_completion_handlers(&mut output,fun,region,ids,(names.0.into(),names.1.into()),outgoing,runtime,true,&mut pending);
        self.run_statement_work(&mut output,fun,pending);output
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    fn completion_with_catch(&self,fun:&CheckedFunction,body:&[Stmt],handler:&CatchClause,ids:(usize,usize),incoming_name:&str,outgoing_name:&str,outgoing:crate::error_effects::FailureSet,runtime:Option<Type>,direct_returns:bool,inline:Option<exception_contract_emit::InlineExit>,inline_abrupt:Option<exception_contract_emit::InlineExit>)->String {
        let mut output=String::new();let mut pending=Vec::new();
        self.schedule_completion_catch(&mut output,fun,body,handler,ids,incoming_name.into(),outgoing_name.into(),outgoing,runtime,direct_returns,inline,inline_abrupt,&mut pending);
        self.run_statement_work(&mut output,fun,pending);output
    }

    #[inline(never)]
    #[cfg(test)]
    fn completion_with_finally(&self,fun:&CheckedFunction,body:&[Stmt],cleanup:&[Stmt],ids:(usize,usize),error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>)->String {
        let pending=self.completion_body(fun,body,ids.0,error_name,contract.clone(),runtime);
        let cleanup=self.completion_body(fun,cleanup,ids.1,error_name,contract,runtime);
        exception_emit::finally_region(&pending,&cleanup)
    }

    /// Pure finally regions still need completion routing, with a plain ABI.
    #[inline(never)]
    fn pure_callable_completion_body(&self,fun:&CheckedFunction)->String {
        let runtime=self.program.records.iter().position(|record|record.name.text=="#RuntimeError" && record.name.kind==crate::syntax::TokenKind::Symbol).map(Type::Record);
        let id=self.completion_next.get();
        let body=self.callable_result_body(fun,&fun.body,id,"std::convert::Infallible",crate::error_effects::FailureSet::default(),runtime);
        format!("match {body} {{Ok(value)=>value,Err(error)=>match error{{}}}}")
    }

    /// Finish a checked callable's completion under its Result return type.
    #[inline(never)]
    fn callable_result_body(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>)->String {
        let _callable=self.exception_regions.callable_boundary();
        let _controls=self.control_targets.callable_boundary();
        let pending=self.completion_body_frame(fun,body,id,error_name,contract,runtime,true);
        let normal=if fun.result==Type::Void {"Ok(())"}else{"unreachable!(\"checked non-void callable cannot fall through\")"};
        let suite_scope=crate::error_effects::checked_expressions(fun).iter().any(|expression|matches!(expression.target,Some(CallTarget::NativeBinding(id)) if crate::native_bindings::BINDINGS[id].module=="std:testing-runtime" && matches!(crate::native_bindings::BINDINGS[id].name,"beginSuite"|"beginFile")));
        let result = format!("{{ match {pending} {{ TsrCompletion::Normal(())=>{normal}, TsrCompletion::Return(value)=>Ok(value), TsrCompletion::Throw(error)=>Err(error), TsrCompletion::Break(_)|TsrCompletion::Continue(_)=>unreachable!(\"checked control cannot escape its callable\") }} }}");
        if suite_scope {
            let slot = fun.bindings.iter().position(|binding| self.test_registry_type(binding.ty)).expect("registry capture");
            format!("tsrust_testing_scope({}, |value{slot}| {result})", self.binding(fun, slot, Mode::Mutate))
        } else { result }
    }

    #[inline(never)]
    #[cfg(test)]
    fn completion_body(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>)->String {
        self.completion_body_frame(fun,body,id,error_name,contract,runtime,false)
    }

    #[inline(never)]
    fn completion_body_frame(&self,fun:&CheckedFunction,body:&[Stmt],id:usize,error_name:&str,contract:crate::error_effects::FailureSet,runtime:Option<Type>,initialize_parameters:bool)->String {
        let mut output=String::new();let mut pending=Vec::new();
        self.schedule_completion_body(&mut output,fun,body,id,error_name.into(),contract,runtime,initialize_parameters,&mut pending);
        self.run_statement_work(&mut output,fun,pending);output
    }

    fn block(&self, output: &mut String, fun: &CheckedFunction, body: &[Stmt], depth: usize) {
        self.control_block(output, fun, body, depth, None);
    }

    fn control_block(
        &self,
        output: &mut String,
        fun: &CheckedFunction,
        body: &[Stmt],
        depth: usize,
        loop_label: Option<usize>,
    ) {
        self.control_block_epochs(output, fun, body, depth, loop_label, None);
    }

    fn control_block_epochs(&self, output: &mut String, fun: &CheckedFunction, body: &[Stmt], depth: usize, loop_label: Option<usize>, declaring_body: Option<Declarations<'_, '_>>) {
        use std::rc::Rc;
        use statement_work::Work;
        let declarations = Rc::new(declaring_body.unwrap_or(Declarations::Body(body)).iter().collect::<Vec<_>>());
        self.run_statement_work(output, fun, vec![Work::Body(body, declarations, depth, loop_label)]);
    }

    fn run_statement_work<'a, 's: 'a>(&'s self, output: &mut String, fun: &'a CheckedFunction, mut pending: Vec<statement_work::Work<'a, 's>>) {
        use std::rc::Rc;
        use statement_work::Work;
        let mut fun=fun;
        let mut functions=Vec::new();
        let mut buffers=Vec::new();
        let mut values=Vec::new();
        while let Some(work) = pending.pop() {
            let (stmt, body, visible, depth, loop_label) = match work {
                Work::Body(body, declarations, depth, label) => {
                    pending.extend(body.iter().rev().map(|stmt| Work::Statement(stmt, body, declarations.clone(), depth, label)));
                    continue;
                },
                Work::Text(code) => { output.push_str(&code); continue; },
                Work::Capture => { buffers.push(std::mem::take(output)); continue; },
                Work::Captured => { values.push(std::mem::replace(output,buffers.pop().expect("completion output buffer"))); continue; },
                Work::Resume(resume) => { resume(output,&mut pending,&mut values); continue; },
                Work::Function(function) => {functions.push(fun);fun=function;continue;},
                Work::RestoreFunction => {fun=functions.pop().expect("enclosing emission function");continue;},
                Work::Region(scope) => { drop(scope); continue; },
                Work::Scope(scope) => { drop(scope); continue; },
                Work::StorageScope(scope) => { drop(scope); continue; },
                Work::Storage(work) => { self.resume_storage_statement(output, fun, work, &mut pending); continue; },
                Work::Iteration(work) => { self.resume_map_iteration(output, fun, work, &mut pending); continue; },
                Work::Post(stmt, depth) => { self.map_post_entry_epochs(output, fun, stmt.span, depth); continue; },
                Work::BlockReturn(stmt, places) => {
                    let StmtKind::Block(inner) = &stmt.kind else { unreachable!() };

                    let values = places.iter().map(|place| self.map_branch_exit_value(fun, inner, place)).collect::<Vec<_>>().join(",");
                    let comma = if places.is_empty() { "" } else { "," };
                    let _ = writeln!(output, "    ({values}{comma})\n}};");
                    let boundary = places.iter().filter_map(|place| self.map_storage_transfer_at_exit(fun, inner, place)).map(|span| span.end).max().unwrap_or(stmt.span.end).max(stmt.span.end);
                    self.exception_regions.renew_storage(&places, boundary);
                    continue;
                },
                Work::LoopTail(stmt, depth) => {
                    let StmtKind::Loop { condition, update, post_test, tail_reachable, .. } = &stmt.kind else { unreachable!() };

                    let _ = writeln!(output, "    }}");
                    pending.push(Work::Post(stmt, depth));
                    pending.push(Work::Text(format!("}}\n")));
                    if *tail_reachable {
                        if *post_test && let Some(condition) = condition { pending.push(Work::LoopTest(condition, stmt.span.start)); }
                        pending.push(Work::Body(update, Rc::new(update.iter().collect()), depth + 1, None));
                    }
                    continue;
                },
                Work::LoopTest(condition, label) => {

                    let _ = writeln!(output, "    if !({}) {{ break 'loop_{label}; }}", self.expr(fun, condition, Mode::Copy));
                    continue;
                },
                Work::Statement(stmt, body, visible, depth, label) => (stmt, body, visible, depth, label),
            };
            let declarations = Declarations::Visible(&visible);

            for epoch in fun.storage_exits.array_epochs(stmt.span).iter().filter(|epoch| epoch.before) {
                let _ = writeln!(output, "{}", self.array_storage_epoch_code(fun, epoch));
            }
            let shared_block_span=matches!(&stmt.kind,StmtKind::Block(inner) if inner.iter().any(|item|item.span==stmt.span));
            for epoch in fun.storage_exits.map_entry_epochs(stmt.span).iter().filter(|epoch|epoch.before && !shared_block_span) {
                let _=writeln!(output,"{}",self.map_entry_epoch_code(fun,epoch));
                self.exception_regions.renew_storage(&[format!("value{}",epoch.place.binding)],stmt.span.start);
            }
            if let Some(code)=self.map_iteration_control_code(fun,stmt){let _=writeln!(output,"{code}");continue;}
            if self.schedule_storage_block(output,fun,stmt,visible.clone(),depth,loop_label,&mut pending){continue;}
            if self.schedule_map_iteration(output,fun,stmt,visible.clone(),depth,&mut pending){continue;}
            if self.schedule_empty_map_loop(output, fun, stmt, visible.clone(), depth, &mut pending) { continue; }
            if self.schedule_map_try_storage(fun, stmt, declarations, &mut pending) { continue; }
            if self.schedule_map_try_prefix(output, fun, stmt, declarations, depth, &mut pending) { continue; }
            if let Some(reset) = self.local_map_clear_epoch(fun, Declarations::Body(body), stmt)
                .or_else(|| self.local_map_clear_epoch(fun, declarations, stmt))
                .or_else(|| self.map_clear_block_epoch(fun, declarations, stmt)) {
                let _ = writeln!(output, "{reset}");
                continue;
            }
            if let StmtKind::Block(inner) = &stmt.kind {
                let places = if stmt.labels.is_empty() { self.map_block_return_places(fun, declarations, inner) } else { Vec::new() };
                if !places.is_empty() {
                    let pattern = places.iter().map(|place| format!("mut {place}")).collect::<Vec<_>>().join(",");
                    let comma = if places.is_empty() { "" } else { "," };
                    let _ = writeln!(output, "let ({pattern}{comma}) = {{");
                    pending.push(Work::BlockReturn(stmt, places));
                    pending.push(Work::Body(inner, visible.clone(), depth + 1, loop_label));
                    continue;
                }
                let body = inner;
                if !stmt.labels.is_empty() { pending.push(Work::Scope(self.control_targets.enter(ControlTarget::Block(stmt.span.start)))); }
                if stmt.labels.is_empty() {let _ = writeln!(output, "{{");}
                else {let _ = writeln!(output,"'block_{}: {{",stmt.span.start); }
                // A counted for loop is represented by a lexical block. Keep
                // visible declarations available to its storage-epoch proof.
                let mut visible = declarations.iter().take_while(|item| item.span != stmt.span).collect::<Vec<_>>();
                visible.extend(body.iter());
                pending.push(Work::Text(format!("}}\n")));
                pending.push(Work::Body(body, Rc::new(visible), depth + 1, loop_label));
                continue;
            }
            if let StmtKind::Loop {
                condition,
                body,
                post_test,
                ..
            } = &stmt.kind
            {
                let label = stmt.span.start;
                pending.push(Work::Scope(self.control_targets.enter(ControlTarget::Tail(label))));
                let _ = writeln!(output, "'loop_{label}: loop {{");
                if !post_test && let Some(condition) = condition {
                    let _ = writeln!(
                        output,
                        "    if !({}) {{ break 'loop_{label}; }}",
                        self.expr(fun, condition, Mode::Copy)
                    );
                }
                let _ = writeln!(output, "    'body_{label}: {{");
                pending.push(Work::LoopTail(stmt, depth));
                pending.push(Work::Body(body, Rc::new(body.iter().collect()), depth + 2, Some(label)));
                continue;
            }
            if let StmtKind::ForOf { body, .. } = &stmt.kind {
                pending.push(Work::Post(stmt, depth));
                self.schedule_for_of(output, fun, stmt, Rc::new(body.iter().collect()), depth, &[], &mut pending);
                continue;
            }
            if let StmtKind::While { condition, body } = &stmt.kind {
                pending.push(Work::Scope(self.control_targets.enter(ControlTarget::Loop(stmt.span.start))));
                let _ = writeln!(
                    output,
                    "'loop_{}: while {} {{",
                    stmt.span.start, self.expr(fun, condition, Mode::Copy)
                );
                pending.push(Work::Post(stmt, depth));
                pending.push(Work::Text(format!("}}\n")));
                pending.push(Work::Body(body, Rc::new(body.iter().collect()), depth + 1, None));
                continue;
            }
            if let StmtKind::If {
                condition,
                then_body,
                else_body,
            } = &stmt.kind
            {
                if self.schedule_storage_if(output, fun, stmt, visible.clone(), depth, loop_label, &mut pending) { continue; }
                let _ = writeln!(
                    output,
                    "if {} {{",
                    self.expr(fun, condition, Mode::Copy)
                );
                pending.push(Work::Post(stmt, depth));
                pending.push(Work::Text(format!("}}\n")));
                pending.push(Work::Body(else_body, Rc::new(else_body.iter().collect()), depth + 1, loop_label));
                pending.push(Work::Text(format!("}} else {{\n")));
                pending.push(Work::Body(then_body, Rc::new(then_body.iter().collect()), depth + 1, loop_label));
                continue;
            }
            if let Some(code) = self.array_result_write_code(fun, stmt) {
                let _ = writeln!(output, "{code}");
                self.map_post_entry_epochs(output, fun, stmt.span, depth);
                continue;
            }
            if let StmtKind::Try(region)=&stmt.kind {
                pending.push(Work::Post(stmt,depth));
                pending.push(Work::Text("\n".into()));
                self.schedule_nested_completion(output,fun,stmt,region,&mut pending);
                continue;
            }
            let code = match &stmt.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Try(_) => unreachable!("exception region scheduled above"),
                StmtKind::Throw(value) => {
                    let code=self.expr(fun,value,Mode::Consume);
                    let unions=crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                    self.exception_regions.throw_value(&code,value.ty.expect("checked throw type"),&unions,stmt.span).expect("checked exception transport")
                },
                StmtKind::If { .. }
                | StmtKind::While { .. }
                | StmtKind::ForOf { .. }
                | StmtKind::Block(_)
                | StmtKind::Loop { .. } => {
                    unreachable!("control flow emitted above")
                }
                StmtKind::Break => match stmt.control_target {
                    Some(target)=>self.exception_regions.control_at(target,false,stmt.span),
                    None=>loop_label.map_or_else(||"break;".into(),|label|format!("break 'loop_{label};")),
                },
                StmtKind::Continue => match stmt.control_target {
                    Some(target)=>self.exception_regions.control_at(target,true,stmt.span),
                    None=>loop_label.map_or_else(||"continue;".into(),|label|format!("break 'body_{label};")),
                },
                StmtKind::Let { value, binding, .. } => {
                    let index = binding.expect("checked binding");
                    let b = &fun.bindings[index];
                    if self.finally_moves.contains(fun,index) {
                        let initialized=if value.is_deferred(){"None".to_owned()}else{format!("Some({})",self.expr(fun,value,Mode::Consume))};
                        let _=writeln!(output,"let mut value{index}={initialized};");
                        continue;
                    }
                    if value.is_deferred() {
                        if let Some(declarations)=self.owned_record_deferred(fun,index){let _=writeln!(output,"{declarations}");continue;}
                        // Assigned later; definite assignment was checked.
                        let _ = writeln!(output, "let mut value{index}: {};", self.local_callback_environment_type(b.ty));
                        continue;
                    }
                    let scoped_alias =
                        matches!(b.ty, Type::BorrowedUnion(_)) || b.borrowed_place.is_some() || b.mutable_place;
                    // Keep selected fresh owners in the alias binding's scope,
                    // rather than draining them into the initializer block.
                    let expression_depth = self.expression_depth.get();
                    if scoped_alias {
                        self.expression_depth.set(expression_depth + 1);
                    }
                    let initializer = self.binding_shape(index).as_ref().map_or_else(
                        || {
                            self.expr(
                                fun,
                                value,
                                if b.mutable_place {
                                    Mode::Mutate
                                } else if scoped_alias {
                                    Mode::Read
                                } else {
                                    Mode::Consume
                                },
                            )
                        },
                        |shape| self.fixed_literal(fun, value, shape),
                    );
                    self.expression_depth.set(expression_depth);
                    if let Some(declaration)=self.owned_record_declaration(fun,index,&initializer){let _=writeln!(output,"{declaration}");continue;}
                    format!(
                        "let {}value{index}: {} = {};",
                        if b.mutable || self.finally_moves.partial_root(fun,index) { "mut " } else { "" },
                        if matches!(b.ty, Type::Union(id) if self.program.unions[id.index()].members.iter().any(|member| matches!(member,Type::Callback(_))))
                            && Self::absent_callable_initializer(value)
                        {
                            format!("Union{}<TsrAbsentCallable>", b.ty.union().unwrap().index())
                        } else if matches!(b.ty, Type::Callback(_)) {
                            "_".into()
                        } else if b.mutable_place {
                            "_".into()
                        } else if b.borrowed_place.is_some() {
                            if matches!(b.ty,Type::Record(id) if self.borrowing_records.contains(&self.program.scoped_records.declaration(id))) {
                                // A projection can be a record view rather than its owned layout.
                                "&_".into()
                            }else{format!("&{}", self.local_callback_environment_type(b.ty))}
                        } else {
                            self.binding_shape(index).as_ref().map_or_else(
                                || self.local_borrowed_family_type(fun,b.ty).unwrap_or_else(||self.local_callback_environment_type(b.ty)),
                                |shape| self.shape_type(shape),
                            )
                        },
                        initializer
                    )
                }
                StmtKind::Assign {
                    target,
                    value,
                    operator: Some((op, span)),
                } => format!(
                    "{};",
                    self.compound_assignment(fun, target, value, *op, *span)
                ),
                StmtKind::Assign {target,value,operator:None} if target.binding.is_some_and(|id|self.restored_capture(fun,id)) => {
                    format!("{}value{} = Some({});",if fun.bindings[target.binding.unwrap()].capture_reference {"*"}else{""},target.binding.unwrap(),self.expr(fun,value,Mode::Consume))
                }
                StmtKind::Assign { target, value, .. }
                    if matches!(
                        target.ty,
                        Some(
                            Type::Int | Type::Number | Type::Boolean | Type::Null | Type::Undefined
                        )
                    ) =>
                {
                    format!("{};", self.scalar_assignment(fun, target, value))
                }
                StmtKind::Assign { target, value, .. }
                    if !matches!(target.kind, ExprKind::Name(_)) =>
                {
                    self.storage_assignment(fun, target, value)
                }
                StmtKind::Assign { target, value, .. }
                    if matches!(target.kind,ExprKind::Name(_)) && target.binding.is_some_and(|id|self.owned_record_layout(fun,id).is_some()) => {
                        self.owned_record_assignment(fun,target.binding.unwrap(),&self.expr(fun,value,Mode::Consume)).unwrap()
                    }
                StmtKind::Assign { target, value, .. } => format!(
                    "{} = {};",
                    if let ExprKind::Name(_) = target.kind {
                        let id = target.binding.expect("checked assignment binding");
                        format!(
                            "{}value{id}",
                            if fun.bindings[id].capture_reference {
                                "*"
                            } else {
                                ""
                            }
                        )
                    } else {
                        self.expr(fun, target, Mode::Consume)
                    },
                    self.expr(fun, value, Mode::Consume)
                ),
                StmtKind::Return(value) => {
                    let value=value.as_ref().map(|value|self.expr(fun,value,Mode::Consume));
                    if self.exception_regions.current_id().is_some() {
                        self.exception_regions.return_value(value.as_deref().unwrap_or("()"),stmt.span)
                    }else{
                        value.map_or_else(||"return;".into(),|value|format!("return {value};"))
                    }
                },
                StmtKind::Expr(value) if fun.storage_exits.comma_operand(value.span) => self.comma_discard_code(fun, value),
                StmtKind::Expr(value) => {
                    let code = self.expr(
                        fun,
                        value,
                        if matches!(value.ty, Some(Type::BorrowedUnion(_))) {
                            Mode::Read
                        } else {
                            Mode::Consume
                        },
                    );
                    if value.ty == Some(Type::Void) {
                        format!("{code};")
                    } else if value.ty.is_some_and(Type::is_copy) {
                        format!("let _ = {code};")
                    } else {
                        format!("std::mem::drop({code});")
                    }
                }
            };
            let declarations = self
                .conditional_temporaries
                .borrow_mut()
                .drain(..)
                .collect::<Vec<_>>()
                .join(" ");
            if declarations.is_empty() {
                let _ = writeln!(output, "{code}");
            } else if matches!(&stmt.kind, StmtKind::Let { binding: Some(id), .. }
                if fun.bindings[*id].borrowed_place.is_some()
                    || matches!(fun.bindings[*id].ty, Type::BorrowedUnion(_)))
            {
                let _ = writeln!(output, "{declarations} {code}");
            } else {
                let _ = writeln!(output, "{{ {declarations} {code} }}");
            }
            self.map_post_entry_epochs(output,fun,stmt.span,depth);
        }
    }

    fn temporary_storage(mut expr: &Expr) -> bool {
        loop {
            match &expr.kind {
                ExprKind::Field(base,_) if matches!(base.ty,Some(Type::Union(_)))=>return true,
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
        !matches!(expr.kind, ExprKind::Name(_)) && !matches!(&expr.kind,ExprKind::SpreadValue(selected) if selected.selection==SpreadSelection::Receiver)
    }

    fn borrowed_place_expression(fun: &CheckedFunction, expr: &Expr) -> bool {
        if matches!(expr.ty,Some(Type::BorrowedUnion(_))) {return true;}
        match &expr.kind {
            ExprKind::Field(base,_) if matches!(base.ty,Some(Type::Union(_)))=>true,
            ExprKind::Project{value,..}|ExprKind::Unary{op:UnaryOp::NonNull,value,..}=>Self::borrowed_place_expression(fun,value),
            ExprKind::Sequence(values)=>values.last().is_some_and(|last|Self::borrowed_place_expression(fun,last)),
            ExprKind::TypeAssertion { value, .. } => Self::borrowed_place_expression(fun, value),
            ExprKind::Optional { body, .. } => Self::borrowed_place_expression(fun, body),
            ExprKind::CoalesceAssign { .. } => true,
            ExprKind::Selected { owned, .. } => !owned,
            ExprKind::SpreadValue(selected) => !selected.owned,
            ExprKind::Name(_) => expr.binding.is_some_and(|id| {
                fun.bindings[id].borrowed_place.is_some() || fun.bindings[id].borrowed_element.is_some()
            }),
            // A string position is a new string, not a place in the text.
            ExprKind::Index(base, _) if base.ty.map(Type::scalar_base) == Some(Type::String) => false,
            ExprKind::Field(base, _) | ExprKind::Index(base, _) => {
                Self::borrowed_place_expression(fun, base)
            }
            _ => false,
        }
    }

    fn temporary_storage_access(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        let (base, index) = match &expr.kind {
            ExprKind::Field(base, _) => (base.as_ref(), None),
            ExprKind::Index(base, index) => (base.as_ref(), Some(index.as_ref())),
            _ => unreachable!("temporary selection"),
        };
        let borrowed = Self::borrowed_place_expression(fun, base);
        let fixed = if index.is_some() && !borrowed {
            self.fixed.temporary_shape(self.program, fun, base)
        } else {
            None
        };
        let array = fixed.map_or_else(
            || self.expr(fun, base, if borrowed { Mode::Read } else { Mode::Consume }),
            |shape| self.fixed_literal(fun, base, shape),
        );
        let index = index.map_or_else(String::new, |index| self.expr(fun, index, Mode::Copy));
        self.finish_temporary_storage_access(expr, mode, borrowed, fixed, array, index)
    }

    fn finish_temporary_storage_access(
        &self,
        expr: &Expr,
        mode: Mode,
        borrowed: bool,
        fixed: Option<&crate::fixed_storage::Shape>,
        array: String,
        index: String,
    ) -> String {
        let owned = match &expr.kind {
            ExprKind::Field(base, name) => {
                let Type::Record(record) = base.ty.expect("checked temporary record") else {
                    unreachable!()
                };
                format!("({array}).field{}", self.fields[record][name.as_str()])
            }
            ExprKind::Index(base, _) => {
                if borrowed {
                    let failure = Self::array_failure(expr.span);
                    let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, expr.span)
                        .unwrap_or_else(|| format!("panic!({failure:?})"));
                    let selected = if expr.ty.is_some_and(Type::is_copy) {
                        "*"
                    } else {
                        ""
                    };
                    return format!(
                        "{{ let temporary_array = {array}; let temporary_index: i64 = {index}; let temporary_index = match usize::try_from(temporary_index) {{ Ok(index) => index, Err(_) => {failed} }}; {selected}(match temporary_array.get(temporary_index) {{ Some(value) => value, None => {failed} }}) }}"
                    );
                }
                let annotation = fixed.map_or_else(String::new, |shape| format!(": {}", self.shape_type(shape)));
                let failure = Self::array_failure(expr.span);
                let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, expr.span)
                    .unwrap_or_else(|| format!("panic!({failure:?})"));
                let ty = base.ty.expect("checked temporary array");
                let vector = fixed.is_none()
                    && !numeric_array(ty)
                    && !inline_array(ty)
                    && !matches!(ty, Type::Array(ArrayElement::Record(record, 0)) if self.small_record(record) || self.record_runtime.recursive(record));
                let selected = if vector {
                    r#"temporary_array.into_iter().nth(temporary_index).expect("checked temporary array index")"#.to_owned()
                } else if expr.ty.is_some_and(Type::is_copy) {
                    r#"*temporary_array.get(temporary_index).expect("checked temporary array index")"#.to_owned()
                } else {
                    "temporary_array.remove(temporary_index)".to_owned()
                };
                let mutable = if !vector && !expr.ty.is_some_and(Type::is_copy) {
                    "mut "
                } else {
                    ""
                };
                format!(
                    "{{ let {mutable}temporary_array{annotation} = {array}; let temporary_index: i64 = {index}; let temporary_index = match usize::try_from(temporary_index) {{ Ok(index) => index, Err(_) => {failed} }}; if temporary_index >= temporary_array.len() {{ {failed}; }} {selected} }}"
                )
            }
            _ => unreachable!("temporary selection"),
        };
        match mode {
            Mode::Read => format!("&({owned})"),
            Mode::Mutate => format!("&mut ({owned})"),
            _ => owned,
        }
    }

    fn storage_steps<'a>(&self, expr: &'a Expr) -> (&'a Expr, Vec<StorageStep<'a>>) {
        let mut steps = Vec::new();
        let mut prefixes = Vec::new();
        let mut root = expr;
        let mut assertion = None;
        loop {
            match &root.kind {
                ExprKind::Sequence(values) => {
                    let (last, leading) = values.split_last().expect("nonempty sequence");
                    prefixes.extend(leading.iter().map(StorageStep::Discard));
                    root = last;
                    assertion = None;
                }
            ExprKind::Unary { op: UnaryOp::NonNull, value, operator } => {
                    assertion = Some(*operator);
                    root = value;
                }
                ExprKind::Index(base, index) => {
                    steps.push(StorageStep::Index(index, root.span));
                    root = base;
                }
                ExprKind::Project {
                    value,
                    union,
                    narrowed,
                } => {
                    if root.ty != Some(Type::Union(*union)) {
                        let variant = self.program.unions[union.index()]
                            .members
                            .iter()
                            .position(|ty| ty == narrowed)
                            .expect("proven payload alternative");
                        steps.push(StorageStep::Union(*union, variant, assertion.take()));
                    }
                    root = value;
                }
                ExprKind::Field(base, name) => {
                    let Type::Record(record) = base.ty.expect("checked field receiver") else {
                        unreachable!()
                    };
                    steps.push(StorageStep::Field(self.fields[record][name.as_str()]));
                    root = base;
                }
                _ => break,
            }
        }
        prefixes.extend(steps.into_iter().rev());
        (root, prefixes)
    }

    /// Shared selections keep each parent stable through later operands. The
    /// exclusive path is reconstructed from saved indices after those operands.
    fn storage_selection(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        assignment: bool,
    ) -> StorageSelection {
        self.storage_selection_with(fun, expr, assignment, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn storage_selection_with(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        assignment: bool,
        render: &mut impl FnMut(&Expr, Mode, bool) -> String,
    ) -> StorageSelection {
        let (root, steps) = self.storage_steps(expr);
        if let ExprKind::SpreadValue(selected)=&root.kind && steps.is_empty() && selected.selection==SpreadSelection::Receiver {
            let name=selected.selection.native_name(selected.slot);
            return StorageSelection {reselected_shared:format!("&(*{name})"),place:format!("*{name}"),setup:String::new(),shared:format!("&(*{name})"),exclusive:format!("&mut (*{name})"),final_conversion:String::new()};
        }
        let id = root.binding.expect("storage path has a named root");
        let deferred = if assignment {
            let final_step = steps.len() - 1;
            // A record field immediately after an index retains the established
            // replacement-before-final-bounds-check contract. Other fields keep
            // all earlier parents selected before evaluating the replacement.
            if matches!(steps[final_step], StorageStep::Field(_))
                && final_step > 0
                && matches!(steps[final_step - 1], StorageStep::Index(_, _))
            {
                final_step - 1
            } else {
                final_step
            }
        } else {
            steps.len()
        };
        // Build places without borrowing whole parent records. A sibling field
        // may already have moved; references begin only at the selected field or
        // an indexed parent that must stay stable through later operands.
        let binding = &fun.bindings[id];
        let view_root = matches!(binding.ty, Type::BorrowedUnion(_))
            || (matches!(binding.ty,Type::Union(id) if !id.copy)
                && binding.parameter
                && binding.mode == Mode::Read
                && !binding.capture_reference);
        let referenced = !view_root
            && ((binding.parameter && matches!(binding.mode, Mode::Read | Mode::Mutate)
                && (!(binding.omission.is_omittable() || (binding.mode==Mode::Mutate && self.multiple_data_union(binding.ty))) || (binding.initializer.is_some() && !matches!(binding.ty,Type::Union(_)|Type::Callback(_)))))
                || binding.capture_reference
                || binding.borrowed_element.is_some()
                || binding.borrowed_place.is_some());
        let mut leaf_path=Vec::new();let mut entry_leaf=None;
        for (position,step) in steps.iter().enumerate() {
            let StorageStep::Field(field)=step else{break;};leaf_path.push(*field);
            if let Some(place)=self.owned_record_leaf_place(fun,id,&leaf_path).or_else(||self.record_parameter_leaf_place(fun,id,&leaf_path)){entry_leaf=Some((position+1,place));break;}
        }
        let skipped=entry_leaf.as_ref().map_or(0,|(count,_)|*count);
        let mut shared = entry_leaf.map_or_else(||format!("{}value{id}", if referenced { "*" } else { "" }),|(_,place)|place);
        let mut exclusive = shared.clone();
        if self.finally_moves.contains(fun,id) {
            shared=format!("*value{id}.as_ref().expect(\"initialized owned slot\")");
            exclusive=format!("*value{id}.as_mut().expect(\"initialized owned slot\")");
        }
        let mut reselected = shared.clone();
        let mut result = StorageSelection {
            reselected_shared: String::new(),
            place: String::new(),
            setup: String::new(),
            shared: String::new(),
            exclusive: String::new(),
            final_conversion: String::new(),
        };
        for (position, step) in steps.iter().enumerate().skip(skipped) {
            match step {
                StorageStep::Discard(prefix) => {
                    let code=render(prefix, Mode::Read, true);
                    result.setup.push_str(&code);
                }
                StorageStep::Field(field) => {
                    shared = format!("({shared}).field{field}");
                    exclusive = format!("({exclusive}).field{field}");
                    reselected = format!("({reselected}).field{field}");
                }
                StorageStep::Union(union, variant, assertion) => {
                    let failure = assertion.map_or_else(|| "checked nullable guard".to_owned(), |span| format!("Non-null assertion failed: value is null or undefined at source bytes {}..{}", span.start, span.end));
                    let failed = assertion.and_then(|span| self.runtime_failure_exit("TypeError", "non-null", &failure, span))
                        .unwrap_or_else(|| format!("panic!({failure:?})"));
                    if view_root && position == 0 {
                        shared = format!(
                            "*(match {shared} {{ ViewUnion{}::V{variant}(payload)=>payload,_=>{failed} }})",
                            union.index()
                        );
                        exclusive = shared.clone();
                        reselected = shared.clone();
                        continue;
                    }
                    let boxed=self.boxed_union_member(*union,self.program.unions[union.index()].members[*variant]);
                    let read_payload=if boxed {"&**payload"}else{"payload"};
                    let write_payload=if boxed {"&mut **payload"}else{"payload"};
                    shared = format!(
                        "*(match &({shared}) {{ Union{}::V{variant}(payload) => {read_payload}, _ => {failed} }})",
                        union.index()
                    );
                    reselected = format!(
                        "*(match &({reselected}) {{ Union{}::V{variant}(payload) => {read_payload}, _ => {failed} }})",
                        union.index()
                    );
                    exclusive = format!(
                        "*(match &mut ({exclusive}) {{ Union{}::V{variant}(payload) => {write_payload}, _ => {failed} }})",
                        union.index()
                    );
                }
                StorageStep::Index(index, span) => {
                    let failure = Self::array_failure(*span);
                    let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, *span)
                        .unwrap_or_else(|| format!("panic!({failure:?})"));
                    let _ = write!(
                        result.setup,
                        "let storage_index{position} = {}; ",
                        render(index, Mode::Copy, false)
                    );
                    let conversion = format!(
                        "let storage_index{position} = match usize::try_from(storage_index{position}) {{ Ok(index) => index, Err(_) => {failed} }}; "
                    );
                    if position == deferred {
                        result.final_conversion = conversion;
                    } else {
                        result.setup.push_str(&conversion);
                        let _ = write!(
                            result.setup,
                            "let storage_receiver{position} = match ({}).get(storage_index{position}) {{ Some(value) => value, None => {failed} }}; ",
                            shared
                        );
                        shared = format!("*storage_receiver{position}");
                    }
                    reselected =
                        format!("*(match ({reselected}).get(storage_index{position}) {{ Some(value) => value, None => {failed} }})");
                    exclusive = format!(
                        "*(match ({exclusive}).get_mut(storage_index{position}) {{ Some(value) => value, None => {failed} }})"
                    );
                }
            }
        }
        result.reselected_shared = format!("&({reselected})");
        result.place = exclusive.clone();
        result.shared = format!("&({shared})");
        result.exclusive = format!("&mut ({exclusive})");
        result
    }

    fn storage_assignment(&self, fun: &CheckedFunction, target: &Expr, value: &Expr) -> String {
        self.storage_assignment_with(fun, target, value, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn storage_assignment_with(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        let selection = self.storage_selection_with(fun, target, true, render);
        let code = render(value, Mode::Consume, false);
        let value=self.record_parameter_replacement(fun,target,value,&code).unwrap_or(code);
        format!(
            "{{ {}let storage_value = {value}; {}({}) = storage_value; }}",
            selection.setup, selection.final_conversion, selection.place
        )
    }

    #[inline(never)]
    fn checked_integer_result(&self, operation: &str, reason: &str, span: crate::Span) -> String {
        let message = format!("integer {reason} at source bytes {}..{}", span.start, span.end);
        let code = if reason.starts_with("shift count") { "shift" }
            else if reason.starts_with("exponent must") { "exponent" }
            else if reason.contains("division") || reason.contains("remainder") { "arithmetic" }
            else { "overflow" };
        if let Some(exit) = self.runtime_failure_exit("RangeError", code, &message, span) {
            format!("(match {operation} {{ Some(value) => value, None => {exit} }})")
        } else {
            format!("{operation}.expect({message:?})")
        }
    }

    fn checked_operator(op: BinaryOp) -> Option<(&'static str, &'static str)> {
        match op {
            BinaryOp::Add => Some(("checked_add", "addition overflow")),
            BinaryOp::Subtract => Some(("checked_sub", "subtraction overflow")),
            BinaryOp::Multiply => Some(("checked_mul", "multiplication overflow")),
            BinaryOp::Divide => Some(("checked_div", "division by zero or overflow")),
            BinaryOp::Remainder => Some(("checked_rem", "remainder by zero or overflow")),
            _ => None,
        }
    }

    fn update_expression(&self, fun: &CheckedFunction, target: &Expr, increment: bool, prefix: bool, span: crate::diagnostic::Span) -> String {
        self.update_expression_with(fun, target, increment, prefix, span, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn update_expression_with(&self, fun: &CheckedFunction, target: &Expr, increment: bool, prefix: bool, span: crate::diagnostic::Span, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        let selection = self.storage_selection_with(fun, target, false, render);
        let target_type = target.ty.expect("checked update target");
        let result = if target_type == Type::Int {
            let (method, failure) = Self::checked_operator(if increment {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            })
            .expect("checked update arithmetic");
            self.checked_integer_result(&format!("update_old.{method}(1)"), failure, span)
        } else {
            format!("update_old {} 1.0", if increment { "+" } else { "-" })
        };
        format!(
            "{{ {}let update_old: {} = *({}); let update_new = {result}; ({}) = update_new; {} }}",
            selection.setup,
            self.native_type(target_type),
            selection.shared,
            selection.place,
            if prefix { "update_new" } else { "update_old" }
        )
    }

    fn coalesce_assignment(&self, fun: &CheckedFunction, expr: &Expr, target: &Expr, value: &Expr, branch: crate::syntax::NullishBranch) -> String {
        self.coalesce_assignment_with(fun, expr, target, value, branch, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn coalesce_assignment_with(&self, fun: &CheckedFunction, expr: &Expr, target: &Expr, value: &Expr, branch: crate::syntax::NullishBranch, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        let selection = self.storage_selection_with(fun, target, false, render);
        let physical = target.ty.expect("checked assignment storage");
        let logical = expr.ty.expect("checked assignment result").owning();
        let absent = if let Some(id) = physical.union() {
            let patterns = self.program.unions[id.index()]
                .members
                .iter()
                .enumerate()
                .filter(|(_, ty)| matches!(ty, Type::Null | Type::Undefined))
                .map(|(i, _)| format!("Union{}::V{i}(_)", id.index()))
                .collect::<Vec<_>>();
            if patterns.is_empty() {
                "false".to_owned()
            } else {
                format!("matches!({}, {})", selection.shared, patterns.join(" | "))
            }
        } else {
            matches!(physical, Type::Null | Type::Undefined).to_string()
        };
        let commit = if branch == crate::syntax::NullishBranch::Present {
            String::new()
        } else {
            let actual = value.ty.expect("checked replacement").owning();
            let code = render(value, Mode::Consume, false);
            let stored = if let Some(to) = physical.union() {
                if let Some(from) = actual.union() {
                    if from == to {
                        code
                    } else {
                        let arms = self.program.unions[from.index()]
                            .members
                            .iter()
                            .enumerate()
                            .map(|(i, ty)| {
                                let j = self.program.unions[to.index()]
                                    .members
                                    .iter()
                                    .position(|m| m == ty)
                                    .expect("assignment subset");
                                format!(
                                    "Union{}::V{i}(payload)=>Union{}::V{j}({})",
                                    from.index(),to.index(),self.union_transfer_payload(from,to,*ty,"payload")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("match {code}{{{arms}}}")
                    }
                } else {
                    let j = self.program.unions[to.index()]
                        .members
                        .iter()
                        .position(|m| {
                            *m == actual
                                || matches!(m, Type::Callback(_))
                                    && matches!(actual, Type::Closure(_) | Type::Callback(_))
                        })
                        .expect("assignment member");
                    format!("Union{}::V{j}({})", to.index(),self.union_store_payload(to,actual,&code))
                }
            } else {
                code
            };
            if branch == crate::syntax::NullishBranch::Absent {
                format!(
                    "let _nullish_absent={absent}; let nullish_assigned={stored}; ({})=nullish_assigned;",
                    selection.place
                )
            } else {
                format!(
                    "if {absent} {{let nullish_assigned={} ;({})=nullish_assigned;}}",
                    stored, selection.place
                )
            }
        };
        let result = if let Some(from) = physical.union() {
            let allowed = logical
                .union()
                .map(|id| self.program.unions[id.index()].members.clone())
                .unwrap_or_else(|| vec![logical]);
            let arms = self.program.unions[from.index()]
                .members
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let selected = if !allowed.contains(ty) {
                        "unreachable!(\"checked assignment result\")".to_owned()
                    } else if let Some(to) = logical.union() {
                        let j = self.program.unions[to.index()]
                            .members
                            .iter()
                            .position(|m| m == ty)
                            .unwrap();
                        format!(
                            "{}{}::V{j}({}payload)",
                            if logical.is_copy() {
                                "Union"
                            } else {
                                "ViewUnion"
                            },
                            to.index(),
                            if logical.is_copy() { "*" } else { "" }
                        )
                    } else if logical.is_copy() {
                        "*payload".into()
                    } else {
                        "payload".into()
                    };
                    format!("Union{}::V{i}(payload)=>{selected}", from.index())
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("match {}{{{arms}}}", selection.reselected_shared)
        } else if logical.is_copy() {
            format!("*({})", selection.reselected_shared)
        } else {
            selection.reselected_shared
        };
        format!("{{ {}{commit} {result} }}", selection.setup)
    }

    fn scalar_assignment(&self, fun: &CheckedFunction, target: &Expr, value: &Expr) -> String {
        self.scalar_assignment_with(fun, target, value, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn scalar_assignment_with(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        // Check every selected bound before RHS effects, without reading old data.
        // Saved indices reconstruct the destination after permitted RHS mutation.
        let selection = self.storage_selection_with(fun, target, false, render);
        let rhs = render(value, Mode::Copy, false);
        format!(
            "{{ {}let assigned_value: {} = {rhs}; ({}) = assigned_value; assigned_value }}",
            selection.setup,
            self.native_type(target.ty.expect("checked scalar assignment target")),
            selection.place,
        )
    }

    fn logical_assignment(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, and: bool) -> String {
        self.logical_assignment_with(fun, target, value, and, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn logical_assignment_with(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, and: bool, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        let selection = self.storage_selection_with(fun, target, false, render);
        let rhs = render(value, Mode::Copy, false);
        format!(
            "{{ {}let logical_old: bool = *({}); if {}logical_old {{ let logical_new: bool = {rhs}; ({}) = logical_new; logical_new }} else {{ logical_old }} }}",
            selection.setup,
            selection.shared,
            if and { "" } else { "!" },
            selection.place
        )
    }

    fn compound_assignment(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, op: BinaryOp, span: crate::diagnostic::Span) -> String {
        self.compound_assignment_with(fun, target, value, op, span, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn compound_assignment_with(&self, fun: &CheckedFunction, target: &Expr, value: &Expr, op: BinaryOp, span: crate::diagnostic::Span, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> String {
        // All bounds and the old target value precede the RHS. Saved indices
        // reconstruct the destination after shared reads have ended.
        let selection = self.storage_selection_with(fun, target, false, render);
        let target_type = target.ty.expect("checked compound target");
        let string = target_type == Type::String;
        let annotation = if string {
            "&String".to_owned()
        } else {
            self.native_type(target_type)
        };
        let old = if string {
            selection.shared
        } else {
            format!("*({})", selection.shared)
        };
        let right = render(value, if string { Mode::Read } else { Mode::Copy }, false);
        let result = if string {
            let failure = format!(
                "string addition capacity overflow at source bytes {}..{}",
                span.start, span.end
            );
            format!("tsr_string_add(compound_left, compound_right, {failure:?})")
        } else if target_type == Type::Number && op.bitwise_or_power() {
            Self::number_operation(op, "compound_left", "compound_right")
        } else if target_type == Type::Int && op.bitwise_or_power() {
            self.int_bitwise_or_power(op, "compound_left", "compound_right", span)
        } else if target_type == Type::Int {
            let (method, failure) =
                Self::checked_operator(op).expect("checked int compound operator");
            self.checked_integer_result(&format!("compound_left.{method}(compound_right)"), failure, span)
        } else {
            let operator = match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Remainder => "%",
                _ => unreachable!("checked arithmetic compound"),
            };
            format!("compound_left {operator} compound_right")
        };
        format!(
            "{{ {}let compound_left: {annotation} = {old}; let compound_right: {annotation} = {right}; let compound_result = {result}; ({}) = compound_result; {} }}",
            selection.setup,
            selection.place,
            if string { "" } else { "compound_result" }
        )
    }

    fn storage_access(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        self.storage_access_with(fun, expr, mode, &mut |operand, mode, discard| {
            if discard { self.comma_discard_code(fun, operand) } else { self.expr(fun, operand, mode) }
        })
    }

    fn storage_access_with(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        mode: Mode,
        render: &mut impl FnMut(&Expr, Mode, bool) -> String,
    ) -> String {
        let(root,steps)=self.storage_steps(expr);
        let path=steps.iter().map(|step|if let StorageStep::Field(field)=step{Some(*field)}else{None}).collect::<Option<Vec<_>>>();
        if let (Some(id),Some(path))=(root.binding,path) && let Some(value)=self.owned_record_value(fun,id,&path,mode){return value;}
        if matches!(expr.ty,Some(Type::Record(_))) {
            let(root,steps)=self.storage_steps(expr);
            let path=steps.iter().map(|step|if let StorageStep::Field(field)=step{Some(*field)}else{None}).collect::<Option<Vec<_>>>();
            if let (Some(id),Some(path))=(root.binding,path) && let Some(value)=self.record_parameter_borrowed_value(fun,id,&path,mode){return value;}
        }
        if mode == Mode::Consume && !expr.ty.is_some_and(Type::is_copy) {
            let (root, steps) = self.storage_steps(expr);
            let id = root.binding.expect("owned field root");
            let mut place = format!("value{id}");
            let mut owned_path = Some(Vec::new());
            let mut setup=String::new();
            for step in steps {
                match step {
                    StorageStep::Field(field) => {
                        let leaf = owned_path.as_mut().and_then(|path| {
                            path.push(field);
                            self.owned_record_leaf_place(fun, id, path)
                        });
                        place = leaf.unwrap_or_else(|| format!("({place}).field{field}"));
                    },
                    StorageStep::Discard(prefix) => {
                        let code=render(prefix, Mode::Read, true);
                        setup.push_str(&code);
                    }
                    StorageStep::Union(union,variant,assertion) => {
                        owned_path = None;
                        let member=self.program.unions[union.index()].members[variant];
                        let message=assertion.map_or_else(||"checked nullable guard".to_owned(),|span|format!("Non-null assertion failed: value is null or undefined at source bytes {}..{}",span.start,span.end));
                        let failure=assertion.and_then(|span|self.runtime_failure_exit("TypeError","non-null",&message,span)).unwrap_or_else(||format!("panic!({message:?})"));
                        let payload=self.union_move_payload(union,member,"payload");
                        place=format!("(match {place} {{Union{}::V{variant}(payload)=>{payload},_=>{failure}}})",union.index());
                    }
                    _ => unreachable!("indexed storage cannot move"),
                }
            }
            return if setup.is_empty(){place}else{format!("{{{setup}{place}}}")};
        }
        let selection = self.storage_selection_with(fun, expr, false, render);
        let value = if mode == Mode::Mutate {
            selection.exclusive
        } else if matches!(expr.ty, Some(Type::BorrowedUnion(_))) {
            format!("({}).tsr_view()", selection.shared)
        } else if expr.ty.is_some_and(Type::is_copy) {
            format!("*({})", selection.shared)
        } else {
            selection.shared
        };
        format!("{{ {}{value} }}", selection.setup)
    }

    fn array_range_mutation(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        fill: bool,
    ) -> String {
        let array_type = base.ty.expect("checked range-mutation receiver");
        let Type::Array(element) = array_type else {
            unreachable!("checked array")
        };
        let compact = numeric_array(array_type)
            || inline_array(array_type)
            || matches!(element, ArrayElement::Record(index, 0) if self.small_record(index));
        let mut operands = String::new();
        for (index, argument) in args.iter().enumerate() {
            let value = self.expr(
                fun,
                argument,
                if fill && index == 0 {
                    Mode::Consume
                } else {
                    Mode::Copy
                },
            );
            write!(operands, "let range_arg{index} = {value}; ").unwrap();
        }
        let end = if args.len() == 3 {
            "Some(range_arg2)"
        } else {
            "None::<i64>"
        };
        let normalization = if fill {
            let start = if args.len() >= 2 {
                "range_arg1"
            } else {
                "0_i64"
            };
            format!(
                "let range_len = range_array.len(); let range_start = tsr_search_start(range_len, {start}); let range_end = {end}.map_or(range_len, |end| tsr_search_start(range_len, end)).max(range_start); "
            )
        } else {
            format!(
                "let range_len = range_array.len(); let range_target = tsr_search_start(range_len, range_arg0); let range_start = tsr_search_start(range_len, range_arg1); let range_end = {end}.map_or(range_len, |end| tsr_search_start(range_len, end)).max(range_start); let range_count = (range_end - range_start).min(range_len - range_target); let range_end = range_start + range_count; "
            )
        };
        // Fixed primitive proofs use real arrays and retain their width. Owned
        // proofs conservatively fall back, as with reverse.
        let fixed = matches!(
            self.expression_shape(base),
            Some(crate::fixed_storage::Shape::ScalarArray { .. })
        );
        let operation = if fill {
            if compact && !fixed {
                "range_array.tsr_fill(range_arg0, range_start, range_end);".to_owned()
            } else {
                "range_array[range_start..range_end].fill(range_arg0);".to_owned()
            }
        } else if compact && !fixed {
            "range_array.tsr_copy_within(range_start, range_end, range_target);".to_owned()
        } else if element.ty().is_copy() {
            "range_array.copy_within(range_start..range_end, range_target);".to_owned()
        } else {
            "if range_start != range_end && range_start != range_target { let range_snapshot = range_array[range_start..range_end].to_vec(); for (destination, value) in range_array[range_target..range_target + range_count].iter_mut().zip(range_snapshot) { *destination = value; } }".to_owned()
        };
        if Self::temporary_storage(base) {
            let receiver = self.expr(fun, base, Mode::Mutate);
            return format!(
                "{{ let range_array = {receiver}; {operands}{normalization}{operation} }}"
            );
        }
        let selection = self.storage_selection(fun, base, false);
        format!(
            "{{ {}{operands}let range_array = {}; {normalization}{operation} }}",
            selection.setup, selection.exclusive
        )
    }

    fn array_to_spliced(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let array_type = base.ty.expect("checked toSpliced receiver");
        let Type::Array(element) = array_type else {
            unreachable!("checked array")
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let mut output = format!("{{ let splicing_array = {receiver}; ");
        for (index, argument) in args.iter().enumerate() {
            let value = self.expr(
                fun,
                argument,
                if index < 2 { Mode::Copy } else { Mode::Consume },
            );
            let argument_type = if index < 2 {
                "i64".to_owned()
            } else {
                self.native_type(element.ty())
            };
            write!(
                output,
                "let splicing_arg{index}: {argument_type} = {value}; "
            )
            .unwrap();
        }
        let start = if args.is_empty() {
            "0_i64"
        } else {
            "splicing_arg0"
        };
        write!(output, "let splicing_len = splicing_array.len(); let splicing_start = tsr_search_start(splicing_len, {start}); ").unwrap();
        let deleted = match args.len() {
            0 => "0".to_owned(),
            1 => "splicing_len - splicing_start".to_owned(),
            _ => "usize::try_from(splicing_arg1.max(0)).unwrap_or(usize::MAX).min(splicing_len - splicing_start)".to_owned(),
        };
        let inserted = args.len().saturating_sub(2);
        let failure = format!(
            "toSpliced capacity overflow at source bytes {}..{}",
            span.start, span.end
        );
        write!(output, "let splicing_deleted = {deleted}; let splicing_end = splicing_start + splicing_deleted; let splicing_capacity = (splicing_len - splicing_deleted).checked_add({inserted}).expect({failure:?}); let mut splicing_values: Vec<{}> = Vec::with_capacity(splicing_capacity); ", self.native_type(element.ty())).unwrap();
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let prefix = if literal {
            "splicing_array.values.iter().take(splicing_start).map(|value| String::Borrowed(value))"
        } else {
            "splicing_array.iter().take(splicing_start).cloned()"
        };
        let suffix = if literal {
            "splicing_array.values.iter().skip(splicing_end).map(|value| String::Borrowed(value))"
        } else {
            "splicing_array.iter().skip(splicing_end).cloned()"
        };
        // Skip deleted storage before cloning. Insertions move into the result.
        write!(output, "splicing_values.extend({prefix}); ").unwrap();
        for index in 2..args.len() {
            write!(output, "splicing_values.push(splicing_arg{index}); ").unwrap();
        }
        write!(output, "splicing_values.extend({suffix}); ").unwrap();
        let result = if numeric_array(array_type) {
            "NumericArray::Heap(splicing_values)".to_owned()
        } else if inline_array(array_type) {
            "Array::Heap(splicing_values)".to_owned()
        } else if let ArrayElement::Record(id, 0) = element {
            if self.record_runtime.recursive(id) {
                format!("RecordArray{id}(splicing_values)")
            } else if self.small_record(id) {
                "Array::Heap(splicing_values)".to_owned()
            } else {
                "splicing_values".to_owned()
            }
        } else {
            "splicing_values".to_owned()
        };
        write!(output, "{result} }}").unwrap();
        output
    }

    fn array_copying(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let array_type = base.ty.expect("checked copying receiver");
        let Type::Array(element) = array_type else {
            unreachable!("checked array")
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let mut output = format!("{{ let copying_array = {receiver}; ");
        let values = if args.is_empty() {
            if literal {
                "copying_array.values.iter().rev().map(|value| String::Borrowed(value)).collect()"
                    .to_owned()
            } else {
                "copying_array.iter().rev().cloned().collect()".to_owned()
            }
        } else {
            let index = self.expr(fun, &args[0], Mode::Copy);
            let replacement = self.expr(fun, &args[1], Mode::Consume);
            let failure = Self::array_failure(span);
            let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, span)
                .unwrap_or_else(|| format!("panic!({failure:?})"));
            // Bind both operands before validating the index. unsigned_abs and
            // checked_sub handle i64::MIN without clamping a failed index.
            write!(output, "let copying_from: i64 = {index}; let mut copying_replacement = Some({replacement}); let copying_index = match (if copying_from >= 0 {{ usize::try_from(copying_from).ok().filter(|index| *index < copying_array.len()) }} else {{ usize::try_from(copying_from.unsigned_abs()).ok().and_then(|distance| copying_array.len().checked_sub(distance)) }}) {{ Some(index) => index, None => {failed} }}; ").unwrap();
            let (iterator, clone) = if literal {
                ("copying_array.values.iter()", "String::Borrowed(value)")
            } else {
                ("copying_array.iter()", "value.clone()")
            };
            format!(
                "{iterator}.enumerate().map(|(index, value)| if index == copying_index {{ copying_replacement.take().expect(\"replacement used once\") }} else {{ {clone} }}).collect()"
            )
        };
        write!(
            output,
            "let copying_values: Vec<{}> = {values}; ",
            self.native_type(element.ty())
        )
        .unwrap();
        let result = if numeric_array(array_type) {
            "NumericArray::Heap(copying_values)".to_owned()
        } else if inline_array(array_type) {
            "Array::Heap(copying_values)".to_owned()
        } else if let ArrayElement::Record(id, 0) = element {
            if self.record_runtime.recursive(id) {
                format!("RecordArray{id}(copying_values)")
            } else if self.small_record(id) {
                "Array::Heap(copying_values)".to_owned()
            } else {
                "copying_values".to_owned()
            }
        } else {
            "copying_values".to_owned()
        };
        write!(output, "{result} }}").unwrap();
        output
    }

    fn array_callback_counted_method(&self, callback: Type, method: Option<TraversalMethod>) -> bool {
        method.is_some() && matches!(callback, Type::Closure(id) if matches!(self.program.closures[id].forward, Some(ClosureForward::RestAdapter { counted: true, .. })))
    }

    fn array_callback_supplied_count(&self, callback: Type, method: Option<TraversalMethod>, fixed: usize, supplied: usize) -> usize {
        let rest = match callback {
            Type::Closure(id) => {
                let closure = &self.program.closures[id];
                let (function, parameters) = method.map_or((&closure.function, &closure.parameters), |method| {
                    let selected = &closure.methods[method.id];
                    (&selected.function, &selected.parameters)
                });
                parameters.last().is_some_and(|&id| function.bindings[id].rest)
            }
            Type::Callback(id) => self.program.callback_contracts[id].params.last().is_some_and(|parameter| parameter.rest),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.last().is_some_and(|parameter| parameter.rest),
            _ => unreachable!("checked array callback"),
        };
        if rest { supplied } else { fixed }
    }

    /// Adapt the real array tuple to checked input contracts. Defaults run only
    /// inside the callback; positions beyond the real tuple receive undefined.
    fn array_callback_arguments(
        &self,
        callback: Type,
        method: Option<TraversalMethod>,
        arguments: Vec<String>,
        supplied: &[Type],
    ) -> Vec<String> {
        let parameters = match callback {
            Type::Closure(id) => {
                let closure = &self.program.closures[id];
                let (function, parameters) = method.map_or((&closure.function, &closure.parameters), |method| {
                    let selected = &closure.methods[method.id];
                    (&selected.function, &selected.parameters)
                });
                parameters.iter().map(|&parameter| ArgumentContract::from(&function.bindings[parameter])).collect::<Vec<_>>()
            }
            Type::Callback(id) => self.program.callback_contracts[id].params.clone(),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.clone(),
            _ => unreachable!("checked array callback"),
        };
        // Mutable callable elements use protected tuples without the source
        // array. Omitted fixed slots receive undefined rather than that array.
        let supplied = if method.is_some_and(|method| method.access == Mode::Mutate) && supplied.len() > 2 {
            &supplied[..supplied.len() - 1]
        } else {
            supplied
        };
        let arguments = &arguments[..arguments.len().min(supplied.len())];
        parameters.iter().enumerate().map(|(index, parameter)| {
            if parameter.rest {
                let items = arguments.iter().skip(index).cloned().collect::<Vec<_>>().join(", ");
                return format!("<{}>::from([{items}])", self.native_type(parameter.ty));
            }
            let actual = supplied.get(index).copied().unwrap_or(Type::Undefined);
            let value = arguments.get(index).cloned().unwrap_or_else(|| "TsrUndefined".to_owned());
            let Type::Union(union) = parameter.ty else { return value; };
            // A union element already carries its tag; a readonly input views it.
            if actual == parameter.ty {
                return if !union.copy && parameter.mode == Mode::Read {
                    format!("({value}).tsr_data_view()")
                } else {
                    value
                };
            }
            let variant = self.program.unions[union.index()].members.iter().position(|member| {
                *member == actual || matches!(member, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) && matches!(actual, Type::Closure(_) | Type::StoredCallable(_))
            }).expect("checked array input union member");
            let view = !union.copy && parameter.mode == Mode::Read;
            let value = if view && actual.is_copy() { format!("&({value})") } else { value };
            let carrier = if matches!(actual, Type::Undefined | Type::Null) && self.callback_payload(parameter.ty).is_some() {
                if parameter.mode==Mode::Mutate {"::<&mut TsrAbsentCallable>"}else{"::<TsrAbsentCallable>"}
            } else { "" };
            format!("{}Union{}{carrier}::V{variant}({value})", if view { "View" } else { "" }, union.index())
        }).collect()
    }

    fn object_has_own(&self, fun: &CheckedFunction, args: &[Expr], key_first: bool) -> String {
        let value=self.expr(fun,&args[0],Mode::Read);
        let bounded=|length:String|format!("own_key_text == \"length\" || own_key_text.parse::<usize>().is_ok_and(|index| index < {length} && index.to_string() == own_key_text)");
        let check=match args[0].ty.expect("checked hasOwn receiver") {
            Type::Record(id) if !self.program.records[id].is_tuple()=>{
                let fields=self.program.records[id].fields.iter().map(|field|format!("{:?}",field.name.text)).collect::<Vec<_>>();
                if fields.is_empty(){"false".into()}else{format!("matches!(own_key_text,{})",fields.join(" | "))}
            },
            Type::Record(id)=>bounded(self.program.records[id].fields.len().to_string()),
            Type::Array(_)=>bounded("own_value.len()".into()),
            Type::String=>bounded("own_value.as_ref().encode_utf16().count()".into()),
            _=>"false".into(),
        };
        if let Some(union)=args[1].ty.and_then(Type::union) {
            // Keep borrowed text inside the argument-storage block; a union's
            // string key is inspected rather than cloned by String conversion.
            let storage=self.value_builtin_bindings(fun,&args[1..]);
            let input=if union.copy {"&builtin_argument_0"}else{"builtin_argument_0"};
            let text=self.union_observation(union,input,CallTarget::ObjectHasOwn);
            format!("{{ let own_value={value}; {storage} let own_key={text}; let own_key_text: &str=own_key.as_ref(); {check} }} }}")
        }else{
            let key=if args[1].ty==Some(Type::String) {self.expr(fun,&args[1],Mode::Read)} else {self.value_builtin_call(fun,CallTarget::StringConversion,&args[1..])};
            if key_first {
                format!("{{ let own_key={key}; let own_value={value}; let own_key_text: &str=own_key.as_ref(); {check} }}")
            } else {
                format!("{{ let own_value={value}; let own_key={key}; let own_key_text: &str=own_key.as_ref(); {check} }}")
            }
        }
    }

    fn array_sort(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        copying: bool,
    ) -> String {
        let array_type = base.ty.expect("checked sorting receiver");
        let Type::Array(element) = array_type else {
            unreachable!()
        };
        let temporary = Self::temporary_at_receiver(base);
        let mut code = String::from("{ ");
        let receiver = if copying {
            let receiver = self.expr(fun, base, Mode::Read);
            write!(code, "let sorting_source = {receiver}; ").unwrap();
            "sorting_source".to_owned()
        } else if temporary {
            let receiver = self.expr(fun, base, Mode::Consume);
            write!(code, "let mut sorting_owned = {receiver}; ").unwrap();
            "sorting_owned".to_owned()
        } else {
            let selected = self.storage_selection(fun, base, false);
            code.push_str(&selected.setup);
            selected.exclusive
        };
        let mut fallible = false;
        let compare = if let Some(callback) = args.first() {
            let callback_type = callback.ty.expect("checked comparator");
            let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
            let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
            let environment = self.expr(fun, callback, if read_callback{Mode::Read}else{Mode::Mutate});
            write!(code, "let sorting_callback = {environment}; ").unwrap();
            let (count, result) = match callback_type {
                Type::Closure(id) => (
                    self.program.closures[id].parameters.len(),
                    self.program.closures[id].function.result,
                ),
                Type::Callback(id) => (
                    self.program.callback_contracts[id].params.len(),
                    self.program.callback_contracts[id].result,
                ),
                Type::StoredCallable(family) | Type::BorrowedCallable(family) => {
                    let contract = &self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")];
                    (contract.params.len(), contract.result)
                },
                _ => unreachable!(),
            };
            let count = self.array_callback_supplied_count(callback_type, callback.traversal_method, count, 2);
            let values = ["sorting_left", "sorting_right"]
                .into_iter()
                .take(count)
                .map(|name| {
                    if element.ty().is_copy() {
                        format!("*{name}")
                    } else {
                        name.to_owned()
                    }
                })
                .collect::<Vec<_>>();
            let values = self.array_callback_arguments(callback_type, callback.traversal_method, values, &[element.ty(), element.ty()]);
            let call = if self.array_callback_counted_method(callback_type, callback.traversal_method) {
                format!("sorting_callback.call{}_with_count({}{})", callback.traversal_method.expect("checked counted method").id, 2, if values.is_empty() { String::new() } else { format!(", {}", values.join(", ")) })
            } else if self.counted_calls && callback.traversal_method.is_none() {
                if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                    self.counted_invocation(invocation, "sorting_callback", &tuple(values.into_iter()), 2)
                } else {
                    self.counted_call("sorting_callback", &values.join(", "), 2)
                }
            } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                format!(
                    "{invocation}(sorting_callback, {})",
                    tuple(values.into_iter())
                )
            } else {
                format!("sorting_callback.call{}({})", callback.traversal_method.map_or(String::new(), |method| method.id.to_string()), values.join(", "))
            };
            fallible = self.registered_traversal_call(call.clone(), callback) != call;
            if fallible {
                let order = if result == Type::Int {
                    "sorting_order.cmp(&0)"
                } else {
                    "if sorting_order < 0.0 { std::cmp::Ordering::Less } else if sorting_order > 0.0 { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Equal }"
                };
                format!("|sorting_left, sorting_right| {{ ({call}).map(|sorting_order| {order}) }}")
            } else             if result == Type::Int {
                format!(
                    "|sorting_left, sorting_right| {{ let sorting_order: i64 = {call}; sorting_order.cmp(&0) }}"
                )
            } else {
                format!(
                    "|sorting_left, sorting_right| {{ let sorting_order: f64 = {call}; if sorting_order < 0.0 {{ std::cmp::Ordering::Less }} else if sorting_order > 0.0 {{ std::cmp::Ordering::Greater }} else {{ std::cmp::Ordering::Equal }} }}"
                )
            }
        } else {
            match element {
                ArrayElement::Number(0) => "|a: &f64, b: &f64| a.total_cmp(b)".to_owned(),
                ArrayElement::Int(0) | ArrayElement::Boolean(0) | ArrayElement::String(0) => {
                    "|a, b| a.cmp(b)".to_owned()
                }
                ArrayElement::Null(0) | ArrayElement::Undefined(0) => {
                    "|_, _| std::cmp::Ordering::Equal".to_owned()
                }
                _ => unreachable!("checked default sort"),
            }
        };
        if copying {
            let literal =
                self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
            let values = if literal {
                "sorting_source.values.iter().map(|value| String::Borrowed(value)).collect()"
            } else {
                "sorting_source.iter().cloned().collect()"
            };
            write!(
                code,
                "let sorting_values: Vec<{}> = {values}; ",
                self.native_type(element.ty())
            )
            .unwrap();
            let wrapped = if numeric_array(array_type) {
                "NumericArray::Heap(sorting_values)".to_owned()
            } else if inline_array(array_type) {
                "Array::Heap(sorting_values)".to_owned()
            } else if let ArrayElement::Record(id, 0) = element {
                if self.record_runtime.recursive(id) {
                    format!("RecordArray{id}(sorting_values)")
                } else if self.small_record(id) {
                    "Array::Heap(sorting_values)".to_owned()
                } else {
                    "sorting_values".to_owned()
                }
            } else {
                "sorting_values".to_owned()
            };
            let call = if fallible {
                self.registered_traversal_call(format!("sorting_copy.tsr_try_sort({compare})"), &args[0])
            } else { format!("sorting_copy.tsr_sort({compare})") };
            write!(code, "let mut sorting_copy = {wrapped}; {call}; sorting_copy }}").unwrap();
        } else {
            let call = if fallible {
                self.registered_traversal_call(format!("({receiver}).tsr_try_sort({compare})"), &args[0])
            } else { format!("({receiver}).tsr_sort({compare})") };
            write!(code, "{call}; }}").unwrap();
        }
        code
    }

    fn array_reverse(&self, fun: &CheckedFunction, base: &Expr) -> String {
        if Self::temporary_storage(base) {
            format!("({}).reverse()", self.expr(fun, base, Mode::Mutate))
        } else {
            let selection = self.storage_selection(fun, base, false);
            format!(
                "{{ {}({}).reverse(); }}",
                selection.setup, selection.exclusive
            )
        }
    }

    fn array_splice(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let array_type = base.ty.expect("checked splice array");
        if args.is_empty() {
            return format!(
                "{{ let _splice_receiver = {}; let splice_empty: {} = [].into(); splice_empty }}",
                self.conditional_arm(fun, base, Mode::Read),
                self.native_type(array_type)
            );
        }
        let (setup, destination) = if Self::temporary_storage(base) {
            (
                format!(
                    "let mut splice_owned = {};",
                    self.expr(fun, base, Mode::Consume)
                ),
                "splice_owned".to_owned(),
            )
        } else {
            let selection = self.storage_selection(fun, base, false);
            (selection.setup, selection.exclusive)
        };
        let start = self.expr(fun, &args[0], Mode::Copy);
        let delete = args.get(1).map_or("None".to_owned(), |value| {
            format!("Some({})", self.expr(fun, value, Mode::Copy))
        });
        let values = args
            .iter()
            .skip(2)
            .map(|arg| self.expr(fun, arg, Mode::Consume))
            .collect::<Vec<_>>()
            .join(", ");
        let failure = format!(
            "splice capacity or int length overflow at source bytes {}..{}",
            span.start, span.end
        );
        let result = if let Type::Array(ArrayElement::Record(record, 0)) = array_type
            && self.record_runtime.recursive(record)
        {
            format!("RecordArray{record}(splice_removed)")
        } else {
            "splice_removed".to_owned()
        };
        format!(
            "{{ {setup} let splice_start:i64={start}; let splice_delete:Option<i64>={delete}; let splice_added=[{values}]; let splice_removed=({destination}).tsr_splice(splice_start,splice_delete,splice_added,{failure:?}); {result} }}"
        )
    }

    fn array_unshift(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let failure = format!(
            "{:?}",
            format!(
                "array length does not fit int at source bytes {}..{}",
                span.start, span.end
            )
        );
        if args.is_empty() {
            return format!(
                "i64::try_from(({}).len()).expect({failure})",
                self.conditional_arm(fun, base, Mode::Read)
            );
        }
        let (setup, destination) = if Self::temporary_storage(base) {
            (
                format!(
                    "let mut unshift_owned = {};",
                    self.expr(fun, base, Mode::Consume)
                ),
                "unshift_owned".to_owned(),
            )
        } else {
            let selection = self.storage_selection(fun, base, false);
            (selection.setup, selection.exclusive)
        };
        let values = args
            .iter()
            .map(|arg| self.expr(fun, arg, Mode::Consume))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{{ {setup} let unshift_values = [{values}]; ({destination}).tsr_unshift(unshift_values, {failure}) }}"
        )
    }

    fn array_push(&self, fun: &CheckedFunction, base: &Expr, value: &Expr) -> String {
        let value = self.expr(fun, value, Mode::Consume);
        if Self::temporary_storage(base) {
            let array = self.expr(fun, base, Mode::Mutate);
            return format!(
                "{{ let temporary_array = {array}; let storage_value = {value}; temporary_array.push(storage_value); }}"
            );
        }
        if !matches!(base.kind, ExprKind::Name(_)) {
            let selection = self.storage_selection(fun, base, false);
            return format!(
                "{{ {}let storage_value = {value}; ({}).push(storage_value); }}",
                selection.setup, selection.exclusive
            );
        }
        let array = self.binding(fun, base.binding.expect("named array"), Mode::Mutate);
        format!("{{ let storage_value = {value}; ({array}).push(storage_value); }}")
    }

    fn array_search(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
        index_of: bool,
    ) -> String {
        let array = self.expr(fun, base, Mode::Read);
        let value = self.expr(fun, &args[0], Mode::Read);
        let from = args
            .get(1)
            .map_or_else(|| "0i64".to_owned(), |arg| self.expr(fun, arg, Mode::Copy));
        let string = args[0].ty.map(Type::scalar_base) == Some(Type::String);
        let value_reference = if string {
            "search_value"
        } else {
            "&search_value"
        };
        let method = if index_of {
            "tsr_index_of"
        } else {
            "tsr_includes"
        };
        let failure = if index_of {
            format!(
                ", {:?}",
                format!(
                    "array search index exceeds int range at source bytes {}..{}",
                    span.start, span.end
                )
            )
        } else {
            String::new()
        };
        if let Some(Type::Array(ArrayElement::Union(union, 0))) = base.ty {
            let equal = self.union_search_predicate(union, args[0].ty.expect("checked search value"), !index_of);
            let result = if index_of {
                format!("tsr_search_result(search_start, search_array.iter().skip(search_start).position(|item| {equal}){failure})")
            } else {
                format!("search_array.iter().skip(search_start).any(|item| {equal})")
            };
            return format!(
                "{{ let search_array = {array}; let search_value = {value}; let search_from = {from}; let search_start = tsr_search_start(search_array.len(), search_from); {result} }}"
            );
        }
        // Evaluate receiver and arguments once, then compare standard contents.
        let search = if matches!(
            self.expression_shape(base),
            Some(crate::fixed_storage::Shape::ScalarArray { .. })
        ) || matches!(
            base.ty,
            Some(Type::Array(
                ArrayElement::Number(0) | ArrayElement::Null(0) | ArrayElement::Undefined(0) | ArrayElement::Enum(_, _, 0)
            ))
        ) {
            if index_of {
                format!(
                    "{{ let search_start = tsr_search_start(search_array.len(), search_from); tsr_search_result(search_start, search_array[search_start..].iter().position(|item| item == &search_value){failure}) }}"
                )
            } else if matches!(base.ty, Some(Type::Array(ArrayElement::Enum(_, crate::syntax::EnumKind::Number, 0)))) {
                "search_array[tsr_search_start(search_array.len(), search_from)..].iter().any(|item| *item == search_value || (item.tsr_value().is_nan() && search_value.tsr_value().is_nan()))".into()
            } else if matches!(base.ty, Some(Type::Array(ArrayElement::Number(0)))) {
                // includes() compares with SameValueZero: NaN is found, and
                // signed zeros are equal. indexOf() keeps strict equality.
                "search_array[tsr_search_start(search_array.len(), search_from)..].iter().any(|item| *item == search_value || (item.is_nan() && search_value.is_nan()))".into()
            } else {
                "search_array[tsr_search_start(search_array.len(), search_from)..].contains(&search_value)".into()
            }
        } else if string {
            format!(
                "search_array.{method}(search_value, search_from, |left, right| left == right{failure})"
            )
        } else {
            format!("search_array.{method}({value_reference}, search_from{failure})")
        };
        format!(
            "{{ let search_array = {array}; let search_value = {value}; let search_from = {from}; {search} }}"
        )
    }

    /// Compare one union element `item` with `search_value`. Only the
    /// alternative of the searched type can match; int and number compare by
    /// value. `same_value_zero` makes NaN match NaN, as includes() requires.
    fn union_search_predicate(&self, union: UnionId, search: Type, same_value_zero: bool) -> String {
        let index = union.index();
        let arms = self.program.unions[index]
            .members
            .iter()
            .enumerate()
            .filter_map(|(variant, member)| {
                let compare = match (*member, search) {
                    (Type::String, Type::String) => "**payload == **search_value",
                    (Type::Int, Type::Int) | (Type::Boolean, Type::Boolean) => "*payload == search_value",
                    (Type::Number, Type::Number) if same_value_zero => "*payload == search_value || (payload.is_nan() && search_value.is_nan())",
                    (Type::Number, Type::Number) => "*payload == search_value",
                    (Type::Int, Type::Number) => "(*payload as f64) == search_value",
                    (Type::Number, Type::Int) => "*payload == (search_value as f64)",
                    (Type::Null, Type::Null) | (Type::Undefined, Type::Undefined) => "{ let _ = payload; true }",
                    _ => return None,
                };
                Some(format!("Union{index}::V{variant}(payload) => {compare}, "))
            })
            .collect::<String>();
        format!("match item {{ {arms}#[allow(unreachable_patterns)] _ => false }}")
    }

    fn array_last_index_of(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let array = self.expr(fun, base, Mode::Read);
        let value = self.expr(fun, &args[0], Mode::Read);
        let from = args.get(1).map_or_else(
            || "None".to_owned(),
            |arg| format!("Some({})", self.expr(fun, arg, Mode::Copy)),
        );
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let union_equal = match base.ty {
            Some(Type::Array(ArrayElement::Union(union, 0))) => {
                self.union_search_predicate(union, args[0].ty.expect("checked search value"), false)
            }
            _ => String::new(),
        };
        let (iterator, equal) = if !union_equal.is_empty() {
            ("search_array.iter()", union_equal.as_str())
        } else if literal {
            (
                "search_array.values.iter()",
                "*item == search_value.as_ref()",
            )
        } else if args[0].ty.map(Type::scalar_base) == Some(Type::String) {
            ("search_array.iter()", "item == search_value")
        } else {
            ("search_array.iter()", "item == &search_value")
        };
        let failure = format!(
            "array search index exceeds int range at source bytes {}..{}",
            span.start, span.end
        );
        // Reverse before skipping so owned inline iterators need no exact-size
        // contract. checked_sub handles empty arrays and negative signed extremes.
        format!(
            "{{ let search_array = {array}; let search_value = {value}; let search_from: Option<i64> = {from}; let search_len = search_array.len(); let search_index = search_len.checked_sub(1).and_then(|last| match search_from {{ None => Some(last), Some(from) if from >= 0 => Some(usize::try_from(from).unwrap_or(usize::MAX).min(last)), Some(from) => usize::try_from(from.unsigned_abs()).ok().and_then(|distance| search_len.checked_sub(distance)) }}); search_index.map_or(-1i64, |index| {iterator}.rev().skip(search_len - 1 - index).position(|item| {equal}).map_or(-1i64, |offset| i64::try_from(index - offset).expect({failure:?}))) }}"
        )
    }

    fn array_reduce(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        result: Type,
        right: bool,
        span: crate::diagnostic::Span,
    ) -> String {
        let method = args[0].traversal_method;
        let receiver = self.expr(
            fun,
            base,
            if method.is_some_and(|method| method.access == Mode::Mutate) {
                Mode::Mutate
            } else {
                Mode::Read
            },
        );
        let callback_type = args[0].ty.expect("checked reduction callback");
        let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
        let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
        let callback = self.expr(fun, &args[0], if read_callback{Mode::Read}else{Mode::Mutate});
        let parameters = match callback_type {
            Type::Closure(id) => self.program.closures[id]
                .parameters
                .iter()
                .map(|&parameter| {
                    ArgumentContract::from(&self.program.closures[id].function.bindings[parameter])
                })
                .collect::<Vec<_>>(),
            Type::Callback(id) => self.program.callback_contracts[id].params.clone(),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.clone(),
            _ => unreachable!("checked reduction callback"),
        };
        let parameter_count = self.array_callback_supplied_count(callback_type, method, parameters.len(), if method.is_some_and(|method| method.access == Mode::Mutate) { 3 } else { 4 });
        let Type::Array(element) = base.ty.expect("checked reduction receiver") else {
            unreachable!()
        };
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        debug_assert!(!literal || parameter_count < 4);
        let mut arguments = Vec::new();
        if let Some(accumulator) = parameters.first() {
            arguments.push(
                match accumulator.mode {
                    Mode::Copy | Mode::Consume => "reduce_accumulator",
                    Mode::Read => "&reduce_accumulator",
                    Mode::Mutate => "&mut reduce_accumulator",
                }
                .to_owned(),
            );
        }
        if parameter_count > 1 {
            arguments.push(
                if element.ty().is_copy() {
                    "*reduce_value"
                } else {
                    "reduce_value"
                }
                .to_owned(),
            );
        }
        if parameter_count > 2 {
            arguments.push("reduce_index".to_owned());
        }
        if parameter_count > 3 {
            arguments.push("reduce_array".to_owned());
        }
        let arguments = self.array_callback_arguments(callback_type, method, arguments, &[result, element.ty(), Type::Int, base.ty.expect("checked array")]);
        let call = if self.array_callback_counted_method(callback_type, method) {
                format!("reduce_callback.call{}_with_count({}{})", method.expect("checked counted method").id, if method.is_some_and(|method| method.access == Mode::Mutate) { 3 } else { 4 }, if arguments.is_empty() { String::new() } else { format!(", {}", arguments.join(", ")) })
            } else if self.counted_calls && method.is_none() {
            if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                self.counted_invocation(invocation, "reduce_callback", &tuple(arguments.into_iter()), if method.is_some_and(|method| method.access == Mode::Mutate) { 3 } else { 4 })
            } else {
                self.counted_call("reduce_callback", &arguments.join(", "), if method.is_some_and(|method| method.access == Mode::Mutate) { 3 } else { 4 })
            }
        } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
            format!(
                "{invocation}(reduce_callback, {})",
                tuple(arguments.into_iter())
            )
        } else {
            format!(
                "reduce_callback.call{}({})",
                method.map_or(String::new(), |method| method.id.to_string()),
                arguments.join(", ")
            )
        };
        let call = self.registered_traversal_call(call, &args[0]);
        let failure = format!(
            "empty array reduction without an initial value at source bytes {}..{}",
            span.start, span.end
        );
        let (initial, range) = if let Some(initial) = args.get(1) {
            (
                self.expr(fun, initial, Mode::Consume),
                "0..reduce_array.len()".to_owned(),
            )
        } else {
            let seed = if right { "reduce_array.len() - 1" } else { "0" };
            let value = if literal {
                format!("reduce_array.get({seed}).expect(\"reduction seed in bounds\")")
            } else {
                format!(
                    "Clone::clone(reduce_array.get({seed}).expect(\"reduction seed in bounds\"))"
                )
            };
            (
                format!("{{ assert!(reduce_array.len() != 0, {failure:?}); {value} }}"),
                if right {
                    "0..reduce_array.len()-1".to_owned()
                } else {
                    "1..reduce_array.len()".to_owned()
                },
            )
        };
        // len works uniformly for ordinary fixed arrays and existing owning containers.
        let positions = if right {
            format!("({range}).rev()")
        } else {
            range
        };
        let value = if literal {
            "let reduce_literal = reduce_array.get(reduce_position).expect(\"reduction index in bounds\"); let reduce_value = &reduce_literal; "
        } else if method.is_some_and(|method| method.access == Mode::Mutate) {
            "let reduce_value = reduce_array.get_mut(reduce_position).expect(\"reduction index in bounds\"); "
        } else {
            "let reduce_value = reduce_array.get(reduce_position).expect(\"reduction index in bounds\"); "
        };
        let index = if parameter_count > 2 {
            "let reduce_index = i64::try_from(reduce_position).expect(\"reduction index exceeds int range\"); "
        } else {
            ""
        };
        format!(
            "{{ let reduce_array = {receiver}; let reduce_callback = {callback}; let mut reduce_accumulator: {} = {initial}; for reduce_position in {positions} {{ {value}{index}reduce_accumulator = {call}; }} reduce_accumulator }}",
            self.native_type(result)
        )
    }

    fn flat_array_result(&self, result: Type, values: &str) -> String {
        if numeric_array(result) {
            format!("NumericArray::Heap({values})")
        } else if inline_array(result) {
            format!("Array::Heap({values})")
        } else if let Type::Array(ArrayElement::Record(id, 0)) = result {
            if self.record_runtime.recursive(id) {
                format!("RecordArray{id}({values})")
            } else if self.small_record(id) {
                format!("Array::Heap({values})")
            } else {
                values.into()
            }
        } else {
            values.into()
        }
    }

    fn array_flat(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        base: &Expr,
        args: &[Expr],
        levels: u8,
    ) -> String {
        let owning = Self::temporary_at_receiver(base);
        let receiver = self.expr(fun, base, if owning { Mode::Consume } else { Mode::Read });
        let depth = args
            .first()
            .map_or_else(|| "1i64".into(), |arg| self.expr(fun, arg, Mode::Copy));
        let failure = format!(
            "flat capacity overflow at source bytes {}..{}",
            expr.span.start, expr.span.end
        );
        let count = if levels == 0 {
            "let flat_count=flat_array.len();".to_owned()
        } else {
            let mut output = "let mut flat_count=0usize;".to_owned();
            let mut parent = "flat_array".to_owned();
            for level in 0..levels {
                let row = format!("flat_count_row_{level}");
                write!(output, "for {row} in {parent}.iter() {{").unwrap();
                parent = row;
            }
            write!(
                output,
                "flat_count=flat_count.checked_add({parent}.len()).expect({failure:?});"
            )
            .unwrap();
            output.push_str(&"}".repeat(usize::from(levels)));
            output
        };
        let mut append = String::new();
        let mut parent = "flat_array".to_owned();
        for level in 0..levels {
            let row = format!("flat_row_{level}");
            let method = if owning { "tsr_owned_items" } else { "iter" };
            write!(append, "for {row} in {parent}.{method}() {{").unwrap();
            parent = row;
        }
        let iterator = if owning {
            format!("{parent}.tsr_owned_items()")
        } else {
            format!("{parent}.iter().cloned()")
        };
        write!(append, "flat_values.extend({iterator});").unwrap();
        append.push_str(&"}".repeat(usize::from(levels)));
        let result = expr.ty.expect("flat result");
        format!(
            "{{ let flat_array={receiver}; let _flat_depth:i64={depth}; {count} let flat_count=tsr_flat_capacity(0,flat_count,{failure:?}); let mut flat_values:Vec<{}>=Vec::with_capacity(flat_count); {append} {} }}",
            self.native_type(if let Type::Array(e) = result {
                e.ty()
            } else {
                unreachable!()
            }),
            self.flat_array_result(result, "flat_values")
        )
    }
    fn array_flat_map(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        base: &Expr,
        callback: &Expr,
    ) -> String {
        let method = callback.traversal_method;
        let receiver = self.expr(
            fun,
            base,
            if method.is_some_and(|method| method.access == Mode::Mutate) {
                Mode::Mutate
            } else {
                Mode::Read
            },
        );
        let callback_type = callback.ty.expect("flatMap callback");
        let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
        let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
        let environment = self.expr(fun, callback, if read_callback{Mode::Read}else{Mode::Mutate});
        let (parameter_count, returned) = match callback_type {
            Type::Closure(id) => (
                self.program.closures[id].parameters.len(),
                method.map_or(self.program.closures[id].function.result, |method| {
                    self.program.closures[id].methods[method.id].function.result
                }),
            ),
            Type::Callback(id) => (
                self.program.callback_contracts[id].params.len(),
                self.program.callback_contracts[id].result,
            ),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => {
                let contract = &self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")];
                (contract.params.len(), contract.result)
            },
            _ => unreachable!(),
        };
        let parameter_count = self.array_callback_supplied_count(callback_type, method, parameter_count, if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 });
        let Type::Array(element) = base.ty.expect("flatMap receiver") else {
            unreachable!()
        };
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        let iterator = if literal {
            "flatmap_array.literal_iter()"
        } else if method.is_some_and(|method| method.access == Mode::Mutate) {
            "flatmap_array.iter_mut()"
        } else {
            "flatmap_array.iter()"
        };
        let prefix = if literal {
            "let flatmap_literal=String::Borrowed(flatmap_value);let flatmap_value=&flatmap_literal;"
        } else {
            ""
        };
        let mut arguments = Vec::new();
        if parameter_count > 0 {
            arguments.push(if element.ty().is_copy() {
                "*flatmap_value"
            } else {
                "flatmap_value"
            });
        }
        if parameter_count > 1 {
            arguments.push(r#"i64::try_from(flatmap_position).expect("flatMap callback index exceeds int range")"#);
        }
        if parameter_count > 2 {
            arguments.push("flatmap_array");
        }
        let arguments = self.array_callback_arguments(callback_type, method, arguments.into_iter().map(str::to_owned).collect(), &[element.ty(), Type::Int, base.ty.expect("checked array")]);
        let call = if self.array_callback_counted_method(callback_type, method) {
                format!("flatmap_callback.call{}_with_count({}{})", method.expect("checked counted method").id, if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 }, if arguments.is_empty() { String::new() } else { format!(", {}", arguments.join(", ")) })
            } else if self.counted_calls && method.is_none() {
            if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                self.counted_invocation(invocation, "flatmap_callback", &tuple(arguments.into_iter()), if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            } else {
                self.counted_call("flatmap_callback", &arguments.join(", "), if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            }
        } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
            format!(
                "{invocation}(flatmap_callback,{})",
                tuple(arguments.into_iter())
            )
        } else if let Some(method) = method {
            format!("flatmap_callback.call{}({})", method.id, arguments.join(","))
        } else {
            format!("flatmap_callback.call({})", arguments.join(","))
        };
        let call = self.registered_traversal_call(call, callback);
        let failure = format!(
            "flatMap capacity overflow at source bytes {}..{}",
            expr.span.start, expr.span.end
        );
        let append = if matches!(returned, Type::Array(_)) {
            format!(
                "let flatmap_count=flatmap_result.len();tsr_flat_capacity(flatmap_values.len(),flatmap_count,{failure:?});flatmap_values.reserve(flatmap_count);flatmap_values.extend(flatmap_result.tsr_owned_items());"
            )
        } else {
            format!(
                "tsr_flat_capacity(flatmap_values.len(),1,{failure:?});flatmap_values.reserve(1);flatmap_values.push(flatmap_result);"
            )
        };
        let result = expr.ty.expect("flatMap result");
        let Type::Array(output) = result else {
            unreachable!()
        };
        format!(
            "{{let flatmap_array={receiver};let flatmap_callback={environment};let mut flatmap_values:Vec<{}>=Vec::new();for(flatmap_position,flatmap_value)in {iterator}.enumerate(){{{prefix}let flatmap_result={call};{append}}}{} }}",
            self.native_type(output.ty()),
            self.flat_array_result(result, "flatmap_values")
        )
    }

    fn array_transform(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        callback: &Expr,
        result: Type,
        filter: bool,
        from_array: bool,
    ) -> String {
        let method = callback.traversal_method;
        let receiver = self.expr(
            fun,
            base,
            if method.is_some_and(|method| method.access == Mode::Mutate) {
                Mode::Mutate
            } else {
                Mode::Read
            },
        );
        let callback_type = callback.ty.expect("checked array callback");
        let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
        let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
        let environment = self.expr(fun, callback, if read_callback{Mode::Read}else{Mode::Mutate});
        let parameter_count = match callback_type {
            Type::Closure(id) => self.program.closures[id].parameters.len(),
            Type::Callback(id) => self.program.callback_contracts[id].params.len(),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.len(),
            _ => unreachable!("checked reusable callback"),
        };
        let parameter_count = self.array_callback_supplied_count(callback_type, method, parameter_count, if from_array || method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 });
        let receiver_type=base.ty.expect("checked receiver");
        let element=match receiver_type {
            Type::Array(element)=>element,
            Type::Set(id) if from_array=>ArrayElement::from_value_type(self.program.records[id].set_definition.as_ref().unwrap().key).expect("Set mapper element"),
            Type::Map(id) if from_array=>ArrayElement::Record(self.program.records[id].map_definition.as_ref().unwrap().entry.expect("Map mapper entry"),0),
            _=>unreachable!("checked transform source"),
        };
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        debug_assert!(!literal || parameter_count < 3);
        let ordered_map=match receiver_type {Type::Map(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap)=>Some(id),_=>None};
        let iterator = if ordered_map.is_some() {
            "transform_array.source_iter()"
        } else if literal {
            "transform_array.literal_iter()"
        } else if matches!(receiver_type,Type::Map(_)) {
            "transform_array.entry_iter()"
        } else if method.is_some_and(|method| method.access == Mode::Mutate) {
            "transform_array.iter_mut()"
        } else {
            "transform_array.iter()"
        };
        let value = if element.ty().is_copy() {
            "*transform_value"
        } else {
            "transform_value"
        };
        let mut arguments = Vec::new();
        if parameter_count > 0 {
            arguments.push(value.to_owned());
        }
        if parameter_count > 1 {
            arguments.push("transform_index".to_owned());
        }
        if parameter_count > 2 {
            arguments.push("transform_array".to_owned());
        }
        let supplied=if from_array{vec![element.ty(),Type::Int]}else{vec![element.ty(),Type::Int,receiver_type]};
        let arguments = self.array_callback_arguments(callback_type, method, arguments, &supplied);
        let call = if self.array_callback_counted_method(callback_type, method) {
                format!("transform_callback.call{}_with_count({}{})", method.expect("checked counted method").id, if from_array || method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 }, if arguments.is_empty() { String::new() } else { format!(", {}", arguments.join(", ")) })
            } else if self.counted_calls && method.is_none() {
            if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                self.counted_invocation(invocation, "transform_callback", &tuple(arguments.into_iter()), if from_array || method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            } else {
                self.counted_call("transform_callback", &arguments.join(", "), if from_array || method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            }
        } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
            format!(
                "{invocation}(transform_callback, {})",
                tuple(arguments.into_iter())
            )
        } else if let Some(method) = method {
            format!("transform_callback.call{}({})", method.id, arguments.join(", "))
        } else {
            format!("transform_callback.call({})", arguments.join(", "))
        };
        let index = if parameter_count > 1 {
            "let transform_index = i64::try_from(transform_position).expect(\"array callback index exceeds int range\"); "
        } else {
            ""
        };
        let literal_value = if let Some(id)=ordered_map {
            let view=self.ordered_map_entry_view(id,"transform_value.0","transform_value.1");
            format!("let transform_entry={view};let transform_value=&transform_entry;")
        } else if literal {
            "let transform_literal = String::Borrowed(transform_value); let transform_value = &transform_literal; ".into()
        } else {
            String::new()
        };
        let checked_call = self.registered_traversal_call(call.clone(), callback);
        if let Type::Map(id)=result {
            let groups=self.program.records[id].map_definition.as_ref().unwrap().value;
            let single=self.flat_array_result(groups,"vec![Clone::clone(transform_value)]");
            let native=self.native_type(result);
            return format!("{{let transform_array={receiver};let transform_callback={environment};let mut transform_groups={native}::new();for (transform_position,transform_value) in {iterator}.enumerate(){{{index}{literal_value}let transform_key={checked_call};if let Some(group)=transform_groups.get_mut(&transform_key){{group.push(Clone::clone(transform_value));}}else{{transform_groups.set(transform_key,{single});}}}}transform_groups}}");
        }
        if checked_call != call {
            let Type::Array(output) = result else { unreachable!("checked transform result") };
            let append = if filter {
                format!("if {checked_call} {{ transform_values.push(Clone::clone(transform_value)); }}")
            } else {
                format!("transform_values.push({checked_call});")
            };
            return format!(
                "{{ let transform_array={receiver}; let transform_callback={environment}; let mut transform_values:Vec<{}>=Vec::new(); for (transform_position,transform_value) in {iterator}.enumerate() {{ {index}{literal_value}{append} }} {} }}",
                self.native_type(output.ty()), self.flat_array_result(result, "transform_values")
            );
        }
        let operation = if filter {
            format!("if {call} {{ Some(Clone::clone(transform_value)) }} else {{ None }}")
        } else {
            call
        };
        let method = if filter { "filter_map" } else { "map" };
        let Type::Array(output) = result else {
            unreachable!("checked transform result")
        };
        let values = format!(
            "{iterator}.enumerate().{method}(|(transform_position, transform_value)| {{ {index}{literal_value}{operation} }}).collect::<Vec<{}>>()",
            self.native_type(output.ty())
        );
        let wrapped = if numeric_array(result) {
            format!("NumericArray::Heap({values})")
        } else if inline_array(result) {
            format!("Array::Heap({values})")
        } else if let Type::Array(ArrayElement::Record(record, 0)) = result {
            if self.record_runtime.recursive(record) {
                format!("RecordArray{record}({values})")
            } else if self.small_record(record) {
                format!("Array::Heap({values})")
            } else {
                values
            }
        } else {
            values
        };
        format!(
            "{{ let transform_array = {receiver}; let transform_callback = {environment}; {wrapped} }}"
        )
    }

    fn array_join(&self, fun: &CheckedFunction, base: &Expr, args: &[Expr]) -> String {
        let receiver = self.expr(fun, base, Mode::Read);
        let mut code = format!("{{ let join_receiver = {receiver}; ");
        let separator = if let Some(argument) = args.first() {
            let value = self.expr(fun, argument, Mode::Read);
            let _ = write!(code, "let join_separator: &String = {value}; ");
            "join_separator.as_ref()"
        } else {
            "\",\""
        };
        let iterator =
            if self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings) {
                "join_receiver.literal_iter()"
            } else {
                "join_receiver.iter()"
            };
        let _ = write!(code, "tsr_array_join({iterator}, {separator}) }}");
        code
    }

    // Only checked builtin calls qualify; source spelling alone is insufficient.
    fn concat_integer_conversion(expr: &Expr) -> Option<&[Expr]> {
        if expr.target != Some(CallTarget::StringConversion) || expr.ty != Some(Type::String) {
            return None;
        }
        let ExprKind::Call(_, args) = &expr.kind else {
            return None;
        };
        args.first()
            .is_some_and(|argument| argument.ty == Some(Type::Int))
            .then_some(args.as_slice())
    }

    fn string_concat_integers(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let failure = format!(
            "string concat capacity or allocation failure at source bytes {}..{}",
            span.start, span.end
        );
        let mut code = String::from("{ ");
        let mut integers = Vec::with_capacity(args.len() + 1);
        // Snapshot every reached operand before capacity planning or formatting.
        // Conversion extras retain the ordinary builtin evaluation and loan path.
        for (index, operand) in std::iter::once(base).chain(args).enumerate() {
            let (value, integer) = if let Some(arguments) = Self::concat_integer_conversion(operand)
            {
                (
                    format!(
                        "{}builtin_argument_0 }}",
                        self.value_builtin_bindings(fun, arguments)
                    ),
                    true,
                )
            } else {
                (self.expr(fun, operand, Mode::Read), false)
            };
            let kind = if integer { "i64" } else { "&String" };
            let _ = write!(code, "let concat_operand{index}: {kind} = {value}; ");
            integers.push(integer);
        }
        code.push_str("let mut concat_capacity = 0usize; ");
        for (index, integer) in integers.iter().enumerate() {
            let length = if *integer {
                // Standard magnitude arithmetic handles zero and i64::MIN.
                format!(
                    "(concat_operand{index}.unsigned_abs().checked_ilog10().unwrap_or(0) as usize + 1 + usize::from(concat_operand{index} < 0))"
                )
            } else {
                format!("concat_operand{index}.len()")
            };
            let _ = write!(
                code,
                "concat_capacity = concat_capacity.checked_add({length}).expect({failure:?}); "
            );
        }
        let _ = write!(
            code,
            "let mut concat_output = string_owned_capacity(concat_capacity, {failure:?}); use std::fmt::Write as _; "
        );
        for (index, integer) in integers.iter().enumerate() {
            if *integer {
                let _ = write!(
                    code,
                    "write!(&mut concat_output, \"{{}}\", concat_operand{index}).expect(\"standard String formatting\"); "
                );
            } else {
                let _ = write!(code, "concat_output.push_str(concat_operand{index}); ");
            }
        }
        code.push_str("String::Owned(concat_output) }");
        code
    }

    fn string_construct(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        method: CallTarget,
        span: crate::diagnostic::Span,
    ) -> String {
        if method == CallTarget::StringConcat
            && std::iter::once(base)
                .chain(args)
                .any(|operand| Self::concat_integer_conversion(operand).is_some())
        {
            return self.string_concat_integers(fun, base, args, span);
        }
        let receiver = self.expr(fun, base, Mode::Read);
        let mut output = format!("{{ let string_receiver: &String = {receiver}; ");
        for (index, argument) in args.iter().enumerate() {
            let integer = method == CallTarget::StringRepeat
                || (matches!(
                    method,
                    CallTarget::StringPadStart | CallTarget::StringPadEnd
                ) && index == 0)
                || (method == CallTarget::StringSplit && index == 1);
            let value = self.expr(fun, argument, if integer { Mode::Copy } else { Mode::Read });
            let ty = if integer { "i64" } else { "&String" };
            let _ = write!(output, "let string_argument{index}: {ty} = {value}; ");
        }
        let operation = match method {
            CallTarget::StringHtml(wrapper) => {
                let (tag, attribute) = wrapper.markup();
                match attribute {
                    Some(attribute) => format!(
                        "tsr_string_html(string_receiver, {tag:?}, Some(({attribute:?}, string_argument0)))"
                    ),
                    None => format!("tsr_string_html(string_receiver, {tag:?}, None)"),
                }
            }
            CallTarget::StringConcat => {
                let failure = format!(
                    "string concat capacity or allocation failure at source bytes {}..{}",
                    span.start, span.end
                );
                let arguments = (0..args.len())
                    .map(|index| format!("string_argument{index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tsr_string_concat(string_receiver, &[{arguments}], {failure:?})")
            }
            CallTarget::StringPadStart | CallTarget::StringPadEnd => {
                let name = if method == CallTarget::StringPadStart {
                    "padStart"
                } else {
                    "padEnd"
                };
                let failure = format!(
                    "string {name} capacity or allocation failure at source bytes {}..{}",
                    span.start, span.end
                );
                let boundary = format!(
                    "string {name} filler prefix is not on a UTF-8 boundary at source bytes {}..{}",
                    span.start, span.end
                );
                let filler = if args.len() == 2 {
                    "Some(string_argument1)"
                } else {
                    "None"
                };
                format!(
                    "tsr_string_pad(string_receiver, string_argument0, {filler}, {}, {failure:?}, {boundary:?})",
                    method == CallTarget::StringPadStart
                )
            }
            CallTarget::StringRepeat => {
                let failure = format!(
                    "string repeat capacity overflow at source bytes {}..{}",
                    span.start, span.end
                );
                format!("tsr_string_repeat(string_receiver, string_argument0, {failure:?})")
            }
            CallTarget::StringReplace | CallTarget::StringReplaceAll => format!(
                "tsr_string_replace(string_receiver, string_argument0, string_argument1, {})",
                method == CallTarget::StringReplaceAll
            ),
            CallTarget::StringSplit => {
                let separator = if args.is_empty() {
                    "None"
                } else {
                    "Some(string_argument0)"
                };
                let limit = if args.len() < 2 {
                    "None"
                } else {
                    "Some(string_argument1)"
                };
                format!("tsr_string_split(string_receiver, {separator}, {limit})")
            }
            _ => unreachable!("checked string construction method"),
        };
        let _ = write!(output, "{operation} }}");
        output
    }

    fn array_for_each(&self, fun: &CheckedFunction, base: &Expr, callback: &Expr) -> String {
        let method = callback.traversal_method;
        let receiver = self.expr(
            fun,
            base,
            if method.is_some_and(|method| method.access == Mode::Mutate) {
                Mode::Mutate
            } else {
                Mode::Read
            },
        );
        let callback_type = callback.ty.expect("checked array for_each callback");
        let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
        let environment = self.expr(fun, callback, if read_callback{Mode::Read}else{Mode::Mutate});
        let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
        let parameter_count = match callback_type {
            Type::Closure(id) => self.program.closures[id].parameters.len(),
            Type::Callback(id) => self.program.callback_contracts[id].params.len(),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.len(),
            _ => unreachable!("checked reusable callback"),
        };
        let parameter_count = self.array_callback_supplied_count(callback_type, method, parameter_count, if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 });
        let receiver_type=base.ty.expect("checked for_each receiver");
        let set_for_each=matches!(receiver_type,Type::Set(_));
        let map_for_each=matches!(receiver_type,Type::Map(_));
        let (value_type,key_type)=match receiver_type{
            Type::Array(element)=>(element.ty(),Type::Int),
            Type::Set(id)=>{let key=self.program.records[id].set_definition.as_ref().unwrap().key;(key,key)},
            Type::Map(id)=>{let map=self.program.records[id].map_definition.as_ref().unwrap();(map.value,map.key)},
            _=>unreachable!(),
        };
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        debug_assert!(!literal || parameter_count < 3);
        let mut arguments = Vec::new();
        if parameter_count > 0 {
            arguments.push(
                if value_type.is_copy() {
                    "*for_each_value"
                } else {
                    "for_each_value"
                }
                .to_owned(),
            );
        }
        if parameter_count > 1 {
            arguments.push(if map_for_each {if key_type.is_copy(){"*for_each_key"}else{"for_each_key"}}else if set_for_each {if value_type.is_copy(){"*for_each_value"}else{"for_each_value"}}else{"for_each_index"}.to_owned());
        }
        if parameter_count > 2 {
            arguments.push("for_each_array".to_owned());
        }
        let arguments = self.array_callback_arguments(callback_type, method, arguments, &[value_type, key_type, receiver_type]);
        let call = if self.array_callback_counted_method(callback_type, method) {
                format!("for_each_callback.call{}_with_count({}{})", method.expect("checked counted method").id, if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 }, if arguments.is_empty() { String::new() } else { format!(", {}", arguments.join(", ")) })
            } else if self.counted_calls && method.is_none() {
            if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                self.counted_invocation(invocation, "for_each_callback", &tuple(arguments.into_iter()), if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            } else {
                self.counted_call("for_each_callback", &arguments.join(", "), if method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            }
        } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
            format!(
                "{invocation}(for_each_callback, {})",
                tuple(arguments.into_iter())
            )
        } else if let Some(method) = method {
            format!("for_each_callback.call{}({})", method.id, arguments.join(", "))
        } else {
            format!("for_each_callback.call({})", arguments.join(", "))
        };
        let call=self.registered_traversal_call(call,callback);
        let index = if parameter_count > 1 && !set_for_each && !map_for_each {
            "let for_each_index = i64::try_from(for_each_position).expect(\"array callback index exceeds int range\"); "
        } else {
            ""
        };
        let literal_value = if literal {
            "let for_each_literal = String::Borrowed(for_each_value); let for_each_value = &for_each_literal; "
        } else {
            ""
        };
        let iterator = if literal {
            "for_each_array.literal_iter()"
        } else if matches!(receiver_type,Type::Map(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedMap)) {
            "for_each_array.source_iter()"
        } else if method.is_some_and(|method| method.access == Mode::Mutate) {
            "for_each_array.iter_mut()"
        } else {
            "for_each_array.iter()"
        };
        let pattern=if map_for_each{"(for_each_key,for_each_value)"}else{"for_each_value"};
        // A wildcard statement drops the returned value immediately, including
        // owned data/callable owners, before advancing to the next source item.
        format!(
            "{{ let for_each_array = {receiver}; let for_each_callback = {environment}; for (for_each_position, {pattern}) in {iterator}.enumerate() {{ {index}{literal_value}let _ = {call}; }} }}"
        )
    }

    fn array_predicate(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        callback: &Expr,
        expr: &Expr,
    ) -> String {
        let method = expr.target.expect("predicate target");
        let finding = matches!(method, CallTarget::ArrayFind | CallTarget::ArrayFindLast);
        let visitor_method = callback.traversal_method;
        let owned = finding && Self::temporary_at_receiver(base);
        let receiver = self.expr(
            fun,
            base,
            if owned {
                Mode::Consume
            } else if visitor_method.is_some_and(|method| method.access == Mode::Mutate) {
                Mode::Mutate
            } else {
                Mode::Read
            },
        );
        let callback_type = callback.ty.expect("checked array predicate callback");
        let read_callback=self.program.stored_contracts.callback_declaration(callback_type).is_some_and(|id| matches!(self.program.callback_contracts[id].kind,CallbackKind::Read|CallbackKind::SharedRead));
        let invocation=if read_callback{"ReadCallback::invoke_read"}else{"Callback::invoke"};
        let environment = self.expr(fun, callback, if read_callback{Mode::Read}else{Mode::Mutate});
        let parameter_count = match callback_type {
            Type::Closure(id) => self.program.closures[id].parameters.len(),
            Type::Callback(id) => self.program.callback_contracts[id].params.len(),
            Type::StoredCallable(family) | Type::BorrowedCallable(family) => self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.len(),
            _ => unreachable!("checked reusable callback"),
        };
        let parameter_count = self.array_callback_supplied_count(callback_type, visitor_method, parameter_count, if visitor_method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 });
        let Type::Array(element) = base.ty.expect("checked predicate receiver") else {
            unreachable!()
        };
        let literal =
            self.expression_shape(base) == Some(crate::fixed_storage::Shape::LiteralStrings);
        debug_assert!(!literal || parameter_count < 3);
        let mut arguments = Vec::new();
        if parameter_count > 0 {
            arguments.push(
                if element.ty().is_copy() {
                    "*predicate_value"
                } else {
                    "predicate_value"
                }
                .to_owned(),
            );
        }
        if parameter_count > 1 {
            arguments.push("predicate_index".to_owned());
        }
        if parameter_count > 2 {
            arguments.push("predicate_array".to_owned());
        }
        let arguments = self.array_callback_arguments(callback_type, visitor_method, arguments, &[element.ty(), Type::Int, base.ty.expect("checked array")]);
        let call = if self.array_callback_counted_method(callback_type, visitor_method) {
                format!("predicate_callback.call{}_with_count({}{})", visitor_method.expect("checked counted method").id, if visitor_method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 }, if arguments.is_empty() { String::new() } else { format!(", {}", arguments.join(", ")) })
            } else if self.counted_calls && visitor_method.is_none() {
            if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                self.counted_invocation(invocation, "predicate_callback", &tuple(arguments.into_iter()), if visitor_method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            } else {
                self.counted_call("predicate_callback", &arguments.join(", "), if visitor_method.is_some_and(|method| method.access == Mode::Mutate) { 2 } else { 3 })
            }
        } else if matches!(callback_type, Type::Callback(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
            format!(
                "{invocation}(predicate_callback, {})",
                tuple(arguments.into_iter())
            )
        } else if let Some(method) = visitor_method {
            format!("predicate_callback.call{}({})", method.id, arguments.join(", "))
        } else {
            format!("predicate_callback.call({})", arguments.join(", "))
        };
        let index = if parameter_count > 1 {
            "let predicate_index = i64::try_from(predicate_position).expect(\"array callback index exceeds int range\"); "
        } else {
            ""
        };
        let literal_value = if literal {
            "let predicate_literal = String::Borrowed(predicate_value); let predicate_value = &predicate_literal; "
        } else {
            ""
        };
        let iterator = if literal {
            "predicate_array.literal_iter()"
        } else if visitor_method.is_some_and(|method| method.access == Mode::Mutate) {
            "predicate_array.iter_mut()"
        } else {
            "predicate_array.iter()"
        };
        let body = format!("{index}{literal_value}{call}");
        let mut operation = match method {
            CallTarget::ArraySome | CallTarget::ArrayEvery => {
                let operation = if method == CallTarget::ArraySome {
                    "any"
                } else {
                    "all"
                };
                format!(
                    "{iterator}.enumerate().{operation}(|(predicate_position, predicate_value)| {{ {body} }})"
                )
            }
            CallTarget::ArrayFind => format!(
                "{iterator}.enumerate().find_map(|(predicate_position,predicate_value)|{{ {index}{literal_value}if {call}{{Some(predicate_position)}}else{{None}} }})"
            ),
            CallTarget::ArrayFindIndex => format!(
                "{iterator}.enumerate().find_map(|(predicate_position, predicate_value)| {{ {index}{literal_value}if {call} {{ Some(predicate_position) }} else {{ None }} }}).map_or(-1i64, |index| i64::try_from(index).expect(\"array predicate result exceeds int range\"))"
            ),
            CallTarget::ArrayFindLastIndex | CallTarget::ArrayFindLast => {
                // ArrayIter need not be double-ended: reverse original positions and borrow each item.
                let selection = if literal {
                    "let predicate_literal = predicate_array.get(predicate_position).expect(\"array predicate index in bounds\"); let predicate_value = &predicate_literal; "
                } else if visitor_method.is_some_and(|method| method.access == Mode::Mutate) {
                    "let predicate_value = predicate_array.get_mut(predicate_position).expect(\"array predicate index in bounds\"); "
                } else {
                    "let predicate_value = predicate_array.get(predicate_position).expect(\"array predicate index in bounds\"); "
                };
                let selection = format!(
                    "(0..predicate_array.len()).rev().find(|&predicate_position| {{ {index}{selection}{call} }})"
                );
                if method == CallTarget::ArrayFindLast {
                    selection
                } else {
                    format!(
                        "{selection}.map_or(-1i64, |index| i64::try_from(index).expect(\"array predicate result exceeds int range\"))"
                    )
                }
            }
            _ => unreachable!("checked array predicate"),
        };
        let checked_call = self.registered_traversal_call(call.clone(), callback);
        if checked_call != call {
            let reverse = matches!(method, CallTarget::ArrayFindLast | CallTarget::ArrayFindLastIndex);
            let positions = if reverse { "(0..predicate_array.len()).rev()" } else { "0..predicate_array.len()" };
            let selection = if literal {
                "let predicate_literal=predicate_array.get(predicate_position).expect(\"array predicate index in bounds\");let predicate_value=&predicate_literal;"
            } else if visitor_method.is_some_and(|method| method.access == Mode::Mutate) {
                "let predicate_value=predicate_array.get_mut(predicate_position).expect(\"array predicate index in bounds\");"
            } else {
                "let predicate_value=predicate_array.get(predicate_position).expect(\"array predicate index in bounds\");"
            };
            let (initial, update, finish) = match method {
                CallTarget::ArraySome => ("false", "if predicate_matches { predicate_result=true; break; }", "predicate_result"),
                CallTarget::ArrayEvery => ("true", "if !predicate_matches { predicate_result=false; break; }", "predicate_result"),
                CallTarget::ArrayFind | CallTarget::ArrayFindLast => ("None::<usize>", "if predicate_matches { predicate_result=Some(predicate_position); break; }", "predicate_result"),
                CallTarget::ArrayFindIndex | CallTarget::ArrayFindLastIndex => ("None::<usize>", "if predicate_matches { predicate_result=Some(predicate_position); break; }", "predicate_result.map_or(-1i64, |position| i64::try_from(position).expect(\"array predicate result exceeds int range\"))"),
                _ => unreachable!("checked array predicate"),
            };
            operation=format!("{{ let mut predicate_result={initial}; for predicate_position in {positions} {{ {index}{selection}let predicate_matches={checked_call};{update} }} {finish} }}");
        }
        if finding {
            let setup = if owned {
                format!("let mut predicate_owned={receiver};let predicate_array=&predicate_owned;")
            } else {
                format!("let predicate_array={receiver};")
            };
            let result = match expr.ty.expect("optional predicate result") {
                Type::Undefined => "let _=predicate_position; TsrUndefined".to_owned(),
                Type::Union(id) | Type::BorrowedUnion(id) => {
                    let borrowed = matches!(expr.ty, Some(Type::BorrowedUnion(_)));
                    let members = &self.program.unions[id.index()].members;
                    let absent = members
                        .iter()
                        .position(|ty| *ty == Type::Undefined)
                        .expect("undefined member");
                    let present = self.optional_element_present(id, element.ty(), borrowed, "payload");
                    let selected = if element.ty().is_copy() {
                        "predicate_array.get(position).copied()"
                    } else if borrowed {
                        "predicate_array.get(position)"
                    } else {
                        "Some(predicate_owned.remove(position))"
                    };
                    let tag = if borrowed { "ViewUnion" } else { "Union" };
                    let nil = if borrowed {
                        "&TsrUndefined"
                    } else {
                        "TsrUndefined"
                    };
                    format!(
                        "match predicate_position.and_then(|position|{selected}){{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}",
                        id.index()
                    )
                }
                _ => unreachable!("optional predicate"),
            };
            return format!(
                "{{{setup}let predicate_callback={environment};let predicate_position={operation};{result}}}"
            );
        }
        format!(
            "{{ let predicate_array = {receiver}; let predicate_callback = {environment}; {operation} }}"
        )
    }

    fn array_concat(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        args: &[Expr],
        span: crate::diagnostic::Span,
    ) -> String {
        let result = base.ty.expect("checked concat receiver");
        let Type::Array(element) = result else {
            unreachable!()
        };
        let receiver = self.expr(fun, base, Mode::Read);
        let mut output = format!("{{ let concat_array = {receiver}; ");
        for (index, argument) in args.iter().enumerate() {
            let argument_type = argument.ty.expect("checked concat argument");
            let value = self.expr(
                fun,
                argument,
                if argument_type.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
            );
            if matches!(argument_type, Type::Array(_)) {
                let _ = write!(output, "let concat_argument{index} = {value}; ");
            } else {
                let reference = if argument_type.is_copy() { "" } else { "&" };
                let _ = write!(
                    output,
                    "let concat_argument{index}: {reference}{} = {value}; ",
                    self.native_type(argument_type)
                );
            }
        }
        let failure = format!(
            "concat capacity overflow at source bytes {}..{}",
            span.start, span.end
        );
        output.push_str("let mut concat_capacity = concat_array.len(); ");
        for (index, argument) in args.iter().enumerate() {
            let count = if matches!(argument.ty, Some(Type::Array(_))) {
                format!("concat_argument{index}.len()")
            } else {
                "1usize".to_owned()
            };
            let _ = write!(
                output,
                "concat_capacity = concat_capacity.checked_add({count}).expect({failure:?}); "
            );
        }
        let _ = write!(
            output,
            "let mut concat_values: Vec<{}> = Vec::with_capacity(concat_capacity); ",
            self.native_type(element.ty())
        );
        let append = |expression: &Expr, name: &str| {
            if self.expression_shape(expression)
                == Some(crate::fixed_storage::Shape::LiteralStrings)
            {
                format!("concat_values.extend({name}.literal_iter().map(String::Borrowed)); ")
            } else {
                format!("concat_values.extend({name}.iter().cloned()); ")
            }
        };
        output.push_str(&append(base, "concat_array"));
        for (index, argument) in args.iter().enumerate() {
            let name = format!("concat_argument{index}");
            if matches!(argument.ty, Some(Type::Array(_))) {
                output.push_str(&append(argument, &name));
            } else if argument.ty.expect("checked concat argument").is_copy() {
                let _ = write!(output, "concat_values.push({name}); ");
            } else {
                let _ = write!(output, "concat_values.push(Clone::clone({name})); ");
            }
        }
        let wrapped = if numeric_array(result) {
            "NumericArray::Heap(concat_values)".to_owned()
        } else if inline_array(result) {
            "Array::Heap(concat_values)".to_owned()
        } else if let Type::Array(ArrayElement::Record(id, 0)) = result {
            if self.record_runtime.recursive(id) {
                format!("RecordArray{id}(concat_values)")
            } else if self.small_record(id) {
                "Array::Heap(concat_values)".to_owned()
            } else {
                "concat_values".to_owned()
            }
        } else {
            "concat_values".to_owned()
        };
        let _ = write!(output, "{wrapped} }}");
        output
    }

    fn array_entries(&self, fun: &CheckedFunction, base: &Expr, result: Type, pair: usize) -> String {
        let receiver = self.expr(fun,base,Mode::Read);
        let literal = self.expression_shape(base)==Some(crate::fixed_storage::Shape::LiteralStrings);
        let iterator = if literal { "entries_source.literal_iter()" } else { "entries_source.iter()" };
        let value = if literal { "String::Borrowed(entries_value)" } else { "Clone::clone(entries_value)" };
        let result = self.native_type(result);
        format!("{{ let entries_source = {receiver}; let mut entries = <{result}>::from([]); for (entries_index,entries_value) in {iterator}.enumerate() {{ entries.push(Record{pair} {{ field0: i64::try_from(entries_index).expect(\"entries index exceeds int range\"), field1: {value} }}); }} entries }}")
    }

    fn array_slice(&self, fun: &CheckedFunction, base: &Expr, args: &[Expr]) -> String {
        let array = self.expr(fun, base, Mode::Read);
        let start = args
            .first()
            .map_or_else(|| "0i64".into(), |arg| self.expr(fun, arg, Mode::Copy));
        let end = args.get(1).map_or_else(
            || "None".into(),
            |arg| format!("Some({})", self.expr(fun, arg, Mode::Copy)),
        );
        let array_type = base.ty.expect("checked array slice");
        let fixed = self.expression_shape(base);
        let compact = fixed.is_none()
            && !numeric_array(array_type)
            && (inline_array(array_type)
                || matches!(array_type, Type::Array(ArrayElement::Record(index, 0)) if self.small_record(index)));
        let values = match fixed {
            Some(crate::fixed_storage::Shape::LiteralStrings) =>
                "slice_array.values[slice_start..slice_end].iter().map(|value| String::Borrowed(value)).collect::<Vec<_>>()".into(),
            Some(crate::fixed_storage::Shape::ScalarArray { .. } | crate::fixed_storage::Shape::Array { .. }) =>
                "slice_array[slice_start..slice_end].to_vec()".into(),
            _ if numeric_array(array_type) =>
                "slice_array.as_slice()[slice_start..slice_end].to_vec()".into(),
            _ if compact =>
                "slice_array.tsr_slice(slice_start, slice_end)".into(),
            _ => "slice_array.as_slice()[slice_start..slice_end].to_vec()".to_owned(),
        };
        let result = if compact {
            values
        } else if numeric_array(array_type) {
            format!("NumericArray::Heap({values})")
        } else if inline_array(array_type) {
            format!("Array::Heap({values})")
        } else if let Type::Array(ArrayElement::Record(index, 0)) = array_type {
            if self.record_runtime.recursive(index) {
                format!("RecordArray{index}({values})")
            } else if self.small_record(index) {
                format!("Array::Heap({values})")
            } else {
                values
            }
        } else {
            values
        };
        format!(
            "{{ let slice_array = {array}; let slice_from: i64 = {start}; let slice_to: Option<i64> = {end}; let slice_start = tsr_search_start(slice_array.len(), slice_from); let slice_end = slice_to.map_or(slice_array.len(), |end| tsr_search_start(slice_array.len(), end)).max(slice_start); {result} }}"
        )
    }

    fn temporary_at_receiver(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => Self::temporary_at_receiver(consequent) || Self::temporary_at_receiver(alternate),
            _ => Self::temporary_storage(expr),
        }
    }

    fn array_at(&self, fun: &CheckedFunction, expr: &Expr, base: &Expr, index: &Expr) -> String {
        let Type::Array(element) = base.ty.expect("at receiver") else {
            unreachable!()
        };
        let borrowed = matches!(expr.ty, Some(Type::BorrowedUnion(_)));
        let stable = !Self::temporary_storage(base);
        let setup = if matches!(base.kind, ExprKind::Conditional { .. })
            && (borrowed || !Self::temporary_at_receiver(base))
        {
            format!(
                "let at_array={}; let at_index:i64={};",
                self.expr(fun, base, Mode::Read),
                self.expr(fun, index, Mode::Copy)
            )
        } else if stable {
            let selection = self.storage_selection(fun, base, false);
            format!(
                "{} let at_index:i64={}; let at_array={};",
                selection.setup,
                self.expr(fun, index, Mode::Copy),
                selection.shared
            )
        } else {
            format!(
                "let mut at_owned={}; let at_index:i64={}; let at_array=&mut at_owned;",
                self.expr(fun, base, Mode::Consume),
                self.expr(fun, index, Mode::Copy)
            )
        };
        let normalized = "let at_selected=if at_index>=0 { usize::try_from(at_index).ok().filter(|&i|i<at_array.len()) } else { usize::try_from(at_index.unsigned_abs()).ok().and_then(|distance|at_array.len().checked_sub(distance)).filter(|&i|i<at_array.len()) };";
        let selected = if element.ty().is_copy() {
            "at_array.get(at_selected).copied()".to_owned()
        } else if borrowed {
            "at_array.get(at_selected)".to_owned()
        } else {
            "Some(at_array.remove(at_selected))".to_owned()
        };
        let result = match expr.ty.expect("at result") {
            Type::Undefined => "let _=at_selected; TsrUndefined".to_owned(),
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let members = &self.program.unions[id.index()].members;
                let absent = members
                    .iter()
                    .position(|ty| *ty == Type::Undefined)
                    .unwrap();
                let present = self.optional_element_present(id, element.ty(), borrowed, "payload");
                let tag = if borrowed { "ViewUnion" } else { "Union" };
                let absent_payload = if borrowed {
                    "&TsrUndefined"
                } else {
                    "TsrUndefined"
                };
                format!(
                    "let at_result=at_selected.and_then(|at_selected|{selected}); match at_result {{ Some(payload)=>{present}, None=>{tag}{}::V{absent}({absent_payload}) }}",
                    id.index()
                )
            }
            _ => unreachable!(),
        };
        format!("{{ {setup} {normalized} {result} }}")
    }

    /// An independent owned copy of a readable value in its ordinary storage.
    #[inline(never)]
    fn object_spread_value_with(&self,result:usize,args:&[Expr],copied:Vec<String>)->String{
        let mut code=String::from("{");let mut last=std::collections::HashMap::new();
        let count=self.program.records[result].fields.len();
        let _=write!(code,"let mut spread_order=[0usize;{count}];let mut spread_seen=[false;{count}];let mut spread_next=0usize;");
        for (part,(arg,copied)) in args.iter().zip(copied).enumerate(){
            let _=write!(code,"let spread_part{part}={copied};");
            let Type::Record(id)=arg.ty.expect("checked spread part")else{unreachable!()};
            let mut arms=String::new();
            for (field,definition) in self.program.records[id].fields.iter().enumerate(){
                let output=self.fields[result][definition.name.text.as_str()];
                last.insert(definition.name.text.clone(),(part,field));
                let _=write!(arms,"{field}=>{output},");
            }
            if arms.is_empty(){continue;}
            let _=write!(code,"for spread_index in spread_part{part}.key_order{{let output=match spread_index{{{arms}_=>unreachable!()}};if !spread_seen[output]{{spread_seen[output]=true;spread_order[spread_next]=output;spread_next+=1;}}}}");
        }
        let fields=self.program.records[result].fields.iter().enumerate().map(|(index,definition)|{let (part,field)=last[&definition.name.text];let Type::Record(source)=args[part].ty.expect("checked spread part") else{unreachable!()};let copied=format!("spread_part{part}.field{field}");let adapted=self.element_alternative(self.program.records[source].fields[field].ty.expect("checked source field"),definition.ty.expect("checked result field"),&copied);format!("field{index}:{adapted}")}).collect::<Vec<_>>().join(",");
        let _=write!(code,"Record{result}{{{fields}{}key_order:spread_order}}}}",if fields.is_empty(){""}else{","});code
    }

    fn clone_argument(&self, fun: &CheckedFunction, value: &Expr) -> String {
        if let Some(snapshot)=self.record_parameter_owned_clone(fun,value){return snapshot;}
        let argument = self.expr(fun, value, Mode::Read);
        self.finish_clone_argument(fun,value,argument)
    }
    fn finish_clone_argument(&self,fun:&CheckedFunction,value:&Expr,argument:String)->String{
        if self.record_parameter_borrow_leaf(fun,value){return format!("Clone::clone({argument})");}
        match self.expression_shape(value) {
            Some(crate::fixed_storage::Shape::LiteralStrings) => format!("({argument}).clone_owned()"),
            Some(crate::fixed_storage::Shape::Record { .. }) => format!("({argument}).clone_owned()"),
            Some(crate::fixed_storage::Shape::ScalarArray { scalar: crate::fixed_storage::Scalar::Number, .. }) => {
                format!("Vec::from(Clone::clone({argument}))")
            }
            Some(crate::fixed_storage::Shape::ScalarArray { .. }) => format!("NumericArray::from(Clone::clone({argument}))"),
            Some(crate::fixed_storage::Shape::Array { record, .. }) => format!("RecordArray{record}::from(Clone::clone({argument}))"),
            None => format!("Clone::clone({argument})"),
        }
    }

    /// A value of type `from` placed into the element type `to`. A union
    /// element takes the alternative of the value; a union value keeps its tag.
    fn element_alternative(&self, from: Type, to: Type, value: &str) -> String {
        let Type::Union(target) = to else { return value.to_owned() };
        if from == to {
            return value.to_owned();
        }
        let members = &self.program.unions[target.index()].members;
        let position = |member: &Type| members.iter().position(|candidate| candidate == member).expect("element alternative");
        match from {
            Type::Union(source) => {
                let arms = self.program.unions[source.index()]
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, member)| format!("Union{}::V{index}(payload)=>Union{}::V{}({})", source.index(), target.index(), position(member),self.union_transfer_payload(source,target,*member,"payload")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("match {value} {{ {arms} }}")
            }
            _ => format!("Union{}::V{}({})", target.index(), position(&from),self.union_store_payload(target,from,value)),
        }
    }

    /// Object.values and Object.entries copy each field in the value's own
    /// key order. `pair` is the [string, value] tuple record for entries.
    fn object_values(&self, fun: &CheckedFunction, expr: &Expr, value: &Expr, pair: Option<usize>) -> String {
        let result = expr.ty.expect("checked object values result");
        let Some(Type::Record(record)) = value.ty else {
            // An array gives an independent copy of itself.
            return self.clone_argument(fun, value);
        };
        let source = self.expr(fun, value, Mode::Read);
        // A compact local record first takes its ordinary layout.
        let setup = if matches!(self.expression_shape(value), Some(crate::fixed_storage::Shape::Record { .. })) {
            format!("let object_owner = ({source}).clone_owned(); let object_source = &object_owner;")
        } else {
            format!("let object_source = {source};")
        };
        let element = match pair {
            Some(pair) => self.program.records[pair].fields[1].ty.expect("pair value type"),
            None => {
                let Type::Array(leaf) = result else { unreachable!("values array") };
                leaf.ty()
            }
        };
        let definition = &self.program.records[record];
        let tuple = definition.is_tuple();
        let items = definition
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let copied = if matches!(field.ty,Some(Type::Record(_))) {
                    let selected=Expr::checked(ExprKind::Field(Box::new(value.clone()),field.name.text.clone()),value.span,field.ty);
                    self.record_parameter_owned_clone(fun,&selected).unwrap_or_else(||format!("Clone::clone(&object_source.field{index})"))
                }else{format!("Clone::clone(&object_source.field{index})")};
                let item = self.element_alternative(field.ty.expect("checked field"), element, &copied);
                match pair {
                    Some(pair) => {
                        let name = if tuple { index.to_string() } else { field.name.text.clone() };
                        format!("Record{pair} {{ field0: String::Borrowed({name:?}), field1: {item} }}")
                    }
                    None => item,
                }
            })
            .collect::<Vec<_>>();
        let fill = if tuple {
            items.iter().map(|item| format!("object_result.push({item}); ")).collect::<String>()
        } else {
            let arms = items.iter().enumerate().map(|(index, item)| format!("{index} => object_result.push({item}), ")).collect::<String>();
            format!("for object_index in object_source.key_order.iter() {{ match *object_index {{ {arms}_ => unreachable!(\"record field index\") }} }}")
        };
        format!("{{ {setup} let mut object_result = <{}>::from([]); {fill} object_result }}", self.native_type(result))
    }

    /// The present arm of an optional element result. `name` holds the selected
    /// element. A union element keeps its own tag inside the wider result.
    fn optional_element_present(&self, result: UnionId, element: Type, borrowed: bool, name: &str) -> String {
        let tag = if borrowed { "ViewUnion" } else { "Union" };
        let members = &self.program.unions[result.index()].members;
        let Type::Union(source) = element else {
            let present = members
                .iter()
                .position(|member| *member == element)
                .expect("optional element member");
            return format!("{tag}{}::V{present}({name})", result.index());
        };
        let value = if borrowed { format!("({name}).tsr_data_view()") } else { name.to_owned() };
        if source == result {
            return value;
        }
        let arms = self.program.unions[source.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let to = members
                    .iter()
                    .position(|candidate| candidate == member)
                    .expect("optional element alternative");
                format!("{tag}{}::V{index}(payload)=>{tag}{}::V{to}(payload)", source.index(), result.index())
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("match {value} {{ {arms} }}")
    }

    fn array_optional_removal(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        base: &Expr,
        shift: bool,
    ) -> String {
        let (setup, receiver) = if Self::temporary_storage(base) {
            (String::new(), self.conditional_arm(fun, base, Mode::Mutate))
        } else {
            let selection = self.storage_selection(fun, base, false);
            (selection.setup, selection.exclusive)
        };
        let operation = if shift {
            "if removal_array.len() == 0 { None } else { Some(removal_array.remove(0)) }"
        } else {
            "removal_array.pop()"
        };
        let result = match expr.ty.expect("checked optional removal result") {
            Type::Undefined => "let _ = removal_result; TsrUndefined".to_owned(),
            Type::Union(union) => {
                let members = &self.program.unions[union.index()].members;
                let absent = members
                    .iter()
                    .position(|member| *member == Type::Undefined)
                    .expect("optional undefined member");
                let Type::Array(element) = base.ty.expect("array receiver") else {
                    unreachable!()
                };
                let target = members.iter().copied().find(|member| matches!(member, Type::Record(_))).unwrap_or(element.ty());
                let (present_type, projected) = if element.ty() != target && matches!((element.ty(), target), (Type::Record(_), Type::Record(_))) {
                    (target, self.map_entry_payload_projection(element.ty(), target, "value"))
                } else { (element.ty(), "value".to_owned()) };
                let present = self.optional_element_present(union, present_type, false, &projected);
                format!(
                    "match removal_result {{ Some(value)=>{present},None=>Union{}::V{absent}(TsrUndefined) }}",
                    union.index()
                )
            }
            _ => unreachable!("canonical optional result"),
        };
        format!(
            "{{ {setup}let removal_array = &mut ({receiver}); let removal_result = {operation}; {result} }}"
        )
    }

    fn array_remove(
        &self,
        fun: &CheckedFunction,
        base: &Expr,
        index: &Expr,
        span: crate::diagnostic::Span,
    ) -> String {
        if Self::temporary_storage(base) {
            let array = self.expr(fun, base, Mode::Mutate);
            let index = self.expr(fun, index, Mode::Copy);
            let failure = Self::array_failure(span);
            let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, span)
                .unwrap_or_else(|| format!("panic!({failure:?})"));
            return format!(
                "{{ let removal_array = {array}; let removal_index = {index}; let removal_index = match usize::try_from(removal_index) {{ Ok(index) => index, Err(_) => {failed} }}; if removal_index >= removal_array.len() {{ {failed}; }} removal_array.remove(removal_index) }}"
            );
        }
        let selection = self.storage_selection(fun, base, false);
        let index = self.expr(fun, index, Mode::Copy);
        let failure = Self::array_failure(span);
        let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, span)
            .unwrap_or_else(|| format!("panic!({failure:?})"));
        format!(
            "{{ {}let removal_index = {index}; let removal_index = match usize::try_from(removal_index) {{ Ok(index) => index, Err(_) => {failed} }}; let removal_array = {}; if removal_index >= removal_array.len() {{ {failed}; }} removal_array.remove(removal_index) }}",
            selection.setup, selection.exclusive
        )
    }

    fn restored_capture(&self,fun:&CheckedFunction,index:usize)->bool {
        self.restored_slots.contains(fun,index) || self.finally_moves.contains(fun,index)
    }

    fn binding(&self, fun: &CheckedFunction, index: usize, mode: Mode) -> String {
        let b = &fun.bindings[index];
        if let Some(owner) = b.name.strip_prefix(crate::data_lowering::ACCESS) {
            let id = self.program.functions.iter().position(|function| function.name == owner).expect("checked data owner");
            return match mode { Mode::Read => format!("&*TSR_DATA{id}"), Mode::Copy | Mode::Consume => format!("*TSR_DATA{id}"), Mode::Mutate => unreachable!("readonly data") };
        }
        if self.restored_capture(fun,index) {
            return match mode {
                Mode::Consume=>format!("value{index}.take().expect(\"initialized capture\")"),
                Mode::Copy if b.ty.is_copy()=>format!("*value{index}.as_ref().expect(\"initialized capture\")"),
                Mode::Read|Mode::Copy=>format!("value{index}.as_ref().expect(\"initialized capture\")"),
                Mode::Mutate=>format!("value{index}.as_mut().expect(\"initialized capture\")"),
            };
        }
        if let Some(value)=self.owned_record_value(fun,index,&[],mode){return value;}
        if let Some(value)=self.record_parameter_borrowed_value(fun,index,&[],mode){return value;}
        if matches!(b.ty, Type::BorrowedUnion(_))
            || (matches!(b.ty,Type::Union(id) if !id.copy)
                && b.parameter
                && b.mode == Mode::Read
                && !b.capture_reference)
        {
            return format!("value{index}");
        }
        let reference = match mode {
            Mode::Read => "&",
            Mode::Mutate => "&mut ",
            Mode::Consume | Mode::Copy => "",
        };
        let dereference = if (matches!(mode, Mode::Read | Mode::Mutate)
            && ((b.parameter && matches!(b.mode, Mode::Read | Mode::Mutate)
                && (!(b.omission.is_omittable() || (b.mode==Mode::Mutate && self.multiple_data_union(b.ty))) || (b.initializer.is_some() && !matches!(b.ty,Type::Union(_)|Type::Callback(_)))))
                || (b.callback_reference && b.initializer.is_none()
                    && !(b.parameter && (b.omission.is_omittable() || (b.mode==Mode::Mutate && self.multiple_data_union(b.ty))) && b.ty.union().is_some()))
                || b.borrowed_element.is_some()
                || b.borrowed_place.is_some()
                || b.mutable_place))
            || (b.ty.is_copy() && b.capture_reference)
        {
            "*"
        } else {
            ""
        };
        format!("{reference}{dereference}value{index}")
    }

    /// The character at a position of a string, as a new string. Operands
    /// are bound by a match scrutinee so a temporary text outlives the read.
    #[inline(never)]
    fn string_index(&self, fun: &CheckedFunction, expr: &Expr, base: &Expr, index: &Expr, mode: Mode) -> String {
        let failure = format!(
            "string index out of range at source bytes {}..{}",
            expr.span.start, expr.span.end
        );
        let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, expr.span)
            .unwrap_or_else(|| format!("panic!({failure:?})"));
        let text = self.expr(fun, base, Mode::Read);
        let position = self.expr(fun, index, Mode::Copy);
        Self::access_wrapped(expr, mode, format!(
            "(match ({text}, {position}) {{ (indexed_text, indexed_position) => {{ let indexed_text: &String = indexed_text; let indexed_text: &str = &**indexed_text; let indexed_position: i64 = indexed_position; match usize::try_from(indexed_position).ok().and_then(|position| indexed_text.chars().nth(position)) {{ Some(character) => String::Owned(character.to_string()), None => {failed} }} }} }})"
        ))
    }

    fn array_failure(span: crate::diagnostic::Span) -> String {
        format!(
            "array index out of bounds at source bytes {}..{}",
            span.start, span.end
        )
    }

    // Primitive arguments are bound once, in source order, before any Math
    // operation or exceptional-value guard can short-circuit.
    fn math_approx_call(&self, fun: &CheckedFunction, target: CallTarget, args: &[Expr]) -> String {
        let mut bindings = String::new();
        for (index, argument) in args.iter().enumerate() {
            let value = self.expr(fun, argument, Mode::Copy);
            write!(bindings, "let math_arg{index}: f64 = {value}; ").unwrap();
        }
        let result = match target {
            CallTarget::MathAcos => "math_arg0.acos()".into(),
            CallTarget::MathAcosh => "math_arg0.acosh()".into(),
            CallTarget::MathAsin => "math_arg0.asin()".into(),
            CallTarget::MathAsinh => "math_arg0.asinh()".into(),
            CallTarget::MathAtan => "math_arg0.atan()".into(),
            CallTarget::MathAtanh => "math_arg0.atanh()".into(),
            CallTarget::MathCbrt => "math_arg0.cbrt()".into(),
            CallTarget::MathCos => "math_arg0.cos()".into(),
            CallTarget::MathCosh => "math_arg0.cosh()".into(),
            CallTarget::MathExp => "math_arg0.exp()".into(),
            CallTarget::MathExpm1 => "math_arg0.exp_m1()".into(),
            CallTarget::MathLog => "math_arg0.ln()".into(),
            CallTarget::MathLog10 => "math_arg0.log10()".into(),
            CallTarget::MathLog1p => "math_arg0.ln_1p()".into(),
            CallTarget::MathLog2 => "math_arg0.log2()".into(),
            CallTarget::MathSin => "math_arg0.sin()".into(),
            CallTarget::MathSinh => "math_arg0.sinh()".into(),
            CallTarget::MathTan => "math_arg0.tan()".into(),
            CallTarget::MathTanh => "math_arg0.tanh()".into(),
            CallTarget::MathAtan2 => "math_arg0.atan2(math_arg1)".into(),
            CallTarget::MathPow => "tsr_math_pow(math_arg0, math_arg1)".into(),
            CallTarget::MathHypot => {
                let values = (0..args.len())
                    .map(|index| format!("math_arg{index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tsr_math_hypot(&[{values}])")
            }
            _ => unreachable!("checked approximate Math operation"),
        };
        format!("{{ {bindings}{result} }}")
    }

    fn math_exact_call(&self, fun: &CheckedFunction, target: CallTarget, args: &[Expr]) -> String {
        let mut bindings = String::new();
        for (index, argument) in args.iter().enumerate() {
            let value = self.expr(fun, argument, Mode::Copy);
            write!(bindings, "let math_arg{index}: f64 = {value}; ").unwrap();
        }
        let result = match target {
            CallTarget::MathRound => "tsr_math_round(math_arg0)".into(),
            CallTarget::MathSign => "tsr_math_sign(math_arg0)".into(),
            CallTarget::MathFround => "(math_arg0 as f32) as f64".into(),
            CallTarget::MathF16round => "tsr_math_f16round(math_arg0)".into(),
            CallTarget::MathRandom => "tsr_math_random()".into(),
            CallTarget::MathClz32 => "tsr_math_uint32(math_arg0).leading_zeros() as f64".into(),
            CallTarget::MathImul => "(tsr_math_uint32(math_arg0).wrapping_mul(tsr_math_uint32(math_arg1)) as i32) as f64".into(),
            CallTarget::MathMin | CallTarget::MathMax => {
                let values = (0..args.len()).map(|index| format!("math_arg{index}")).collect::<Vec<_>>().join(", ");
                format!("tsr_math_extreme(&[{values}], {})", target == CallTarget::MathMax)
            }
            _ => unreachable!("checked exact Math operation"),
        };
        format!("{{ {bindings}{result} }}")
    }

    fn expr(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        if let Some(code) = self.take_rendered_receiver(expr, mode) { return code; }
        self.emit_expression_work(fun, expr, mode)
    }

    fn finish_expression_work(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode,
        depth: usize, first_temporary: usize, code: String) -> String {
        // Projection, conditional arms and injection already lower borrowed union views.
        let code = if let Some(Type::Union(id)) = expr.ty
            && !id.copy
            && mode == Mode::Read
            && !matches!(&expr.kind,ExprKind::Field(base,_) if matches!(base.ty,Some(Type::Union(_))))
            && !matches!(
                expr.kind,
                ExprKind::Optional { .. }
                    | ExprKind::TypeAssertion { .. }
                    | ExprKind::SpreadValue(_)
                    | ExprKind::Project { .. }
                    | ExprKind::Conditional { .. }
                    | ExprKind::Inject { .. }
                    | ExprKind::Binary { op: BinaryOp::And | BinaryOp::Or | BinaryOp::Coalesce, .. }
                    | ExprKind::Unary { op: UnaryOp::NonNull | UnaryOp::Await | UnaryOp::AwaitCallee, .. }
            )
            && !expr.binding.is_some_and(|i| {
                fun.bindings[i].parameter
                    && fun.bindings[i].mode == Mode::Read
                    && !fun.bindings[i].capture_reference
            }) {
            let method = if self.callback_payload(Type::Union(id)).is_some() { "tsr_view" } else { "tsr_data_view" };
            format!("({code}).{method}()")
        } else {
            code
        };
        self.expression_depth.set(depth);
        if depth == 0
            && (matches!(mode, Mode::Copy | Mode::Consume)
                || matches!(expr.ty, Some(Type::BorrowedUnion(_))))
        {
            let declarations = self
                .conditional_temporaries
                .borrow_mut()
                .drain(first_temporary..)
                .collect::<Vec<_>>()
                .join(" ");
            if !declarations.is_empty() {
                return format!("{{ {declarations} {code} }}");
            }
        }
        code
    }

    fn optional_chain(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        base: &Expr,
        body: &Expr,
        state: (
            usize,
            Option<UnionId>,
            Mode,
            crate::syntax::NullishBranch,
            bool,
        ),
        mode: Mode,
    ) -> String {
        if mode == Mode::Read
            && !expr.ty.is_some_and(Type::is_copy)
            && !matches!(expr.ty, Some(Type::BorrowedUnion(_)))
            && !Self::borrowed_place_expression(fun, body)
        {
            let owned = self.optional_chain(fun, expr, base, body, state, Mode::Consume);
            return if expr.ty.and_then(Type::union).is_some() {
                format!("(&({owned})).tsr_view()")
            } else {
                format!("&({owned})")
            };
        }
        let (slot, union, access, branch, owned) = state;
        let result = expr.ty.expect("optional result");
        let view = matches!(result, Type::BorrowedUnion(_))
            || (result.union().is_some_and(|id| !id.copy) && mode == Mode::Read);
        let body_mode = if view { Mode::Read } else { mode };
        let code = self.conditional_arm(fun, body, body_mode);
        let actual = body.ty.expect("optional body");
        let prefix = if view { "ViewUnion" } else { "Union" };
        let selected = if let Some(to) = result.union() {
            if let Some(from) = actual.union() {
                if from == to {
                    code
                } else {
                    let source_view =
                        matches!(actual, Type::BorrowedUnion(_)) || (!from.copy && view);
                    let arms = self.program.unions[from.index()]
                        .members
                        .iter()
                        .enumerate()
                        .map(|(i, member)| {
                            let j = self.program.unions[to.index()]
                                .members
                                .iter()
                                .position(|ty| {
                                    ty == member
                                        || matches!(ty, Type::Callback(_))
                                            && matches!(
                                                member,
                                                Type::Closure(_) | Type::Callback(_)
                                            )
                                })
                                .expect("optional subset");
                            format!(
                                "{}{}::V{i}(value)=>{prefix}{}::V{j}({})",
                                if source_view { "ViewUnion" } else { "Union" },
                                from.index(),to.index(),if !view&&!source_view{self.union_transfer_payload(from,to,*member,"value")}else{"value".into()}
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("match {code} {{{arms}}}")
                }
            } else {
                let logical = if actual == Type::Void {
                    Type::Undefined
                } else {
                    actual
                };
                let i = self.program.unions[to.index()]
                    .members
                    .iter()
                    .position(|ty| *ty == logical)
                    .expect("optional member");
                let code = if actual == Type::Void {
                    format!("{{{code}; TsrUndefined}}")
                } else {
                    code
                };
                let code = if view && logical.is_copy() {
                    format!("&({code})")
                } else {
                    code
                };
                format!("{prefix}{}::V{i}({})", to.index(),if view{code}else{self.union_store_payload(to,logical,&code)})
            }
        } else if actual == Type::Void && result == Type::Undefined {
            format!("{{{code}; TsrUndefined}}")
        } else {
            code
        };
        let missing = if branch == crate::syntax::NullishBranch::Present {
            "unreachable!(\"checked optional presence\")".into()
        } else if let Some(id) = result.union() {
            let i = self.program.unions[id.index()]
                .members
                .iter()
                .position(|ty| *ty == Type::Undefined)
                .expect("optional undefined");
            format!(
                "{prefix}{}::V{i}({})",
                id.index(),
                if view {
                    "&TsrUndefined"
                } else {
                    "TsrUndefined"
                }
            )
        } else {
            "TsrUndefined".into()
        };
        let access = if base.ty.is_some_and(Type::is_copy) {
            Mode::Copy
        } else if owned {
            Mode::Consume
        } else {
            access
        };
        let source = self.conditional_arm(fun, base, access);
        if let Some(id) = union {
            let source_view = matches!(base.ty, Some(Type::BorrowedUnion(_)))
                || (!id.copy && access == Mode::Read);
            let arms = self.program.unions[id.index()]
                .members
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let body = if matches!(ty, Type::Null | Type::Undefined) {
                        missing.clone()
                    } else if branch == crate::syntax::NullishBranch::Absent {
                        // The selection follows the dead check so this arm still
                        // has the selected type; a bare `unreachable!` leaves the
                        // payload parameter of the result view uninferred.
                        format!("if true {{ unreachable!(\"checked optional absence\") }} {selected}")
                    } else {
                        selected.clone()
                    };
                    let body=if owned && access==Mode::Consume && self.boxed_union_member(id,*ty) {
                        format!("let mut optional_slot{slot}={};{body}",self.union_move_payload(id,*ty,&format!("optional_slot{slot}")))
                    }else{body};
                    format!(
                        "{}{}::V{i}({}optional_slot{slot})=>{{{body}}}",
                        if source_view { "ViewUnion" } else { "Union" },
                        id.index(),
                        if owned && access == Mode::Consume {
                            "mut "
                        } else {
                            ""
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("match {source} {{{arms}}}")
        } else {
            format!(
                "{{let {}optional_slot{slot}={source};{selected}}}",
                if owned { "mut " } else { "" }
            )
        }
    }

    fn pattern_fallback(&self, fun: &CheckedFunction, right: &Expr, mode: Mode) -> String {
        let (access, slot) = self.prepare_pattern_fallback_work(right, mode);
        let owner = self.conditional_arm(fun, right, access);
        self.finish_pattern_fallback_work(right, slot, owner)
    }

    fn coalesce(&self, fun: &CheckedFunction, expr: &Expr, left: &Expr, right: &Expr, mode: Mode) -> String {
        let (plan, left) = self.prepare_coalesce_emission(expr, left, mode);
        let source = if plan.absent_only { self.expr(fun, left, plan.left_mode) }
            else { self.conditional_arm(fun, left, plan.left_mode) };
        let fallback = if plan.undefined_only { self.pattern_fallback(fun, right, plan.right_mode) }
            else { self.conditional_arm(fun, right, plan.right_mode) };
        self.finish_coalesce_emission(plan, right, source, fallback)
    }

    // A present conditional branch fixes the view's callable carrier. Propagate
    // that inference only along its branch spine, never into nested call arguments.
    fn conditional_expression(
        &self,
        fun: &CheckedFunction,
        predicate: &Expr,
        consequent: &Expr,
        alternate: &Expr,
        mode: Mode,
        infer_callable_carrier: bool,
    ) -> String {
        let condition = self.expr(fun, predicate, Mode::Copy);
        let branch = |arm: &Expr| match &arm.kind {
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } if infer_callable_carrier => {
                self.conditional_expression(fun, predicate, consequent, alternate, mode, true)
            }
            ExprKind::Inject { value, union }
                if infer_callable_carrier && matches!(arm.ty, Some(Type::BorrowedUnion(_))) =>
            {
                self.union_injection(fun, arm, value, *union, mode, true)
            }
            _ => self.conditional_arm(fun, arm, mode),
        };
        let then_value = branch(consequent);
        let else_value = branch(alternate);
        format!("(if {condition} {{ {then_value} }} else {{ {else_value} }})")
    }

    fn conditional_arm(&self, fun: &CheckedFunction, arm: &Expr, mode: Mode) -> String {
        let (child_mode, temporary) = self.prepare_conditional_arm_work(fun, arm, mode);
        let code = self.expr(fun, arm, child_mode);
        self.finish_conditional_arm_work(arm, mode, temporary, code)
    }

    fn prepare_conditional_arm_work(&self, fun: &CheckedFunction, arm: &Expr, mode: Mode) -> (Mode, Option<usize>) {
        if Self::borrowed_place_expression(fun, arm) { return (mode, None); }
        if matches!(arm.ty, Some(Type::BorrowedUnion(_))) { return (Mode::Read, None); }
        if matches!(mode, Mode::Read | Mode::Mutate)
            && !matches!(arm.kind, ExprKind::CoalesceAssign { .. } | ExprKind::Conditional { .. }
                | ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or, .. })
            && Self::temporary_storage(arm)
        {
            let id = self.conditional_next.get();
            self.conditional_next.set(id + 1);
            self.conditional_temporaries.borrow_mut().push(format!("let mut conditional_temporary_{id};"));
            (Mode::Consume, Some(id))
        } else { (mode, None) }
    }

    fn finish_conditional_arm_work(&self, arm: &Expr, mode: Mode, temporary: Option<usize>, code: String) -> String {
        let Some(id) = temporary else { return code; };
        let borrow = if mode == Mode::Mutate { "&mut " } else { "&" };
        let code = format!("{{ conditional_temporary_{id} = {code}; {borrow}conditional_temporary_{id} }}");
        if matches!(arm.ty, Some(Type::Union(id)) if !id.copy) && mode == Mode::Read { format!("({code}).tsr_view()") } else { code }
    }

    fn concat_call(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        callee: &Expr,
        args: &[Expr],
        mode: Mode,
    ) -> String {
        let ExprKind::Field(base, _) = &callee.kind else {
            unreachable!("checked concat receiver");
        };
        let code = match expr.target.expect("checked concat call") {
            CallTarget::StringConcat => {
                self.string_construct(fun, base, args, CallTarget::StringConcat, expr.span)
            }
            CallTarget::ArrayConcat => self.array_concat(fun, base, args, expr.span),
            _ => unreachable!("checked concat target"),
        };
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            Mode::Consume | Mode::Copy => code,
        }
    }

    /// Hidden capture arguments for a direct call of a nested function, read
    /// from the calling body's own bindings or forwarded from its own hidden
    /// parameters or captures.
    #[inline(never)]
    fn named_capture_arguments(&self, fun: &CheckedFunction, callee: usize, call: &Expr) -> Vec<String> {
        let Some(accesses) = self.program.named_capture_access.get(&callee).filter(|accesses| !accesses.is_empty()) else {
            return Vec::new();
        };
        let slots = call.capture_slots.as_deref().expect("checked capture slots of a nested function call");
        accesses
            .iter()
            .zip(slots)
            .map(|(access, &slot)| {
                let binding = &fun.bindings[slot];
                if binding.ty.is_copy() && matches!(access.mode, Mode::Read | Mode::Mutate) {
                    let reference = if access.mode == Mode::Mutate { "&mut " } else { "&" };
                    return if binding.capture_reference {
                        format!("{reference}*value{slot}")
                    } else {
                        format!("{reference}value{slot}")
                    };
                }
                let mut name = Expr::new(ExprKind::Name(binding.name.clone()), call.span);
                name.ty = Some(binding.ty);
                name.binding = Some(slot);
                self.expr(fun, &name, if binding.ty.is_copy() { Mode::Copy } else { access.mode })
            })
            .collect()
    }

    #[inline(never)]
    fn native_collection_constructor_expression(&self,fun:&CheckedFunction,expr:&Expr,mode:Mode)->String {
        if let Some(code) = self.http_constructor_expression(fun,expr,mode) {return code;}
        if let Some(code) = self.cache_constructor_expression(fun,expr,mode) {return code;}
        if let Some(code) = self.channel_constructor_expression(fun, expr, mode) { return code; }
        if let Some(Type::Map(_))=expr.ty {
            let native=self.native_type(expr.ty.unwrap());let code=format!("<{native}>::new()");
            return match mode {Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code};
        }
        let Type::Record(id) = expr.ty.expect("checked constructor") else { unreachable!() };
        let id = self.native_record_declaration(id);
        if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Schema) {
            let ExprKind::NativeConstructor(args,_)=&expr.kind else{unreachable!()};
            if let Some(CallTarget::SchemaChoice(plan))=expr.target{return self.schema_choice_constructor(fun,expr,plan,args,mode);}
            let factory=self.schema_factory_initializer(id);
            if let Some(members)=self.schema_enum_members(self.program.records[id].application.as_ref().unwrap().resolved_arguments[0]){
                let mut setup=String::new();let mut fields=Vec::new();
                for(index,arg)in args.iter().enumerate(){
                    let Type::Record(child)=arg.ty.unwrap()else{unreachable!()};let output=self.program.records[child].application.as_ref().unwrap().resolved_arguments[0];
                    let field=members.iter().position(|member|*member==output).expect("enum schema alternative");
                    setup.push_str(&format!("let enum_plan{index}={};",self.expr(fun,arg,Mode::Consume)));fields.push(format!("field{field}:{}",self.schema_child_value(id,&format!("enum_plan{index}"))));
                }
                let code=format!("{{{setup}Record{id}{{{factory}{}}}}}",fields.join(","));return match mode{Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code};
            }
            if matches!(self.program.records[id].application.as_ref().unwrap().resolved_arguments[0],Type::Enum(_,_)) {
                let code=format!("Record{id}{{{factory}}}");return match mode{Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code};
            }
            let child=self.expr(fun,&args[0],Mode::Consume);
            let code=if matches!(self.program.records[id].application.as_ref().unwrap().resolved_arguments[0],Type::Literal(_,_)) {format!("Record{id}{{{factory}expected:{child}}}")}else if self.program.records[id].application.as_ref().unwrap().resolved_arguments[0]==Type::Null{format!("{{let _={child};Record{id}{{{factory}}}}}")}else if let Type::Record(object)=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0] {
                let Type::Record(shape)=args[0].ty.unwrap() else{unreachable!()};
                let fields=self.program.records[object].fields.iter().enumerate().map(|(field,value)|{
                    let source=self.program.records[shape].fields.iter().position(|candidate|candidate.name.text==value.name.text).unwrap();
                    format!("field{field}:{}",self.schema_child_value(id,&format!("plan.field{source}")))
                }).collect::<Vec<_>>().join(",");
                format!("{{let plan={child};Record{id}{{{factory}reject_unknown:true,{fields}}} }}")
            }else{format!("Record{id} {{ {factory} element: {} }}",self.schema_child_value(id,&child))};
            return match mode {Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code};
        }
        let backend=if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::Heap){"TsrHeap"}else if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::OrderedSet){"TsrOrderedSet"}else{"TsrDeque"};
        let code = format!("Record{id} {{ values: {backend}::new() }}");
        match mode { Mode::Read => format!("&({code})"), Mode::Mutate => format!("&mut ({code})"), _ => code }
    }

    fn expr_dispatch(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        if let Some(value)=self.nullable_record_parameter_field(fun,expr,mode) { return value; }
        match &expr.kind {
            ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) => unreachable!("collection constructor sources use the expression work stack"),
            ExprKind::NativeConstructor(_, _) => self.native_collection_constructor_expression(fun,expr,mode),
            ExprKind::TypeAssertion { value, satisfies, .. } => {
                let code = self.expr(fun, value, mode);
                if !satisfies && value.ty == Some(Type::Int) && expr.ty == Some(Type::Number) {
                    format!("({code}) as f64")
                } else { code }
            },
            ExprKind::SpreadCall(_) => unreachable!("spread emission is scheduled by expression work"),
            ExprKind::SpreadValue(selected) => self.spread_selected_expression(expr,selected,mode),
            ExprKind::Unary { op: UnaryOp::RestPack, value, .. } => self.expr(fun, value, mode),
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) => self.array_literal_expression(fun, expr, values, mode),
            _ => self.expr_dispatch_inner(fun, expr, mode),
        }
    }

    fn array_literal_expression(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        values: &[Expr],
        mode: Mode,
    ) -> String {
        let items = values
            .iter()
            .map(|value| self.expr(fun, value, Mode::Consume))
            .collect::<Vec<_>>()
            .join(", ");
        self.finish_array_literal_expression(expr, values, mode, items)
    }

    fn finish_array_literal_expression(&self, expr: &Expr, values: &[Expr], mode: Mode, items: String) -> String {
        // An empty literal still has a checked element/container type.
        // Bind its constructor before borrowing, without changing fixed named arrays.
        let code = if values.is_empty() {
            format!(
                "<{}>::from([])",
                self.native_type(expr.ty.expect("checked array type"))
            )
        } else if let Some(Type::Array(ArrayElement::Record(index, 0))) = expr.ty
            && self.record_runtime.recursive(index)
        {
            format!("RecordArray{index}::from([{items}])")
        } else if numeric_array(expr.ty.expect("checked array type")) {
            format!("NumericArray::from([{items}])")
        } else if inline_array(expr.ty.expect("checked array type"))
            || matches!(expr.ty, Some(Type::Array(ArrayElement::Record(index, 0))) if self.small_record(index))
        {
            format!("Array::from([{items}])")
        } else {
            format!("vec![{items}]")
        };
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            _ => code,
        }
    }

    fn expr_dispatch_inner(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        let mode = if expr.ty.is_some_and(Type::is_copy) {
            Mode::Copy
        } else {
            mode
        };
        match &expr.kind {
            ExprKind::Call(callee, args)
                if matches!(
                    expr.target,
                    Some(CallTarget::StringConcat | CallTarget::ArrayConcat)
                ) =>
            {
                self.concat_call(fun, expr, callee, args, mode)
            }
            ExprKind::Optional {
                base,
                body,
                slot,
                union,
                access,
                branch,
                owned,
            } => self.optional_chain(
                fun,
                expr,
                base,
                body,
                (*slot, *union, *access, *branch, *owned),
                mode,
            ),
            ExprKind::Selected { slot, owned, .. } => {
                let name = format!("optional_slot{slot}");
                if expr.ty.is_some_and(Type::is_copy) {
                    if *owned { name } else { format!("*{name}") }
                } else {
                    match mode {
                        Mode::Read => {
                            if *owned {
                                format!("&{name}")
                            } else {
                                format!("&*{name}")
                            }
                        }
                        Mode::Mutate => {
                            if *owned {
                                format!("&mut {name}")
                            } else {
                                format!("&mut *{name}")
                            }
                        }
                        _ => name,
                    }
                }
            }
            ExprKind::OptionalChain(_) => unreachable!("source optional chain"),
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault,
                left,
                right,
                ..
            } => self.coalesce(fun, expr, left, right, mode),
            ExprKind::Unary {op:UnaryOp::RecordCarrier,value,..}=>{
                let code=if let Some(code)=self.contextual_record_carrier(fun,value,expr.ty.unwrap()) {code}
                else {
                    let input=self.expr(fun,value,Mode::Consume);
                    let source=self.owned_record_source_type(fun,value).unwrap_or_else(||value.ty.unwrap());
                    let converted=self.record_carrier_conversion(source,expr.ty.unwrap(),"record_carrier_input");
                    format!("{{let record_carrier_input={input};{converted}}}")
                };
                match mode{Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code}
            }
            ExprKind::Unary {
                op: UnaryOp::CallableStore,
                value,
                ..
            } => {
                let (Type::StoredCallable(family)|Type::BorrowedCallable(family)) = expr.ty.expect("checked storage wrapper") else {
                    unreachable!();
                };
                let Type::Closure(origin) = value.ty.expect("checked storage origin") else {
                    unreachable!();
                };
                let inner = self.expr(fun, value, Mode::Consume);
                let code = self.local_borrowed_family_constructor(fun,family,&inner).unwrap_or_else(||self.stored_callable_constructor(family, origin, &inner));
                match mode {
                    Mode::Read => format!("&({code})"),
                    Mode::Mutate => format!("&mut ({code})"),
                    _ => code,
                }
            }
            ExprKind::Unary {op:UnaryOp::ParameterDefaultStore,value,..} => {
                let union=expr.ty.unwrap().union().expect("nullable default family");
                let target=&self.program.unions[union.index()].members;
                let family=target.iter().copied().find_map(|ty|if let Type::StoredCallable(id)=ty{Some(id)}else{None}).expect("default family");
                let stored=|member:Type,payload:&str| {
                    if let Type::Closure(origin)=member {
                        self.stored_callable_constructor(family, origin, &payload)
                    } else {payload.to_owned()}
                };
                let source=self.expr(fun,value,Mode::Consume);
                if let Some(from)=value.ty.unwrap().union() {
                    let arms=self.program.unions[from.index()].members.iter().enumerate().map(|(i,&member)|{
                        let to=target.iter().position(|&ty|ty==member || matches!(ty,Type::StoredCallable(_))&&!matches!(member,Type::Null|Type::Undefined)).unwrap();
                        format!("Union{}::V{i}(payload)=>Union{}::V{to}({})",from.index(),union.index(),stored(member,"payload"))
                    }).collect::<Vec<_>>().join(",");
                    format!("match {source} {{{arms}}}")
                } else {
                    let member=value.ty.unwrap();let to=target.iter().position(|&ty|ty==member||matches!(ty,Type::StoredCallable(_))&&!matches!(member,Type::Null|Type::Undefined)).unwrap();
                    format!("Union{}::V{to}({})",union.index(),stored(member,&source))
                }
            }
            ExprKind::Unary {op:UnaryOp::ParameterMutableInput,value,..} => {
                let Type::Union(union)=expr.ty.unwrap() else {unreachable!("mutable slot tag")};
                let target=&self.program.unions[union.index()].members;
                if let Some(code)=self.spread_mutable_input(value,union) {
                    code
                } else if let Some(from)=value.ty.unwrap().union() {
                    let source=self.expr(fun,value,Mode::Mutate);
                    let arms=self.program.unions[from.index()].members.iter().enumerate().map(|(i,member)|{
                        let to=target.iter().position(|ty|ty==member).expect("mutable input subset");
                        let payload=if member.is_copy(){"*payload"}else{"payload"};
                        format!("Union{}::V{i}(payload)=>Union{}::V{to}({payload})",from.index(),union.index())
                    }).collect::<Vec<_>>().join(",");
                    format!("match {source} {{{arms}}}")
                } else {
                    let actual=value.ty.unwrap();
                    let to=target.iter().position(|ty|*ty==actual || matches!(ty,Type::Callback(_))&&matches!(actual,Type::Callback(_)|Type::Closure(_))).expect("mutable input member");
                    let source=self.expr(fun,value,if actual.is_copy(){Mode::Copy}else{Mode::Mutate});
                    let absent=if actual.is_copy() && self.callback_payload(Type::Union(union)).is_some(){"::<&mut TsrAbsentCallable>"}else{""};
                    format!("Union{}{absent}::V{to}({source})",union.index())
                }
            }
            ExprKind::Unary{op:UnaryOp::CallableRestAdapter,value,..}=>{
                let Type::Closure(id)=expr.ty.expect("adapter type")else{unreachable!()};
                let source=self.expr(fun,value,self.program.closures[id].captures[0].storage);
                let code=format!("Closure{id} {{capture0: {source}}}");
                match mode{Mode::Read=>format!("&({code})"),Mode::Mutate=>format!("&mut ({code})"),_=>code}
            }
            ExprKind::Unary {
                op: UnaryOp::CallableOnce | UnaryOp::TaskCreate,
                value,
                ..
            } => {
                let Type::Closure(id) = expr.ty.expect("checked once environment") else {
                    unreachable!();
                };
                let inner = self.expr(fun, value, Mode::Consume);
                let code = format!("Closure{id} {{ capture0: {inner} }}");
                match mode {
                    Mode::Read => format!("&({code})"),
                    Mode::Mutate => format!("&mut ({code})"),
                    _ => code,
                }
            }
            ExprKind::Unary {
                op: UnaryOp::AwaitCallee | UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected,
                value,
                ..
            } => self.expr(fun, value, mode),
                ExprKind::Unary { op: UnaryOp::Await, value, .. } => {
                let code=self.expr(fun,value,mode);
                code
            }
            ExprKind::Unary { op: UnaryOp::NonNull, value, operator } => self.non_null_projection(fun,expr,value,*operator,mode),
            ExprKind::Binary { op, left, right, .. } if matches!(op, BinaryOp::And | BinaryOp::Or)
                && !(left.ty == Some(Type::Boolean) && right.ty == Some(Type::Boolean)) =>
                self.logical_value(fun, expr, *op, left, right, mode),
            ExprKind::Match { id, body } => {
                let outer_temporaries = std::mem::take(&mut *self.conditional_temporaries.borrow_mut());
                let mut output = format!("'match_value{id}: {{");
                self.block(&mut output, fun, body, 0);
                output.push('}');
                *self.conditional_temporaries.borrow_mut() = outer_temporaries;
                match mode { Mode::Read => format!("&({output})"), Mode::Mutate => format!("&mut ({output})"), _ => output }
            }
            ExprKind::Project {
                value,
                union,
                narrowed,
            } => self.union_projection(fun, expr, value, *union, *narrowed, mode, "checked nullable guard"),
            ExprKind::Inject { value, union } => {
                self.union_injection(fun, expr, value, *union, mode, false)
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                let infer_callable_carrier = matches!(expr.ty, Some(Type::BorrowedUnion(_)))
                    && self.callback_payload(expr.ty.unwrap()).is_some()
                    && !Self::absent_callable_initializer(expr);
                self.conditional_expression(
                    fun,
                    predicate,
                    consequent,
                    alternate,
                    mode,
                    infer_callable_carrier,
                )
            }
            ExprKind::CompoundAssign {
                target,
                value,
                op,
                operator,
            } => self.compound_assignment(fun, target, value, *op, *operator),
            ExprKind::CoalesceAssign {
                target,
                value,
                branch,
                ..
            } => self.coalesce_assignment(fun, expr, target, value, *branch),
            ExprKind::Assign { target, value, .. } if expr.ty==Some(Type::Void) => {
                if matches!(target.kind,ExprKind::Name(_)){
                    let id=target.binding.expect("checked assignment binding");
                    let assigned=self.expr(fun,value,Mode::Consume);
                    if self.restored_capture(fun,id) {format!("{{{}value{id}=Some({assigned});}}",if fun.bindings[id].capture_reference {"*"}else{""})} else if let Some(store)=self.owned_record_assignment(fun,id,&assigned){format!("{{{store}}}")}else{format!("{{{}value{id}={assigned};}}",if fun.bindings[id].capture_reference{"*"}else{""})}
                }else{self.storage_assignment(fun,target,value)}
            }
            ExprKind::Assign { target, value, .. } => self.scalar_assignment(fun, target, value),
            ExprKind::LogicalAssign {
                target, value, and, ..
            } => self.logical_assignment(fun, target, value, *and),
            ExprKind::Update {
                target,
                increment,
                prefix,
                operator,
            } => self.update_expression(fun, target, *increment, *prefix, *operator),
            ExprKind::Index(base, index) if base.ty.map(Type::scalar_base) == Some(Type::String) => self.string_index(fun, expr, base, index, mode),
            ExprKind::Index(base, index)
                if self.expression_shape(base)
                    == Some(crate::fixed_storage::Shape::LiteralStrings) =>
            {
                debug_assert!(matches!(mode, Mode::Read));
                let failure = Self::array_failure(expr.span);
                let failed = self.runtime_failure_exit("RangeError", "bounds", &failure, expr.span)
                    .unwrap_or_else(|| format!("panic!({failure:?})"));
                let array = self.expr(fun, base, Mode::Read);
                let index = self.expr(fun, index, Mode::Copy);
                format!(
                    "&{{ let literal_array = {array}; let literal_index = {index}; let literal_index = match usize::try_from(literal_index) {{ Ok(index) => index, Err(_) => {failed} }}; match literal_array.get(literal_index) {{ Some(value) => value, None => {failed} }} }}"
                )
            }
            ExprKind::Field(base, name) if matches!(&base.kind, ExprKind::Name(namespace) if NumericConstant::lookup(namespace, name).is_some()) =>
            {
                let ExprKind::Name(namespace) = &base.kind else {
                    unreachable!("checked numeric namespace")
                };
                NumericConstant::lookup(namespace, name)
                    .expect("checked numeric constant")
                    .rust()
                    .into()
            }
            ExprKind::Field(base, name) if name == "size" && matches!(base.ty, Some(Type::Record(id)) if self.program.records[id].native_element().is_some()) => {
                let receiver=self.conditional_arm(fun,base,Mode::Read);
                format!("({receiver}).values.len() as i64")
            }
            ExprKind::Field(base, name) if name == "size" && matches!(base.ty,Some(Type::Set(_) | Type::Map(_))) => {
                let receiver=self.conditional_arm(fun,base,Mode::Read);
                format!("({receiver}).len() as i64")
            }
            ExprKind::Field(base, name) if name == "length" && matches!(base.ty, Some(Type::Record(id)) if self.program.records[id].is_tuple()) => {
                let Type::Record(id) = base.ty.unwrap() else { unreachable!() };
                let receiver = self.conditional_arm(fun, base, Mode::Read);
                format!("{{ let _tuple = {receiver}; {}i64 }}",self.program.records[id].fields.len())
            }
            ExprKind::Field(base, name) if base.ty.map(Type::scalar_base) == Some(Type::String) && name == "length" => {
                // Fresh receivers survive the complete consuming expression,
                // including console argument staging, then dispose normally.
                let value = self.conditional_arm(fun, base, Mode::Read);
                format!("tsr_length({value})")
            }
            ExprKind::Field(base, name)
                if matches!(base.ty, Some(Type::Array(_))) && name == "length" =>
            {
                let array = self.conditional_arm(fun, base, Mode::Read);
                format!("i64::try_from(({array}).len()).expect(\"array length exceeds int range\")")
            }
            ExprKind::Field(base,name) if matches!(base.ty,Some(Type::Union(_))) => {
                let Type::Union(union)=base.ty.unwrap() else{unreachable!()};
                let receiver=self.expr(fun,base,Mode::Read);
                let copied=expr.ty.unwrap().is_copy();
                let reference=if mode==Mode::Read&&!copied{"&"}else{""};
                let representation=if union.copy{"Union"}else{"ViewUnion"};
                let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,member)|{
                    let Type::Record(record)=member else{unreachable!("checked common record field")};
                    let field=self.fields[*record][name.as_str()];
                    let output=if let Some(Type::Union(target))=expr.ty{
                        let field_type=self.program.records[*record].fields[field].ty.unwrap();
                        let target_kind=if target.copy{"Union"}else{"ViewUnion"};
                        let inject=|member:Type,payload:&str|{
                            let variant=self.program.unions[target.index()].members.iter().position(|ty|*ty==member).expect("common field alternative");
                            format!("{target_kind}{}::V{variant}({payload})",target.index())
                        };
                        if let Type::Union(source)=field_type{
                            let alternatives=self.program.unions[source.index()].members.iter().enumerate().map(|(index,ty)|format!("Union{}::V{index}(payload)=>{}",source.index(),inject(*ty,if target.copy{"*payload"}else{"payload"}))).collect::<Vec<_>>().join(",");
                            format!("match &value.field{field}{{{alternatives}}}")
                        }else{inject(field_type,&format!("{}value.field{field}",if target.copy{""}else{"&"}))}
                    }else{format!("{reference}value.field{field}")};
                    format!("{representation}{}::V{tag}(value)=>{output}",union.index())
                }).collect::<Vec<_>>().join(",");
                let value=format!("match {receiver}{{{arms}}}");
                if copied&&mode==Mode::Read&&!matches!(expr.ty,Some(Type::Union(_))){format!("&({value})")}else{value}
            }
            ExprKind::Index(_, _) | ExprKind::Field(_, _) if Self::temporary_storage(expr) => {
                self.temporary_storage_access(fun, expr, mode)
            }
            ExprKind::Index(_, _) | ExprKind::Field(_, _) => self.storage_access(fun, expr, mode),
            ExprKind::Record(fields) => self.record_literal(fun, expr, fields, mode),
            ExprKind::Template(parts) => self.template_literal(fun, parts, mode),
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) => self.array_literal_expression(fun, expr, values, mode),
            ExprKind::Unary {
                op: UnaryOp::BitNot,
                value,
                ..
            } if value.ty == Some(Type::Number) => {
                let value = self.expr(fun, value, Mode::Copy);
                format!(
                    "{{ let number_value: f64 = {value}; (!tsr_math_uint32(number_value) as i32) as f64 }}"
                )
            }
            ExprKind::Binary {
                op,
                operator,
                left,
                right,
            } if op.bitwise_or_power() && left.ty == Some(Type::Int) => {
                self.int_operator(fun, *op, left, right, *operator)
            }
            ExprKind::Binary {
                op, left, right, ..
            } if op.bitwise_or_power() => self.number_operator(fun, *op, left, right),
            ExprKind::Unary { op: UnaryOp::Not, value, .. }
                if value.ty.is_some_and(|ty| !matches!(ty, Type::Boolean | Type::Null | Type::Undefined)) =>
            {
                self.logical_not(fun, value)
            }
            ExprKind::Unary {
                op,
                operator,
                value,
            } if matches!(
                op,
                UnaryOp::Positive | UnaryOp::Negate | UnaryOp::Not | UnaryOp::BitNot
            ) && !matches!(value.ty, Some(Type::Null | Type::Undefined)) =>
            {
                self.scalar_unary(fun, *op, *operator, value)
            }
            ExprKind::Unary { .. } => self.unary_expression_frame(fun, expr, mode),
            ExprKind::Binary { .. } => self.binary_expression_frame(fun, expr, mode),
            _ => self.expr_inner(fun, expr, mode),
        }
    }

    fn scalar_unary(
        &self,
        fun: &CheckedFunction,
        op: UnaryOp,
        operator: crate::diagnostic::Span,
        value: &Expr,
    ) -> String {
        let number = value.ty == Some(Type::Number);
        let value = self.expr(fun, value, Mode::Copy);
        match op {
            UnaryOp::Positive => value,
            UnaryOp::Negate if number => format!("(-({value}))"),
            UnaryOp::Not | UnaryOp::BitNot => format!("(!({value}))"),
            UnaryOp::Negate => self.checked_integer_result(&format!("({value}).checked_neg()"), "negation overflow", operator),
            _ => unreachable!("scalar unary operation expected"),
        }
    }

    fn int_operator(
        &self,
        fun: &CheckedFunction,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        span: crate::diagnostic::Span,
    ) -> String {
        let left = self.expr(fun, left, Mode::Copy);
        let right = self.expr(fun, right, Mode::Copy);
        self.int_bitwise_or_power(op, &left, &right, span)
    }

    #[inline(never)]
    fn int_bitwise_or_power(
        &self,
        op: BinaryOp,
        left: &str,
        right: &str,
        span: crate::diagnostic::Span,
    ) -> String {
        let result = match op {
            BinaryOp::BitAnd => "integer_left & integer_right".to_owned(),
            BinaryOp::BitOr => "integer_left | integer_right".to_owned(),
            BinaryOp::BitXor => "integer_left ^ integer_right".to_owned(),
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight | BinaryOp::ShiftRightUnsigned => {
                let reason = "shift count must be in 0..=63";
                let shifted = match op {
                    BinaryOp::ShiftLeft => "integer_left.checked_shl(integer_count)",
                    BinaryOp::ShiftRight => "integer_left.checked_shr(integer_count)",
                    BinaryOp::ShiftRightUnsigned => {
                        "(integer_left as u64).checked_shr(integer_count)"
                    }
                    _ => unreachable!("checked int shift"),
                };
                let cast = if op == BinaryOp::ShiftRightUnsigned {
                    " as i64"
                } else {
                    ""
                };
                let count = self.checked_integer_result("u32::try_from(integer_right).ok()", reason, span);
                let shifted = self.checked_integer_result(shifted, reason, span);
                format!("let integer_count = {count}; {shifted}{cast}")
            }
            BinaryOp::Exponent => {
                let exponent = self.checked_integer_result("u32::try_from(integer_right).ok()", "exponent must be in 0..=4294967295", span);
                let powered = self.checked_integer_result("integer_left.checked_pow(integer_exponent)", "exponentiation overflow", span);
                format!("let integer_exponent = {exponent}; {powered}")
            }
            _ => unreachable!("checked int bitwise or power operator"),
        };
        // Explicit bindings preserve single left-to-right evaluation before any
        // count conversion or checked arithmetic, including the compound path.
        format!("{{ let integer_left: i64 = {left}; let integer_right: i64 = {right}; {result} }}")
    }

    fn number_operator(
        &self,
        fun: &CheckedFunction,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> String {
        let left = self.expr(fun, left, Mode::Copy);
        let right = self.expr(fun, right, Mode::Copy);
        Self::number_operation(op, &left, &right)
    }

    fn number_operation(op: BinaryOp, left: &str, right: &str) -> String {
        let result = match op {
            BinaryOp::Exponent => "tsr_math_pow(number_left, number_right)",
            BinaryOp::BitAnd => "((number_left & number_right) as i32) as f64",
            BinaryOp::BitOr => "((number_left | number_right) as i32) as f64",
            BinaryOp::BitXor => "((number_left ^ number_right) as i32) as f64",
            BinaryOp::ShiftLeft => "((number_left << (number_right & 31)) as i32) as f64",
            BinaryOp::ShiftRight => "((number_left as i32) >> (number_right & 31)) as f64",
            BinaryOp::ShiftRightUnsigned => "(number_left >> (number_right & 31)) as f64",
            _ => unreachable!("checked Number operator"),
        };
        let conversions = if op == BinaryOp::Exponent {
            ""
        } else {
            "let number_left = tsr_math_uint32(number_left); let number_right = tsr_math_uint32(number_right);"
        };
        format!(
            "{{ let number_left: f64 = {left}; let number_right: f64 = {right}; {conversions} {result} }}"
        )
    }

    fn order_fields(&self, record: usize, fields: String) -> String {
        if self.record_runtime.ordered.contains(&record) {
            format!("{fields}{}key_order: [usize; {}]", if fields.is_empty() { "" } else { ", " }, self.program.records[record].fields.len())
        } else { fields }
    }

    fn order_values(&self, record: usize, fields: &[(Token, Expr)], values: String) -> String {
        if self.record_runtime.ordered.contains(&record) {
            let order = fields.iter().map(|(key, _)| self.fields[record][key.text.as_str()].to_string()).collect::<Vec<_>>().join(",");
            format!("{values}{}key_order: [{order}]", if values.is_empty() { "" } else { ", " })
        } else { values }
    }

    fn record_literal(
        &self,
        fun: &CheckedFunction,
        expr: &Expr,
        fields: &[(crate::syntax::Token, Expr)],
        mode: Mode,
    ) -> String {
        let code = {
            let Type::Record(index) = expr.ty.expect("checked record type") else {
                unreachable!("record literal")
            };
            let values = fields
                .iter()
                .map(|(key, value)| {
                    format!(
                        "field{}: {}",
                        self.fields[index][key.text.as_str()],
                        self.expr(fun, value, Mode::Consume)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let values = self.order_values(index, fields, values);
            format!("Record{} {{ {values} }}", self.native_record_declaration(index))
        };
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            _ => code,
        }
    }

    fn template_literal(&self, fun: &CheckedFunction, parts: &[Expr], mode: Mode) -> String {
        let capacity: usize = parts
            .iter()
            .step_by(2)
            .map(|part| {
                let ExprKind::String(chunk) = &part.kind else {
                    unreachable!("template chunk")
                };
                chunk.len()
            })
            .sum();
        let mut code = format!(
            "{{ use std::fmt::Write as _; let mut template_output = std::string::String::with_capacity({capacity}); "
        );
        for (index, part) in parts.iter().enumerate() {
            if index % 2 == 0 {
                let ExprKind::String(chunk) = &part.kind else {
                    unreachable!("template chunk")
                };
                code.push_str(&format!("template_output.push_str({chunk:?}); "));
                continue;
            }
            // A selected fresh string may need conditional locals. Keep them in
            // this append scope so its owner drops before the next operand.
            let first_temporary = self.conditional_temporaries.borrow().len();
            let value = self.expr(fun, part, Mode::Read);
            let declarations = self
                .conditional_temporaries
                .borrow_mut()
                .drain(first_temporary..)
                .collect::<Vec<_>>()
                .join(" ");
            code.push_str(&self.finish_template_append(part, &value, &declarations));
        }
        code.push_str("String::Owned(template_output) }");
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            Mode::Consume | Mode::Copy => code,
        }
    }

    fn finish_template_append(&self, part: &Expr, value: &str, declarations: &str) -> String {
        let mut code = String::new();
            code.push_str(&format!("{{ {declarations} "));
            if part.ty == Some(Type::String) {
                code.push_str(&format!(
                    "let template_value: &String = {value}; template_output.push_str(template_value.as_ref()); "
                ));
            } else if part.ty == Some(Type::Number) {
                // Number text has its own spelling for infinities, zero,
                // exponent ranges and tied digits.
                code.push_str(&format!(
                    "let template_value: f64 = {value}; template_output.push_str(tsr_number_to_string(template_value, 10, \"number text\").as_ref()); "
                ));
            } else {
                code.push_str(&format!(
                    "let template_value = {value}; write!(&mut template_output, \"{{}}\", template_value).expect(\"string formatting failed\"); "
                ));
            }
            code.push_str("} ");
        code
    }

    // Operator nesting recurses through these frames, not the general
    // expression frame.
    #[inline(never)]
    fn unary_expression_frame(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        match self.unary_expression_code(fun, expr, mode) {
            Ok(code) => Self::access_wrapped(expr, mode, code),
            Err(complete) => complete,
        }
    }

    // Err carries code that is already complete for the requested access.
    fn unary_expression_code(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> Result<String, String> {
        self.unary_expression_code_with(expr, mode, &mut |value, mode, conditional| {
            if conditional { self.conditional_arm(fun, value, mode) } else { self.expr(fun, value, mode) }
        })
    }

    fn unary_expression_code_with(&self, expr: &Expr, mode: Mode, render: &mut impl FnMut(&Expr, Mode, bool) -> String) -> Result<String, String> {
        let ExprKind::Unary { op, operator, value } = &expr.kind else {
            unreachable!("unary expression")
        };
        Ok({
        if let UnaryOp::VariantIs(variant) = op {
            let union = value.ty.and_then(Type::union).expect("checked variant test");
            let index = self.program.unions[union.index()].members.iter().position(|member| member == variant).expect("checked alternative");
            let family = if !union.copy || matches!(value.ty, Some(Type::BorrowedUnion(_))) { "ViewUnion" } else { "Union" };
            return Err(format!("matches!({}, {family}{}::V{index}(_))", render(value, Mode::Read, false), union.index()));
        }
        if matches!(op, UnaryOp::Typeof | UnaryOp::Void) {
            let unresolved = matches!(&value.kind, ExprKind::Name(_) if value.binding.is_none() && value.ty == Some(Type::Undefined));
            if *op == UnaryOp::Typeof
                && let Some(union) = value.ty.and_then(Type::union)
            {
                let value = render(value, Mode::Read, true);
                return Err(Self::access_wrapped(expr, mode, self.union_typeof(union, &format!("&({value})"))));
            }
            if *op==UnaryOp::Typeof && matches!(value.ty,Some(Type::Record(id)) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::StringView)) {
                let operand=render(value, Mode::Read, false);return Err(Self::access_wrapped(expr,mode,format!("{{let _={operand};String::Borrowed(\"string\")}}")));
            }
            if *op == UnaryOp::Typeof && matches!(value.ty, Some(Type::Record(id)) if matches!(self.program.records[id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) {
                let operand = render(value, Mode::Read, false);
                return Err(Self::access_wrapped(expr, mode, format!("String::Borrowed(tsrust_native_json::type_of(&({operand}).value))")));
            }
            let operand = if unresolved {
                String::new()
            } else {
                format!("let _ = {}; ", render(value, Mode::Read, false))
            };
            if *op == UnaryOp::Void {
                return Err(format!("{{ {operand}TsrUndefined }}"));
            }
            let name = if unresolved {
                match &value.kind {
                    ExprKind::Name(name) if matches!(name.as_str(), "console" | "Math") => {
                        "object"
                    }
                    ExprKind::Name(name)
                        if matches!(
                            name.as_str(),
                            "Number" | "clone" | "String" | "Boolean" | "Array"
                        ) =>
                    {
                        "function"
                    }
                    _ => "undefined",
                }
            } else {
                match value.ty.expect("checked typeof") {
                    Type::Literal(_,kind)=>match kind{crate::literal_types::LiteralKind::String=>"string",crate::literal_types::LiteralKind::Boolean=>"boolean",_=>"number"},
                    Type::Null | Type::Set(_) | Type::Map(_) | Type::Record(_) | Type::Array(_) => "object",
                    Type::Undefined | Type::Void => "undefined",
                    Type::Boolean => "boolean",
                    Type::String | Type::Enum(_, crate::syntax::EnumKind::String) => "string",
                    Type::Int | Type::Number | Type::Enum(_, crate::syntax::EnumKind::Number) => "number",
                    Type::Closure(_)
                    | Type::Callable(_)
                    | Type::Callback(_)
                    | Type::StoredCallable(_) | Type::BorrowedCallable(_) => "function",
                    Type::Union(_) | Type::BorrowedUnion(_) => {
                        unreachable!("nullable typeof dispatched")
                    }
                }
            };
            let code = format!("{{ {operand}String::Borrowed({name:?}) }}");
            return Err(match mode {
                Mode::Read => format!("&({code})"),
                Mode::Mutate => format!("&mut ({code})"),
                _ => code,
            });
        }
        if *op == UnaryOp::Not && matches!(value.ty, Some(Type::Null | Type::Undefined)) {
            return Err(format!("{{ let _ = {}; true }}", render(value, Mode::Copy, false)));
        }
        let number = value.ty == Some(Type::Number);
        let value = render(value, Mode::Copy, false);
        match op {
            UnaryOp::Await | UnaryOp::AwaitCallee | UnaryOp::TaskCreate => unreachable!("task lowering emitted separately"),
            UnaryOp::VariantIs(_) => unreachable!("variant test emitted separately"),
            UnaryOp::PatternSelection=>unreachable!("pattern selection lowered before emission"),
            UnaryOp::Spread => unreachable!("source spread must be lowered by checked call planning"),
            UnaryOp::NonNull
            | UnaryOp::RestPack
            | UnaryOp::CoalesceSelected
            | UnaryOp::ParameterMutableInput
            | UnaryOp::ParameterDefaultStore
            | UnaryOp::CallableRestAdapter
            | UnaryOp::CallableOnce
            | UnaryOp::CallableStore
            | UnaryOp::RecordCarrier
            | UnaryOp::OptionalBoundary
            | UnaryOp::OptionalSelected => {
                unreachable!("coalescing identity emitted separately")
            }
            UnaryOp::Typeof | UnaryOp::Void => {
                unreachable!("reflection emitted separately")
            }
            UnaryOp::Positive => value,
            UnaryOp::Negate if number => format!("(-({value}))"),
            UnaryOp::Not => format!("(!({value}))"),
            UnaryOp::BitNot => format!("(!({value}))"),
            UnaryOp::Negate => self.checked_integer_result(&format!("({value}).checked_neg()"), "negation overflow", *operator),
        }
        })
    }

    /// Strict equality between `equality_left` and `equality_right` where at
    /// least one is a union read as a copy or a view. Only alternatives of
    /// equal type can match; int and number compare by value.
    fn stored_callable_constructor(&self, family: usize, origin: usize, value: &str) -> String {
        if self.program.stored_contracts.scoped_native_family(family).is_some() {
            return format!("NativeViewCallable{family}::V{origin}({value})");
        }
        if self.program.stored_contracts.is_borrowed(family) {
            if self.borrowed_payloads(family).iter().any(|payload| !payload.captures.is_empty() || !payload.owners.is_empty()) {
                let name=self.borrowed_family_type(family).replacen("<","::<",1);
                return format!("{name}::from({value})");
            }
            let origins=self.program.stored_contracts.origins(family).expect("borrowed origins");
            let position=origins.iter().position(|&id|id==origin).expect("registered borrowed origin");
            let variant=crate::borrowed_callable_dispatch::variant(position,origins.len());
            let name=self.borrowed_family_type(family).replacen("<","::<",1);
            return format!("{name}::{variant}({value})");
        }
        let declaration=self.program.stored_contracts.declaration(family).expect("stored declaration");
        let name=self.native_type(Type::StoredCallable(family));
        let singleton=self.program.stored_contracts.singleton_origin(family, origin).expect("registered stored origin");
        if self.program.callback_contracts[declaration].kind == crate::syntax::CallbackKind::SharedRead {
            format!("{name}::from_origin_{origin}({value})")
        } else if singleton {format!("{name}({value})")} else {format!("{name}::V{origin}({value})")}
    }

    fn testing_numeric_length_property(&self, ty: Type, value: &str, presence: bool) -> String {
        if let Some(union) = ty.union() {
            let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(i, member)| {
                let result = self.testing_numeric_length_property(*member, "payload", presence);
                format!("Union{}::V{i}(payload) => {{ let _ = &payload; {result} }}", union.index())
            }).collect::<Vec<_>>().join(",");
            return format!("match {value} {{ {arms} }}");
        }
        if presence { return matches!(ty, Type::Int | Type::Number).to_string(); }
        match ty { Type::Int => format!("*({value}) as f64"), Type::Number => format!("*({value})"), _ => "f64::NAN".into() }
    }

    fn testing_length_observation(&self, ty: Type, value: &str, presence: bool) -> String {
        if let Some(union) = ty.union() {
            let prefix = if union.copy { "Union" } else { "ViewUnion" };
            let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(i, member)| {
                let result = self.testing_length_observation(*member, "payload", presence);
                format!("{prefix}{}::V{i}(payload) => {{ let _ = &payload; {result} }}", union.index())
            }).collect::<Vec<_>>().join(",");
            return format!("match {value} {{ {arms} }}");
        }
        if let Type::Record(id) = ty {
            if !self.program.records[id].is_tuple() {
                if let Some((i, field)) = self.program.records[id].fields.iter().enumerate().find(|(_, field)| field.name.text == "length") {
                    return self.testing_numeric_length_property(field.ty.expect("checked length field"), &format!("&({value}).field{i}"), presence);
                }
            }
        }
        let length = match ty {
            Type::String => Some(format!("({value}).encode_utf16().count() as f64")),
            Type::Array(_) => Some(format!("({value}).len() as f64")),
            Type::Record(id) if self.program.records[id].is_tuple() => Some(format!("{}f64", self.program.records[id].fields.len())),
            _ => None,
        };
        let result = if presence { length.is_some().to_string() } else { length.unwrap_or_else(|| "f64::NAN".into()) };
        format!("{{ let _ = &({value}); {result} }}")
    }

    fn union_object_is(&self, left: Type, right: Type) -> String {
        let side = |ty: Type, name: &str| -> Vec<(String, Type)> {
            match ty.union() {
                Some(id) => self.program.unions[id.index()].members.iter().enumerate().map(|(index, member)|
                    (format!("{}{}::V{index}({name})", if id.copy { "Union" } else { "ViewUnion" }, id.index()), *member)).collect(),
                None => vec![(name.to_owned(), ty)],
            }
        };
        let mut arms = String::new();
        for (lp, lt) in side(left, "left") {
            for (rp, rt) in side(right, "right") {
                let (lt, rt) = (lt.scalar_base(), rt.scalar_base());
                let comparison = if lt == rt && matches!(lt, Type::Int | Type::Boolean) { "*left == *right" }
                else if lt == rt && matches!(lt, Type::Enum(_, crate::syntax::EnumKind::Number)) {
                    "{let l = left.tsr_value(); let r = right.tsr_value(); l.to_bits() == r.to_bits() || (l.is_nan() && r.is_nan())}"
                } else if lt == rt && matches!(lt, Type::Enum(_, crate::syntax::EnumKind::String)) { "left.tsr_value() == right.tsr_value()" }
                else if matches!(lt, Type::Int | Type::Number) && matches!(rt, Type::Int | Type::Number) {
                    "{let l = *left as f64; let r = *right as f64; l.to_bits() == r.to_bits() || (l.is_nan() && r.is_nan())}"
                } else if lt == rt && matches!(lt, Type::Null | Type::Undefined) { "true" }
                else if lt == rt && lt == Type::String { "**left == **right" }
                else if lt == rt && matches!(lt, Type::Record(_) | Type::Array(_)) { "std::ptr::eq(*left, *right)" }
                else { "false" };
                let _ = write!(arms, "({lp}, {rp}) => {{ let _ = (&left, &right); {comparison} }},");
            }
        }
        format!("match (&object_left, &object_right) {{ {arms} }}")
    }

    fn union_equality(&self, left: Type, right: Type) -> String {
        // Each side is a list of (pattern, payload type); a plain value has one.
        let side = |ty: Type, name: &str| -> Vec<(String, Type)> {
            match ty.union() {
                Some(id) => {
                    let tag = if id.copy { "Union" } else { "ViewUnion" };
                    self.program.unions[id.index()]
                        .members
                        .iter()
                        .enumerate()
                        .map(|(variant, member)| (format!("{tag}{}::V{variant}({name})", id.index()), *member))
                        .collect()
                }
                None => vec![(name.to_owned(), ty)],
            }
        };
        let mut arms = String::new();
        for (left_pattern, left_member) in side(left, "left") {
            for (right_pattern, right_member) in side(right, "right") {
                let compare = match (left_member.scalar_base(), right_member.scalar_base()) {
                    (Type::Enum(a, _), Type::Enum(b, _)) if a == b => "*left == *right",
                    (Type::String, Type::String) => "**left == **right",
                    (Type::Int, Type::Int) | (Type::Boolean, Type::Boolean) | (Type::Number, Type::Number) => "*left == *right",
                    (Type::Int, Type::Number) => "(*left as f64) == *right",
                    (Type::Number, Type::Int) => "*left == (*right as f64)",
                    (Type::Null, Type::Null) | (Type::Undefined, Type::Undefined) => "true",
                    _ => continue,
                };
                let typed = |name: &str, member: Type| match member {
                    Type::Null | Type::Undefined => format!("let _ = {name};"),
                    _ => format!("let {name}: &{} = {name};", self.native_type(member)),
                };
                let _ = write!(arms, "({left_pattern}, {right_pattern}) => {{ {} {} {compare} }}, ", typed("left", left_member), typed("right", right_member));
            }
        }
        format!("match (&equality_left, &equality_right) {{ {arms}#[allow(unreachable_patterns)] _ => false }}")
    }

    #[inline(never)]
    fn binary_expression_frame(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        match self.binary_expression_code(fun, expr) {
            Ok(code) => Self::access_wrapped(expr, mode, code),
            Err(complete) => complete,
        }
    }

    // Equality operands are bound by a match scrutinee: its temporaries live
    // for the whole comparison, so a view of an owned call result stays valid.
    fn binary_expression_code(&self, fun: &CheckedFunction, expr: &Expr) -> Result<String, String> {
        self.binary_expression_code_with(expr, &mut |value, mode| self.expr(fun, value, mode))
    }

    fn binary_expression_code_with(&self, expr: &Expr, render: &mut impl FnMut(&Expr, Mode) -> String) -> Result<String, String> {
        let ExprKind::Binary { op, operator, left, right } = &expr.kind else {
            unreachable!("binary expression")
        };
        Ok({
        if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
            && (matches!(left.ty, Some(Type::Null | Type::Undefined))
                || matches!(right.ty, Some(Type::Null | Type::Undefined)))
        {
            let equal = if let Some(union) = left
                .ty
                .and_then(Type::union)
                .or_else(|| right.ty.and_then(Type::union))
            {
                let (name, sentinel) = if left.ty.and_then(Type::union).is_some() {
                    ("equality_left", right.ty.unwrap())
                } else {
                    ("equality_right", left.ty.unwrap())
                };
                let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(index, member)| {
                    let test = if *member == sentinel { "true" } else if sentinel == Type::Null && matches!(member, Type::Record(id) if matches!(self.program.records[*id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) { "payload.value.is_null()" } else { "false" };
                    format!("{}{}::V{index}(payload)=>{test}",if union.copy {"Union"}else{"ViewUnion"},union.index())
                }).collect::<Vec<_>>().join(",");
                format!("match &{name} {{{arms}}}")
            } else if left.ty == Some(Type::Null) && matches!(right.ty, Some(Type::Record(id)) if matches!(self.program.records[id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) {
                "equality_right.value.is_null()".into()
            } else if right.ty == Some(Type::Null) && matches!(left.ty, Some(Type::Record(id)) if matches!(self.program.records[id].native_type, Some(crate::native_signatures::NativeTypeId::JsonValue | crate::native_signatures::NativeTypeId::JsonView))) {
                "equality_left.value.is_null()".into()
            } else {
                (left.ty == right.ty).to_string()
            };
            return Err(format!(
                "match ({}, {}) {{ (equality_left, equality_right) => {{ let _ = (&equality_left, &equality_right); {} }} }}",
                render(left, Mode::Read),
                render(right, Mode::Read),
                if *op == BinaryOp::Equal {
                    equal
                } else {
                    format!("!({equal})")
                }
            ));
        }
        if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
            && (left.ty.and_then(Type::union).is_some() || right.ty.and_then(Type::union).is_some())
        {
            let equal = self.union_equality(left.ty.expect("checked operand"), right.ty.expect("checked operand"));
            return Err(format!(
                "match ({}, {}) {{ (equality_left, equality_right) => {{ let _ = (&equality_left, &equality_right); {} }} }}",
                render(left, Mode::Read),
                render(right, Mode::Read),
                if *op == BinaryOp::Equal { equal } else { format!("!({equal})") }
            ));
        }
        let string = left.ty.map(Type::scalar_base) == Some(Type::String);
        let number = left.ty.map(Type::scalar_base) == Some(Type::Number);
        let left = render(left, if string { Mode::Read } else { Mode::Copy });
        let right = render(right, if string { Mode::Read } else { Mode::Copy });
        let checked = Self::checked_operator(*op);
        if string && *op == BinaryOp::Add {
            let failure = format!(
                "string addition capacity overflow at source bytes {}..{}",
                operator.start, operator.end
            );
            format!(
                "{{ let string_left: &String = {left}; let string_right: &String = {right}; tsr_string_add(string_left, string_right, {failure:?}) }}"
            )
        } else if let Some((method, failure)) = checked.filter(|_| !number) {
            self.checked_integer_result(&format!("({left}).{method}({right})"), failure, *operator)
        } else {
            let operator = match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Remainder => "%",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Coalesce | BinaryOp::PatternDefault => unreachable!("coalescing emitted separately"),
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
                BinaryOp::BitAnd
                | BinaryOp::BitOr
                | BinaryOp::BitXor
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftRight
                | BinaryOp::ShiftRightUnsigned
                | BinaryOp::Exponent => {
                    unreachable!("Number operator dispatched separately")
                }
            };
            if string {
                format!(
                    "{{ let string_left: &String = {left}; let string_right: &String = {right}; string_left {operator} string_right }}"
                )
            } else {
                format!("(({left}) {operator} ({right}))")
            }
        }
        })
    }

    fn expr_inner(&self, fun: &CheckedFunction, expr: &Expr, mode: Mode) -> String {
        let code = match &expr.kind {
            ExprKind::Match { .. } | ExprKind::Sequence(_) | ExprKind::SpreadCall(_)
            | ExprKind::SpreadValue(_)
            | ExprKind::OptionalChain(_)
            | ExprKind::Optional { .. }
            | ExprKind::Selected { .. }
            | ExprKind::Project { .. }
            | ExprKind::Inject { .. }
            | ExprKind::Conditional { .. }
            | ExprKind::Update { .. }
            | ExprKind::LogicalAssign { .. }
            | ExprKind::Assign { .. }
            | ExprKind::CoalesceAssign { .. }
            | ExprKind::CompoundAssign { .. } => {
                unreachable!("updates emitted in their own frame")
            }
            ExprKind::Unary { .. } | ExprKind::Binary { .. } => {
                unreachable!("operators emitted in their own frames")
            }
            ExprKind::Closure { .. } => {
                let Type::Closure(id) = expr.ty.expect("checked closure") else {
                    unreachable!()
                };
                let storage = id;
                let id = expr.closure_origin().unwrap_or(id);
                let fields = self.program.closures[id]
                    .captures
                    .iter()
                    .map(|capture| {
                        format!(
                            "capture{}: {}",
                            capture.inner,
                            if capture.restored {format!("Some({})",self.binding(fun,capture.outer,capture.storage))} else if capture.storage==Mode::Read && matches!(fun.bindings[capture.outer].ty,Type::Union(id)|Type::BorrowedUnion(id) if !id.copy) {
                                let code=self.binding(fun,capture.outer,Mode::Read);
                                if matches!(fun.bindings[capture.outer].ty,Type::BorrowedUnion(_)) || (fun.bindings[capture.outer].parameter && fun.bindings[capture.outer].mode==Mode::Read && !fun.bindings[capture.outer].capture_reference) {code} else {let method=if self.callback_payload(fun.bindings[capture.outer].ty).is_some(){"tsr_view"}else{"tsr_data_view"};format!("({code}).{method}()")}
                            } else if self.record_capture_is_view(capture,&self.program.closures[id]){self.record_capture_value(fun,capture.outer,capture.storage)} else {self.binding(fun, capture.outer, capture.storage)}
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let value = format!("Closure{id} {{ {fields} }}");
                if self.program.polymorphic_storage.contains(storage) {
                    self.program.polymorphic_storage.checked(storage, &self.program.closures, expr.span).expect("checked constructor storage").inject(id, &value, expr.span).expect("checked construction origin")
                } else { value }
            }
            ExprKind::Name(_) => {
                if expr.binding.is_none()
                    && let Some(Type::Closure(id)) = expr.ty
                    && matches!(
                        self.program.closures[id].forward,
                        Some(ClosureForward::Function(_) | ClosureForward::Native(_))
                    )
                {
                    let reference = match mode {
                        Mode::Read => "&",
                        Mode::Mutate => "&mut ",
                        Mode::Consume | Mode::Copy => "",
                    };
                    return format!("{reference}Closure{id} {{}}");
                }
                let binding = expr.binding.expect("checked binding");
                let value = self.binding(fun, binding, mode);
                if let Some(Type::Closure(target)) = expr.ty
                    && let Type::Closure(source) = fun.bindings[binding].ty
                    && target != source
                    && let Some(ClosureForward::Native(native)) =
                        self.program.closures[target].forward
                    && matches!(self.program.closures[source].forward, Some(ClosureForward::Native(id)) if id == native)
                {
                    let reference = match mode {
                        Mode::Read => "&",
                        Mode::Mutate => "&mut ",
                        Mode::Consume | Mode::Copy => "",
                    };
                    // Native origins are capture-free; preserve access to the source value
                    // while materializing the callback's selected argument shape.
                    return format!(
                        "{reference}{{ let _native_origin = {value}; Closure{target} {{}} }}"
                    );
                }
                if let Some(Type::Closure(id)) = expr.ty
                    && let Some(ClosureForward::Once(origin)) = self.program.closures[id].forward
                    && fun.bindings[binding].ty == Type::Closure(origin)
                {
                    return format!("Closure{id} {{ capture0: {value} }}");
                }
                return value;
            }
            ExprKind::Field(_, _) => unreachable!("fields emitted in the small dispatch frame"),
            ExprKind::EnumMember { enumeration, member } => format!("Enum{enumeration}::V{member}"),
            ExprKind::String(value) => self.string_literal(value),

            ExprKind::Integer(value) if expr.ty.map(Type::scalar_base) == Some(Type::Number) => {
                float_literal(value.parse::<f64>().expect("checked number literal"))
            }
            ExprKind::Integer(value) => format!("{value}i64"),
            ExprKind::Number(value) => float_literal(*value),
            ExprKind::Boolean(value) => value.to_string(),
            ExprKind::Null => "TsrNull".into(),
            ExprKind::Undefined => "TsrUndefined".into(),
            ExprKind::SetConstructor(_, _) | ExprKind::MapConstructor(_, _) | ExprKind::NativeConstructor(_, _) | ExprKind::Array(_) | ExprKind::ArrayOf(_, _) | ExprKind::Index(_, _) => {
                unreachable!("arrays emitted in the small dispatch frame")
            }
            ExprKind::Record(_) => unreachable!("records emitted in the small dispatch frame"),
            ExprKind::TypeAssertion { .. } => unreachable!("assertions use the transparent dispatch"),
            ExprKind::Template(_) => unreachable!("templates emitted in their own frame"),
            ExprKind::Call(callee, args) => match expr.target.expect("checked call target") {
                CallTarget::ArrayOf => unreachable!("Array.of lowers to an array expression"),
                CallTarget::SchemaChoice(_) => unreachable!("schema choices use constructor expressions"),
                CallTarget::SchemaDefault => format!("{{let mut schema={};schema.factory=Some(Box::new({}));schema}}",self.expr(fun,&args[0],Mode::Consume),self.expr(fun,&args[1],Mode::Consume)),
                CallTarget::SchemaPresence => {
                    let Type::Record(id)=expr.ty.unwrap() else{unreachable!()};
                    let Type::Record(source)=args[0].ty.unwrap() else{unreachable!()};
                    let source_output=self.program.records[source].application.as_ref().unwrap().resolved_arguments[0];
                    let target_output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
                    if let Some((plan,_))=self.program.schema_choices.for_output(target_output).find(|(_,choice)|choice.presence.is_some()&&choice.branches==[source_output]){
                        return self.schema_choice_constructor(fun,expr,plan,args,mode);
                    }
                    let value=self.expr(fun,&args[0],Mode::Consume);
                    if id==source {value}else{
                        let output=self.program.records[source].application.as_ref().unwrap().resolved_arguments[0];
                        let target=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
                        let mut fields=if self.program.schema_factories.contains_key(&target){String::from("factory:None,")}else{String::new()};
                        let inherited=self.schema_inherited_factories(output);
                        for (source,_) in self.schema_inherited_factories(target){
                            let Type::Union(union)=source else{unreachable!()};
                            let previous=if source==output && self.schema_presence_element(output).is_some() {String::from("previous.factory")}else if inherited.iter().any(|(ty,_)|*ty==source){format!("previous.factory_from{}",union.index())}else{String::from("None")};
                            fields.push_str(&format!("factory_from{}:{previous},",union.index()));
                        }
                        let inner=if self.schema_presence_element(output).is_some(){self.schema_take_child(source,"previous.inner")}else{String::from("previous")};
                        let inner=self.schema_child_value(id,&inner);
                        if self.program.schema_choices.for_output(target).next().is_some(){fields.push_str("choice:None,");}
                        format!("{{let previous={value};Record{}{{{fields}inner:{inner}}}}}",self.native_record_declaration(id))
                    }
                },
                CallTarget::SchemaRetain => {let Type::Record(id)=expr.ty.unwrap() else{unreachable!()};format!("Record{}{{{}inner:{}}}",self.native_record_declaration(id),self.schema_factory_initializer(id),self.schema_child_value(id,&self.expr(fun,&args[0],Mode::Consume)))},
                CallTarget::SchemaStrip => {
                    let Type::Record(id)=expr.ty.unwrap() else{unreachable!()};
                    let output=self.program.records[id].application.as_ref().unwrap().resolved_arguments[0];
                    // An explicit outer policy applies to every attempted branch.
                    // Without it, each branch retains its own unknown-field policy.
                    let policy=if self.schema_has_choices(output){"schema.choice_policy=Some(false);"}else{""};
                    format!("{{let mut schema={};schema.reject_unknown=false;{policy}schema}}",self.expr(fun,&args[0],Mode::Consume))
                },
                CallTarget::SchemaMessagePolicy(retaining) => {
                    let receiver = self.expr(fun, &args[0], Mode::Consume);
                    let Type::Record(wrapper) = expr.ty.unwrap() else { unreachable!() };
                    let child = self.program.schema_pipelines.wrapper(wrapper).unwrap().source;
                    let child_id = self.native_record_declaration(child);
                    let value = if self.program.schema_pipelines.object(child).is_some() {
                        format!("Record{child_id}{{field0:message_plan.field0.field0}}")
                    } else if retaining {
                        format!("Record{child_id}{{{}inner:{}}}", self.schema_factory_initializer(child), self.schema_child_value(child, "message_plan.field0"))
                    } else {
                        let output = self.program.records[child].application.as_ref().unwrap().resolved_arguments[0];
                        let policy = if self.schema_has_choices(output) { "schema.choice_policy=Some(false);" } else { "" };
                        format!("{{let mut schema=message_plan.field0;schema.reject_unknown=false;{policy}schema}}")
                    };
                    let wrapper = self.native_record_declaration(wrapper);
                    format!("{{let message_plan={receiver};Record{wrapper}{{field0:{value},field1:message_plan.field1}}}}")
                },
                CallTarget::SchemaConstraint(index) => {
                    if matches!(expr.ty,Some(Type::Record(id)) if self.program.schema_pipelines.wrapper(id).is_some_and(|plan| matches!(plan.kind,crate::stdlib::schema::pipelines::WrapperKind::Constraint(_)))) {
                        return self.schema_constraint_options_value(fun,expr,args,index);
                    }
                    let receiver = self.expr(fun, &args[0], Mode::Consume);
                    let limit = self.expr(fun, &args[1], Mode::Copy);
                    let name = self.native_call_name(index, args);
                    let call = format!("{name}(message_plan.field0, message_limit)");
                    let call = if let Some((_, source_name)) = self.result_functions.borrow().get(&index) {
                        let unions = crate::unions::UnionArena::from_snapshot_with_records(self.program.unions.clone(), &self.program.records);
                        self.exception_regions.function_result_call(&call, index, source_name, &self.program.error_effects, &unions, expr.span)
                            .expect("checked message constraint Result call")
                    } else { call };
                    let Type::Record(id) = expr.ty.unwrap() else { unreachable!() };
                    let id = self.native_record_declaration(id);
                    format!("{{let message_plan={receiver};let message_limit={limit};let constrained={call};Record{id}{{field0:constrained,field1:message_plan.field1}}}}")
                },
                CallTarget::SchemaAddIssue(id) => {
                    use crate::stdlib::schema::pipelines::DiagnosticSegment;
                    let call=self.program.schema_pipelines.issue_call(id);
                    let path=call.path.iter().map(|segment|match segment {
                        DiagnosticSegment::Field(name)=>format!("tsrust_schema_issues::Segment::Field({name:?})"),
                        DiagnosticSegment::Index(index)=>format!("tsrust_schema_issues::Segment::Index({index})"),
                    }).collect::<Vec<_>>().join(",");
                    let kind=self.schema_kind(call.output);
                    format!("{{let issue_context={};let issue_message={};let start=issue_context.issues.items.len();issue_context.issues.push(&[{path}],tsrust_schema_issues::Code::Refinement,tsrust_schema_issues::Kind::{kind},tsrust_schema_issues::Kind::{kind});if issue_context.issues.items.len()>start{{issue_context.issues.items[start].message=Some(issue_message.into_owned());}}}}",self.expr(fun,&args[0],Mode::Mutate),self.expr(fun,&args[1],Mode::Consume))
                },
                CallTarget::SchemaValidate => self.schema_validate(fun,expr,args),
                CallTarget::MatchYield(id) => format!("break 'match_value{id} {}", self.expr(fun, &args[0], Mode::Consume)),
                CallTarget::ResultUnwrap(error) => self.result_unwrap(fun, expr, &args[0], error),
                CallTarget::SchemaExport(failure) => self.schema_export_call(fun,expr,args,failure),
                CallTarget::JsonEncode => self.json_typed_encode(fun,expr,args),
                CallTarget::JsonDecode => self.json_typed_decode(fun,expr,args),
                CallTarget::NativeBinding(id) => {
                    if let Some(call)=self.sql_native(fun,expr,id,args){return Self::access_wrapped(expr,mode,call);}
                    if let Some(call)=self.sqlite_native(fun,expr,id,args){return Self::access_wrapped(expr,mode,call);}
                    if let Some(call)=self.http_native(fun,expr,id,args){return Self::access_wrapped(expr,mode,call);}
                    if let Some(call) = self.future_native(fun, expr, id, args) { return call; }
                    if let Some(call)=self.cache_native(fun,expr,id,args){return Self::access_wrapped(expr,mode,call);}
                    if let Some(call) = self.thread_native(fun, expr, id, args) { return call; }
                    if let Some(call) = self.program_testing_native(fun, expr, id, args) { return call; }
                    if crate::native_bindings::BINDINGS[id].parameters.first().is_some_and(|p| p.ty == crate::native_bindings::NativeType::ObservedValue) {
                        let binding = &crate::native_bindings::BINDINGS[id];
                        if binding.name == "assertionDetails" {
                            let actual=self.expr(fun,&args[0],Mode::Read);
                            let expected=self.expr(fun,&args[1],Mode::Read);
                            let message=self.expr(fun,&args[2],Mode::Read);
                            let render=|ty:Type,name:&str| match ty {
                                Type::String=>format!("std::format!(\"{{:?}}\",({name}).as_ref())"),
                                Type::Int|Type::Boolean=>format!("({name}).to_string()"),
                                Type::Number=>format!("std::format!(\"{{:?}}\",{name})"),
                                Type::Null=>"std::string::String::from(\"null\")".into(),
                                Type::Undefined=>"std::string::String::from(\"undefined\")".into(),
                                Type::Record(id) if self.record_inspectable(id)=>format!("tsr_console_format(&[{}])",self.console_value(name,ty,"0",false)),
                                Type::Array(element) if matches!(element.ty(),Type::Int|Type::Number|Type::Boolean|Type::String|Type::Null|Type::Undefined)=>format!("tsr_console_format(&[{}])",self.console_value(name,ty,"0",false)),
                                _=>"std::string::String::from(\"<value not expanded>\")".into(),
                            };
                            let actual_text=render(args[0].ty.unwrap(),"assert_actual");
                            let expected_text=render(args[1].ty.unwrap(),"assert_expected");
                            let rendered=format!("{{ let assert_actual={actual}; let assert_expected={expected}; let assert_message={message}; String::Owned(if std::env::var(\"TSRUST_TEST_REPORTER\").as_deref()==Ok(\"pretty\") {{std::format!(\"{{}}\\nExpected: {{}}\\nReceived: {{}}\",assert_message.as_ref(),{expected_text},{actual_text})}} else {{assert_message.as_ref().to_owned()}}) }}");
                            return if mode == Mode::Read {format!("&({rendered})")} else {rendered};
                        }
                        if binding.name == "containsValue" {
                            if args[0].ty == Some(Type::String) {
                                return format!("{{ let actual = {}; let expected = {}; actual.contains(expected.as_ref()) }}", self.expr(fun, &args[0], Mode::Read), self.expr(fun, &args[1], Mode::Read));
                            }
                            return format!("({}) >= 0", self.array_search(fun, &args[0], &args[1..], expr.span, true));
                        }
                        let argument = &args[0];
                        let ty = argument.ty.expect("checked numeric observation");
                        let value = self.expr(fun, argument, Mode::Read);
                        let binding = &crate::native_bindings::BINDINGS[id];
                        if matches!(binding.name, "lengthValue" | "hasLength") {
                            let observed = self.testing_length_observation(ty, "length_actual", binding.name == "hasLength");
                            return format!("{{ let length_actual = {value}; {observed} }}");
                        }
                        if let Some(union) = ty.union() {
                            let prefix = if union.copy { "Union" } else { "ViewUnion" };
                            let arms = self.program.unions[union.index()].members.iter().enumerate().map(|(i, member)| {
                                let result = match member { Type::Int if union.copy => "numeric as f64", Type::Int => "*numeric as f64", Type::Number if union.copy => "numeric", Type::Number => "*numeric", Type::Enum(_, crate::syntax::EnumKind::Number) => "numeric.tsr_value()", _ => "f64::NAN" };
                                format!("{prefix}{}::V{i}(numeric) => {{ let _ = &numeric; {result} }}", union.index())
                            }).collect::<Vec<_>>().join(",");
                            return format!("match {value} {{ {arms} }}");
                        }
                        return match ty { Type::Enum(_, crate::syntax::EnumKind::Number) => format!("({value}).tsr_value()"), Type::Int => format!("({value}) as f64"), Type::Number => value, _ => format!("{{ let _ = {value}; f64::NAN }}") };
                    }
                    self.native_bindings.borrow_mut().insert(id);
                    let binding = &crate::native_bindings::BINDINGS[id];
                    let arguments = args
                        .iter()
                        .enumerate()
                        .map(|(position, argument)| { let parameter=binding.parameters.get(position).or_else(||binding.parameters.last().filter(|p|p.rest)).expect("checked native arity"); self.expr(fun, argument, parameter.access) })
                        .collect::<Vec<_>>();
                    if binding.module=="std:testing-runtime" && binding.name=="checkAssertion" {
                        if let Some(call)=self.typed_native_assertion(&arguments,expr.span,None){return call;}
                    }
                    if binding.module=="std:testing-runtime" && binding.name=="finishTests" {
                        if let Some(call)=self.typed_runner_result("tsrust_testing_finish_tests_result()",expr.span,None){return call;}
                    }
                    if binding.module=="std:testing-runtime" && binding.name=="runTests" {
                        if let Some(call)=self.typed_runner_execution(expr.span,None){return call;}
                    }
                    if binding.module=="std:testing-runtime" && matches!(binding.name,"registerTest"|"registerHook") {
                        if let Some(call)=self.typed_test_registration(&arguments,args[1].ty.expect("checked registration callback"),expr.span,None,binding.name=="registerHook"){return call;}
                    }
                    if binding.module=="std:testing-runtime" && matches!(binding.name,"beginSuite"|"beginFile"|"endSuite") {
                        let call=if binding.name=="beginSuite"{format!("tsrust_testing_begin_suite_result({})",arguments[0])}else if binding.name=="beginFile"{format!("tsrust_testing_begin_file_result({})",arguments[0])}else{"tsrust_testing_end_suite_result()".into()};
                        if let Some(call)=self.typed_runner_result(&call,expr.span,None){return call;}
                    }
                    if binding.module=="std:ui-runtime" && binding.name=="button" { return self.ui_press_adapter(&arguments,args[3].ty.expect("UI handler"),expr.span); }
                    if binding.module=="std:path" && binding.name=="relative" {
                        let call=crate::native_emit::call(binding,arguments);
                        self.typed_native_result(&call,expr.span,None,"Error","path-relative").unwrap_or_else(||format!("{call}.unwrap_or_else(|message|panic!(\"path.relative: {{message}}\"))"))
                    }else{crate::native_emit::call(binding, arguments)}
                }
                CallTarget::JsonStringView => {
                    let receiver=self.expr(fun,&args[0],Mode::Read);
                    let text=self.json_text_read(args[0].ty.unwrap(),"json_source");
                    let Type::Record(id)=expr.ty.unwrap() else{unreachable!()};
                    format!("{{let json_source={receiver};Record{id}{{text:{text}}}}}")
                }
                CallTarget::JsonBoolean => self.json_boolean_projection(fun, &args[0]),
                CallTarget::JsonStringLength => {
                    let receiver=self.expr(fun,&args[0],Mode::Read);
                    let text=self.json_text_read(args[0].ty.unwrap(),"json_source");
                    format!("{{let json_source={receiver};let text:&str={text};tsr_length(text)}}")
                }
                target @ (CallTarget::BooleanConversion
                | CallTarget::StringConversion
                | CallTarget::ArrayIsArray) => self.value_builtin_call(fun, target, args),

                target @ (CallTarget::MathRound
                | CallTarget::MathSign
                | CallTarget::MathFround
                | CallTarget::MathF16round
                | CallTarget::MathRandom
                | CallTarget::MathMin
                | CallTarget::MathMax
                | CallTarget::MathClz32
                | CallTarget::MathImul) => self.math_exact_call(fun, target, args),
                target @ (CallTarget::MathAcos
                | CallTarget::MathAcosh
                | CallTarget::MathAsin
                | CallTarget::MathAsinh
                | CallTarget::MathAtan
                | CallTarget::MathAtanh
                | CallTarget::MathCbrt
                | CallTarget::MathCos
                | CallTarget::MathCosh
                | CallTarget::MathExp
                | CallTarget::MathExpm1
                | CallTarget::MathLog
                | CallTarget::MathLog10
                | CallTarget::MathLog1p
                | CallTarget::MathLog2
                | CallTarget::MathSin
                | CallTarget::MathSinh
                | CallTarget::MathTan
                | CallTarget::MathTanh
                | CallTarget::MathAtan2
                | CallTarget::MathPow
                | CallTarget::MathHypot) => self.math_approx_call(fun, target, args),
                CallTarget::StringRaw => format!(
                    "{{ let raw_segments = {}; let raw_substitutions = {}; let mut raw_output = std::string::String::new(); for (raw_index, raw_segment) in raw_segments.iter().enumerate() {{ if raw_index != 0 {{ if let Some(value) = raw_substitutions.get(raw_index - 1) {{ raw_output.push_str(&value); }} }} raw_output.push_str(&raw_segment); }} String::Owned(raw_output) }}",
                    self.expr(fun, &args[0], Mode::Read),
                    self.expr(fun, &args[1], Mode::Read)
                ),
                CallTarget::StringFromCodePoint => format!(
                    "{{ let string_codes = {}; tsr_string_from_code_points(string_codes.iter().copied(), \"String.fromCodePoint code point must be between 0 and 1114111 at source bytes {}..{}\") }}",
                    self.expr(fun, &args[0], Mode::Read),
                    expr.span.start,
                    expr.span.end
                ),
                CallTarget::StringFromCharCode => format!(
                    "{{ let string_codes = {}; tsr_string_from_code_units(string_codes.iter().map(|code| *code as u16)) }}",
                    self.expr(fun, &args[0], Mode::Read)
                ),
                CallTarget::ObjectValues => self.object_values(fun, expr, &args[0], None),
                CallTarget::ObjectEntries(pair) => self.object_values(fun, expr, &args[0], Some(pair)),
                CallTarget::ObjectHasOwn => self.object_has_own(fun,args,matches!(&callee.kind,ExprKind::Field(_,name) if name=="hasOwn#in")),
                CallTarget::ObjectKeys => {
                    let value = self.expr(fun, &args[0], Mode::Read);
                    let keys = match args[0].ty.expect("checked Object.keys input") {
                        Type::Record(id) if !self.program.records[id].is_tuple() => {
                            let names = self.program.records[id].fields.iter().map(|field| format!("{:?}", field.name.text)).collect::<Vec<_>>().join(",");
                            format!("{{ let key_names: [&str; {}] = [{names}]; object_keys_value.key_order.iter().map(|index| String::Borrowed(key_names[*index])).collect::<Vec<_>>() }}",self.program.records[id].fields.len())
                        }
                        Type::Record(id) => format!("(0..{}).map(|index| String::Owned(index.to_string())).collect::<Vec<_>>()", self.program.records[id].fields.len()),
                        Type::Array(_) => "(0..object_keys_value.len()).map(|index| String::Owned(index.to_string())).collect::<Vec<_>>()".into(),
                        Type::String => "(0..object_keys_value.as_ref().encode_utf16().count()).map(|index| String::Owned(index.to_string())).collect::<Vec<_>>()".into(),
                        _ => "Vec::<String>::new()".into(),
                    };
                    format!("{{ let object_keys_value = {value}; let mut object_keys_result = <{}>::from([]); for key in {keys} {{ object_keys_result.push(key); }} object_keys_result }}", self.native_type(expr.ty.expect("checked keys result")))
                }
                CallTarget::OrderedSetMethod(method)=>self.ordered_set_method_expression(fun,callee,args,method),
                CallTarget::HeapMethod(method) => self.heap_method_expression(fun,expr,callee,args,method),
                CallTarget::DequeMethod(method) => self.deque_method_expression(fun,expr,callee,args,method),
                CallTarget::MapMethod(method) => self.map_method_expression(fun,expr,callee,args,method),
                    CallTarget::SetMethod(method) => self.set_method_expression(fun, expr, callee, args, method),
                CallTarget::ObjectIs => {
                    let (left_type, right_type) = (args[0].ty.expect("checked operand").scalar_base(), args[1].ty.expect("checked operand").scalar_base());
                    let numeric = |ty: Type| matches!(ty, Type::Int | Type::Number);
                    let union_compare;
                    let compare = if left_type.union().is_some() || right_type.union().is_some() {
                        union_compare = self.union_object_is(left_type, right_type);
                        union_compare.as_str()
                    } else if left_type == right_type && matches!(left_type, Type::Int | Type::Boolean | Type::String) {
                        "object_left == object_right"
                    } else if left_type == right_type && matches!(left_type, Type::Enum(_, crate::syntax::EnumKind::Number)) {
                        "{ let left = object_left.tsr_value(); let right = object_right.tsr_value(); left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan()) }"
                    } else if left_type == right_type && matches!(left_type, Type::Enum(_, crate::syntax::EnumKind::String)) {
                        "object_left.tsr_value() == object_right.tsr_value()"
                    } else if numeric(left_type) && numeric(right_type) {
                        // Same value: NaN is itself, and the two zeros differ.
                        "{ let left = object_left as f64; let right = object_right as f64; left.to_bits() == right.to_bits() || (left.is_nan() && right.is_nan()) }"
                    } else if left_type == right_type && matches!(left_type, Type::Null | Type::Undefined) {
                        "true"
                    } else if left_type == right_type {
                        "std::ptr::eq(object_left, object_right)"
                    } else {
                        "false"
                    };
                    format!(
                        "{{ let object_left = {}; let object_right = {}; {compare} }}",
                        self.expr(fun, &args[0], Mode::Read),
                        self.expr(fun, &args[1], Mode::Read)
                    )
                }
                CallTarget::NumberParseInt => format!(
                    "{{ let parse_int_text: &String = {}; let parse_int_radix: Option<i64> = {}; tsr_number_parse_int(parse_int_text.as_ref(), parse_int_radix) }}",
                    self.expr(fun, &args[0], Mode::Read),
                    args.get(1).map_or_else(|| "None".to_owned(), |radix| format!("Some({})", self.expr(fun, radix, Mode::Copy)))
                ),
                CallTarget::NumberParseFloat => format!(
                    "{{ let parse_float_text: &String = {}; tsr_number_parse_float(parse_float_text.as_ref()) }}",
                    self.expr(fun, &args[0], Mode::Read)
                ),
                CallTarget::Number => match args.first() {
                    None => "0.0f64".into(),
                    Some(argument) => match argument.ty.expect("checked Number argument").scalar_base() {
                        Type::Union(union) | Type::BorrowedUnion(union) => self.union_observation(union, &self.expr(fun, argument, Mode::Read), CallTarget::Number),
                        Type::Null => format!(
                            "{{ let _ = {}; 0.0f64 }}",
                            self.expr(fun, argument, Mode::Copy)
                        ),
                        Type::Undefined => format!(
                            "{{ let _ = {}; f64::NAN }}",
                            self.expr(fun, argument, Mode::Copy)
                        ),
                        Type::Enum(_, crate::syntax::EnumKind::Number) => format!("({}).tsr_value()", self.expr(fun, argument, Mode::Copy)),
                        Type::Enum(_, crate::syntax::EnumKind::String) => format!("tsr_number_from_text(({}).tsr_value())", self.expr(fun, argument, Mode::Copy)),
                        Type::Number => self.expr(fun, argument, Mode::Copy),
                        Type::Int => format!("({}) as f64", self.expr(fun, argument, Mode::Copy)),
                        Type::Boolean => format!(
                            "if {} {{1.0f64}} else {{0.0f64}}",
                            self.expr(fun, argument, Mode::Copy)
                        ),
                        Type::String => format!(
                            "{{ let number_text: &String = {}; tsr_number_from_text(number_text.as_ref()) }}",
                            self.expr(fun, argument, Mode::Read)
                        ),
                        _ => unreachable!("checked Number argument"),
                    },
                },
                target @ (CallTarget::NumberIsFinite
                | CallTarget::NumberIsNaN
                | CallTarget::NumberIsInteger
                | CallTarget::NumberIsSafeInteger) => {
                    let value = self.expr(fun, &args[0], Mode::Copy);
                    let predicate = match target {
                        CallTarget::NumberIsFinite => "number_value.is_finite()",
                        CallTarget::NumberIsNaN => "number_value.is_nan()",
                        CallTarget::NumberIsInteger => {
                            "number_value.is_finite() && number_value.fract() == 0.0f64"
                        }
                        CallTarget::NumberIsSafeInteger => {
                            "number_value.is_finite() && number_value.fract() == 0.0f64 && number_value.abs() <= 9007199254740991.0f64"
                        }
                        _ => unreachable!("checked number predicate"),
                    };
                    format!("{{ let number_value: f64 = {value}; {predicate} }}")
                }
                target @ (CallTarget::MathAbs
                | CallTarget::MathFloor
                | CallTarget::MathCeil
                | CallTarget::MathTrunc
                | CallTarget::MathSqrt) => {
                    let value = self.expr(fun, &args[0], Mode::Copy);
                    let method = match target {
                        CallTarget::MathAbs => "abs",
                        CallTarget::MathFloor => "floor",
                        CallTarget::MathCeil => "ceil",
                        CallTarget::MathTrunc => "trunc",
                        CallTarget::MathSqrt => "sqrt",
                        _ => unreachable!("checked Math operation"),
                    };
                    format!("{{ let math_value: f64 = {value}; math_value.{method}() }}")
                }
                CallTarget::ArrayLastIndexOf => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_last_index_of(fun, base, args, expr.span)
                }
                CallTarget::ArrayIncludes | CallTarget::ArrayIndexOf => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_search(
                        fun,
                        base,
                        args,
                        expr.span,
                        expr.target == Some(CallTarget::ArrayIndexOf),
                    )
                }
                CallTarget::StringStartsWith
                | CallTarget::StringEndsWith
                | CallTarget::StringIncludes
                | CallTarget::StringIndexOf
                | CallTarget::StringLastIndexOf => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Read);
                    let needle = self.expr(fun, &args[0], Mode::Read);
                    let position = args.get(1).map_or_else(
                        || "None".into(),
                        |arg| format!("Some({})", self.expr(fun, arg, Mode::Copy)),
                    );
                    let method = match expr.target {
                        Some(CallTarget::StringStartsWith) => "tsr_string_starts_with",
                        Some(CallTarget::StringEndsWith) => "tsr_string_ends_with",
                        Some(CallTarget::StringIncludes) => "tsr_string_includes",
                        Some(CallTarget::StringIndexOf) => "tsr_string_index_of",
                        Some(CallTarget::StringLastIndexOf) => "tsr_string_last_index_of",
                        _ => unreachable!("checked string search"),
                    };
                    let text=self.json_text_read(base.ty.unwrap(),"string_source");
                    let needle_text=self.json_text_read(args[0].ty.unwrap(),"needle_source");
                    format!(
                        "{{ let string_source = {receiver}; let string_receiver: &str = {text}; let needle_source = {needle}; let string_needle: &str = {needle_text}; let string_position = {position}; {method}(string_receiver, string_needle, string_position) }}"
                    )
                }
                CallTarget::ArrayReduce | CallTarget::ArrayReduceRight => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_reduce(
                        fun,
                        base,
                        args,
                        expr.ty.expect("checked reduction result"),
                        expr.target == Some(CallTarget::ArrayReduceRight),
                        expr.span,
                    )
                }
                CallTarget::ArrayForEach => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_for_each(fun, base, &args[0])
                }
                CallTarget::ArrayFlat(levels) => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_flat(fun, expr, base, args, levels)
                }
                CallTarget::ArrayFlatMap => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_flat_map(fun, expr, base, &args[0])
                }
                CallTarget::ArrayMap | CallTarget::ArrayFilter | CallTarget::MapGroupBy => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_transform(
                        fun,
                        base,
                        &args[0],
                        expr.ty.expect("checked transform result"),
                        expr.target == Some(CallTarget::ArrayFilter),
                        matches!(&callee.kind,ExprKind::Field(_,name) if name=="map#arrayFrom" || name=="groupBy#map"),
                    )
                }
                CallTarget::NumberToFixed
                | CallTarget::NumberToExponential
                | CallTarget::NumberToPrecision => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Copy);
                    let precision = args.first().map_or_else(
                        || "None".into(),
                        |argument| format!("Some({})", self.expr(fun, argument, Mode::Copy)),
                    );
                    let (helper, method, range) = match expr
                        .target
                        .expect("checked number formatting")
                    {
                        CallTarget::NumberToFixed => ("tsr_number_fixed", "toFixed", "0 and 100"),
                        CallTarget::NumberToExponential => {
                            ("tsr_number_exponential", "toExponential", "0 and 100")
                        }
                        CallTarget::NumberToPrecision => {
                            ("tsr_number_precision", "toPrecision", "1 and 100")
                        }
                        _ => unreachable!("checked number formatting"),
                    };
                    let failure = format!(
                        "Number.{method} precision must be between {range} at source bytes {}..{}",
                        expr.span.start, expr.span.end
                    );
                    format!(
                        "{{ let number_receiver: f64 = {receiver}; let number_precision: Option<i64> = {precision}; {helper}(number_receiver, number_precision, {failure:?}) }}"
                    )
                }
                CallTarget::IntToLocaleString | CallTarget::NumberToLocaleString => {
                    let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
                    let receiver = self.expr(fun, base, Mode::Copy);
                    let helper = if expr.target == Some(CallTarget::IntToLocaleString) {
                        "tsr_int_locale_string"
                    } else {
                        "tsr_number_locale_string"
                    };
                    format!("{helper}({receiver})")
                }
                CallTarget::NumberToString => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Copy);
                    let radix = args.first().map_or_else(
                        || "10i64".into(),
                        |argument| self.expr(fun, argument, Mode::Copy),
                    );
                    let failure = format!(
                        "number.toString radix must be between 2 and 36 at source bytes {}..{}",
                        expr.span.start, expr.span.end
                    );
                    format!(
                        "{{ let number_receiver: f64 = {receiver}; let number_radix: i64 = {radix}; tsr_number_to_string(number_receiver, number_radix, {failure:?}) }}"
                    )
                }
                CallTarget::IntToString => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Copy);
                    let radix = args.first().map_or_else(
                        || "10i64".into(),
                        |argument| self.expr(fun, argument, Mode::Copy),
                    );
                    let failure = format!(
                        "int.toString radix must be between 2 and 36 at source bytes {}..{}",
                        expr.span.start, expr.span.end
                    );
                    format!(
                        "{{ let integer_receiver: i64 = {receiver}; let integer_radix: i64 = {radix}; tsr_int_to_string(integer_receiver, integer_radix, {failure:?}) }}"
                    )
                }
                CallTarget::StringIsWellFormed => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Read);
                    // Valid UTF-8 storage guarantees the predicate, but receiver
                    // construction, effects, and ordinary temporary disposal remain.
                    format!(
                        "{{ let well_formed_receiver: &String = {receiver}; let _ = well_formed_receiver; true }}"
                    )
                }
                CallTarget::StringIdentity
                | CallTarget::BooleanIdentity
                | CallTarget::BooleanToString
                | CallTarget::NumberIdentity
                | CallTarget::IntIdentity => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    match expr.target.expect("checked primitive identity") {
                        CallTarget::StringIdentity => {
                            format!("tsr_string_identity({})", self.expr(fun, base, Mode::Read))
                        }
                        CallTarget::BooleanToString => {
                            format!("tsr_boolean_string({})", self.expr(fun, base, Mode::Copy))
                        }
                        _ => self.expr(fun, base, Mode::Copy),
                    }
                }
                CallTarget::StringHtml(_)
                | CallTarget::StringConcat
                | CallTarget::StringPadStart
                | CallTarget::StringPadEnd
                | CallTarget::StringRepeat
                | CallTarget::StringReplace
                | CallTarget::StringReplaceAll
                | CallTarget::StringSplit => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.string_construct(
                        fun,
                        base,
                        args,
                        expr.target.expect("checked string constructor"),
                        expr.span,
                    )
                }
                CallTarget::ArraySome
                | CallTarget::ArrayEvery
                | CallTarget::ArrayFindIndex
                | CallTarget::ArrayFindLastIndex
                | CallTarget::ArrayFind
                | CallTarget::ArrayFindLast => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_predicate(fun, base, &args[0], expr)
                }
                CallTarget::StringNormalize => {
                    let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
                    let receiver=self.expr(fun,base,Mode::Read);
                    let (evaluate,form)=args.first().map_or_else(||(std::string::String::new(),"None".to_owned()), |argument| {
                        let temporary=matches!(argument.ty,Some(Type::Union(id)) if !id.copy) && Self::temporary_storage(argument);
                        let value=self.expr(fun,argument,if temporary { Mode::Consume } else { Mode::Read });
                        let setup=if temporary { format!("let normalization_owner = {value}; let normalization_form = normalization_owner.tsr_view();") } else { format!("let normalization_form = {value};") };
                        let form=match argument.ty.expect("checked normalization form") {
                            Type::Undefined=>"None".to_owned(),
                            Type::String=>"Some(normalization_form.as_ref())".to_owned(),
                            Type::Union(id)|Type::BorrowedUnion(id)=> {
                                let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(index,member)| {
                                    let result=if *member==Type::String { "Some(payload.as_ref())" } else { "None" };
                                    format!("ViewUnion{}::V{index}(payload)=>{result}",id.index())
                                }).collect::<Vec<_>>().join(",");
                                format!("match normalization_form {{ {arms} }}")
                            }
                            _=>unreachable!("checked normalization form"),
                        };
                        (setup,form)
                    });
                    let failure=format!("normalize form must be NFC, NFD, NFKC or NFKD at source bytes {}..{}",expr.span.start,expr.span.end);
                    format!("{{ let normalization_receiver = {receiver}; {evaluate} tsr_string_normalize(normalization_receiver.as_ref(), {form}, {failure:?}) }}")
                }
                CallTarget::ArrayToLocaleString => {
                    let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
                    let receiver=self.expr(fun, base, Mode::Read);
                    let iterator=if self.expression_shape(base)==Some(crate::fixed_storage::Shape::LiteralStrings) { "literal_iter()" } else { "iter()" };
                    format!("{{ let locale_receiver = {receiver}; tsr_array_locale(locale_receiver.{iterator}) }}")
                }
                CallTarget::ArrayJoin => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_join(fun, base, args)
                }
                CallTarget::ArrayConcat => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_concat(fun, base, args, expr.span)
                }
                CallTarget::RecordSpread(_) => unreachable!("object spread children use expression work stack"),
                CallTarget::PatternRest(result,start) => {
                    let ExprKind::Field(base,_) = &callee.kind else {unreachable!()};
                    let record_rest=start==usize::MAX;
                    let fields=self.program.records[result].fields.iter().enumerate().map(|(index,field)|{
                        let key=if record_rest {field.name.text.clone()}else{(index+start).to_string()};
                        let selected=Expr::checked(ExprKind::Field(base.clone(),key),base.span,field.ty);
                        let copied=if field.ty.expect("checked rest field").is_copy(){self.expr(fun,&selected,Mode::Copy)}else{self.clone_argument(fun,&selected)};
                        format!("field{index}:{copied}")
                    }).collect::<Vec<_>>().join(",");
                    if !record_rest {format!("Record{result}{{{fields}}}")}
                    else if self.program.records[result].fields.is_empty(){format!("Record{result}{{key_order:[]}}")}
                    else {
                        let Type::Record(source)=base.ty.expect("checked rest source") else{unreachable!()};
                        let order=self.record_parameter_order_access(fun,base).unwrap_or_else(||format!("({}).key_order",self.expr(fun,base,Mode::Consume)));
                        let count=self.program.records[result].fields.len();
                        let arms=self.program.records[result].fields.iter().enumerate().map(|(index,field)|format!("{}=>{{rest_order[rest_next]={index};rest_next+=1;}},",self.fields[source][field.name.text.as_str()])).collect::<String>();
                        format!("{{let mut rest_order=[0usize;{count}];let mut rest_next=0usize;for rest_index in ({order}).iter() {{match rest_index{{{arms}_=>{{}}}}}}Record{result}{{{fields},key_order:rest_order}}}}")
                    }
                }
                CallTarget::ArraySlice => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_slice(fun, base, args)
                }
                CallTarget::ArrayToSpliced => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_to_spliced(fun, base, args, expr.span)
                }
                CallTarget::ArrayEntries(pair) => {
                    let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
                    self.array_entries(fun,base,expr.ty.expect("checked entries result"),pair)
                }
                CallTarget::ArrayKeys => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let receiver = self.expr(fun, base, Mode::Read);
                    let keys = self.native_type(expr.ty.expect("checked keys result"));
                    format!("{{ let keys_length = ({receiver}).len(); let mut keys = <{keys}>::from([]); let mut key = 0i64; while (key as usize) < keys_length {{ keys.push(key); key += 1; }} keys }}")
                }
                CallTarget::ArrayToReversed | CallTarget::ArrayWith => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_copying(fun, base, args, expr.span)
                }
                CallTarget::ArrayFill | CallTarget::ArrayCopyWithin => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_range_mutation(
                        fun,
                        base,
                        args,
                        expr.target == Some(CallTarget::ArrayFill),
                    )
                }
                CallTarget::ArraySort | CallTarget::ArrayToSorted => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_sort(
                        fun,
                        base,
                        args,
                        expr.target == Some(CallTarget::ArrayToSorted),
                    )
                }
                CallTarget::ArrayReverse => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_reverse(fun, base)
                }
                CallTarget::ArraySplice => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_splice(fun, base, args, expr.span)
                }
                CallTarget::ArrayUnshift => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_unshift(fun, base, args, expr.span)
                }
                CallTarget::ArrayPush => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_push(fun, base, &args[0])
                }
                CallTarget::ArrayAt => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_at(fun, expr, base, &args[0])
                }
                CallTarget::ArrayPop | CallTarget::ArrayShift => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_optional_removal(
                        fun,
                        expr,
                        base,
                        expr.target == Some(CallTarget::ArrayShift),
                    )
                }
                CallTarget::ArrayRemove => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.array_remove(fun, base, &args[0], expr.span)
                }
                CallTarget::Clone if matches!(&args[0].kind,ExprKind::Project { narrowed:Type::Union(target),union,.. } if target!=union) =>
                {
                    let ExprKind::Project {
                        value,
                        union,
                        narrowed: Type::Union(target),
                    } = &args[0].kind
                    else {
                        unreachable!()
                    };
                    let source = self.conditional_arm(fun, value, Mode::Read);
                    let members = &self.program.unions[union.index()].members;
                    let arms=self.program.unions[target.index()].members.iter().enumerate().map(|(to,ty)| {
                        let from=members.iter().position(|member|member==ty).expect("proved clone alternative");
                        format!("ViewUnion{}::V{from}(payload) => Union{}::V{to}(Clone::clone(payload))",union.index(),target.index())
                    }).collect::<Vec<_>>().join(", ");
                    format!(
                        "match {source} {{ {arms}, _ => unreachable!(\"checked nullable guard\") }}"
                    )
                }
                CallTarget::Clone if matches!(args[0].ty,Some(Type::Union(id)|Type::BorrowedUnion(id)) if !id.copy) =>
                {
                    let argument = self.expr(fun, &args[0], Mode::Read);
                    format!("({argument}).tsr_owned()")
                }
                CallTarget::Clone => self.clone_argument(fun, &args[0]),
                CallTarget::Log => self.console_call(fun, callee, args),
                CallTarget::Trim
                | CallTarget::TrimStart
                | CallTarget::TrimEnd
                | CallTarget::Lower
                | CallTarget::Upper => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!("method receiver")
                    };
                    if expr.target == Some(CallTarget::Lower)
                        && base.target == Some(CallTarget::Trim)
                        && let ExprKind::Call(receiver, _) = &base.kind
                        && let ExprKind::Field(input, _) = &receiver.kind
                    {
                        format!("tsr_normalize({})", self.expr(fun, input, Mode::Read))
                    } else {
                        let method = match expr.target {
                            Some(CallTarget::Trim) => "tsr_trim",
                            Some(CallTarget::TrimStart) => "tsr_trim_start",
                            Some(CallTarget::TrimEnd) => "tsr_trim_end",
                            Some(CallTarget::Lower) => "tsr_lower",
                            Some(CallTarget::Upper) => "tsr_upper",
                            _ => unreachable!("checked string transform"),
                        };
                        format!("{method}({})", self.expr(fun, base, Mode::Read))
                    }
                }
                CallTarget::StringCodePointSlot | CallTarget::StringCharCodeAt => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!("string receiver")
                    };
                    let receiver = self.expr(fun, base, Mode::Read);
                    let position = args
                        .first()
                        .map_or_else(|| "0i64".into(), |arg| self.expr(fun, arg, Mode::Copy));
                    // Positions count whole characters; a negative position names none.
                    let character = "if code_position < 0 { None } else { code_text.as_ref().chars().nth(code_position as usize) }";
                    if expr.target == Some(CallTarget::StringCodePointSlot) {
                        let slot = self.native_type(expr.ty.expect("checked code point slot"));
                        format!("{{ let code_text: &String = {receiver}; let code_position: i64 = {position}; let mut code_slot = <{slot}>::from([]); if let Some(character) = {character} {{ code_slot.push(character as u32 as i64); }} code_slot }}")
                    } else {
                        format!("{{ let code_text: &String = {receiver}; let code_position: i64 = {position}; match {character} {{ Some(character) => {{ let mut units = [0u16; 2]; character.encode_utf16(&mut units)[0] as f64 }} None => f64::NAN }} }}")
                    }
                }
                CallTarget::StringSlice
                | CallTarget::StringSubstring
                | CallTarget::StringSubstr
                | CallTarget::StringCharAt => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!("string receiver")
                    };
                    let receiver = self.expr(fun, base, Mode::Read);
                    let start = args
                        .first()
                        .map_or_else(|| "0i64".into(), |arg| self.expr(fun, arg, Mode::Copy));
                    if matches!(
                        expr.target,
                        Some(
                            CallTarget::StringSlice
                                | CallTarget::StringSubstring
                                | CallTarget::StringSubstr
                        )
                    ) {
                        let end = args.get(1).map_or_else(
                            || "None".into(),
                            |arg| format!("Some({})", self.expr(fun, arg, Mode::Copy)),
                        );
                        let operation = if matches!(
                            expr.target,
                            Some(CallTarget::StringSubstring | CallTarget::StringSubstr)
                        ) {
                            let method = if expr.target == Some(CallTarget::StringSubstr) {
                                "substr"
                            } else {
                                "substring"
                            };
                            let failure = format!(
                                "string {method} is not on UTF-8 boundaries at source bytes {}..{}",
                                expr.span.start, expr.span.end
                            );
                            format!(
                                "tsr_{method}(string_receiver, string_start, string_end, {failure:?})"
                            )
                        } else {
                            "tsr_slice(string_receiver, string_start, string_end)".into()
                        };
                        format!(
                            "{{ let string_receiver = {receiver}; let string_start = {start}; let string_end = {end}; {operation} }}"
                        )
                    } else {
                        format!(
                            "{{ let string_receiver = {receiver}; let string_index = {start}; tsr_char_at(string_receiver, string_index) }}"
                        )
                    }
                }
                CallTarget::ForwardFunction(_) | CallTarget::ClosureMethod(_) => {
                    unreachable!("callable children are scheduled by expression work stack")
                }
                CallTarget::ConditionalClosure(_) | CallTarget::ConditionalCallback(_)
                | CallTarget::Closure(_) | CallTarget::StoredCallable(_)
                | CallTarget::StoredCallableField(_) | CallTarget::Callback(_) => {
                    unreachable!("callable children are scheduled by expression work stack")
                }
                CallTarget::Function(_) => {
                    unreachable!("ordinary function call has its own emission frame")
                }
            },
        };
        Self::access_wrapped(expr, mode, code)
    }

    fn access_wrapped(expr: &Expr, mode: Mode, code: String) -> String {
        if matches!(expr.ty, Some(Type::BorrowedUnion(_))) {
            return code;
        }
        match mode {
            Mode::Read => format!("&({code})"),
            Mode::Mutate => format!("&mut ({code})"),
            Mode::Consume | Mode::Copy => code,
        }
    }
}

#[cfg(test)]
mod exception_region_tests {
    #[test]
    fn read_method_reborrows_an_exclusively_stored_capture() {
        let mut checked=crate::check("function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action=select;const one=action(1);const text=action('x');}console.log(calls);}").unwrap();
        let origin=*checked.program.deferred_generic_values.iter().next().unwrap();
        let method=&mut checked.program.closures[origin].methods[0];
        method.invocation=crate::Mode::Read;
        method.function.body.remove(0);
        let slot=method.captures[0];
        method.function.bindings[slot].mode=crate::Mode::Read;
        method.function.bindings[slot].mutable=false;
        let rust=super::emit(&checked.program);
        let directory=std::env::temp_dir().join(format!("tsrust-method-reborrow-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("source.rs");let binary=directory.join("source");std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let output=std::process::Command::new(binary).output().unwrap();assert!(output.status.success());assert_eq!(output.stdout,b"1\n");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn materialized_methods_use_their_own_receiver_access() {
        let mut checked = crate::check("function identity<T>(value:T):T{return value;}function main(){const action=identity;console.log(action(1));console.log(action(true));console.log(action('text'));}").unwrap();
        let origin = *checked.program.deferred_generic_values.iter().next().unwrap();
        checked.program.closures[origin].invocation = crate::Mode::Mutate;
        checked.program.closures[origin].methods[0].invocation = crate::Mode::Read;
        checked.program.closures[origin].methods[1].invocation = crate::Mode::Mutate;
        checked.program.closures[origin].methods[2].invocation = crate::Mode::Consume;
        for function in &mut checked.program.functions {
            for binding in &mut function.bindings {
                if binding.ty == crate::syntax::Type::Closure(origin) { binding.mutable = true; }
            }
        }
        let rust = super::emit(&checked.program);
        assert!(rust.contains("fn call0(&self,"), "read method must borrow its receiver");
        assert!(rust.contains("fn call1(&mut self,"), "mutating method must borrow exclusively");
        assert!(rust.contains("fn call2(self,"), "consuming method must own its receiver");
        let directory = std::env::temp_dir().join(format!("tsrust-method-access-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("source.rs");
        let binary = directory.join("source");
        std::fs::write(&source, rust).unwrap();
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let compiled = std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let output = std::process::Command::new(binary).output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"1\ntrue\ntext\n");
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn source_generic_replacements_construct_and_dispatch_native_family_variants() {
        let source = crate::format("function identity<T>(value:T):T{return value;}function unused<T>(){let action=identity;action=identity;}function relay<T>(value:T):T{let action=identity;action=identity;return action<T>(value);}function main(){let action=identity;console.log(action<int>(3));const saved=action;action=identity;console.log(action<string>('owned'));console.log(saved<int>(7));console.log(relay<int>(9));console.log(relay<string>('kept'));}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let families = checked.program.polymorphic_storage.families(&checked.program.closures, crate::Span::default()).unwrap();
        assert_eq!(families.len(), 2, "main and capture-free relay retain separate storage families");
        let rust = super::emit(&checked.program);
        for family in families {
            assert!(family.origins.len() >= 2, "fresh replacement origins remain registered");
            for (variant, origin) in family.origins.iter().enumerate() {
                assert!(rust.contains(&format!("TsrGenericFamily{}::V{variant}(Closure{origin}", family.id)),
                    "source construction must retain each concrete environment in its native storage variant");
            }
        }
        assert!(rust.contains("match method_environment"), "source calls must dispatch through the checked family");
        crate::scoped_record_places::tests::native_output(&rust, "3\nowned\n7\n9\nkept\n");
    }

    #[test]
    fn registered_generic_storage_uses_checked_native_environments() {
        let mut checked = crate::check("function first<T>(value:T):T{return value;}function second<T>(value:T):T{return value;}function inactive<T>(left:T,right:T):T{return right;}function main(){const one=first;const two=second;const unused=inactive;one<int>(3);two<string>('warm');two<int>(4);}").unwrap();
        checked.program.polymorphic_storage.observe(0, 1);
        checked.program.polymorphic_storage.observe(1, 2);
        assert!(checked.program.closures[2].methods.is_empty());
        let plan = crate::polymorphic_invocation_plans::Plan::checked([
            crate::polymorphic_invocation_plans::Pair { origin: 0, method: 0 },
            crate::polymorphic_invocation_plans::Pair { origin: 1, method: 1 },
        ], &checked.program.closures, crate::Span::default()).unwrap();
        let invocation = plan.render("&family", &["4i64".into()], "1", |_| false, false, Some((0, &[0, 1, 2])), crate::Span::default()).unwrap();
        let mut rust = super::emit(&checked.program);
        assert!(rust.contains("enum TsrGenericFamily0<E0, E1, E2> { V0(E0), V1(E1), V2(E2) }"));
        rust.push_str(&format!("fn carrier_witness() {{ let mut family: TsrGenericFamily0<Closure0,Closure1,Closure2> = TsrGenericFamily0::V0(Closure0{{}}); assert_eq!({invocation},4); let saved=family; family=TsrGenericFamily0::V1(Closure1{{}}); assert_eq!({invocation},4); let family=saved; assert_eq!({invocation},4); let inactive: TsrGenericFamily0<Closure0,Closure1,Closure2> = TsrGenericFamily0::V2(Closure2{{}}); drop(inactive); }}"));
        let main=rust.find("fn main()").unwrap();
        let opening=main+rust[main..].find('{').unwrap()+1;
        rust.insert_str(opening,"carrier_witness();");
        crate::scoped_record_places::tests::native(&rust);
    }
    #[test]
    fn selected_generic_method_uses_its_own_result_transport() {
        let source=crate::format("function first<T>(value:T):int{return 1;}function second<T>(value:T):int{return 8/0;}function main(){const one=first;const two=second;one<int>(3);two<int>(4);}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let mut checked=crate::check(&source).unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut checked.program);
        let main=checked.program.functions.iter().position(|function|function.name=="main").unwrap();
        let calls=checked.program.functions[main].body.iter().filter_map(|statement|match &statement.kind {crate::syntax::StmtKind::Expr(call)=>Some(call),_=>None}).collect::<Vec<_>>();
        let crate::syntax::ExprKind::Call(callee,_)=&calls[1].kind else {panic!("second call");};
        let Some(crate::syntax::Type::Closure(origin))=callee.ty else {panic!("second environment");};
        let Some(crate::syntax::CallTarget::ClosureMethod(method))=calls[1].target else {panic!("second method");};
        let mut statement=checked.program.functions[main].body.iter().find(|statement|matches!(statement.kind,crate::syntax::StmtKind::Expr(_))).unwrap().clone();
        let crate::syntax::StmtKind::Expr(call)=&mut statement.kind else {unreachable!()};
        let id=checked.program.polymorphic_invocations.publish([crate::polymorphic_invocation_plans::Pair{origin,method}],&checked.program.closures,call.span).unwrap();
        let crate::syntax::ExprKind::Call(callee,_)=&mut call.kind else {unreachable!()};
        callee.generic_invocation=Some(id);
        let emitter=super::emitter_for(&checked.program,"");
        emitter.result_methods.borrow_mut().insert((origin,method),"SelectedMethodError".into());
        let contract=checked.program.error_effects.method_errors[&(origin,method)].clone();
        let body=emitter.callable_result_body(&checked.program.functions[main],&[statement],123,"DestinationError",contract,Some(runtime));
        assert!(body.contains("SelectedMethodError"),"selected Result transport missing: {body}");
    }
    #[test]
    fn source_exception_pipeline_executes_catch_finally_and_propagation() {
        let source="function unused(action:Mutable<Function<void>>):void{action();}function risky():int{throw \"zero\";}function value():int{try{return risky();}catch(error){if(error===\"zero\"){return 4;}return 0;}finally{console.log(7);}}function main(){console.log(value());}";
        let checked=crate::check(source).unwrap();
        let rust=super::emit(&checked.program);
        let directory=std::env::temp_dir().join(format!("tsrust-source-exception-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("source.rs");let binary=directory.join("source");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert_eq!(executed.stdout,b"7\n4\n");assert!(executed.stderr.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn abstract_forwarder_captures_have_checked_callback_bounds() {
        let checked=crate::check("function keep(f:Function<int,[n:int]>):(n:int)=>int{return f;}function main(){}").unwrap();
        let emitter=super::emitter_for(&checked.program, "");
        let closure=checked.program.closures.iter().find(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Abstract(_)))).unwrap();
        let bounds=emitter.captured_callback_bounds(closure);
        assert!(bounds.iter().any(|bound|bound.contains("Callback<(i64,), i64>")),"{bounds:?}");
    }
    #[test]
    fn shared_read_forwarder_capture_bounds_preserve_immutable_access() {
        let mut checked=crate::check("function keep(f:Function<int,[n:int]>):(n:int)=>int{return f;}function main(){}").unwrap();
        let outer=checked.program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Abstract(_)))).unwrap();
        let Some(crate::analysis::ClosureForward::Abstract(contract))=checked.program.closures[outer].forward else {unreachable!()};
        // Shared-read contracts are internal owned immutable environments.
        checked.program.callback_contracts[contract].kind=crate::syntax::CallbackKind::SharedRead;
        let emitter=super::emitter_for(&checked.program, "");
        let bounds=emitter.captured_callback_bounds(&checked.program.closures[outer]);
        assert!(bounds.iter().any(|bound|bound.contains("ReadCallback<(i64,), i64>")),"shared immutable capture requires read invocation: {bounds:?}");
    }
    #[test]
    fn concrete_closure_calls_use_registered_result_contract() {
        let mut program=crate::check("function value():int{const leaf=(n:int):int=>n/0;return leaf(64);}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program, "");
        let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let contract=program.program.error_effects.closure_errors[0].clone();
        let (_,name)=emitter.completion_error_layout("ClosureError",&contract,Some(runtime),function.body[0].span);
        emitter.result_closures.borrow_mut().insert(0,name.clone());
        let contract=program.program.error_effects.function_errors[program.program.functions.iter().position(|function|function.name=="value").unwrap()].clone();
        let (_,caller_name)=emitter.completion_error_layout("CallerError",&contract,Some(runtime),function.body[0].span);
        let body=emitter.callable_result_body(function,&function.body,73,&caller_name,contract,Some(runtime));
        assert!(body.contains("tsr_from_result"),"{body}");
        assert!(body.contains(".map_err("),"{body}");
        assert!(body.contains(".call(64i64)"),"{body}");
    }
    #[test]
    fn registered_closure_signature_executes_native_result() {
        let mut program=crate::check("function value():int{const leaf=(n:int):int=>64/n;return 64;}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let mut emitter=super::emitter_for(&program.program, "");
        emitter.callback_traits=true;emitter.counted_calls=true;
        let closure=&program.program.closures[0];
        let contract=program.program.error_effects.closure_errors[0].clone();
        let (declaration,name)=emitter.completion_error_layout("ClosureError",&contract,Some(runtime),closure.function.body[0].span);
        emitter.result_closures.borrow_mut().insert(0,name);
        let mut rust=emitter.program();
        assert!(rust.contains("-> Result<i64,ClosureError<Record0>>"),"registered closure signature missing");
        rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declaration);
        rust.push_str("fn closure_witness(){let leaf=Closure0{};assert!(matches!(leaf.call(1),Ok(64)));assert!(matches!(leaf.call_with_count(1,1),Ok(64)));assert!(matches!(OnceCallback::invoke_once(Closure0{},(1,)),Ok(64)));assert!(matches!(OnceCallback::invoke_once_with_count(Closure0{},(1,),1),Ok(64)));match leaf.call(0){Err(ClosureError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"closure error lost\")}} ");
        let mut caller_program=crate::check("function value(a:int):int{const leaf=(n:int):int=>64/n;return leaf(a);}function main(){}").unwrap();
        let caller_runtime=crate::error_effects::materialize_runtime_record(&mut caller_program.program);
        let mut caller_emitter=super::emitter_for(&caller_program.program, "");
        caller_emitter.callback_traits=true;caller_emitter.counted_calls=true;
        caller_emitter.result_closures.borrow_mut().insert(0,"ClosureError<Record0>".into());
        let index=caller_program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let function=&caller_program.program.functions[index];
        let contract=caller_program.program.error_effects.function_errors[index].clone();
        let (caller_declaration,caller_name)=caller_emitter.completion_error_layout("CallerError",&contract,Some(caller_runtime),function.body[0].span);
        let caller_body=caller_emitter.callable_result_body(function,&function.body,80,&caller_name,contract,Some(caller_runtime));
        assert!(caller_body.contains(".call_with_count("),"{caller_body}");
        rust.push_str(&caller_declaration);
        rust.push_str(&format!("fn source_closure_caller(value0:i64)->Result<i64,{caller_name}>{{{caller_body}}}fn caller_witness(){{assert!(matches!(source_closure_caller(1),Ok(64)));match source_closure_caller(0){{Err(CallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"source closure caller lost failure\")}}}}"));
        let mut conditional_program=crate::check("function value(a:int,flag:boolean):int{const leaf=(n:int):int=>64/n;return (flag?leaf:leaf)(a);}function main(){}").unwrap();
        let conditional_runtime=crate::error_effects::materialize_runtime_record(&mut conditional_program.program);
        let mut conditional_emitter=super::emitter_for(&conditional_program.program, "");conditional_emitter.counted_calls=true;conditional_emitter.callback_traits=true;
        conditional_emitter.result_closures.borrow_mut().insert(0,"ClosureError<Record0>".into());
        let index=conditional_program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let function=&conditional_program.program.functions[index];let contract=conditional_program.program.error_effects.function_errors[index].clone();
        let (declaration,name)=conditional_emitter.completion_error_layout("ConditionalCallerError",&contract,Some(conditional_runtime),function.body[0].span);
        let body=conditional_emitter.callable_result_body(function,&function.body,82,&name,contract,Some(conditional_runtime));
        assert!(body.contains("tsr_from_result"),"{body}");
        rust.push_str(&declaration);rust.push_str(&format!("fn conditional_caller(value0:i64,value1:bool)->Result<i64,{name}>{{{body}}}fn conditional_witness(){{for flag in [false,true]{{assert!(matches!(conditional_caller(1,flag),Ok(64)));match conditional_caller(0,flag){{Err(ConditionalCallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"conditional error lost\")}}}}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"closure_witness();caller_witness();conditional_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-closure-result-signature-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("closure.rs");let binary=directory.join("closure");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert!(executed.stdout.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn registered_function_value_forwards_native_result() {
        let mut program=crate::check("function leaf(n:int):int{return 64/n;}function value():int{const saved=leaf;return 64;}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program, "");
        let leaf=program.program.functions.iter().position(|function|function.name=="leaf").unwrap();
        let closure=program.program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Function(index)) if index==leaf)).unwrap();
        let function=&program.program.functions[leaf];
        let contract=program.program.error_effects.function_errors[leaf].clone();
        let (leaf_declaration,leaf_name)=emitter.completion_error_layout("LeafError",&contract,Some(runtime),function.body[0].span);
        let (forward_declaration,forward_name)=emitter.completion_error_layout("ForwardError",&program.program.error_effects.closure_errors[closure],Some(runtime),function.body[0].span);
        emitter.result_functions.borrow_mut().insert(leaf,("result_leaf".into(),leaf_name.clone()));
        emitter.result_closures.borrow_mut().insert(closure,forward_name);
        let mut rust=emitter.program();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&leaf_declaration);rust.push_str(&forward_declaration);
        rust.push_str(&format!("fn forward_witness(){{let saved=Closure{closure}{{}};assert!(matches!(saved.call(1),Ok(64)));match saved.call(0){{Err(ForwardError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"forwarded error lost\")}}}}"));
        let mut caller_program=crate::check("function leaf(n:int):int{return 64/n;}function value(a:int):int{const saved=leaf;return saved(a);}function main(){}").unwrap();
        let caller_runtime=crate::error_effects::materialize_runtime_record(&mut caller_program.program);
        let caller_emitter=super::emitter_for(&caller_program.program, "");
        let caller_leaf=caller_program.program.functions.iter().position(|function|function.name=="leaf").unwrap();
        caller_emitter.result_functions.borrow_mut().insert(caller_leaf,("result_leaf".into(),leaf_name.clone()));
        let caller_index=caller_program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let caller=&caller_program.program.functions[caller_index];let caller_contract=caller_program.program.error_effects.function_errors[caller_index].clone();
        let (caller_declaration,caller_name)=caller_emitter.completion_error_layout("ForwardCallerError",&caller_contract,Some(caller_runtime),caller.body[0].span);
        let caller_body=caller_emitter.callable_result_body(caller,&caller.body,91,&caller_name,caller_contract,Some(caller_runtime));
        assert!(caller_body.contains("result_leaf("),"{caller_body}");
        rust.push_str(&caller_declaration);
        rust.push_str(&format!("fn source_forward_caller(value0:i64)->Result<i64,{caller_name}>{{{caller_body}}}fn forward_caller_witness(){{assert!(matches!(source_forward_caller(1),Ok(64)));match source_forward_caller(0){{Err(ForwardCallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"forward call lost error\")}}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"forward_witness();forward_caller_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-forward-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("forward.rs");let binary=directory.join("forward");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert!(executed.stdout.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn registered_once_adapter_converts_direct_and_counted_errors() {
        let mut program=crate::check("function value():void{const saved:Once<Function<int,[x:int]>>=(n:int):int=>64/n;}function main(){}").unwrap();
        let adapter=program.program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Once(_)))).unwrap();
        let Some(crate::analysis::ClosureForward::Once(inner))=program.program.closures[adapter].forward else {unreachable!()};
        program.program.functions.iter_mut().find(|function|function.name=="value").unwrap().body.clear();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let mut emitter=super::emitter_for(&program.program, "");emitter.callback_traits=true;emitter.counted_calls=true;
        let span=program.program.closures[inner].span;
        let (inner_declaration,inner_name)=emitter.completion_error_layout("InnerError",&program.program.error_effects.closure_errors[inner],Some(runtime),span);
        let (adapter_declaration,adapter_name)=emitter.completion_error_layout("AdapterError",&program.program.error_effects.closure_errors[adapter],Some(runtime),span);
        emitter.result_closures.borrow_mut().insert(inner,inner_name);emitter.result_closures.borrow_mut().insert(adapter,adapter_name);
        let mut rust=emitter.program();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&inner_declaration);rust.push_str(&adapter_declaration);
        rust.push_str(&format!("fn adapter_witness(){{assert!(matches!(Closure{adapter}{{capture0:Closure{inner}{{}}}}.call(1),Ok(64)));assert!(matches!(Closure{adapter}{{capture0:Closure{inner}{{}}}}.call_with_count(1,1),Ok(64)));match (Closure{adapter}{{capture0:Closure{inner}{{}}}}).call(0){{Err(AdapterError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"direct adapter error lost\")}}match (Closure{adapter}{{capture0:Closure{inner}{{}}}}).call_with_count(1,0){{Err(AdapterError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"counted adapter error lost\")}}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"adapter_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-once-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("once.rs");let binary=directory.join("once");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert!(executed.stdout.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn exception_saved_family_registration_rejects_unready_members_atomically() {
        let mut checked=crate::check("function value():void{const actions:((n:int)=>int)[]=[(n:int):int=>64/n,(n:int):int=>n];}function main(){}").unwrap();
        crate::error_effects::materialize_runtime_record(&mut checked.program);
        let emitter=super::emitter_for(&checked.program, "");
        let (family,_,_)=checked.program.stored_contracts.families().find(|(_,_,origins)|origins.len()==2).unwrap();
        let span=crate::Span{start:4,end:9};
        let error=emitter.prepare_storage_result(family,"StoredError",span).unwrap_err();
        assert_eq!(error.code,"error-effect");assert_eq!(error.span,span);
        assert!(emitter.result_storage.borrow().is_empty());
        assert!(emitter.prepare_storage_result(usize::MAX,"Missing",span).is_err());
        assert!(emitter.result_storage.borrow().is_empty());
    }
    #[test]
    fn registered_stored_family_emits_native_result_dispatch() {
        let mut program=crate::check("function value(a:int,field:boolean):int{const actions:((n:int)=>int)[]=[(n:int):int=>64/n,(n:int):int=>n];if(field){return actions[0](a);}let selected=actions.remove(0);return selected(a);}function main(){}").unwrap();
        let caller_index=program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let caller_contract=program.program.error_effects.function_errors[caller_index].clone();
        assert!(!caller_contract.opaque,"stored caller must have a resolved saved contract");
        let original=program.program.functions.iter_mut().find(|function|function.name=="value").unwrap();let caller=crate::analysis::CheckedFunction{storage_exits:original.storage_exits.clone(),native_returns:original.native_returns.clone(), native_mutations:original.native_mutations.clone(),template:original.template,specialization:original.specialization,name:original.name.clone(),bindings:original.bindings.clone(),parameter_count:original.parameter_count,result:original.result,body:original.body.clone()};original.body.clear();original.result=crate::syntax::Type::Void;
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let mut emitter=super::emitter_for(&program.program, "");emitter.counted_calls=true;emitter.callback_traits=true;
        let (family,_,origins)=program.program.stored_contracts.families().find(|(_,_,origins)|origins.len()==2).unwrap();
        let failing=*origins.iter().find(|&&origin|program.program.error_effects.closure_errors[origin].runtime).unwrap();
        let pure=*origins.iter().find(|&&origin|!program.program.error_effects.closure_errors[origin].runtime).unwrap();
        let contract=program.program.error_effects.closure_errors[failing].clone();let span=program.program.closures[failing].span;
        let (member_declaration,member_name)=emitter.completion_error_layout("MemberError",&contract,Some(runtime),span);
        emitter.result_closures.borrow_mut().insert(failing,member_name);
        let (family_declaration,_)=emitter.prepare_storage_result(family,"StoredError",span).unwrap();
        let mut rust=emitter.program();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&member_declaration);rust.push_str(&family_declaration);
        rust.push_str(&format!("fn stored_witness(){{let mut failing=StoredCallable{family}::V{failing}(Closure{failing}{{}});assert!(matches!(failing.call(1),Ok(64)));match failing.call_with_count(1,0){{Err(StoredError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"stored error lost\")}}let mut pure=StoredCallable{family}::V{pure}(Closure{pure}{{}});assert!(matches!(pure.call(64),Ok(64)));assert!(matches!(pure.call_with_count(1,64),Ok(64)));}}"));
        let (caller_declaration,caller_name)=emitter.completion_error_layout("StoredCallerError",&caller_contract,Some(runtime),caller.body[0].span);
        let caller_body=emitter.callable_result_body(&caller,&caller.body,96,&caller_name,caller_contract,Some(runtime));
        assert!(caller_body.contains("tsr_from_result"),"{caller_body}");
        rust.push_str(&caller_declaration);rust.push_str(&format!("fn source_stored_caller(value0:i64,value1:bool)->Result<i64,{caller_name}>{{{caller_body}}}fn stored_caller_witness(){{for field in [false,true]{{assert!(matches!(source_stored_caller(1,field),Ok(64)));match source_stored_caller(0,field){{Err(StoredCallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"stored source error lost\")}}}}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"stored_witness();stored_caller_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-stored-emitter-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("stored.rs");let binary=directory.join("stored");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert!(executed.stdout.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn exception_fixed_result_dispatch_preserves_specialized_borrowed_arrays() {
        let mut checked=crate::check("function leaf(values:int[],n:int):int{return values[n];}function value(n:int):int{const values=[64];return leaf(values,n);}function bump(values:int[],n:int):int{values[n]+=1;return values[n];}function mutate(n:int):int{const values=[64];return bump(values,n);}type Box={value:int;children:Box[];};function boxed(box:Box,n:int):int{return box.children[n].value;}function records(n:int):int{const box:Box={value:0,children:[{value:64,children:[]}]};return boxed(box,n);}function text(values:string[],n:int):int{return values[n].length;}function strings(n:int):int{const values=[\"abcd\"];return text(values,n);}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut checked.program);
        let emitter=super::emitter_for(&checked.program, "");let mut declarations=String::new();
        for (source,symbol,error) in [("leaf","result_leaf","LeafError"),("value","result_caller","CallerError"),("bump","result_bump","BumpError"),("mutate","result_mutator","MutatorError"),("boxed","result_boxed","BoxError"),("records","result_records","RecordsError"),("text","result_text","TextError"),("strings","result_strings","StringsError")] {
            let id=checked.program.functions.iter().position(|function|function.name==source).unwrap();let function=&checked.program.functions[id];
            let (declaration,name)=emitter.completion_error_layout(error,&checked.program.error_effects.function_errors[id],Some(runtime),function.body[0].span);
            declarations.push_str(&declaration);emitter.result_functions.borrow_mut().insert(id,(symbol.into(),name));
        }
        let mut rust=emitter.program();assert!(rust.contains("fn result_leaf_fixed"),"fixed Result specialization missing");
        assert!(rust.contains("fn result_boxed_fixed"),"fixed record specialization missing");
        assert!(rust.contains("fn result_text_fixed"),"fixed string specialization missing");
        rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declarations);
        rust.push_str("fn fixed_witness(){assert!(matches!(result_caller(0),Ok(64)));match result_caller(1){Err(CallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"bounds\"),_=>panic!(\"fixed failure lost\")}assert!(matches!(result_mutator(0),Ok(65)));match result_mutator(1){Err(MutatorError::V0(error))=>assert_eq!(error.field1.as_ref(),\"bounds\"),_=>panic!(\"fixed mutation failure lost\")}assert!(matches!(result_records(0),Ok(64)));match result_records(1){Err(RecordsError::V0(error))=>assert_eq!(error.field1.as_ref(),\"bounds\"),_=>panic!(\"fixed record failure lost\")}assert!(matches!(result_strings(0),Ok(4)));match result_strings(1){Err(StringsError::V0(error))=>assert_eq!(error.field1.as_ref(),\"bounds\"),_=>panic!(\"fixed string failure lost\")}}");
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"fixed_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-fixed-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("fixed.rs");let binary=directory.join("fixed");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert!(executed.stdout.is_empty());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn exception_registered_entry_returns_failure_without_native_panic() {
        let mut checked=crate::check("function main(){const n=1/0;}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut checked.program);
        let emitter=super::emitter_for(&checked.program, "cleanup_witness(); pure_entry::verify();");let id=checked.program.functions.iter().position(|function|function.name=="main").unwrap();
        let function=&checked.program.functions[id];let (declaration,name)=emitter.completion_error_layout("EntryError",&checked.program.error_effects.function_errors[id],Some(runtime),function.body[0].span);
        emitter.result_functions.borrow_mut().insert(id,("result_main".into(),name));
        let mut rust=emitter.program();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declaration);
        rust.push_str("static ENTRY_DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);static ENTRY_REPORTS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);struct EntryOwner;impl Drop for EntryOwner{fn drop(&mut self){ENTRY_DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}fn cleanup_witness(){use std::sync::atomic::Ordering::SeqCst;let reporter=Box::new(EntryOwner);let success=tsr_entry_result::<Box<EntryOwner>,Box<EntryOwner>>(Ok(Box::new(EntryOwner)),move |_|{ENTRY_REPORTS.fetch_add(1,SeqCst);drop(reporter);});assert_eq!(success,std::process::ExitCode::SUCCESS);assert_eq!(ENTRY_DROPS.load(SeqCst),2);assert_eq!(ENTRY_REPORTS.load(SeqCst),0);let reporter=Box::new(EntryOwner);let failure=tsr_entry_result::<Box<EntryOwner>,Box<EntryOwner>>(Err(Box::new(EntryOwner)),move |_|{assert_eq!(ENTRY_DROPS.load(SeqCst),2);ENTRY_REPORTS.fetch_add(1,SeqCst);drop(reporter);});assert_eq!(failure,std::process::ExitCode::FAILURE);assert_eq!(ENTRY_DROPS.load(SeqCst),4);assert_eq!(ENTRY_REPORTS.load(SeqCst),1);}");
        let pure=crate::check("function main(){}").unwrap();let pure_emitter=super::emitter_for(&pure.program, "");
        let pure_main=pure.program.functions.iter().position(|function|function.name=="main").unwrap();
        pure_emitter.result_functions.borrow_mut().insert(pure_main,("result_main".into(),"std::convert::Infallible".into()));
        let mut pure_rust=pure_emitter.program();pure_rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);pure_rust.push_str("pub fn verify(){assert_eq!(main(),std::process::ExitCode::SUCCESS);}");
        rust.push_str(&format!("mod pure_entry{{{pure_rust}}}"));
        let directory=std::env::temp_dir().join(format!("tsrust-entry-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("entry.rs");let binary=directory.join("entry");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert_eq!(executed.status.code(),Some(1));assert!(executed.stdout.is_empty());let stderr=String::from_utf8(executed.stderr).unwrap();assert!(stderr.starts_with("Uncaught RangeError:"),"{stderr}");assert!(!stderr.contains("panicked"),"{stderr}");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn exception_result_declaration_failure_preserves_all_registries() {
        let mut checked=crate::check("function invoke(f:Function<int,[n:int]>,n:int):int{return f(n);}function main(){}").unwrap();
        // An unused template is skipped; exercise an emitted unresolved consumer.
        checked.program.functions.iter_mut().find(|function|function.name=="invoke").unwrap().template=false;
        crate::error_effects::materialize_runtime_record(&mut checked.program);
        let emitter=super::emitter_for(&checked.program, "");
        emitter.result_functions.borrow_mut().insert(99,("saved".into(),"SavedError".into()));
        emitter.result_callbacks.borrow_mut().insert(99,("SavedCallbackError".into(),Default::default()));
        let error=emitter.prepare_result_declarations().unwrap_err();assert_eq!(error.code,"error-effect");
        assert_eq!(emitter.result_functions.borrow().get(&99).unwrap().0,"saved");
        assert_eq!(emitter.result_callbacks.borrow().get(&99).unwrap().0,"SavedCallbackError");
        assert_eq!(emitter.result_callbacks.borrow().len(),1);
        assert_eq!(emitter.result_functions.borrow().len(),1);assert!(emitter.result_closures.borrow().is_empty());assert!(emitter.result_storage.borrow().is_empty());
    }
    #[test]
    fn exception_result_declarations_connect_entire_native_call_chain() {
        let mut checked=crate::check("function leaf(n:int):int{return 64/n;}function stored(n:int):int{const actions:((x:int)=>int)[]=[(x:int):int=>64/x,(x:int):int=>x];return actions[0](n);}function main(){console.log(leaf(1));console.log(stored(1));console.log(stored(0));}").unwrap();
        crate::error_effects::materialize_runtime_record(&mut checked.program);
        let emitter=super::emitter_for(&checked.program, "");
        let declarations=emitter.prepare_result_declarations().unwrap();
        let mut rust=emitter.program();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declarations);
        let directory=std::env::temp_dir().join(format!("tsrust-result-collection-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("result.rs");let binary=directory.join("result");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert_eq!(executed.status.code(),Some(1));assert_eq!(executed.stdout,b"64\n64\n");let stderr=String::from_utf8(executed.stderr).unwrap();assert!(stderr.starts_with("Uncaught RangeError:"),"{stderr}");assert!(!stderr.contains("panicked"),"{stderr}");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn exception_abstract_callback_result_bounds_propagate_native_failures() {
        let mut checked=crate::check("function invoke(f:Function<int,[n:int]>,n:int):int{return f(n);}function conditional(f:Function<int,[n:int]>,n:int):int{return(n>0?f:f)(n);}function main(){console.log(invoke((n:int):int=>64/n,1));console.log(conditional((n:int):int=>64/n,2));console.log(invoke((n:int):int=>n,7));console.log(conditional((n:int):int=>64/n,0));}").unwrap();
        // Exercise the retained generic checked body alongside concrete instantiations.
        for generic in checked.program.functions.iter_mut().filter(|function|matches!(function.name.as_str(),"invoke" | "conditional") && function.template){
            assert!(matches!(generic.bindings[0].ty,crate::syntax::Type::Callback(_)));
            generic.template=false;
        }
        crate::error_effects::materialize_runtime_record(&mut checked.program);
        checked.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut checked.program);
        assert!(checked.program.error_effects.function_errors.iter().all(|contract|!contract.opaque));
        let emitter=super::emitter_for(&checked.program, "direct_failure_witness();");
        let declarations=emitter.prepare_result_declarations().unwrap();
        let callback_name=emitter.result_callbacks.borrow().get(&0).expect("saved callback contract must be prepared").0.clone();
        let mut rust=emitter.program();assert!(rust.contains(&format!("F0: Callback<(i64,), Result<i64,{callback_name}>>")),"callback Result bound missing: {:?}",rust.lines().filter(|line|line.starts_with("fn tsr_result_function")).collect::<Vec<_>>());rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declarations);
        let invoke=checked.program.functions.iter().position(|function|function.name=="invoke" && !function.template).unwrap();
        let argument=match checked.program.functions[invoke].bindings[0].mode {super::Mode::Read=>"&callback",super::Mode::Mutate=>"&mut callback",_=>"callback"};
        rust.push_str(&format!("fn direct_failure_witness(){{let mut callback=Closure0{{}};match tsr_result_function{invoke}({argument},0){{Err(TsrFunctionError{invoke}::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"direct callback error lost\")}}}}"));
        let conditional=checked.program.functions.iter().position(|function|function.name=="conditional" && !function.template).unwrap();
        let argument=match checked.program.functions[conditional].bindings[0].mode {super::Mode::Read=>"&callback",super::Mode::Mutate=>"&mut callback",_=>"callback"};
        rust.push_str(&format!("fn conditional_failure_witness(){{let mut callback=Closure1{{}};match tsr_result_function{conditional}({argument},0){{Err(TsrFunctionError{conditional}::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"conditional callback error lost\")}}}}"));
        rust.push_str(&format!("fn pure_callback_witness(){{let direct:i64=Closure2{{}}.call(9);assert_eq!(direct,9);let mut callback=Closure2{{}};assert!(matches!(tsr_result_function{invoke}(callback,8),Ok(8)));}}"));
        rust=rust.replace("direct_failure_witness();", "direct_failure_witness();conditional_failure_witness();pure_callback_witness();");
        let mut subset=crate::check("function mixed(f:Function<int,[n:int]>,n:int):int{return f(n);}function main(){mixed((n:int):int=>64/n,1);mixed((n:int):int=>n,1);}").unwrap();
        // Stage a checked string throw while public exception checking remains disabled.
        let mut payload=crate::syntax::Expr::new(crate::syntax::ExprKind::String("subset".into()),crate::Span::default());payload.ty=Some(crate::syntax::Type::String);
        subset.program.closures[1].function.body[0].kind=crate::syntax::StmtKind::Throw(payload);
        crate::error_effects::materialize_runtime_record(&mut subset.program);
        subset.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut subset.program);
        let mut subset_emitter=super::emitter_for(&subset.program, "");subset_emitter.counted_calls=true;
        let subset_declarations=subset_emitter.prepare_result_declarations().unwrap();
        let (target_name,target)=subset_emitter.result_callbacks.borrow().get(&0).unwrap().clone();
        let unions=crate::unions::UnionArena::from_snapshot(subset.program.unions.clone());
        let runtime=subset.program.records.iter().position(|record|record.name.text=="#RuntimeError").map(crate::syntax::Type::Record).unwrap();
        let payloads=target.native_payloads(&unions,Some(runtime),crate::Span::default()).unwrap();
        let runtime_variant=payloads.iter().position(|ty|*ty==runtime).unwrap();let string_variant=payloads.iter().position(|ty|*ty==crate::syntax::Type::String).unwrap();
        let mut subset_rust=subset_emitter.program();subset_rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);subset_rust.push_str(&subset_declarations);
        let target_path=target_name.split('<').next().unwrap();
        let runtime_source=subset_emitter.result_closures.borrow().get(&0).unwrap().clone();
        let string_source=subset_emitter.result_closures.borrow().get(&1).unwrap().clone();
        assert_ne!(runtime_source,target_name);assert_ne!(string_source,target_name);
        subset_rust.push_str(&format!("fn narrow_direct_witness(){{let runtime:Result<i64,{runtime_source}>=Closure0{{}}.call(1);assert!(matches!(runtime,Ok(64)));let text:Result<i64,{string_source}>=Closure1{{}}.call(1);assert!(text.is_err());}}"));
        subset_rust.push_str(&format!("pub fn verify(){{narrow_direct_witness();let mut callback=Closure0{{}};let success:Result<i64,{target_name}>=ReadCallback::invoke_read(&callback,(1,));assert!(matches!(success,Ok(64)));let error:Result<i64,{target_name}>=Callback::invoke(&mut callback,(0,));assert!(matches!(error,Err({target_path}::V{runtime_variant}(error)) if error.field1.as_ref()==\"arithmetic\"));let error:Result<i64,{target_name}>=OnceCallback::invoke_once_with_count(callback,(0,),1);assert!(matches!(error,Err({target_path}::V{runtime_variant}(error)) if error.field1.as_ref()==\"arithmetic\"));let mut callback=Closure1{{}};let error:Result<i64,{target_name}>=ReadCallback::invoke_read_with_count(&callback,(1,),1);assert!(matches!(error,Err({target_path}::V{string_variant}(error)) if error.as_ref()==\"subset\"));let error:Result<i64,{target_name}>=Callback::invoke_with_count(&mut callback,(1,),1);assert!(matches!(error,Err({target_path}::V{string_variant}(error)) if error.as_ref()==\"subset\"));let error:Result<i64,{target_name}>=OnceCallback::invoke_once(callback,(1,));assert!(matches!(error,Err({target_path}::V{string_variant}(error)) if error.as_ref()==\"subset\"));}}"));
        rust.push_str(&format!("mod subset_bridge{{{subset_rust}}}"));
        rust=rust.replace("pure_callback_witness();", "pure_callback_witness();subset_bridge::verify();");
        let mut forwarded=crate::check("function keep(f:Function<int,[n:int]>):(n:int)=>int{return f;}function main(){const f=keep((n:int):int=>64/n);}").unwrap();
        let (outer,contract)=forwarded.program.closures.iter().enumerate().find_map(|(id,closure)|match closure.forward{Some(crate::analysis::ClosureForward::Abstract(contract))=>Some((id,contract)),_=>None}).unwrap();
        forwarded.program.closures[outer].function.template=false;
        let inner=forwarded.program.closures.iter().position(|closure|closure.forward.is_none()).unwrap();
        crate::error_effects::materialize_runtime_record(&mut forwarded.program);
        forwarded.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut forwarded.program);
        assert!(!forwarded.program.error_effects.closure_errors[outer].opaque);
        let mut forward_emitter=super::emitter_for(&forwarded.program, "");forward_emitter.counted_calls=true;
        let declarations=forward_emitter.prepare_result_declarations().unwrap();
        let name=forward_emitter.result_callbacks.borrow().get(&contract).unwrap().0.clone();let path=name.split('<').next().unwrap();
        let mut forwarded_rust=forward_emitter.program();forwarded_rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);forwarded_rust.push_str(&declarations);
        forwarded_rust.push_str(&format!("pub fn verify(){{let mut f=Closure{outer}{{capture0:Closure{inner}{{}}}};let value:Result<i64,{name}>=f.call(1);assert!(matches!(value,Ok(64)));let error:Result<i64,{name}>=f.call_with_count(1,0);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));}}"));
        rust.push_str(&format!("mod abstract_forward_bridge{{{forwarded_rust}}}"));rust=rust.replace("subset_bridge::verify();", "subset_bridge::verify();abstract_forward_bridge::verify();");
        for (module,annotation,expression,fallible) in [("pure_abstract_forward","Function<int,[n:int]>","n",false),("once_abstract_forward","Once<Function<int,[n:int]>>","64/n",true)] {
            let source=format!("function keep(f:{annotation}):(n:int)=>int{{return f;}}function main(){{const f=keep((n:int):int=>{expression});}}");
            let mut checked=crate::check(&source).unwrap();
            let (outer,contract)=checked.program.closures.iter().enumerate().find_map(|(id,closure)|match closure.forward{Some(crate::analysis::ClosureForward::Abstract(contract))=>Some((id,contract)),_=>None}).unwrap();
            checked.program.closures[outer].function.template=false;
            let inner=checked.program.closures.iter().position(|closure|closure.forward.is_none()).unwrap();
            crate::error_effects::materialize_runtime_record(&mut checked.program);
            checked.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut checked.program);
            let mut emitter=super::emitter_for(&checked.program, "");emitter.counted_calls=true;
            let declarations=emitter.prepare_result_declarations().unwrap();let mut code=emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
            let verification=if fallible {
                let name=emitter.result_callbacks.borrow().get(&contract).unwrap().0.clone();let path=name.split('<').next().unwrap();
                format!("let value:Result<i64,{name}>=Closure{outer}{{capture0:Closure{inner}{{}}}}.call(1);assert!(matches!(value,Ok(64)));let error:Result<i64,{name}>=Closure{outer}{{capture0:Closure{inner}{{}}}}.call_with_count(1,0);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));")
            }else{format!("let mut f=Closure{outer}{{capture0:Closure{inner}{{}}}};let value:i64=f.call(7);assert_eq!(value,7);let value:i64=f.call_with_count(1,9);assert_eq!(value,9);")};
            code.push_str(&format!("pub fn verify(){{{verification}}}"));rust.push_str(&format!("mod {module}{{{code}}}"));
            rust=rust.replace("abstract_forward_bridge::verify();",&format!("abstract_forward_bridge::verify();{module}::verify();"));
        }
        for (module,source_text) in [
            ("rest_result_bridge","function size(...values:undefined[]):int{return 64/values.length;}function main(){const action:Function<int,[value?:undefined]>=size;}"),
            ("stored_rest_result_bridge","function size(...values:undefined[]):int{return 64/values.length;}function main(){const actions:(Function<int,[...values:undefined[]]>)[]=[size];let selected=actions.remove(0);const action:Mutable<Function<int,[value?:undefined]>>=selected;}"),
            ("stored_rest_family_result_bridge","function size(...values:undefined[]):int{return 64/values.length;}function main(){const actions:(Function<int,[...values:undefined[]]>)[]=[size,(...values:undefined[]):int=>values.length];let selected=actions.remove(0);const action:Mutable<Function<int,[value?:undefined]>>=selected;}")
        ] {
        let mut rest=crate::check(source_text).unwrap();
        let (adapter,descriptor)=rest.program.closures.iter().enumerate().find(|(_,closure)|matches!(closure.forward,Some(crate::analysis::ClosureForward::RestAdapter{counted:true,..}))).unwrap();
        let capture=match descriptor.function.bindings[0].ty {
            crate::syntax::Type::Closure(source)=>format!("Closure{source}{{}}"),
            crate::syntax::Type::StoredCallable(family)=>{
                let (_,_,origins)=rest.program.stored_contracts.families().find(|(id,_,_)|*id==family).unwrap();
                let source=*origins.iter().find(|&&origin|rest.program.error_effects.closure_errors[origin].runtime).unwrap();
                if origins.len()==1 {format!("StoredCallable{family}(Closure{source}{{}})")}else{format!("StoredCallable{family}::V{source}(Closure{source}{{}})")}
            }
            other=>panic!("finite rest source {other:?}"),
        };
        let pure_capture=if let crate::syntax::Type::StoredCallable(family)=descriptor.function.bindings[0].ty {
            let (_,_,origins)=rest.program.stored_contracts.families().find(|(id,_,_)|*id==family).unwrap();
            origins.iter().find(|&&origin|!rest.program.error_effects.closure_errors[origin].runtime).map(|source|format!("StoredCallable{family}::V{source}(Closure{source}{{}})"))
        }else{None};
        crate::error_effects::materialize_runtime_record(&mut rest.program);rest.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut rest.program);
        let rest_emitter=super::emitter_for(&rest.program, "");let declarations=rest_emitter.prepare_result_declarations().unwrap();
        let name=rest_emitter.result_closures.borrow().get(&adapter).unwrap().clone();let path=name.split('<').next().unwrap();
        let mut code=rest_emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
        code.push_str(&format!("pub fn verify(){{let mut action=Closure{adapter}{{capture0:{capture}}};let value:Result<i64,{name}>=action.call(TsrUndefined);assert!(matches!(value,Ok(64)));let error:Result<i64,{name}>=action.call_with_count(0,TsrUndefined);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));let value:Result<i64,{name}>=action.call_with_count(1,TsrUndefined);assert!(matches!(value,Ok(64)));}}"));
        if let Some(capture)=pure_capture {
            code.push_str(&format!("pub fn verify_pure(){{let mut action=Closure{adapter}{{capture0:{capture}}};let direct:Result<i64,{name}>=action.call(TsrUndefined);assert!(matches!(direct,Ok(1)));let absent:Result<i64,{name}>=action.call_with_count(0,TsrUndefined);assert!(matches!(absent,Ok(0)));let supplied:Result<i64,{name}>=action.call_with_count(1,TsrUndefined);assert!(matches!(supplied,Ok(1)));}}"));
            rust=rust.replace("subset_bridge::verify();",&format!("subset_bridge::verify();{module}::verify_pure();"));
        }
        rust.push_str(&format!("mod {module}{{{code}}}"));rust=rust.replace("subset_bridge::verify();", &format!("subset_bridge::verify();{module}::verify();"));
        }
        for (module,access) in [("abstract_rest_result_bridge","ReadOnly"),("mutable_rest_result_bridge","Mutable"),("once_rest_result_bridge","Once"),("shared_read_rest_result_bridge","SharedRead")] {
        let annotation=if access=="SharedRead"{"ReadOnly"}else{access};
        let source_text=format!("function size(...values:undefined[]):int{{return 64/values.length;}}function use(action:{annotation}<Function<int,[value?:undefined]>>):int{{return action(undefined);}}function forward(action:{annotation}<Function<int,[...values:undefined[]]>>):int{{return use(action);}}function main(){{forward(size);}}");
        let mut rest=crate::check(&source_text).unwrap();
        let adapter=rest.program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::RestAdapter{counted:true,..})) && matches!(closure.function.bindings[0].ty,crate::syntax::Type::Callback(_))).expect("raw abstract rest adapter");
        rest.program.closures[adapter].function.template=false;
        let crate::syntax::Type::Callback(contract)=rest.program.closures[adapter].function.bindings[0].ty else {unreachable!()};
        if access=="SharedRead" {
            // Stage the internal owned immutable contract on this checked adapter.
            rest.program.callback_contracts[contract].kind=crate::syntax::CallbackKind::SharedRead;
            rest.program.closures[adapter].captures[0].storage=crate::Mode::Consume;
            rest.program.closures[adapter].lifetime_count=0;
        }
        let source=rest.program.closures.iter().position(|closure|matches!(closure.forward,Some(crate::analysis::ClosureForward::Function(_)))).unwrap();
        crate::error_effects::materialize_runtime_record(&mut rest.program);rest.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut rest.program);
        let emitter=super::emitter_for(&rest.program, "");let declarations=emitter.prepare_result_declarations().unwrap();
        let name=emitter.result_closures.borrow().get(&adapter).unwrap().clone();let path=name.split('<').next().unwrap();
        let mut code=emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
        let verification=if access=="SharedRead" {
            let (arguments,lifetimes)=emitter.native_argument_tuple(&rest.program.callback_contracts[contract].params);
            assert!(lifetimes.is_empty(),"owned rest tuple");
            let result=emitter.callback_result_type(contract,crate::syntax::Type::Int);
            code.push_str(&format!("struct ImmutableOrigin(Closure{source});impl ReadCallback<{arguments},{result}> for ImmutableOrigin{{fn invoke_read(&self,args:{arguments})->{result}{{ReadCallback::invoke_read(&self.0,args)}}}}"));
            format!("let action=Closure{adapter}{{capture0:ImmutableOrigin(Closure{source}{{}})}};assert!(matches!(action.call(TsrUndefined),Ok(64)));let error:Result<i64,{name}>=action.call_with_count(0,TsrUndefined);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));assert!(matches!(action.call_with_count(1,TsrUndefined),Ok(64)));")
        }else if access=="Once" {
            format!("assert!(matches!(Closure{adapter}{{capture0:Closure{source}{{}}}}.call(TsrUndefined),Ok(64)));let error:Result<i64,{name}>=Closure{adapter}{{capture0:Closure{source}{{}}}}.call_with_count(0,TsrUndefined);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));assert!(matches!(Closure{adapter}{{capture0:Closure{source}{{}}}}.call_with_count(1,TsrUndefined),Ok(64)));")
        }else{
            let borrow=if access=="Mutable"{"&mut "}else{"&"};
            format!("let mut source=Closure{source}{{}};let mut action=Closure{adapter}{{capture0:{borrow}source}};assert!(matches!(action.call(TsrUndefined),Ok(64)));let error:Result<i64,{name}>=action.call_with_count(0,TsrUndefined);assert!(matches!(error,Err({path}::V0(error)) if error.field1.as_ref()==\"arithmetic\"));assert!(matches!(action.call_with_count(1,TsrUndefined),Ok(64)));")
        };
        code.push_str(&format!("pub fn verify(){{{verification}}}"));
        rust.push_str(&format!("mod {module}{{{code}}}"));rust=rust.replace("subset_bridge::verify();", &format!("subset_bridge::verify();{module}::verify();"));
        }
        let mut native=crate::check("function main(){const action=(path:string):string=>\"unused\";}").unwrap();
        let binding=crate::native_bindings::BINDINGS.iter().position(|binding|binding.module=="std:path" && binding.name=="basename").unwrap();
        native.program.closures[0].forward=Some(crate::analysis::ClosureForward::Native(binding));native.program.closures[0].function.body.clear();
        native.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut native.program);
        let mut emitter=super::emitter_for(&native.program, "");emitter.callback_traits=true;let declarations=emitter.prepare_result_declarations().unwrap();
        assert!(!emitter.result_closures.borrow().contains_key(&0),"pure native direct ABI stays plain");
        let mut code=emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
        code.push_str("pub fn verify(){let action=Closure0{};let input=String::Borrowed(\"/tmp/report.txt\");let result:String=action.call(&input);assert_eq!(result.as_ref(),\"report.txt\");let result:String=ReadCallback::invoke_read(&action,(&input,));assert_eq!(result.as_ref(),\"report.txt\");}");
        rust.push_str(&format!("mod native_path_origin_bridge{{{code}}}"));rust=rust.replace("subset_bridge::verify();","subset_bridge::verify();native_path_origin_bridge::verify();");
        let mut assertion=crate::check("function main(){const action=(condition:boolean,message:string):void=>{};}").unwrap();
        let binding=crate::native_bindings::BINDINGS.iter().position(|binding|binding.module=="std:testing-runtime" && binding.name=="checkAssertion").unwrap();
        assertion.program.closures[0].forward=Some(crate::analysis::ClosureForward::Native(binding));assertion.program.closures[0].function.body.clear();
        crate::error_effects::materialize_runtime_record(&mut assertion.program);
        assertion.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut assertion.program);
        assert!(!assertion.program.error_effects.closure_errors[0].opaque,"native assertion needs a resolved runtime error contract");
        let emitter=super::emitter_for(&assertion.program, "");let declarations=emitter.prepare_result_declarations().unwrap();
        let name=emitter.result_closures.borrow().get(&0).unwrap().clone();let path=name.split('<').next().unwrap();
        let mut code=emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
        code.push_str(&format!("pub fn verify(){{let action=Closure0{{}};let message=String::Owned(\"assertion witness\".to_owned());assert!(matches!(action.call(true,&message),Ok(())));let failure:Result<(),{name}>=action.call(false,&message);assert!(matches!(failure,Err({path}::V0(error)) if error.field0.as_ref()==\"AssertionError\" && error.field1.as_ref()==\"assertion\" && error.field2.as_ref()==\"assertion witness\"));assert_eq!(message.as_ref(),\"assertion witness\");}}"));
        rust.push_str(&format!("mod native_assertion_bridge{{{code}}}"));rust=rust.replace("subset_bridge::verify();","subset_bridge::verify();native_assertion_bridge::verify();");
        let mut assertion=crate::check("function nativeCheck(condition:boolean,message:string):void{}function wrapper(condition:boolean,message:string):void{nativeCheck(condition,message);}function main(){}").unwrap();
        let wrapper=assertion.program.functions.iter().position(|function|function.name=="wrapper").unwrap();
        let crate::syntax::StmtKind::Expr(call)=&mut assertion.program.functions[wrapper].body[0].kind else{panic!("native assertion call");};call.target=Some(crate::syntax::CallTarget::NativeBinding(binding));
        crate::error_effects::materialize_runtime_record(&mut assertion.program);assertion.program.error_effects=crate::error_effects::annotate_inferred_callbacks(&mut assertion.program);
        let emitter=super::emitter_for(&assertion.program, "");let declarations=emitter.prepare_result_declarations().unwrap();
        let (symbol,name)=emitter.result_functions.borrow().get(&wrapper).unwrap().clone();let path=name.split('<').next().unwrap();
        let mut code=emitter.program();code.push_str(crate::runtime::EXCEPTION_COMPLETIONS);code.push_str(&declarations);
        code.push_str(&format!("pub fn verify(){{let message=String::Owned(\"direct assertion\".to_owned());assert!(matches!({symbol}(true,&message),Ok(())));let failure:Result<(),{name}>={symbol}(false,&message);assert!(matches!(failure,Err({path}::V0(error)) if error.field0.as_ref()==\"AssertionError\" && error.field2.as_ref()==\"direct assertion\"));assert_eq!(message.as_ref(),\"direct assertion\");}}"));
        rust.push_str(&format!("mod direct_native_assertion_bridge{{{code}}}"));rust=rust.replace("subset_bridge::verify();","subset_bridge::verify();direct_native_assertion_bridge::verify();");
        let directory=std::env::temp_dir().join(format!("tsrust-abstract-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("callback.rs");let binary=directory.join("callback");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert_eq!(executed.status.code(),Some(1),"{}",String::from_utf8_lossy(&executed.stderr));assert_eq!(executed.stdout,b"64\n32\n7\n");let stderr=String::from_utf8(executed.stderr).unwrap();assert!(stderr.starts_with("Uncaught RangeError:"),"{stderr}");assert!(!stderr.contains("panicked"),"{stderr}");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn declared_result_calls_propagate_native_callee_failures() {
        let mut program=crate::check("function mark(value:int):int{console.log(value);return value;}function leaf(a:int,b:int):int{if(a===1){return 1/0;}return b;}function value(a:int):int{return leaf(mark(a),mark(64));}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program,"");
        let leaf=program.program.functions.iter().position(|function|function.name=="leaf").unwrap();
        let caller=program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let contract=program.program.error_effects.function_errors[leaf].clone();
        let (leaf_declaration,leaf_name)=emitter.completion_error_layout("LeafError",&contract,Some(runtime),program.program.functions[leaf].body[0].span);
        emitter.result_functions.borrow_mut().insert(leaf,("result_leaf".into(),leaf_name.clone()));
        let caller_function=&program.program.functions[caller];let contract=program.program.error_effects.function_errors[caller].clone();
        let (caller_declaration,caller_name)=emitter.completion_error_layout("CallerError",&contract,Some(runtime),caller_function.body[0].span);
        let caller_body=emitter.callable_result_body(caller_function,&caller_function.body,11,&caller_name,contract,Some(runtime));
        assert!(caller_body.contains("result_leaf("),"{caller_body}");
        emitter.result_functions.borrow_mut().insert(caller,("result_caller".into(),caller_name.clone()));
        let mut rust=emitter.program();
        assert!(rust.contains("fn result_caller("),"registered function signature missing");rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&leaf_declaration);rust.push_str(&caller_declaration);
        rust.push_str(&format!("fn call_witness(){{assert!(matches!(result_caller(0),Ok(64)));match result_caller(1){{Err(CallerError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"callee error lost\")}}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"call_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-declared-result-call-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("call.rs");let binary=directory.join("call");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert!(executed.stderr.is_empty());assert_eq!(executed.stdout,b"0\n64\n1\n64\n");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn callable_parameter_defaults_recover_and_skip_supplied_values() {
        use crate::syntax::Type;
        let mut program=crate::check("function value(item:int=1/0):int{return item;}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program,"");let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let Type::Union(input)=function.bindings[0].input_ty.unwrap() else {panic!("default input");};
        let members=&program.program.unions[input.index()].members;
        let absent=members.iter().position(|ty|*ty==Type::Undefined).unwrap();let present=members.iter().position(|ty|*ty==Type::Int).unwrap();
        let contract=crate::error_effects::FailureSet{runtime:true,..Default::default()};
        let (declaration,name)=emitter.completion_error_layout("DefaultError",&contract,Some(runtime),function.body[0].span);
        let expression=emitter.callable_result_body(function,&function.body,8,&name,contract,Some(runtime));
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declaration);
        rust.push_str(&format!("fn default_value(value0:Union{}) -> Result<i64,{name}> {{ {expression} }} fn default_witness(){{assert!(matches!(default_value(Union{}::V{present}(64)),Ok(64)));match default_value(Union{}::V{absent}(TsrUndefined)){{Err(DefaultError::V0(error))=>assert_eq!(error.field1.as_ref(),\"arithmetic\"),_=>panic!(\"missing default failure\")}}}}",input.index(),input.index(),input.index()));
        let pattern_program=crate::check("type Point={n:int|undefined;};function mark():int{console.log(1);return 64;}function value({n=mark()}:Point):int{return n;}function main(){}").unwrap();
        let pattern_emitter=super::emitter_for(&pattern_program.program,"");
        let pattern_function=pattern_program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let parameter=&pattern_function.bindings[0];let Type::Record(record)=parameter.input_ty.unwrap() else {panic!("pattern input");};
        let Type::Union(field)=pattern_program.program.records[record].fields[0].ty.unwrap() else {panic!("optional pattern field");};
        let absent_field=pattern_program.program.unions[field.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
        let argument=match parameter.mode {super::Mode::Read=>"&argument",super::Mode::Mutate=>"&mut argument",_=>"argument"};
        let pattern=pattern_emitter.callable_result_body(pattern_function,&pattern_function.body,9,"std::convert::Infallible",crate::error_effects::FailureSet::default(),None);
        let mut pattern_rust=pattern_program.emit_rust();pattern_rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);
        pattern_rust.push_str(&format!("pub fn verify(){{let mut argument=Record{record}{{field0:Union{}::V{absent_field}(TsrUndefined)}};let value0={argument};let result:Result<i64,std::convert::Infallible>={pattern};assert!(matches!(result,Ok(64)));}}",field.index()));
        rust.push_str(&format!("mod pattern_case{{{pattern_rust}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"default_witness();pattern_case::verify();");
        let directory=std::env::temp_dir().join(format!("tsrust-default-recovery-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("default.rs");let binary=directory.join("default");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));assert_eq!(executed.stdout,b"1\n");std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn callable_result_body_handles_void_fallthrough_and_restores_boundaries() {
        use crate::syntax::ControlTarget;
        let program=crate::check("function value(){}function main(){}").unwrap();
        let emitter=super::emitter_for(&program.program,"");
        let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let contract=crate::error_effects::FailureSet::default();
        let _outer=emitter.exception_regions.enter_typed(4,vec![],"Outer".into(),contract.clone(),None);
        let _control=emitter.control_targets.enter(ControlTarget::Loop(3));
        let expression=emitter.callable_result_body(function,&function.body,5,"std::convert::Infallible",contract,None);
        assert_eq!(emitter.exception_regions.current_id(),Some(4));
        assert_eq!(emitter.control_targets.snapshot(),vec![ControlTarget::Loop(3)]);
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);
        rust.push_str(&format!("fn callable_witness(){{let result:Result<(),std::convert::Infallible>={expression};assert!(result.is_ok());}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"callable_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-void-result-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("void.rs");let binary=directory.join("void");std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        let executed=std::process::Command::new(binary).output().unwrap();assert!(executed.status.success(),"{}",String::from_utf8_lossy(&executed.stderr));std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn runtime_operation_failures_enter_typed_native_completion() {
        let mut batch = String::new();
        let mut calls = String::new();
        let mut stdout = String::new();
        let mut case = 0;
        for (source, expected_code) in [
            ("function value():int{const items=[1];const item=items[2]!;return item;}function main(){}", "bounds"),
            ("function value():int{const items=[1];const item=items[-1]!;return item;}function main(){}", "bounds"),
            ("function value():int{const items=[1];const item=items[2];return item!;}function main(){}", "bounds"),
            ("function input():int|undefined{return undefined;}function value():int{const item=input();return item!;}function main(){}", "non-null"),
            ("function input():int{return 9223372036854775807;}function value():int{return input()+1;}function main(){}", "overflow"),
            ("function input():int{return 0;}function value():int{return 1/input();}function main(){}", "arithmetic"),
            ("function input():int{return 0;}function value():int{return 1%input();}function main(){}", "arithmetic"),
            ("function input():int{return -9223372036854775808;}function value():int{return input()-1;}function main(){}", "overflow"),
            ("function input():int{return 9223372036854775807;}function value():int{return input()*2;}function main(){}", "overflow"),
            ("function input():int{return -9223372036854775808;}function value():int{return -input();}function main(){}", "overflow"),
            ("function input():int{return 9223372036854775807;}function value():int{let item=input();item++;return item;}function main(){}", "overflow"),
            ("function input():int{return 9223372036854775807;}function value():int{let item=input();item+=1;return item;}function main(){}", "overflow"),
            ("function input():int{return -9223372036854775808;}function value():int{return input()/-1;}function main(){}", "arithmetic"),
            ("function input():int{return -1;}function value():int{return 1<<input();}function main(){}", "shift"),
            ("function input():int{return 64;}function value():int{return 1>>input();}function main(){}", "shift"),
            ("function input():int{return 4294967296;}function value():int{return 1>>>input();}function main(){}", "shift"),
            ("function input():int{return -1;}function value():int{return 2**input();}function main(){}", "exponent"),
            ("function input():int{return 4294967296;}function value():int{return 2**input();}function main(){}", "exponent"),
            ("function input():int{return 63;}function value():int{return 2**input();}function main(){}", "overflow"),
            ("function input():int{return 64;}function value():int{let item=1;item<<=input();return item;}function main(){}", "shift"),
            ("function input():int{return 63;}function value():int{let item=2;item**=input();return item;}function main(){}", "overflow"),
            ("function input():int{return 6;}function value():int{return 1<<input();}function main(){}", "success"),
            ("function input():int{return 6;}function value():int{return 2**input();}function main(){}", "success"),
            ("function input():int{return 6;}function value():int{let item=1;item<<=input();return item;}function main(){}", "success"),
            ("function input():int{return 6;}function value():int{let item=2;item**=input();return item;}function main(){}", "success"),
            ("type Box={n:int;};function input():Box|undefined{return undefined;}function value():int{let item=input();item!.n=2;return 0;}function main(){}", "non-null"),
            ("type Box={n:int;};function input():Box|undefined{return undefined;}function value():int{let item=input();item!.n++;return 0;}function main(){}", "non-null"),
            ("type Box={n:int;};function input():Box|undefined{return undefined;}function value():int{let item=input();item!.n+=1;return 0;}function main(){}", "non-null"),
            ("type Box={n:int;};function input():Box|undefined{return {n:63};}function value():int{let item=input();item!.n=64;return item!.n;}function main(){}", "success"),
            ("type Box={n:int;};function input():Box|undefined{return {n:63};}function value():int{let item=input();item!.n++;return item!.n;}function main(){}", "success"),
            ("type Box={n:int;};function input():Box|undefined{return {n:63};}function value():int{let item=input();item!.n+=1;return item!.n;}function main(){}", "success"),
            ("type Box={n:int;label:string;};function input():Box|undefined{return undefined;}function value():int{const items=[input()];const item=items[0];return item!.n;}function main(){}", "non-null"),
            ("function value():int{return [1][2];}function main(){}", "bounds"),
            ("function input():int[]{return [1];}function value():int{return input()[-1];}function main(){}", "bounds"),
            ("function input():string[]{return [\"owned\"]; }function value():int{const item=input()[2];return item.length;}function main(){}", "bounds"),
            ("function value():int{const words=[\"literal\"];return words[2].length;}function main(){}", "bounds"),
            ("function value():int{return [64][0];}function main(){}", "success"),
            ("function input():string[]{return [\"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"]; }function value():int{const item=input()[0];return item.length;}function main(){}", "success"),
            ("function value():int{const items=[1];const copy=items.with(1,64);return copy[0];}function main(){}", "bounds"),
            ("function value():int{const items=[1];const copy=items.with(-2,64);return copy[0];}function main(){}", "bounds"),
            ("function value():int{const items=[1];const copy=items.with(-9223372036854775808,64);return copy[0];}function main(){}", "bounds"),
            ("function value():int{const items=[1];const copy=items.with(-1,64);return copy[0];}function main(){}", "success"),
            ("function receiver():int[]{console.log(1);return [10];}function mark(value:int):int{console.log(value);return value;}function value():int{const copy=receiver().with(mark(3),mark(4));return copy[0];}function main(){}", "bounds-order"),
            ("function value():int{let items=[1];return items.remove(1);}function main(){}", "bounds"),
            ("function value():int{let items=[1];return items.remove(-1);}function main(){}", "bounds"),
            ("function value():int{let items=[64];return items.remove(0);}function main(){}", "success"),
            ("type Box={items:int[];};function value():int{let box:Box={items:[1]};return box.items.remove(1);}function main(){}", "bounds"),
            ("type Box={items:int[];};function value():int{let box:Box={items:[64]};return box.items.remove(0);}function main(){}", "success"),
        ] {
        let mut program=crate::check(source).unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program,"");let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let index=program.program.functions.iter().position(|function|function.name=="value").unwrap();
        let plan=emitter.fixed.plans.iter().position(|plan|plan.function==index).unwrap();emitter.active_plan.set(Some(plan));
        let contract=crate::error_effects::FailureSet{runtime:true,..Default::default()};let (declaration,name)=emitter.completion_error_layout("RuntimeFailure",&contract,Some(runtime),function.body[0].span);
        let expression=emitter.callable_result_body(function,&function.body,999,&name,contract,Some(runtime));
        let expected_stdout = if expected_code == "bounds-order" { "1\n3\n4\n" } else { "" };
        let expected_code = if expected_code == "bounds-order" { "bounds" } else { expected_code };
        let verification = if expected_code == "success" {
            "assert!(matches!(result,Ok(64)));".to_owned()
        } else {
            format!("match result{{Err(RuntimeFailure::V0(error))=>{{assert_eq!(error.field1.as_ref(),{expected_code:?});assert!(error.field5>error.field4);}},_=>panic!(\"missing runtime failure\")}}")
        };
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declaration);rust.push_str(&format!("fn recovery_witness(){{let result:Result<i64,{name}>={expression};{verification}}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"recovery_witness();");
        batch.push_str(&format!("mod case_{case} {{ {rust} pub fn run() {{ main(); }} }}\n"));
        calls.push_str(&format!("case_{case}::run();"));
        stdout.push_str(expected_stdout);
        case += 1;
        }
        batch.push_str(&format!("fn main() {{ {calls} }}"));
        let directory = std::env::temp_dir().join(format!("tsrust-runtime-recovery-batch-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source = directory.join("recovery.rs");
        let binary = directory.join("recovery");
        std::fs::write(&source, batch).unwrap();
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let compiled = std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
        let executed = std::process::Command::new(binary).output().unwrap();
        assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
        assert!(executed.stderr.is_empty(), "{}", String::from_utf8_lossy(&executed.stderr));
        assert_eq!(executed.stdout, stdout.as_bytes());
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn source_closure_error_layout_compiles_with_capture_lifetime() {
        let program=crate::check("function main(){const label=\"payload\";const saved=()=>label.length;const size=saved();}").unwrap();
        let emitter=super::emitter_for(&program.program,"");let function=program.program.functions.iter().find(|function|function.name=="main").unwrap();
        let (binding,saved)=function.bindings.iter().enumerate().find(|(_,binding)|binding.name=="saved").unwrap();
        let contract=crate::error_effects::FailureSet{explicit:vec![saved.ty],..Default::default()};
        let span=function.body[0].span;let (declaration,name)=emitter.completion_error_layout("CapturedError",&contract,None,span);
        let unions=crate::unions::UnionArena::from_snapshot(program.program.unions.clone());
        let injection=super::exception_contract_emit::checked_throw_value(&format!("value{binding}"),saved.ty,&name,&contract,&unions,None,span).unwrap();
        let conversion=super::exception_contract_emit::checked_catch_value("error",&name,&contract,&unions,None,span).unwrap();
        let mut rust=program.emit_rust();rust.push_str(&declaration);
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;
        let mut depth=1;let mut closing=opening;for (offset,ch) in rust[opening..].char_indices(){if ch=='{'{depth+=1;}else if ch=='}'{depth-=1;if depth==0{closing=opening+offset;break;}}}
        rust.insert_str(closing,&format!("let error:{name}={injection};let caught={conversion};assert_eq!(caught.call(),7);"));
        let directory=std::env::temp_dir().join(format!("tsrust-source-closure-layout-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();let source=directory.join("closure.rs");let binary=directory.join("closure");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn abrupt_finally_can_remove_pending_throw_transport() {
        use crate::syntax::StmtKind;
        let program=crate::check("function value():int{return 4;}function cleanup():int{return 6;}function main(){}").unwrap();
        let emitter=super::emitter_for(&program.program,"");let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let mut body=function.body.clone();let StmtKind::Return(Some(value))=&body[0].kind else {panic!("return");};body[0].kind=StmtKind::Throw(value.clone());
        let region=crate::syntax::ExceptionRegion{body,catch:None,finally:Some(function.body.clone())};
        let nested=crate::syntax::Stmt{labels:vec![],control_target:None,kind:StmtKind::Try(Box::new(region.clone())),span:function.body[0].span};
        let cleanup=program.program.functions.iter().find(|function|function.name=="cleanup").unwrap();
        let outer=crate::syntax::ExceptionRegion{body:vec![nested],catch:None,finally:Some(cleanup.body.clone())};
        let outer_statement=crate::syntax::Stmt{labels:vec![],control_target:None,kind:StmtKind::Try(Box::new(outer)),span:cleanup.body[0].span};
        let expression=emitter.callable_result_body(function,&[outer_statement],100,"std::convert::Infallible",crate::error_effects::FailureSet::default(),None);
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&format!("fn suppressed_witness(){{let result:Result<i64,std::convert::Infallible>={expression};assert!(matches!(result,Ok(6)));}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"suppressed_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-suppressed-throw-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();let source=directory.join("suppressed.rs");let binary=directory.join("suppressed");std::fs::write(&source,rust).unwrap();let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn checked_throw_and_catch_statement_composition_executes() {
        use crate::syntax::{StmtKind,Type};
        let mut program=crate::check("function value(error:int):int{return error+1;}function main(){}").unwrap();
        let runtime=crate::error_effects::materialize_runtime_record(&mut program.program);
        let emitter=super::emitter_for(&program.program,"");
        let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let mut throwing=function.body.clone();
        let StmtKind::Return(Some(value))=&throwing[0].kind else {panic!("return");};
        throwing[0].kind=StmtKind::Throw(value.clone());
        let mut parsed=crate::syntax::parse("function main(){try{}catch(error){}}").unwrap();
        let StmtKind::Try(region)=&mut parsed.functions[0].body[0].kind else {panic!("region");};
        let handler=region.catch.as_mut().unwrap();handler.binding=Some(0);handler.body=function.body.clone();handler.error_type=Some(Type::Int);
        let incoming=crate::error_effects::FailureSet{explicit:vec![Type::Int],..Default::default()};handler.error_contract=Some(Box::new(incoming));
        let (declaration,name)=emitter.completion_error_layout("Incoming",handler.error_contract.as_deref().unwrap(),None,throwing[0].span);
        assert_eq!(name,"Incoming<i64>");
        let (empty,empty_name)=emitter.completion_error_layout("Unused",&crate::error_effects::FailureSet::default(),None,throwing[0].span);
        assert!(empty.is_empty());assert_eq!(empty_name,"std::convert::Infallible");
        let expression=emitter.completion_with_catch(function,&throwing,handler,(20,21),&name,"std::convert::Infallible",crate::error_effects::FailureSet::default(),None,false,None,None);
        region.body=throwing.clone();region.finally=Some(function.body.clone());
        let combined=emitter.completion_handlers(function,region,(22,23,24),(&name,"std::convert::Infallible"),crate::error_effects::FailureSet::default(),Some(runtime));
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);rust.push_str(&declaration);
        rust.push_str(&format!("fn catch_witness(value0:i64) {{let result:TsrCompletion<(),i64,std::convert::Infallible>={expression};assert!(matches!(result,TsrCompletion::Return(5)));}}"));
        rust.push_str(&format!("fn handlers_witness(value0:i64) {{let result:TsrCompletion<(),i64,std::convert::Infallible>={combined};assert!(matches!(result,TsrCompletion::Return(4)));}}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"catch_witness(3);handlers_witness(3);");
        let directory=std::env::temp_dir().join(format!("tsrust-checked-catch-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();let source=directory.join("catch.rs");let binary=directory.join("catch");std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));assert!(std::process::Command::new(binary).status().unwrap().success());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn checked_finally_changes_outer_state_and_preserves_pending_return() {
        let program=crate::check("function value():int {let trace=0;trace=trace+5;return trace;} function main(){}").unwrap();
        let emitter=super::emitter_for(&program.program,"");
        let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let expression=emitter.completion_with_finally(function,&function.body[1..],&function.body[1..2],(10,11),"std::convert::Infallible",crate::error_effects::FailureSet::default(),None);
        let replaced=emitter.completion_with_finally(function,&function.body[1..],&function.body[1..],(12,13),"std::convert::Infallible",crate::error_effects::FailureSet::default(),None);
        let mut setup=String::new();emitter.block(&mut setup,function,&function.body[..1],0);
        let mut rust=program.emit_rust();rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);
        rust.push_str(&format!("fn checked_cleanup_witness() {{ {setup} let pending:TsrCompletion<(),i64,std::convert::Infallible>={expression}; assert!(matches!(pending,TsrCompletion::Return(5))); assert_eq!(value0,10); }}"));
        rust.push_str(&format!("fn checked_replacement_witness() {{ {setup} let pending:TsrCompletion<(),i64,std::convert::Infallible>={replaced}; assert!(matches!(pending,TsrCompletion::Return(10))); assert_eq!(value0,10); }}"));
        let main=rust.find("fn main()").unwrap();let opening=main+rust[main..].find('{').unwrap()+1;rust.insert_str(opening,"checked_cleanup_witness();checked_replacement_witness();");
        let directory=std::env::temp_dir().join(format!("tsrust-checked-finally-{}",std::process::id()));std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("cleanup.rs");let binary=directory.join("cleanup");std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let compiled=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();
        assert!(compiled.status.success(),"{}",String::from_utf8_lossy(&compiled.stderr));
        assert!(std::process::Command::new(binary).status().unwrap().success());std::fs::remove_dir_all(directory).unwrap();
        assert_eq!(emitter.exception_regions.current_id(),None);
    }
    #[test]
    fn checked_return_statement_emits_completion_inside_typed_region() {
        let program=crate::check("function value(): int {return 4;} function main(){}").unwrap();
        let emitter=super::emitter_for(&program.program,"");
        let function=program.program.functions.iter().find(|function|function.name=="value").unwrap();
        let body=emitter.completion_body(function,&function.body,7,"std::convert::Infallible",crate::error_effects::FailureSet::default(),None);
        assert!(body.contains("break 'exception_region7 TsrCompletion::Return(4i64);"),"{body}");
        let mut rust=program.emit_rust();
        rust.push_str(crate::runtime::EXCEPTION_COMPLETIONS);
        rust.push_str(&format!("fn region_witness()->TsrCompletion<(),i64,std::convert::Infallible> {{ {body} }}"));
        let main=rust.find("fn main()").unwrap();
        let opening=main+rust[main..].find('{').unwrap()+1;
        rust.insert_str(opening,"assert!(matches!(region_witness(),TsrCompletion::Return(4)));");
        let directory=std::env::temp_dir().join(format!("tsrust-statement-region-{}",std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let source=directory.join("region.rs");let binary=directory.join("region");
        std::fs::write(&source,rust).unwrap();
        let compiler=std::env::var_os("RUSTC").unwrap_or_else(||"rustc".into());
        let output=std::process::Command::new(compiler).arg("--edition=2024").arg(source).arg("-o").arg(&binary).output().unwrap();
        assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
        assert!(std::process::Command::new(binary).status().unwrap().success());
        std::fs::remove_dir_all(directory).unwrap();
        assert_eq!(emitter.exception_regions.current_id(),None);
        let mut ordinary=String::new();
        emitter.block(&mut ordinary,function,&function.body,0);
        assert!(ordinary.contains("return 4i64;"),"{ordinary}");
    }
}

mod set_emit;
mod map_emit;
#[path = "../stdlib/collections/emit.rs"]
mod deque_emit;
#[path = "../stdlib/collections/heap_emit.rs"]
mod heap_emit;
mod map_storage_epochs;


impl Emitter<'_>{
    #[inline(never)]
    fn comma_discard_code(&self,function:&CheckedFunction,operand:&Expr)->String{
        let first=self.conditional_temporaries.borrow().len();
        let value=self.expr(function,operand,Mode::Read);
        let declarations=self.conditional_temporaries.borrow_mut().drain(first..).collect::<Vec<_>>().join(" ");
        format!("{{{declarations} let _={value};}}")
    }
}

mod map_return_storage;
mod map_catch_finally_storage;
mod map_exception_epochs;
mod map_finally_storage_epochs;

mod map_loop_storage_epochs;
mod iteration_emit;
mod map_entry_epoch_emit;
mod array_storage_epoch_emit;

mod map_iteration_emit;

mod map_cleanup_control_emit;

mod map_control_storage;

mod map_cleanup_storage;

#[cfg(test)]
mod nullable_recursive_tests;

#[path = "../stdlib/collections/ordered_set_emit.rs"]
mod ordered_set_emit;
