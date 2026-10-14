//! Private, bounded specialization for local aggregate literals with proved uses.
//! Source types and ownership analysis stay unchanged. Any unproved use falls
//! back to ordinary owned storage, including moves, length changes, and captures. Primitive indexed writes preserve width.
use crate::analysis::{CheckedFunction, CheckedProgram, Mode};
use crate::record_runtime::RecordRuntime;
use crate::syntax::*;
use std::collections::{BTreeMap, BTreeSet};

const MAX_STACK_BYTES: usize = 4096;
const MAX_VARIANTS: usize = 64;
const MAX_PROOF_STEPS: usize = 10_000;
const MAX_PROOF_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Scalar {
    Int,
    Number,
    Boolean,
}
impl Scalar {
    fn bytes(self) -> usize {
        if self == Self::Boolean { 1 } else { 8 }
    }
    fn array(ty: Type) -> Option<Self> {
        match ty {
            Type::Array(ArrayElement::Int(0)) => Some(Self::Int),
            Type::Array(ArrayElement::Number(0)) => Some(Self::Number),
            Type::Array(ArrayElement::Boolean(0)) => Some(Self::Boolean),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Shape {
    LiteralStrings,
    ScalarArray {
        scalar: Scalar,
        width: usize,
    },
    Record {
        record: usize,
        widths: Vec<Option<usize>>,
    },
    Array {
        record: usize,
        width: usize,
    },
}

pub(crate) struct FunctionPlan {
    pub(crate) function: usize,
    pub(crate) parameters: Vec<Option<Shape>>,
    pub(crate) bindings: BTreeMap<usize, Shape>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Body {
    Function(usize),
    Closure(usize),
}

pub(crate) struct FixedStorage {
    pub(crate) plans: Vec<FunctionPlan>,
    pub(crate) records: Vec<Shape>,
    temporary_arrays: BTreeMap<(Body, usize, usize, Scalar), Shape>,
}

impl FixedStorage {
    pub(crate) fn new(program: &CheckedProgram, runtime: &RecordRuntime) -> Self {
        let mut temporary_arrays = BTreeMap::new();
        let locals = program
            .functions
            .iter()
            .enumerate()
            .map(|(function, fun)| {
                // Uninstantiated callback templates need no backend layout.
                if fun.template {
                    return BTreeMap::new();
                }
                let mut bindings = BTreeMap::new();
                let mut bytes = 0;
                visit_statements(&fun.body, &mut |stmt| {
                    if let StmtKind::Let {
                        value,
                        binding: Some(id),
                        ..
                    } = &stmt.kind
                        && let Some((shape, size)) =
                            literal_shape(program, runtime, value).or_else(|| {
                                if value.target != Some(CallTarget::Clone) {
                                    return None;
                                }
                                let ExprKind::Call(_, args) = &value.kind else {
                                    return None;
                                };
                                let shape @ Shape::ScalarArray { scalar, width } =
                                    shape_of(program, &bindings, &args[0])?
                                else {
                                    return None;
                                };
                                let bytes = width.checked_mul(scalar.bytes())?;
                                Some((shape, bytes))
                            })
                        && bytes + size <= MAX_STACK_BYTES
                        && Proof::new(program, matches!(shape, Shape::ScalarArray { .. }))
                            .binding(fun, *id)
                    {
                        bytes += size;
                        bindings.insert(*id, shape);
                    }
                });
                temporary_arrays.extend(plan_temporary_arrays(&fun.body, bytes).into_iter().map(
                    |((start, end, scalar), shape)| {
                        ((Body::Function(function), start, end, scalar), shape)
                    },
                ));
                bindings
            })
            .collect::<Vec<_>>();
        let mut storage = Self {
            plans: program
                .functions
                .iter()
                .enumerate()
                .filter(|(_, f)| !f.template)
                .map(|(function, fun)| FunctionPlan {
                    function,
                    parameters: vec![None; fun.parameter_count],
                    bindings: locals[function].clone(),
                })
                .collect(),
            records: Vec::new(),
            temporary_arrays,
        };
        for (id, closure) in program.closures.iter().enumerate() {
            if !closure.function.template {
                storage.temporary_arrays.extend(
                    plan_temporary_arrays(&closure.function.body, 0)
                        .into_iter()
                        .map(|((start, end, scalar), shape)| {
                            ((Body::Closure(id), start, end, scalar), shape)
                        }),
                );
            }
        }
        let original_count = storage.plans.len();
        let mut cursor = 0;
        while cursor < storage.plans.len() {
            let plan = &storage.plans[cursor];
            let mut calls = Vec::new();
            visit_function_expressions(&program.functions[plan.function], &mut |expr| {
                if let ExprKind::Call(_, args) = &expr.kind
                    && let Some(
                        CallTarget::Function(function) | CallTarget::ForwardFunction(function),
                    ) = expr.target
                {
                    let parameters = args
                        .iter()
                        .enumerate()
                        .map(|(index, arg)| {
                            let parameter = program.functions[function].bindings.get(index)?;
                            // A supplied literal does not prove the omitted/default
                            // branch has the same native parameter representation.
                            if parameter.omission.is_omittable()
                                || parameter.input_ty.is_some_and(|ty| ty != parameter.ty)
                            {
                                return None;
                            }
                            shape_of(program, &plan.bindings, arg)
                        })
                        .collect::<Vec<_>>();
                    if parameters.iter().any(Option::is_some) {
                        calls.push((function, parameters));
                    }
                }
            });
            for (function, parameters) in calls {
                if storage
                    .plans
                    .iter()
                    .any(|p| p.function == function && p.parameters == parameters)
                {
                    continue;
                }
                // Fail closed for the entire optimization if code growth exceeds
                // its budget. Never emit a half-specialized call graph.
                if storage.plans.len() - original_count >= MAX_VARIANTS {
                    for plan in storage.plans.iter_mut().take(original_count) {
                        plan.bindings.clear();
                    }
                    storage.plans.truncate(original_count);
                    storage.temporary_arrays.clear();
                    return storage;
                }
                let mut bindings = locals[function].clone();
                for (id, shape) in parameters.iter().enumerate() {
                    if let Some(shape) = shape {
                        bindings.insert(id, shape.clone());
                    }
                }
                storage.plans.push(FunctionPlan {
                    function,
                    parameters,
                    bindings,
                });
            }
            cursor += 1;
        }
        let mut records = BTreeSet::new();
        for plan in &storage.plans {
            for shape in plan.bindings.values() {
                if matches!(shape, Shape::Record { .. }) {
                    records.insert(shape.clone());
                }
            }
        }
        storage.records = records.into_iter().collect();
        storage
    }

    pub(crate) fn temporary_shape(
        &self,
        program: &CheckedProgram,
        function: &CheckedFunction,
        expr: &Expr,
    ) -> Option<&Shape> {
        let scalar = Scalar::array(expr.ty?)?;
        // Specialized bodies can share source spans while having different local
        // plans. Resolve the checked body identity before looking up its budget.
        let body = program
            .functions
            .iter()
            .position(|candidate| std::ptr::eq(candidate, function))
            .map(Body::Function)
            .or_else(|| {
                program
                    .closures
                    .iter()
                    .position(|candidate| std::ptr::eq(&candidate.function, function))
                    .map(Body::Closure)
            })?;
        self.temporary_arrays
            .get(&(body, expr.span.start, expr.span.end, scalar))
    }

    pub(crate) fn record_id(&self, shape: &Shape) -> usize {
        self.records
            .iter()
            .position(|s| s == shape)
            .expect("planned fixed record")
    }

    pub(crate) fn call(&self, function: usize, parameters: &[Option<Shape>]) -> usize {
        self.plans
            .iter()
            .position(|p| p.function == function && p.parameters == parameters)
            .expect("planned function specialization")
    }
}

fn selected_shape(program:&CheckedProgram,bindings:&BTreeMap<usize,Shape>,source:&SelectedSource)->Option<Shape> {
            let (root, fields) = source.place.as_ref()?;
            let mut shape = bindings.get(root)?.clone();
            for name in fields {
                let Shape::Record { record, widths } = shape else {
                    return None;
                };
                let field = program.records[record]
                    .fields
                    .iter()
                    .position(|field| field.name.text == *name)?;
                let width = widths[field]?;
                let Type::Array(ArrayElement::Record(record, 0)) =
                    program.records[record].fields[field].ty?
                else {
                    return None;
                };
                shape = Shape::Array { record, width };
            }
            Some(shape)
}

pub(crate) fn shape_of(
    program: &CheckedProgram,
    bindings: &BTreeMap<usize, Shape>,
    expr: &Expr,
) -> Option<Shape> {
    match &expr.kind {
        ExprKind::Name(_) => bindings.get(&expr.binding?).cloned(),
        ExprKind::Unary {
            op: UnaryOp::OptionalSelected,
            value,
            ..
        }
        | ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => shape_of(program, bindings, value),
        ExprKind::Selected { source, .. } => selected_shape(program, bindings, source),
        ExprKind::SpreadValue(selected) if selected.selection == SpreadSelection::Receiver => selected_shape(program, bindings, &selected.source),
        ExprKind::Field(base, name) => {
            let Shape::Record { record, widths } = shape_of(program, bindings, base)? else {
                return None;
            };
            let field = program.records[record]
                .fields
                .iter()
                .position(|f| f.name.text == *name)?;
            let width = widths[field]?;
            let Type::Array(ArrayElement::Record(record, 0)) = expr.ty? else {
                return None;
            };
            Some(Shape::Array { record, width })
        }
        _ => None,
    }
}

fn scalar_literal_shape(expr: &Expr) -> Option<(Shape, usize)> {
    if let ExprKind::Unary {
        op: UnaryOp::OptionalSelected,
        value,
        ..
    } = &expr.kind
    {
        return scalar_literal_shape(value);
    }
    let ExprKind::Array(items) = &expr.kind else {
        return None;
    };
    let scalar = Scalar::array(expr.ty?)?;
    let bytes = items.len().checked_mul(scalar.bytes())?;
    (bytes <= MAX_STACK_BYTES).then_some((
        Shape::ScalarArray {
            scalar,
            width: items.len(),
        },
        bytes,
    ))
}

// A scalar literal selected directly by indexing cannot expose its storage
// before selection. Budget each occurrence together with the function's locals.
fn plan_temporary_arrays(
    body: &[Stmt],
    mut bytes: usize,
) -> BTreeMap<(usize, usize, Scalar), Shape> {
    enum Node<'a> {
        Statement(&'a Stmt),
        Expression(&'a Expr),
    }
    let mut pending = body
        .iter()
        .rev()
        .map(|s| (Node::Statement(s), 0usize))
        .collect::<Vec<_>>();
    let mut remaining = MAX_PROOF_STEPS;
    let mut arrays = BTreeMap::new();
    while let Some((node, depth)) = pending.pop() {
        if remaining == 0 || depth >= MAX_PROOF_DEPTH {
            return BTreeMap::new();
        }
        remaining -= 1;
        let next = depth + 1;
        match node {
            Node::Statement(statement) => match &statement.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
                StmtKind::Try(_) => return BTreeMap::new(),
                StmtKind::Throw(value) => pending.push((Node::Expression(value), next)),
                StmtKind::Let { value, .. }
                | StmtKind::Expr(value)
                | StmtKind::Return(Some(value)) => pending.push((Node::Expression(value), next)),
                StmtKind::Assign { target, value, .. } => {
                    pending.push((Node::Expression(value), next));
                    pending.push((Node::Expression(target), next));
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    pending.extend(else_body.iter().rev().map(|s| (Node::Statement(s), next)));
                    pending.extend(then_body.iter().rev().map(|s| (Node::Statement(s), next)));
                    pending.push((Node::Expression(condition), next));
                }
                StmtKind::While { condition, body } => {
                    pending.extend(body.iter().rev().map(|s| (Node::Statement(s), next)));
                    pending.push((Node::Expression(condition), next));
                }
                StmtKind::Block(body) => {
                    pending.extend(body.iter().rev().map(|s| (Node::Statement(s), next)))
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    pending.extend(update.iter().rev().map(|s| (Node::Statement(s), next)));
                    pending.extend(body.iter().rev().map(|s| (Node::Statement(s), next)));
                    if let Some(condition) = condition {
                        pending.push((Node::Expression(condition), next));
                    }
                }
                StmtKind::ForOf { iterable, body, .. } => {
                    pending.extend(body.iter().rev().map(|s| (Node::Statement(s), next)));
                    pending.push((Node::Expression(iterable), next));
                }
                StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => (),
            },
            Node::Expression(expression) => {
                if let ExprKind::Index(base, _) = &expression.kind
                    && let Some((shape @ Shape::ScalarArray { scalar, .. }, size)) =
                        scalar_literal_shape(base)
                    && bytes
                        .checked_add(size)
                        .is_some_and(|total| total <= MAX_STACK_BYTES)
                {
                    bytes += size;
                    arrays.insert((base.span.start, base.span.end, scalar), shape);
                }
                match &expression.kind {
                    ExprKind::Match { body, .. } => pending.extend(body.iter().map(|value| (Node::Statement(value), next))),
                    ExprKind::SpreadCall(plan) => pending.extend(plan.expressions().map(|value| (Node::Expression(value), next))),
                    ExprKind::SpreadValue(_) => (),
                    ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value)
                    | ExprKind::Project { value, .. }
                    | ExprKind::Inject { value, .. }
                    | ExprKind::Unary { value, .. }
                    | ExprKind::Update { target: value, .. }
                    | ExprKind::Field(value, _) => pending.push((Node::Expression(value), next)),
                    ExprKind::Conditional {
                        predicate,
                        consequent,
                        alternate,
                    } => {
                        pending.push((Node::Expression(alternate), next));
                        pending.push((Node::Expression(consequent), next));
                        pending.push((Node::Expression(predicate), next));
                    }
                    ExprKind::Optional {
                        base: left,
                        body: right,
                        ..
                    }
                    | ExprKind::Binary { left, right, .. }
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
                        pending.push((Node::Expression(right), next));
                        pending.push((Node::Expression(left), next));
                    }
                    ExprKind::Call(callee, args) => {
                        pending.extend(args.iter().rev().map(|a| (Node::Expression(a), next)));
                        pending.push((Node::Expression(callee), next));
                    }
                    ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
                        pending.extend(values.iter().rev().map(|a| (Node::Expression(a), next)))
                    }
                    ExprKind::Record(fields) => pending.extend(
                        fields
                            .iter()
                            .rev()
                            .map(|(_, a)| (Node::Expression(a), next)),
                    ),
                    ExprKind::Selected { .. }
                    | ExprKind::Closure { .. }
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
    arrays
}

fn literal_shape(
    program: &CheckedProgram,
    runtime: &RecordRuntime,
    expr: &Expr,
) -> Option<(Shape, usize)> {
    if let Some(shape) = scalar_literal_shape(expr) {
        return Some(shape);
    }
    if let ExprKind::Array(items) = &expr.kind
        && let Some(Type::Array(ArrayElement::Record(record, 0))) = expr.ty
    {
        let bytes =
            layout_bound(program, runtime, Type::Record(record), 0)?.checked_mul(items.len())?;
        return (bytes <= MAX_STACK_BYTES).then_some((
            Shape::Array {
                record,
                width: items.len(),
            },
            bytes,
        ));
    }
    if let ExprKind::Array(items) = &expr.kind
        && expr.ty == Some(Type::Array(ArrayElement::String(0)))
        && items
            .iter()
            .all(|item| matches!(&item.kind, ExprKind::String(_)))
    {
        // Charge both the private view and static reference payload against
        // the aggregate budget. Width does not create another ABI variant.
        let bytes = items
            .len()
            .checked_add(1)?
            .checked_mul(std::mem::size_of::<&str>())?;
        return (bytes <= MAX_STACK_BYTES).then_some((Shape::LiteralStrings, bytes));
    }
    let ExprKind::Record(values) = &expr.kind else {
        return None;
    };
    let Type::Record(record) = expr.ty? else {
        return None;
    };
    let mut widths = vec![None; program.records[record].fields.len()];
    let mut bytes = 8_usize;
    for (name, value) in values {
        let field = program.records[record]
            .fields
            .iter()
            .position(|f| f.name.text == name.text)?;
        let size = if let Some(Type::Array(ArrayElement::Record(child, 0))) = value.ty
            && let ExprKind::Array(items) = &value.kind
        {
            widths[field] = Some(items.len());
            layout_bound(program, runtime, Type::Record(child), 0)?.checked_mul(items.len())?
        } else {
            layout_bound(program, runtime, value.ty?, 0)?
        };
        bytes = bytes.checked_add(size)?.checked_add(8)?;
        if bytes > MAX_STACK_BYTES {
            return None;
        }
    }
    widths
        .iter()
        .any(Option::is_some)
        .then_some((Shape::Record { record, widths }, bytes))
}

fn layout_bound(
    program: &CheckedProgram,
    runtime: &RecordRuntime,
    ty: Type,
    depth: usize,
) -> Option<usize> {
    if depth > 32 {
        return None;
    }
    match ty {
        Type::Null | Type::Undefined => Some(0),
        Type::Int | Type::Number | Type::Boolean => Some(8),
        Type::String => Some(crate::storage::string_storage_bound()),
        Type::Record(record) => {
            if program.records[record].native_type.is_some_and(|id| !crate::native_types::declaration(id).source_constructible()) { return None; }
            let mut bytes = 8_usize;
            for field in &program.records[record].fields {
                bytes = bytes
                    .checked_add(layout_bound(program, runtime, field.ty?, depth + 1)?)?
                    .checked_add(8)?;
                if bytes > MAX_STACK_BYTES {
                    return None;
                }
            }
            Some(bytes)
        }
        Type::Array(ArrayElement::Record(record, 0)) if runtime.recursive(record) => {
            Some(std::mem::size_of::<Vec<()>>())
        }
        Type::Array(element) => Some(crate::storage::array_storage_bound(element)),
        _ => None,
    }
}

// Recheck every syntactic use, not only final flow-state flags: a move on an
// exited branch or a capture can otherwise disappear from that final state.
struct Proof<'a> {
    program: &'a CheckedProgram,
    visiting: BTreeSet<(usize, usize)>,
    remaining: usize,
    depth: usize,
    scalar: bool,
}
impl<'a> Proof<'a> {
    fn new(program: &'a CheckedProgram, scalar: bool) -> Self {
        Self {
            program,
            visiting: BTreeSet::new(),
            remaining: MAX_PROOF_STEPS,
            depth: 0,
            scalar,
        }
    }
    fn binding(&mut self, fun: &CheckedFunction, id: usize) -> bool {
        fun.bindings.iter().all(|binding| {
            binding.initializer.as_deref().is_none_or(|value| self.expr(id, value, Mode::Consume))
        }) && self.body(id, &fun.body)
    }
    fn body(&mut self, id: usize, body: &[Stmt]) -> bool {
        if self.depth >= MAX_PROOF_DEPTH {
            return false;
        }
        self.depth += 1;
        let valid = body.iter().all(|stmt| match &stmt.kind {
            StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
            StmtKind::Try(_) => false,
            StmtKind::Throw(value) => self.expr(id,value,Mode::Consume),
            StmtKind::Let { value, .. } | StmtKind::Expr(value) => {
                self.expr(id, value, Mode::Consume)
            }
            StmtKind::Assign {
                target,
                value,
                operator,
            } => {
                let mode = if operator.is_some() {
                    if target.ty == Some(Type::String) {
                        Mode::Read
                    } else {
                        Mode::Copy
                    }
                } else {
                    Mode::Consume
                };
                self.expr(id, target, Mode::Mutate) && self.expr(id, value, mode)
            }
            StmtKind::Return(value) => value
                .as_ref()
                .is_none_or(|value| self.expr(id, value, Mode::Consume)),
            StmtKind::ForOf { iterable, body, .. } => {
                self.expr(id, iterable, Mode::Read) && self.body(id, body)
            }
            StmtKind::While { condition, body } => {
                self.expr(id, condition, Mode::Copy) && self.body(id, body)
            }
            StmtKind::Block(body) => self.body(id, body),
            StmtKind::Loop {
                condition,
                body,
                update,
                ..
            } => {
                condition
                    .as_ref()
                    .is_none_or(|condition| self.expr(id, condition, Mode::Copy))
                    && self.body(id, body)
                    && self.body(id, update)
            }
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.expr(id, condition, Mode::Copy)
                    && self.body(id, then_body)
                    && self.body(id, else_body)
            }
            StmtKind::Break | StmtKind::Continue => true,
        });
        self.depth -= 1;
        valid
    }
    fn parameter(&mut self, function: usize, id: usize) -> bool {
        if !self.visiting.insert((function, id)) {
            return true;
        }
        if self.visiting.len() > 64 {
            self.visiting.remove(&(function, id));
            return false;
        }
        let valid = self.binding(&self.program.functions[function], id);
        self.visiting.remove(&(function, id));
        valid
    }
    fn expr(&mut self, id: usize, expr: &Expr, mode: Mode) -> bool {
        if self.remaining == 0 || self.depth >= MAX_PROOF_DEPTH {
            return false;
        }
        self.remaining -= 1;
        self.depth += 1;
        let mode = if mode != Mode::Mutate && expr.ty.is_some_and(Type::is_copy) {
            Mode::Copy
        } else {
            mode
        };
        let valid = match &expr.kind {
            ExprKind::Match { body, .. } => self.body(id, body),
            ExprKind::SpreadCall(plan) => {
                let receiver = plan.callee.as_ref().is_none_or(|callee| self.expr(id, &callee.value, callee.access));
                // A spread consumer needs addressable homogeneous elements. Scalar
                // stack arrays provide them; LiteralStrings and record SoA
                // layouts fall back for this specific source use.
                receiver && plan.segments.iter().all(|segment| self.expr(id, &segment.value, if segment.spread && !self.scalar {Mode::Consume}else{segment.access}))
                    && self.expr(id, &plan.call, mode)
            }
            ExprKind::SpreadValue(selected) => !selected.source.roots.contains(&id)
                || expr.ty.is_some_and(Type::is_copy) || mode == Mode::Read,
            ExprKind::Name(_) => expr.binding != Some(id) || mode == Mode::Read,
            ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => self.expr(id, value, mode),
            ExprKind::Optional {
                base, body, access, ..
            } => {
                // Selecting a shared or mutable receiver does not itself resize
                // it. The reached body must prove every actual use separately.
                let selection = if *access == Mode::Consume {
                    Mode::Consume
                } else {
                    Mode::Read
                };
                self.expr(id, base, selection) && self.expr(id, body, mode)
            }
            ExprKind::Selected { source, .. } => {
                !source.roots.contains(&id)
                    || expr.ty.is_some_and(Type::is_copy)
                    || mode == Mode::Read
            }
            ExprKind::Integer(_)
            | ExprKind::EnumMember { .. } | ExprKind::Number(_)
            | ExprKind::Boolean(_)
            | ExprKind::String(_)
            | ExprKind::Null
            | ExprKind::Undefined => true,
            // Captures retain the ordinary generated closure layout. A function
            // containing closures conservatively retains ordinary aggregates.
            ExprKind::Closure { .. } => false,
            ExprKind::Project { value, .. } => self.expr(id, value, mode),
            ExprKind::Inject { value, .. } => self.expr(
                id,
                value,
                if value.ty.is_some_and(Type::is_copy) {
                    Mode::Copy
                } else {
                    Mode::Consume
                },
            ),
            ExprKind::Unary {
                op: UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::OptionalBoundary | UnaryOp::NonNull,
                value,
                ..
            } => self.expr(id, value, mode),
            ExprKind::Binary {
                op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or,
                left,
                right,
                ..
            } => self.expr(id, left, Mode::Consume) && self.expr(id, right, Mode::Consume),
            ExprKind::Unary { op, value, .. } => self.expr(
                id,
                value,
                if matches!(op, UnaryOp::Typeof | UnaryOp::Void) {
                    Mode::Read
                } else {
                    Mode::Copy
                },
            ),
            ExprKind::Update { target, .. } => self.expr(id, target, Mode::Mutate),
            ExprKind::LogicalAssign { target, value, .. }
            | ExprKind::Assign { target, value, .. }
            | ExprKind::CoalesceAssign { target, value, .. }
            | ExprKind::CompoundAssign { target, value, .. } => {
                self.expr(id, target, Mode::Mutate) && self.expr(id, value, Mode::Copy)
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                // Conditional aggregate arms must share their ordinary ABI.
                // Do not retain a fixed representation for either possible owner.
                let branch_mode = if expr.ty.is_some_and(Type::is_copy) {
                    mode
                } else {
                    Mode::Consume
                };
                self.expr(id, predicate, Mode::Copy)
                    && self.expr(id, consequent, branch_mode)
                    && self.expr(id, alternate, branch_mode)
            }
            ExprKind::Binary { left, right, .. } => {
                let mode = if left.ty == Some(Type::String) {
                    Mode::Read
                } else {
                    Mode::Copy
                };
                self.expr(id, left, mode) && self.expr(id, right, mode)
            }
            ExprKind::Field(base, name) => {
                let access = if mode == Mode::Mutate {
                    Mode::Mutate
                } else if expr.ty.is_some_and(Type::is_copy)
                    || (matches!(base.ty, Some(Type::Array(_))) && name == "length")
                {
                    Mode::Read
                } else {
                    mode
                };
                self.expr(id, base, access)
            }
            ExprKind::Index(base, index) => {
                let mut source = base.as_ref();
                while let ExprKind::Field(parent, _)
                | ExprKind::Index(parent, _)
                | ExprKind::Project { value: parent, .. } = &source.kind
                {
                    source = parent;
                }
                let borrowed_string = expr.ty == Some(Type::String)
                    && matches!(&source.kind,ExprKind::Selected{source,..} if source.roots.contains(&id));
                self.expr(
                    id,
                    base,
                    if borrowed_string {
                        Mode::Consume
                    } else if mode == Mode::Copy
                        || (self.scalar
                            && mode == Mode::Mutate
                            && expr.ty.is_some_and(Type::is_copy))
                    {
                        Mode::Read
                    } else {
                        mode
                    },
                ) && self.expr(id, index, Mode::Copy)
            }
            ExprKind::Template(parts) => parts.iter().all(|part| self.expr(id, part, Mode::Read)),
            ExprKind::Sequence(_) => false,
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) => values.iter().all(|v| self.expr(id, v, Mode::Consume)),
            ExprKind::Record(fields) => fields.iter().all(|(_, v)| self.expr(id, v, Mode::Consume)),
            ExprKind::Call(callee, args) => match expr.target.expect("checked call") {
                CallTarget::ArrayOf => unreachable!("Array.of lowers to an array expression"),
                CallTarget::NativeBinding(binding) => args
                    .iter()
                    .zip(crate::native_bindings::BINDINGS[binding].parameters)
                    .all(|(argument, parameter)| self.expr(id, argument, parameter.access)),
                // Generated JSON visitors borrow the ordinary aggregate ABI.
                CallTarget::SchemaStrip | CallTarget::SchemaRetain | CallTarget::SchemaPresence => self.expr(id,&args[0],Mode::Consume),
                CallTarget::SchemaDefault | CallTarget::SchemaChoice(_) | CallTarget::SchemaConstraint(_) | CallTarget::SchemaMessagePolicy(_) => args.iter().all(|argument|self.expr(id,argument,Mode::Consume)),
                CallTarget::SchemaAddIssue(_) => self.expr(id,&args[0],Mode::Mutate) && self.expr(id,&args[1],Mode::Consume),
                CallTarget::ResultUnwrap(_) | CallTarget::MatchYield(_) => self.expr(id,&args[0],Mode::Consume),
                CallTarget::SchemaExport(_) => self.expr(id,&args[0],Mode::Read),
                CallTarget::SchemaValidate => self.expr(id,&args[0],Mode::Read) && self.expr(id,&args[1],Mode::Consume),
                CallTarget::JsonEncode => args.iter().all(|a| self.expr(id, a, Mode::Consume)),
                CallTarget::JsonDecode | CallTarget::JsonStringLength | CallTarget::JsonStringView | CallTarget::JsonBoolean | CallTarget::BooleanConversion | CallTarget::ArrayIsArray | CallTarget::ObjectHasOwn | CallTarget::ObjectIs | CallTarget::ObjectKeys | CallTarget::ObjectValues | CallTarget::ObjectEntries(_)
                | CallTarget::StringRaw | CallTarget::StringFromCodePoint | CallTarget::StringFromCharCode => {
                    args.iter().all(|a| self.expr(id, a, Mode::Read))
                }
                CallTarget::RecordSpread(_) => args.iter().all(|arg|if let ExprKind::Unary{op:UnaryOp::Spread,value,..}=&arg.kind{self.expr(id,value,Mode::Read)}else{self.expr(id,arg,Mode::Consume)}),
                CallTarget::StringConversion => args.iter().all(|a| self.expr(id, a, Mode::Read)),
                CallTarget::NumberParseFloat => self.expr(id, &args[0], Mode::Read),
                CallTarget::NumberParseInt => {
                    self.expr(id, &args[0], Mode::Read)
                        && args.get(1).is_none_or(|radix| self.expr(id, radix, Mode::Copy))
                }
                CallTarget::Number => args
                    .first()
                    .is_none_or(|argument| self.expr(id, argument, Mode::Read)),
                CallTarget::NumberIsFinite
                | CallTarget::NumberIsNaN
                | CallTarget::NumberIsInteger
                | CallTarget::NumberIsSafeInteger => self.expr(id, &args[0], Mode::Copy),
                CallTarget::MathAbs
                | CallTarget::MathFloor
                | CallTarget::MathCeil
                | CallTarget::MathTrunc
                | CallTarget::MathSqrt
                | CallTarget::MathRound
                | CallTarget::MathSign
                | CallTarget::MathFround
                | CallTarget::MathF16round
                | CallTarget::MathRandom
                | CallTarget::MathMin
                | CallTarget::MathMax
                | CallTarget::MathClz32
                | CallTarget::MathImul
                | CallTarget::MathAcos
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
                | CallTarget::MathHypot => args
                    .iter()
                    .all(|argument| self.expr(id, argument, Mode::Copy)),
                CallTarget::Clone => self.expr(id, &args[0], Mode::Read),
                // Structured console inspection uses ordinary aggregate layouts.
                CallTarget::Log => args.iter().all(|argument| {
                    !matches!(argument.ty, Some(Type::Array(_) | Type::Record(_)))
                        && self.expr(id, argument, Mode::Read)
                }),
                CallTarget::ArrayIncludes
                | CallTarget::ArrayIndexOf
                | CallTarget::ArrayLastIndexOf
                | CallTarget::StringStartsWith
                | CallTarget::StringEndsWith
                | CallTarget::StringIncludes
                | CallTarget::StringIndexOf
                | CallTarget::StringLastIndexOf => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && self.expr(id, &args[0], Mode::Read)
                        && args.get(1).is_none_or(|arg| self.expr(id, arg, Mode::Copy))
                }
                CallTarget::IntToLocaleString
                | CallTarget::NumberToLocaleString
                | CallTarget::IntToString
                | CallTarget::NumberToString
                | CallTarget::NumberToFixed
                | CallTarget::NumberToExponential
                | CallTarget::NumberToPrecision => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Copy)
                        && args
                            .first()
                            .is_none_or(|radix| self.expr(id, radix, Mode::Copy))
                }
                CallTarget::StringIdentity
                | CallTarget::StringIsWellFormed
                | CallTarget::BooleanIdentity
                | CallTarget::BooleanToString
                | CallTarget::NumberIdentity
                | CallTarget::IntIdentity => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(
                        id,
                        base,
                        if matches!(
                            expr.target,
                            Some(CallTarget::StringIdentity | CallTarget::StringIsWellFormed)
                        ) {
                            Mode::Read
                        } else {
                            Mode::Copy
                        },
                    )
                }
                CallTarget::StringNormalize => {
                    let ExprKind::Field(base, _) = &callee.kind else { unreachable!() };
                    self.expr(id, base, Mode::Read) && args.iter().all(|argument|self.expr(id, argument, Mode::Read))
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
                    self.expr(id, base, Mode::Read)
                        && args.iter().enumerate().all(|(index, argument)| {
                            let integer = expr.target == Some(CallTarget::StringRepeat)
                                || (matches!(
                                    expr.target,
                                    Some(CallTarget::StringPadStart | CallTarget::StringPadEnd)
                                ) && index == 0)
                                || (expr.target == Some(CallTarget::StringSplit) && index == 1);
                            self.expr(id, argument, if integer { Mode::Copy } else { Mode::Read })
                        })
                }
                CallTarget::ArrayReduce | CallTarget::ArrayReduceRight => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let parameters = match args[0].ty.expect("checked reduction callback") {
                        Type::Closure(index) => self.program.closures[index].parameters.len(),
                        Type::Callback(index) => {
                            self.program.callback_contracts[index].params.len()
                        }
                        Type::StoredCallable(family) => {
                            self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.len()
                        }
                        _ => unreachable!("checked reduction callback"),
                    };
                    self.expr(
                        id,
                        base,
                        if parameters == 4 {
                            Mode::Consume
                        } else {
                            Mode::Read
                        },
                    ) && self.expr(id, &args[0], Mode::Mutate)
                        && args
                            .get(1)
                            .is_none_or(|initial| self.expr(id, initial, Mode::Consume))
                }
                CallTarget::ArrayForEach
                | CallTarget::ArrayMap
                | CallTarget::ArrayFlatMap
                | CallTarget::ArrayFilter
                | CallTarget::ArraySome
                | CallTarget::ArrayEvery
                | CallTarget::ArrayFindIndex
                | CallTarget::ArrayFindLastIndex
                | CallTarget::ArrayFind
                | CallTarget::ArrayFindLast => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let parameters = match args[0].ty.expect("checked callback") {
                        Type::Closure(index) => self.program.closures[index].parameters.len(),
                        Type::Callback(index) => {
                            self.program.callback_contracts[index].params.len()
                        }
                        Type::StoredCallable(family) => {
                            self.program.callback_contracts[self.program.stored_contracts.declaration(family).expect("checked traversal storage")].params.len()
                        }
                        _ => unreachable!("checked callback type"),
                    };
                    // A callback receiving the whole array uses its ordinary array ABI.
                    self.expr(
                        id,
                        base,
                        if parameters == 3
                            || (matches!(
                                expr.target,
                                Some(
                                    CallTarget::ArrayFind
                                        | CallTarget::ArrayFindLast
                                        | CallTarget::ArrayFlatMap
                                )
                            ) && !self.scalar)
                        {
                            Mode::Consume
                        } else {
                            Mode::Read
                        },
                    ) && self.expr(id, &args[0], Mode::Mutate)
                }
                CallTarget::ArrayToLocaleString | CallTarget::ArrayJoin => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && args
                            .first()
                            .is_none_or(|argument| self.expr(id, argument, Mode::Read))
                }
                CallTarget::ArrayFlat(_) => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    // Retained array children require the ordinary owning ABI.
                    // Scalar flattening can read fixed arrays directly.
                    let compatible =
                        matches!(expr.ty, Some(Type::Array(element)) if element.ty().is_copy());
                    self.expr(
                        id,
                        base,
                        if compatible {
                            Mode::Read
                        } else {
                            Mode::Consume
                        },
                    ) && args
                        .first()
                        .is_none_or(|arg| self.expr(id, arg, Mode::Copy))
                }
                CallTarget::ArrayConcat => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && args.iter().all(|arg| {
                            self.expr(
                                id,
                                arg,
                                if arg.ty.expect("checked concat argument").is_copy() {
                                    Mode::Copy
                                } else if matches!(arg.ty, Some(Type::Record(_))) {
                                    // Scalar records use the ordinary record ABI in concat.
                                    // This is a layout escape barrier, not source consumption.
                                    Mode::Consume
                                } else {
                                    Mode::Read
                                },
                            )
                        })
                }
                CallTarget::ArraySlice | CallTarget::PatternRest(_,_) => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && args.iter().all(|arg| self.expr(id, arg, Mode::Copy))
                }
                CallTarget::ArrayToSpliced => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && args.iter().enumerate().all(|(index, argument)| {
                            self.expr(
                                id,
                                argument,
                                if index < 2 { Mode::Copy } else { Mode::Consume },
                            )
                        })
                }
                CallTarget::ArrayToReversed | CallTarget::ArrayKeys | CallTarget::ArrayEntries(_) | CallTarget::ArrayWith => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(id, base, Mode::Read)
                        && args.iter().enumerate().all(|(index, arg)| {
                            self.expr(
                                id,
                                arg,
                                if index == 0 {
                                    Mode::Copy
                                } else {
                                    Mode::Consume
                                },
                            )
                        })
                }
                CallTarget::ArrayFill | CallTarget::ArrayCopyWithin => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(
                        id,
                        base,
                        if self.scalar {
                            Mode::Read
                        } else {
                            Mode::Mutate
                        },
                    ) && args.iter().enumerate().all(|(index, arg)| {
                        self.expr(
                            id,
                            arg,
                            if expr.target == Some(CallTarget::ArrayFill) && index == 0 {
                                Mode::Consume
                            } else {
                                Mode::Copy
                            },
                        )
                    })
                }
                CallTarget::ArrayAt => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    // Optional owned references require stable ordinary payload slots.
                    let escape = matches!(expr.ty, Some(Type::BorrowedUnion(_))) && !self.scalar;
                    self.expr(id, base, if escape { Mode::Consume } else { Mode::Read })
                        && args.iter().all(|arg| self.expr(id, arg, Mode::Copy))
                }
                CallTarget::ArraySort | CallTarget::ArrayToSorted => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let copying = expr.target == Some(CallTarget::ArrayToSorted);
                    // Sorting preserves width, but owned slots must be ordinary mutable values.
                    let mode = if copying || self.scalar {
                        Mode::Read
                    } else {
                        Mode::Mutate
                    };
                    self.expr(id, base, mode)
                        && args.iter().all(|arg| self.expr(id, arg, Mode::Mutate))
                }
                CallTarget::ArrayReverse => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    self.expr(
                        id,
                        base,
                        if self.scalar {
                            Mode::Read
                        } else {
                            Mode::Mutate
                        },
                    )
                }
                CallTarget::Trim
                | CallTarget::TrimStart
                | CallTarget::TrimEnd
                | CallTarget::Lower
                | CallTarget::Upper
                | CallTarget::StringSlice
                | CallTarget::StringSubstring
                | CallTarget::StringSubstr
                | CallTarget::StringCharAt
                | CallTarget::StringCodePointSlot
                | CallTarget::StringCharCodeAt
                | CallTarget::ArrayPush
                | CallTarget::ArrayUnshift
                | CallTarget::ArraySplice
                | CallTarget::ArrayRemove
                | CallTarget::ArrayPop
                | CallTarget::ArrayShift => {
                    let ExprKind::Field(base, _) = &callee.kind else {
                        unreachable!()
                    };
                    let access = if (matches!(
                        expr.target,
                        Some(CallTarget::ArrayUnshift | CallTarget::ArraySplice)
                    ) && !args.is_empty())
                        || matches!(
                            expr.target,
                            Some(
                                CallTarget::ArrayPush
                                    | CallTarget::ArrayRemove
                                    | CallTarget::ArrayPop
                                    | CallTarget::ArrayShift
                            )
                        ) {
                        Mode::Mutate
                    } else {
                        Mode::Read
                    };
                    self.expr(id, base, access)
                        && args.iter().all(|v| self.expr(id, v, Mode::Consume))
                }
                CallTarget::Function(function) | CallTarget::ForwardFunction(function) => {
                    // A capturing nested function also receives bindings of
                    // this body as hidden arguments. Their storage is not
                    // proved here, so no binding keeps a fixed shape.
                    self.program.named_capture_access.get(&function).is_none_or(Vec::is_empty)
                        && (expr.target != Some(CallTarget::ForwardFunction(function))
                        || self.expr(id, callee, Mode::Read))
                        && args.iter().enumerate().all(|(param, arg)| {
                            let mode = self.program.functions[function].bindings[param].mode;
                            let stable = !affected(arg, id)
                                || arg.ty.is_some_and(Type::is_copy)
                                || self.parameter(function, param);
                            let access = if self.scalar && mode == Mode::Mutate && stable {
                                Mode::Read
                            } else {
                                mode
                            };
                            stable && self.expr(id, arg, access)
                        })
                }
                CallTarget::ConditionalClosure(_) | CallTarget::ConditionalCallback(_) => {
                    self.expr(id, callee, Mode::Mutate)
                        && args
                            .iter()
                            .all(|arg| !affected(arg, id) && self.expr(id, arg, Mode::Consume))
                }
                CallTarget::ClosureMethod(_) => {
                    self.expr(id, callee, Mode::Mutate)
                        && args
                            .iter()
                            .all(|arg| !affected(arg, id) && self.expr(id, arg, Mode::Consume))
                }
                CallTarget::Closure(_) | CallTarget::Callback(_) | CallTarget::StoredCallable(_) => args
                    .iter()
                    .all(|arg| !affected(arg, id) && self.expr(id, arg, Mode::Consume)),
                CallTarget::StoredCallableField(_) | CallTarget::HeapMethod(_) | CallTarget::OrderedSetMethod(_) | CallTarget::DequeMethod(_) | CallTarget::MapMethod(_) | CallTarget::SetMethod(_) | CallTarget::MapGroupBy => false,
            },
        };
        self.depth -= 1;
        valid
    }
}

fn affected(expr: &Expr, id: usize) -> bool {
    match &expr.kind {
        ExprKind::Optional { base, body, .. } => affected(base, id) || affected(body, id),
        ExprKind::TypeAssertion {value,..} | ExprKind::OptionalChain(value) => affected(value, id),
        ExprKind::Selected { source, .. } => source.roots.contains(&id),
        ExprKind::Conditional {
            consequent,
            alternate,
            ..
        } => affected(consequent, id) || affected(alternate, id),
        ExprKind::Binary {
            op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or,
            left,
            right,
            ..
        } => affected(left, id) || affected(right, id),
        ExprKind::Unary {
            op: UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::OptionalBoundary | UnaryOp::NonNull,
            value,
            ..
        } => affected(value, id),
        ExprKind::Name(_) => expr.binding == Some(id),
        ExprKind::Project { value: base, .. }
        | ExprKind::Inject { value: base, .. }
        | ExprKind::Field(base, _)
        | ExprKind::Index(base, _) => affected(base, id),
        _ => false,
    }
}

pub(crate) fn visit_statements(body: &[Stmt], visit: &mut impl FnMut(&Stmt)) {
    let mut work: Vec<&Stmt> = body.iter().rev().collect();
    while let Some(stmt) = work.pop() {
        visit(stmt);
        match &stmt.kind {
            StmtKind::ForOf { body, .. } | StmtKind::While { body, .. } | StmtKind::Block(body) => work.extend(body.iter().rev()),
            StmtKind::Loop { body, update, .. } => { work.extend(update.iter().rev()); work.extend(body.iter().rev()); },
            StmtKind::If { then_body, else_body, .. } => { work.extend(else_body.iter().rev()); work.extend(then_body.iter().rev()); },
            StmtKind::Try(region) => {
                if let Some(finally) = &region.finally { work.extend(finally.iter().rev()); }
                if let Some(catch) = &region.catch { work.extend(catch.body.iter().rev()); }
                work.extend(region.body.iter().rev());
            },
            _ => {},
        }
    }
}

pub(crate) fn visit_function_expressions(function: &CheckedFunction, visit: &mut impl FnMut(&Expr)) {
    for binding in &function.bindings {
        if let Some(initializer) = &binding.initializer {
            visit_expression(initializer, visit);
        }
    }
    visit_expressions(&function.body, visit);
}

fn visit_expression(expr: &Expr, visit: &mut impl FnMut(&Expr)) {
    let mut work = vec![expr];
    while let Some(expr) = work.pop() {
        visit(expr);
        match &expr.kind {
            ExprKind::Match { body, .. } => visit_expressions(body, visit),
            ExprKind::SpreadCall(plan) => work.extend(plan.expressions().collect::<Vec<_>>().into_iter().rev()),
            ExprKind::SpreadValue(_) => (),
            ExprKind::TypeAssertion { value, .. } | ExprKind::OptionalChain(value)
            | ExprKind::Project { value, .. } | ExprKind::Inject { value, .. }
            | ExprKind::Unary { value, .. } | ExprKind::Update { target: value, .. }
            | ExprKind::Field(value, _) => work.push(value),
            ExprKind::Conditional { predicate, consequent, alternate } => { work.push(alternate); work.push(consequent); work.push(predicate); },
            ExprKind::Optional { base: left, body: right, .. } | ExprKind::Binary { left, right, .. }
            | ExprKind::CompoundAssign { target: left, value: right, .. } | ExprKind::Assign { target: left, value: right, .. }
            | ExprKind::CoalesceAssign { target: left, value: right, .. } | ExprKind::LogicalAssign { target: left, value: right, .. }
            | ExprKind::Index(left, right) => { work.push(right); work.push(left); },
            ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => work.extend(values.iter().rev()),
            ExprKind::Record(fields) => work.extend(fields.iter().rev().map(|(_, value)| value)),
            ExprKind::Call(callee, args) => { work.extend(args.iter().rev()); work.push(callee); },
            ExprKind::Selected { .. } | ExprKind::Closure { .. } | ExprKind::Name(_) | ExprKind::String(_) | ExprKind::Integer(_)
            | ExprKind::EnumMember { .. } | ExprKind::Number(_) | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {},
        }
    }
}

fn visit_expressions(body: &[Stmt], visit: &mut impl FnMut(&Expr)) {
    visit_statements(body, &mut |stmt| match &stmt.kind {
        StmtKind::Let { value, .. } | StmtKind::Expr(value) => visit_expression(value, visit),
        StmtKind::Assign { target, value, .. } => {
            visit_expression(target, visit);
            visit_expression(value, visit);
        }
        StmtKind::Return(Some(value)) | StmtKind::Throw(value) => visit_expression(value, visit),
        StmtKind::While { condition, .. } | StmtKind::If { condition, .. } => {
            visit_expression(condition, visit)
        }
        StmtKind::ForOf { iterable, .. } => visit_expression(iterable, visit),
        StmtKind::Loop {
            condition: Some(condition),
            ..
        } => visit_expression(condition, visit),
        _ => {}
    });
}

#[cfg(test)]
mod native_layout_tests {
    use super::*;
    #[test]
    fn opaque_native_storage_is_not_inferred_from_empty_public_fields() {
        let mut program = crate::syntax::parse("type Empty={};function main(){}").unwrap();
        let native = [crate::native_signatures::NativeTypeId::Bytes, crate::native_signatures::NativeTypeId::ByteView, crate::native_signatures::NativeTypeId::ByteViewMut]
            .map(|id| crate::native_type_resolution::intern(&mut program.records, id, crate::Span::new(0, 0)).unwrap());
        let checked = crate::analysis::analyze(program).unwrap();
        let runtime = RecordRuntime::new(&checked.records, &checked.unions);
        for id in native { assert_eq!(layout_bound(&checked, &runtime, Type::Record(id), 0), None); }
        assert_eq!(layout_bound(&checked, &runtime, Type::Record(0), 0), Some(8));
    }
}

#[cfg(test)]
mod traversal_tests {
    use super::*;
    #[test]
    fn deep_storage_visits_preserve_preorder_and_closure_boundaries() {
        let span = crate::Span::new(0, 1);
        let mut value = Expr::new(ExprKind::Integer("0".into()), span);
        for index in 1..=16_384 {
            value = Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(index.to_string()), span)), vec![value]), span);
        }
        let mut names = Vec::new();
        visit_expression(&value, &mut |expr| if let ExprKind::Name(name) = &expr.kind { names.push(name.parse::<usize>().unwrap()); });
        assert_eq!(names, (1..=16_384).rev().collect::<Vec<_>>());
        let mut body = vec![Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Expr(value), span }];
        for index in 1..=16_384 {
            body = vec![Stmt { labels: Vec::new(), control_target: None, kind: StmtKind::Block(body), span: crate::Span::new(index, index + 1) }];
        }
        let mut order = Vec::new();
        visit_statements(&body, &mut |stmt| order.push(stmt.span.start));
        assert_eq!(order, (0..=16_384).rev().collect::<Vec<_>>());
        let mut calls = 0;
        visit_expressions(&body, &mut |expr| if matches!(expr.kind, ExprKind::Call(..)) { calls += 1; });
        assert_eq!(calls, 16_384);
        drop(body);
        let program = crate::syntax::parse("function main(){const f=()=>hidden();}").unwrap();
        let StmtKind::Let { value, .. } = &program.functions[0].body[0].kind else { panic!("closure") };
        let mut visits = 0;
        visit_expression(value, &mut |_| visits += 1);
        assert_eq!(visits, 1);
    }
}
