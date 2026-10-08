//! Callable bodies are indexed without executing declared capture writes.
use super::*;
use crate::local_capture_plan::BindingId;
#[path = "generic_call_aggregate_eligibility.rs"]
mod aggregate_eligibility;
#[path = "generic_call_capture_requirements.rs"]
mod capture_requirements;

struct Scope {
    name: String,
    range: Span,
    binding: BindingId,
}
pub(super) struct Parameter {
    pub binding: Binding,
    pub initializer: ParameterInitializer,
    pub rest: bool,
}

#[derive(Default)]
pub(super) struct Index {
    bodies: HashMap<(usize, usize), Vec<Stmt>>,
    parameters: HashMap<(usize, usize), Vec<Parameter>>,
    uses: HashMap<usize, BindingId>,
    declarations: HashMap<usize, BindingId>,
    scopes: Vec<Scope>,
    aggregate_writes: HashSet<(usize,usize)>,
    written_bindings: HashSet<BindingId>,
    required_captures: HashMap<(usize,usize), HashSet<BindingId>>,
}
impl Index {
    pub(super) fn new(program: &Program) -> Result<Self, Diagnostic> {
        let mut index = Self::default();
        let mut source_writes = Vec::new();
        for plan in &program.local_capture_plans {
            index
                .uses
                .extend(plan.uses.iter().map(|(&span, &binding)| (span, binding)));
        }
        for binding in index.uses.values() {
            index
                .declarations
                .insert(binding.declaration_start, *binding);
        }
        let owners = program
            .local_declarations
            .iter()
            .map(|declaration| {
                (
                    declaration.emitted_name.as_str(),
                    declaration.function_owner,
                )
            })
            .collect::<HashMap<_, _>>();
        let mut next_owner = program.functions.len();
        for (root, function) in program.functions.iter().enumerate() {
            let owner = owners
                .get(function.name.text.as_str())
                .copied()
                .unwrap_or(root);
            index.bodies.insert(
                (function.name.span.start, function.name.span.end),
                function.body.clone(),
            );
            for (name, _, _) in &function.params {
                index.add(name, function.scope_span, owner);
            }
            index.locals(&function.body, owner);
            let parameters = function.params.iter().enumerate().map(|(position, (name, annotation, callback))| {
                let callback = callback.clone().or_else(|| crate::callable_annotations::callback(annotation));
                index.parameter(name, function.scope_span, callback.as_ref(), function.parameter_initializers.get(position).cloned().unwrap_or(ParameterInitializer::Required), function.rest_parameter && position + 1 == function.params.len())
            }).collect();
            index.parameters.insert((function.name.span.start, function.name.span.end), parameters);
            let mut probe = function.clone();
            targets::function_writes(&mut probe, &mut source_writes)?;
            crate::source_walk::function_body(&mut probe, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Closure { params, body, .. } = &expression.kind {
                    // Trusted named-value wrappers forward to the original body.
                    // Their generated parameters share the value-use span and
                    // must not acquire independent source binding identities.
                    if targets::value(expression)
                        .iter()
                        .any(|target| matches!(target, Target::Name(..)))
                    {
                        return Ok(());
                    }
                    let owner = next_owner;
                    next_owner += 1;
                    index
                        .bodies
                        .insert((expression.span.start, expression.span.end), body.clone());
                    for parameter in params {
                        index.add(&parameter.name, expression.span, owner);
                    }
                    index.locals(body, owner);
                    let parameters = params.iter().map(|parameter| {
                        let callback = match &parameter.ty {
                            ParameterAnnotation::Callback(callback) => Some(callback.as_ref().clone()),
                            ParameterAnnotation::Named(annotation) => crate::callable_annotations::callback(annotation),
                            ParameterAnnotation::Inferred(_) => None,
                        };
                        index.parameter(&parameter.name, expression.span, callback.as_ref(), parameter.initializer.clone(), parameter.rest)
                    }).collect();
                    index.parameters.insert((expression.span.start, expression.span.end), parameters);
                }
                Ok(())
            })?;
        }
        for plan in &program.local_capture_plans {
            for group in &plan.groups {
                let owners = group.members.iter().map(|&member| program.local_declarations[member].function_owner).collect::<HashSet<_>>();
                let captures = group.captures.iter().filter(|capture| !owners.contains(&capture.binding.owner)).map(|capture| capture.binding).collect::<HashSet<_>>();
                for &member in &group.members {
                    let declaration = &program.local_declarations[member];
                    if let Some(function) = program.functions.iter().find(|function| function.name.text == declaration.emitted_name) {
                        index.required_captures.insert((function.name.span.start,function.name.span.end),captures.clone());
                    }
                }
            }
        }
        let named = program.functions.iter().map(|function| (function.name.span.start,function.name.span.end)).collect::<HashSet<_>>();
        let arrow_captures = index.bodies.iter().filter(|(key,_)| !named.contains(*key)).map(|(&key,body)| {
            capture_requirements::arrow(body,index.parameters.get(&key).map(Vec::as_slice).unwrap_or(&[]),&index).map(|captures|(key,captures))
        }).collect::<Result<Vec<_>,Diagnostic>>()?;
        index.required_captures.extend(arrow_captures);
        for (&key, body) in &index.bodies {
            if aggregate_eligibility::writes(body, index.parameters.get(&key).map(Vec::as_slice).unwrap_or(&[]))? { index.aggregate_writes.insert(key); }
        }
        for write in source_writes {
            if let Some(identity) = index.binding(&write.name, write.span) { index.written_bindings.insert(identity); }
        }
        Ok(index)
    }
    pub(super) fn may_write(&self, identity: BindingId) -> bool { self.written_bindings.contains(&identity) }
    pub(super) fn captures(&self, declaration: Span) -> Option<&HashSet<BindingId>> { self.required_captures.get(&(declaration.start,declaration.end)) }
    pub(super) fn captures_available(&self, declaration: Span, bindings: &[Binding]) -> bool {
        self.required_captures.get(&(declaration.start,declaration.end)).is_none_or(|captures| captures.iter().all(|capture| bindings.iter().any(|binding| binding.identity == Some(*capture))))
    }
    pub(super) fn aggregate_writes(&self, declaration: Span) -> bool { self.aggregate_writes.contains(&(declaration.start,declaration.end)) }
    fn add(&mut self, name: &Token, range: Span, owner: usize) {
        let binding = *self
            .declarations
            .entry(name.span.start)
            .or_insert(BindingId {
                owner,
                declaration_start: name.span.start,
            });
        self.scopes.push(Scope {
            name: name.text.clone(),
            range,
            binding,
        });
    }
    pub(super) fn in_scope(&self, name: &str, range: Span) -> Option<BindingId> {
        self.scopes
            .iter()
            .find(|scope| scope.name == name && scope.range == range)
            .map(|scope| scope.binding)
    }
    pub(super) fn declared(&self, span: Span) -> Option<BindingId> {
        self.declarations.get(&span.start).copied()
    }
    fn locals(&mut self, body: &[Stmt], owner: usize) {
        let mut pending = vec![body];
        while let Some(body) = pending.pop() {
            let end = crate::source_walk::body_end(body);
            for statement in body {
                match &statement.kind {
                    StmtKind::Let { name, value, .. } => {
                        self.add(name, Span::new(value.span.end, end), owner)
                    }
                    StmtKind::Block(body) | StmtKind::While { body, .. } => pending.push(body),
                    StmtKind::If {
                        then_body,
                        else_body,
                        ..
                    } => pending.extend([then_body.as_slice(), else_body.as_slice()]),
                    StmtKind::Loop { body, update, .. } => {
                        pending.extend([body.as_slice(), update.as_slice()])
                    }
                    StmtKind::ForOf { name, body, .. } => {
                        if let Some(first) = body.first() {
                            self.add(
                                name,
                                Span::new(first.span.start, crate::source_walk::body_end(body)),
                                owner,
                            );
                        }
                        pending.push(body);
                    }
                    StmtKind::Try(region) => {
                        pending.push(&region.body);
                        if let Some(handler) = &region.catch {
                            if let (Some(name), Some(first)) = (&handler.name, handler.body.first())
                            {
                                self.add(
                                    name,
                                    Span::new(
                                        first.span.start,
                                        crate::source_walk::body_end(&handler.body),
                                    ),
                                    owner,
                                );
                            }
                            pending.push(&handler.body);
                        }
                        if let Some(body) = &region.finally {
                            pending.push(body);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    pub(super) fn binding(&self, name: &str, span: Span) -> Option<BindingId> {
        // Named captures include hoisted uses before the binding's source
        // initializer. Their resolver provenance outranks textual ranges.
        self.uses.get(&span.start).copied().or_else(|| {
            self.scopes
                .iter()
                .filter(|scope| {
                    scope.name == name
                        && scope.range.start <= span.start
                        && span.end <= scope.range.end
                })
                .min_by_key(|scope| scope.range.end - scope.range.start)
                .map(|scope| scope.binding)
        })
    }
    fn parameter(&self, name: &Token, scope: Span, callback: Option<&CallbackAnnotation>, initializer: ParameterInitializer, rest: bool) -> Parameter {
        Parameter { binding: Binding { alternatives: None,
            identity: self.declared(name.span), name: name.text.clone(), scope,
            targets: vec![Target::Unknown], origins: vec![body_origins::Origin::Unknown],
            selection: Selection::default(), elements: None, mutable: false, contract: callback.map(|callback| parameters(&callback.params, name.span)),
        }, initializer, rest }
    }
    pub(super) fn parameters(&self, declaration: Span) -> Option<&[Parameter]> {
        self.parameters.get(&(declaration.start, declaration.end)).map(Vec::as_slice)
    }
    pub(super) fn body(&self, declaration: Span) -> Option<&[Stmt]> {
        self.bodies
            .get(&(declaration.start, declaration.end))
            .map(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parsed(source: &str) -> (String, Program, Index) {
        let source = crate::format(source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let index = Index::new(&program).unwrap();
        (source, program, index)
    }
    fn assignment(body: &[Stmt]) -> &Expr {
        body.iter()
            .find_map(|statement| match &statement.kind {
                StmtKind::Assign { target, .. } => Some(target),
                _ => None,
            })
            .unwrap()
    }
    #[test]
    fn hoisted_capture_and_local_shadow_resolve_to_distinct_declarations() {
        let (_, program, index) = parsed(
            "function outer<T>(){function change(){action=two;}let action=one;function shadow(){let action=two;action=one;}}function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function main(){}",
        );
        let outer = program
            .functions
            .iter()
            .find(|function| function.name.text == "outer")
            .unwrap();
        let StmtKind::Let {
            name: outer_name, ..
        } = &outer
            .body
            .iter()
            .find(|statement| matches!(statement.kind, StmtKind::Let { .. }))
            .unwrap()
            .kind
        else {
            unreachable!()
        };
        let change = program
            .functions
            .iter()
            .find(|function| function.name.text.ends_with("#change"))
            .unwrap();
        let target = assignment(&change.body);
        assert_eq!(
            index
                .binding("action", target.span)
                .unwrap()
                .declaration_start,
            outer_name.span.start
        );
        assert!(index.body(change.name.span).is_some());
        let shadow = program
            .functions
            .iter()
            .find(|function| function.name.text.ends_with("#shadow"))
            .unwrap();
        let StmtKind::Let { name, .. } = &shadow.body[0].kind else {
            unreachable!()
        };
        let target = assignment(&shadow.body);
        assert_eq!(
            index
                .binding("action", target.span)
                .unwrap()
                .declaration_start,
            name.span.start
        );
        assert_ne!(
            index
                .binding("action", target.span)
                .unwrap()
                .declaration_start,
            outer_name.span.start
        );
    }
    #[test]
    fn arrow_capture_and_parameter_shadow_keep_lexical_identity() {
        let (_, program, index) = parsed(
            "function outer<T>(){let action=one;const change=()=>{action=two;};const shadow=(action:Function<int>):void=>{action=()=>1;};}function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function main(){}",
        );
        let outer = program
            .functions
            .iter()
            .find(|function| function.name.text == "outer")
            .unwrap();
        let StmtKind::Let { name, .. } = &outer.body[0].kind else {
            unreachable!()
        };
        for (statement, captured) in [(&outer.body[1], true), (&outer.body[2], false)] {
            let StmtKind::Let { value, .. } = &statement.kind else {
                unreachable!()
            };
            let ExprKind::Closure { params, body, .. } = &value.kind else {
                unreachable!()
            };
            assert!(index.body(value.span).is_some());
            let binding = index.binding("action", assignment(body).span).unwrap();
            let declaration = if captured {
                name.span
            } else {
                params[0].name.span
            };
            assert_eq!(Some(binding), index.declared(declaration));
            assert_eq!(
                binding.declaration_start,
                if captured {
                    name.span.start
                } else {
                    params[0].name.span.start
                }
            );
        }
    }
    #[test]
    fn synthetic_forwarders_use_original_bodies_not_shared_parameter_spans() {
        let (_, program, index) = parsed(
            "function outer<T>(value:T){function pick(first:T,second:T):T{String(value);return second;}const saved=pick;}function main(){}",
        );
        let outer = program
            .functions
            .iter()
            .find(|function| function.name.text == "outer")
            .unwrap();
        let wrapper = outer
            .body
            .iter()
            .find_map(|statement| match &statement.kind {
                StmtKind::Let { value, .. } if matches!(value.kind, ExprKind::Closure { .. }) => {
                    Some(value)
                }
                _ => None,
            })
            .unwrap();
        assert!(
            index.body(wrapper.span).is_none(),
            "synthetic forwarding arguments share spans and have no independent body identity"
        );
        let original = program
            .functions
            .iter()
            .find(|function| function.name.text.ends_with("#pick"))
            .unwrap();
        assert!(index.body(original.name.span).is_some());
    }
}

#[cfg(test)]
mod eligibility_tests {
    use super::*;
    #[test]
    fn aggregate_write_metadata_covers_statements_expressions_and_defaults() {
        for (source, writes) in [
            ("function action(items:int[]){items[0]=1;}function main(){}",true),
            ("function action(items:int[]){const saved=(items[0]=1);}function main(){}",true),
            ("function action(items:int[],saved:int=(items[0]=1)){}function main(){}",true),
            ("function action(items:int[]){if(true){items[0]++;}}function main(){}",true),
            ("function action(value:int){value=1;}function main(){}",false),
        ] {
            let source=crate::format(source).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();let index=Index::new(&program).unwrap();
            let declaration=program.functions.iter().find(|function|function.name.text=="action").unwrap().name.span;
            assert_eq!(index.aggregate_writes(declaration),writes,"{source}");
        }
    }
}
