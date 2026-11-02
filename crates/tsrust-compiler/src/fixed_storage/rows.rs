//! Owned fixed-width rows inferred across a closed set of named calls.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Element { Int, Number, Boolean, String }
impl Element {
    pub(crate) fn native(self) -> &'static str {
        match self { Self::Int => "i64", Self::Number => "f64", Self::Boolean => "bool", Self::String => "String" }
    }
    fn of(ty: Type, depth: u8) -> Option<Self> {
        match ty {
            Type::Array(ArrayElement::Int(n)) if n == depth => Some(Self::Int),
            Type::Array(ArrayElement::Number(n)) if n == depth => Some(Self::Number),
            Type::Array(ArrayElement::Boolean(n)) if n == depth => Some(Self::Boolean),
            Type::Array(ArrayElement::String(n)) if n == depth => Some(Self::String),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Fact { shape: Shape, origins: BTreeSet<usize> }

#[derive(Default)]
pub(super) struct Facts {
    values: BTreeMap<(usize, usize), Fact>,
    outputs: BTreeMap<usize, Fact>,
    invalid: BTreeSet<usize>,
}

impl Facts {
    pub(super) fn initializer(&self, function: usize, binding: usize, value: &Expr) -> bool {
        self.expression(function, value).is_some_and(|fact| self.binding(function, binding) == Some(&fact.shape))
    }
    pub(super) fn returned(&self, function: usize, value: &Expr) -> bool {
        self.expression(function, value).is_some_and(|fact| self.result(function) == Some(&fact.shape))
    }
    pub(super) fn argument(&self, caller: usize, target: usize, parameter: usize, value: &Expr) -> bool {
        self.expression(caller, value).is_some_and(|fact| self.binding(target, parameter) == Some(&fact.shape))
    }
    pub(super) fn push(&self, function: usize, base: &Expr, value: &Expr) -> bool {
        matches!(self.expression(function, base), Some(Fact { shape: Shape::Rows { width, .. }, .. }) if row_width(value) == Some(width))
    }
    fn approve(&self, function: usize, value: &Expr, approved: &mut BTreeSet<(usize,usize)>) {
        if self.expression(function, value).is_none() { return; }
        match &value.kind {
            ExprKind::Call(_, _) if matches!(value.target, Some(CallTarget::Function(_))) => { approved.insert((value.span.start,value.span.end)); }
            ExprKind::Call(_, args) if value.target == Some(CallTarget::Clone) => self.approve(function,&args[0],approved),
            ExprKind::TypeAssertion { value, .. } | ExprKind::Index(value, _) => self.approve(function,value,approved),
            _ => (),
        }
    }
    pub(super) fn validated(mut self, program: &CheckedProgram) -> Self {
        if self.values.is_empty() && self.outputs.is_empty() { return self; }
        for (&(function,binding), fact) in &self.values {
            let mut proof = Proof::new(program, matches!(fact.shape, Shape::Row { element: Element::Int | Element::Number | Element::Boolean, .. }));
            proof.row_facts = Some(&self);
            if !proof.binding(&program.functions[function],binding) { self.invalid.extend(fact.origins.iter().copied()); }
        }
        for (function, fun) in program.functions.iter().enumerate() {
            let mut approved = BTreeSet::new();
            statements(fun, &mut |statement| match &statement.kind {
                StmtKind::Let { binding:Some(binding),value,.. } if self.initializer(function,*binding,value) => self.approve(function,value,&mut approved),
                StmtKind::Return(Some(value)) if self.returned(function,value) => self.approve(function,value,&mut approved),
                StmtKind::Expr(value) | StmtKind::ForOf { iterable:value,.. } => self.approve(function,value,&mut approved),
                _ => (),
            });
            visit_function_expressions(fun, &mut |value| {
                if let ExprKind::Call(_, args) = &value.kind && let Some(CallTarget::Function(target)) = value.target {
                    for (parameter,arg) in args.iter().enumerate() {
                        if self.argument(function,target,parameter,arg) { self.approve(function,arg,&mut approved); }
                    }
                }
                if let ExprKind::Field(base,name) = &value.kind && name == "length" { self.approve(function,base,&mut approved); }
                if let ExprKind::Index(base,_) = &value.kind
                    && matches!(self.expression(function,base),Some(Fact { shape:Shape::Row { element:Element::Number,.. },.. })) {
                    self.approve(function,base,&mut approved);
                }
            });
            visit_function_expressions(fun, &mut |value| {
                if let Some(CallTarget::Function(target)) = value.target
                    && let Some(fact) = self.outputs.get(&target)
                    && !approved.contains(&(value.span.start,value.span.end)) {
                    self.invalid.extend(fact.origins.iter().copied());
                }
            });
        }
        loop {
            let before = self.invalid.len();
            for fact in self.values.values().chain(self.outputs.values()) {
                if !fact.origins.is_disjoint(&self.invalid) { self.invalid.extend(fact.origins.iter().copied()); }
            }
            if before == self.invalid.len() { break; }
        }
        self.values.retain(|_,fact| fact.origins.is_disjoint(&self.invalid));
        self.outputs.retain(|_,fact| fact.origins.is_disjoint(&self.invalid));
        self
    }
    pub(super) fn binding(&self, function: usize, binding: usize) -> Option<&Shape> {
        self.values.get(&(function, binding)).map(|fact| &fact.shape)
    }
    pub(super) fn result(&self, function: usize) -> Option<&Shape> {
        self.outputs.get(&function).map(|fact| &fact.shape)
    }
    fn merge(facts: impl IntoIterator<Item = Fact>) -> Option<Fact> {
        let mut facts = facts.into_iter();
        let mut result = facts.next()?;
        for fact in facts {
            if fact.shape != result.shape { return None; }
            result.origins.extend(fact.origins);
        }
        Some(result)
    }
    fn expression(&self, function: usize, value: &Expr) -> Option<Fact> {
        match &value.kind {
            ExprKind::Array(items) if value.ty == Some(Type::Array(ArrayElement::Number(0))) => {
                let fact=self.outputs.get(&function)?;
                matches!(fact.shape,Shape::Row { element:Element::Number,width } if width==items.len()).then(||fact.clone())
            }
            ExprKind::Name(_) => self.values.get(&(function, value.binding?)).cloned(),
            ExprKind::Selected { source, .. } => {
                let (binding, fields) = source.place.as_ref()?;
                fields.is_empty().then(|| self.values.get(&(function, *binding)).cloned()).flatten()
            }
            ExprKind::TypeAssertion { value, .. } => self.expression(function, value),
            ExprKind::Call(_, args) if value.target == Some(CallTarget::Clone) => self.expression(function, &args[0]),
            ExprKind::Call(_, _) => match value.target? {
                CallTarget::Function(target) => self.outputs.get(&target).cloned(),
                _ => None,
            },
            ExprKind::Index(base, _) => {
                let mut fact = self.expression(function, base)?;
                let Shape::Rows { element, width } = fact.shape else { return None; };
                fact.shape = Shape::Row { element, width };
                Some(fact)
            }
            _ => None,
        }
    }
    fn put(&mut self, key: (usize, usize), fact: Fact) -> bool {
        if let Some(previous) = self.values.get(&key) {
            if previous.shape != fact.shape {
                self.invalid.extend(previous.origins.iter().chain(&fact.origins).copied());
                return false;
            }
            let joined = Self::merge([previous.clone(), fact]).unwrap();
            if &joined == previous { return false; }
            self.values.insert(key, joined);
        } else { self.values.insert(key, fact); }
        true
    }
    pub(super) fn infer(program: &CheckedProgram) -> Self {
        if !program.functions.iter().any(|fun| fun.result==Type::Array(ArrayElement::Number(0)) || fun.bindings.iter().any(|binding| Element::of(binding.ty,1).is_some())) { return Self::default(); }
        if crate::emit::needs_exception_transport(program) { return Self::default(); }
        let mut facts = Self::default();
        let mut seeds = 0;
        let mut forbidden = program.closures.iter().filter_map(|closure| match closure.forward {
            Some(crate::analysis::ClosureForward::Function(id)) => Some(id), _ => None,
        }).collect::<BTreeSet<_>>();
        for closure in &program.closures {
            for fun in std::iter::once(&closure.function).chain(closure.methods.iter().map(|method| &method.function)) {
                visit_function_expressions(fun, &mut |value| {
                    if let Some(CallTarget::Function(id) | CallTarget::ForwardFunction(id)) = value.target { forbidden.insert(id); }
                });
            }
        }
        let mut calls = vec![Vec::new(); program.functions.len()];
        for (caller, fun) in program.functions.iter().enumerate() {
            visit_function_expressions(fun, &mut |value| {
                if let Some(CallTarget::ForwardFunction(id)) = value.target { forbidden.insert(id); }
                if let Some(CallTarget::Function(id)) = value.target && let ExprKind::Call(_, args) = &value.kind {
                    calls[id].push((caller, args.as_slice()));
                }
            });
        }
        let eligible = |id: usize| !program.functions[id].template
            && !program.error_effects.function_errors[id].opaque
            && program.error_effects.function_errors[id].explicit.is_empty()
            && program.named_capture_access.get(&id).is_none_or(Vec::is_empty)
            && !forbidden.contains(&id);
        for (function, fun) in program.functions.iter().enumerate() {
            if !eligible(function) { continue; }
            if fun.result==Type::Array(ArrayElement::Number(0)) && seeds<MAX_VARIANTS {
                let mut widths=Vec::new();let mut complete=true;
                statements(fun,&mut |statement|if let StmtKind::Return(value)=&statement.kind {
                    match value.as_ref().and_then(row_width) {Some(width)=>widths.push(width),None=>complete=false}
                });
                if complete && let Some(&width)=widths.first()
                    && width!=0 && widths.iter().all(|other|*other==width)
                    && width.checked_mul(8).is_some_and(|bytes|bytes<=MAX_STACK_BYTES) {
                    facts.outputs.insert(function,Fact {shape:Shape::Row {element:Element::Number,width},origins:BTreeSet::from([seeds])});
                    seeds+=1;
                }
            }
            let mut pushes = BTreeMap::<usize, Vec<Option<usize>>>::new();
            visit_function_expressions(fun, &mut |expression| {
                if expression.target != Some(CallTarget::ArrayPush) { return; }
                let ExprKind::Call(callee, args) = &expression.kind else { return; };
                let ExprKind::Field(base, _) = &callee.kind else { return; };
                if let Some(binding) = base.binding { pushes.entry(binding).or_default().push(args.first().and_then(row_width)); }
            });
            visit_statements(&fun.body, &mut |statement| {
                let StmtKind::Let { binding: Some(binding), value, .. } = &statement.kind else { return; };
                let Some(element) = Element::of(fun.bindings[*binding].ty, 1) else { return; };
                let ExprKind::Array(initial) = &value.kind else { return; };
                let Some(mut widths) = initial.iter().map(row_width).collect::<Option<Vec<_>>>() else { return; };
                if let Some(appended) = pushes.get(binding) {
                    let Some(appended) = appended.iter().copied().collect::<Option<Vec<_>>>() else { return; };
                    widths.extend(appended);
                }
                let Some(&width) = widths.first() else { return; };
                let bytes = if element == Element::String { crate::storage::string_storage_bound() } else { 8 };
                if widths.iter().any(|candidate| *candidate != width)
                    || (matches!(element, Element::Int | Element::Boolean) && width <= 4)
                    || width.checked_mul(bytes).is_none_or(|size| size > MAX_STACK_BYTES)
                    || initial.len().checked_mul(width).and_then(|size| size.checked_mul(bytes)).is_none_or(|size| size > MAX_STACK_BYTES)
                    || seeds >= MAX_VARIANTS { return; }
                facts.put((function, *binding), Fact { shape: Shape::Rows { element, width }, origins: BTreeSet::from([seeds]) });
                seeds += 1;
            });
        }
        if seeds == 0 { return facts; }
        let mut remaining = MAX_PROOF_STEPS;
        loop {
            let mut changed = false;
            for (function, fun) in program.functions.iter().enumerate() {
                if !eligible(function) { continue; }
                statements(fun, &mut |statement| {
                    remaining = remaining.saturating_sub(1);
                    match &statement.kind {
                        StmtKind::Let { binding: Some(binding), value, .. } => {
                            if let Some(fact) = facts.expression(function, value) { changed |= facts.put((function, *binding), fact); }
                        }
                        StmtKind::ForOf { binding: Some(binding), iterable, .. } => {
                            if let Some(mut fact) = facts.expression(function, iterable)
                                && let Shape::Rows { element, width } = fact.shape {
                                fact.shape = Shape::Row { element, width };
                                changed |= facts.put((function, *binding), fact);
                            }
                        }
                        _ => (),
                    }
                });
                let mut returns = Vec::new();
                let mut complete = true;
                statements(fun, &mut |statement| {
                    if let StmtKind::Return(value) = &statement.kind {
                        match value.as_ref().and_then(|value| facts.expression(function, value)) {
                            Some(fact) => returns.push(fact), None => complete = false,
                        }
                    }
                });
                if complete && let Some(fact) = Self::merge(returns)
                    && facts.outputs.get(&function) != Some(&fact) {
                    facts.outputs.insert(function, fact); changed = true;
                }
                if remaining == 0 { return Self::default(); }
            }
            for target in 0..program.functions.len() {
                if !eligible(target) { continue; }
                for parameter in 0..program.functions[target].parameter_count {
                    let binding = &program.functions[target].bindings[parameter];
                    if Element::of(binding.ty, 0).is_none() && Element::of(binding.ty, 1).is_none() { continue; }
                    if program.functions[target].bindings[parameter].omission.is_omittable() { continue; }
                    if binding.input_ty.is_some_and(|input| input != binding.ty) { continue; }
                    let mut supplied = Vec::new(); let mut complete = true;
                    for &(caller, args) in &calls[target] {
                            remaining = remaining.saturating_sub(1);
                            match args.get(parameter).and_then(|arg| facts.expression(caller, arg)) {
                                Some(fact) => supplied.push(fact), None => complete = false,
                            }
                    }
                    if complete && let Some(fact) = Self::merge(supplied) { changed |= facts.put((target, parameter), fact); }
                    if remaining == 0 { return Self::default(); }
                }
            }
            if remaining == 0 { return Self::default(); }
            if !changed { break; }
        }
        facts
    }
}

// Match-arm return statements exit the enclosing function too.
fn statements<'a>(fun:&'a CheckedFunction,visit:&mut impl FnMut(&'a Stmt)) {
    visit_statements(&fun.body,visit);
    visit_function_expressions(fun,&mut |value|if let ExprKind::Match {body,..}=&value.kind {visit_statements(body,visit);});
}

fn row_width(value: &Expr) -> Option<usize> {
    let ExprKind::Array(values) = &value.kind else { return None; };
    Some(values.len())
}

pub(super) fn expression_shape(bindings: &BTreeMap<usize, Shape>, results: &BTreeMap<usize, Shape>, value: &Expr) -> Option<Shape> {
    match &value.kind {
        ExprKind::Call(_, args) if value.target == Some(CallTarget::Clone) => expression_shape(bindings, results, &args[0]),
        ExprKind::Call(_, _) => match value.target? { CallTarget::Function(id) => results.get(&id).cloned(), _ => None },
        ExprKind::Index(base, _) => match expression_shape(bindings, results, base)? {
            Shape::Rows { element, width } => Some(Shape::Row { element, width }), _ => None,
        },
        ExprKind::Name(_) => bindings.get(&value.binding?).filter(|shape| matches!(shape,Shape::Rows{..}|Shape::Row{..})).cloned(),
        ExprKind::Selected { source, .. } => {
            let (binding,fields) = source.place.as_ref()?;
            fields.is_empty().then(|| bindings.get(binding).filter(|shape|matches!(shape,Shape::Rows{..}|Shape::Row{..})).cloned()).flatten()
        }
        ExprKind::TypeAssertion { value, .. } => expression_shape(bindings, results, value),
        _ => None,
    }
}
