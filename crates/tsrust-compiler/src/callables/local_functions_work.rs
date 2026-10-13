//! Lexical resolution keeps continuations and scope rollback on the heap.
use super::*;

type Undo = Vec<(String, Option<Symbol>)>;
fn bind(env: &mut Environment, undo: &mut Undo, name: String, symbol: Symbol) {
    let prior = env.insert(name.clone(), symbol);
    undo.push((name, prior));
}
#[derive(Clone, Copy)]
struct Context { owner: usize, pending: bool }
enum Work<'a> {
    Function(&'a mut Function, Context),
    Body(&'a mut [Stmt], Context, bool),
    Statement(&'a mut Stmt, Context),
    Expression(&'a mut Expr, Context),
    Bind(&'a Token, Context),
    Restore(usize),
    Lift(String),
}

impl Resolver {
    pub(super) fn function(&mut self, function: &mut Function, owner: usize) -> Result<(), Diagnostic> {
        let mut env = Environment::new();
        let mut undo = Undo::new();
        let mut work = vec![Work::Function(function, Context { owner, pending: false })];
        while let Some(next) = work.pop() {
            match next {
                Work::Restore(mark) => while undo.len() > mark {
                    let (name, prior) = undo.pop().unwrap();
                    if let Some(symbol) = prior { env.insert(name, symbol); } else { env.remove(&name); }
                },
                Work::Lift(name) => self.lift_order.push(name),
                Work::Bind(name, context) => bind(&mut env, &mut undo, name.text.clone(), Symbol::Variable { owner: context.owner, span: name.span }),
                Work::Function(function, context) => {
                    work.push(Work::Restore(undo.len()));
                    work.push(Work::Body(&mut function.body, context, true));
                    for ((name, _, _), initializer) in function.params.iter().zip(&mut function.parameter_initializers).rev() {
                        work.push(Work::Bind(name, context));
                        if let ParameterInitializer::Default { value, .. } = initializer { work.push(Work::Expression(value, context)); }
                    }
                },
                Work::Body(body, context, function_body) => {
                    work.push(Work::Restore(undo.len()));
                    self.declare(body, &mut env, &mut undo, context.owner, function_body)?;
                    work.extend(body.iter_mut().rev().map(|stmt| Work::Statement(stmt, context)));
                },
                Work::Statement(statement, context) => match &mut statement.kind {
                    StmtKind::LocalFunction(function) => {
                        let symbol = env[&function.name.text].clone();
                        let Symbol::Function { emitted, id } = &symbol else { unreachable!() };
                        let local_owner = self.declarations[*id].function_owner;
                        let written = written_name(&function.name.text).to_owned();
                        work.push(Work::Restore(undo.len()));
                        if written != function.name.text { bind(&mut env, &mut undo, written, symbol.clone()); }
                        function.name.text = emitted.clone();
                        work.push(Work::Lift(emitted.clone()));
                        work.push(Work::Function(function, Context { owner: local_owner, pending: true }));
                    },
                    StmtKind::Let { value, .. } | StmtKind::Throw(value) | StmtKind::Expr(value) | StmtKind::Return(Some(value)) => work.push(Work::Expression(value, context)),
                    StmtKind::Assign { target, value, .. } => { work.push(Work::Expression(value, context)); work.push(Work::Expression(target, context)); },
                    StmtKind::Block(body) => work.push(Work::Body(body, context, false)),
                    StmtKind::If { condition, then_body, else_body } => {
                        work.push(Work::Body(else_body, context, false)); work.push(Work::Body(then_body, context, false)); work.push(Work::Expression(condition, context));
                    },
                    StmtKind::While { condition, body } => { work.push(Work::Body(body, context, false)); work.push(Work::Expression(condition, context)); },
                    StmtKind::Loop { condition, body, update, .. } => {
                        work.push(Work::Body(update, context, false)); work.push(Work::Body(body, context, false));
                        if let Some(condition) = condition { work.push(Work::Expression(condition, context)); }
                    },
                    StmtKind::ForOf { name, iterable, body, .. } => {
                        work.push(Work::Restore(undo.len()));
                        work.push(Work::Body(body, context, false)); work.push(Work::Bind(name, context)); work.push(Work::Expression(iterable, context));
                    },
                    StmtKind::Try(region) => {
                        if let Some(finally) = &mut region.finally { work.push(Work::Body(finally, context, false)); }
                        if let Some(catch) = &mut region.catch {
                            work.push(Work::Restore(undo.len()));
                            work.push(Work::Body(&mut catch.body, context, false));
                            if let Some(name) = &catch.name { work.push(Work::Bind(name, context)); }
                        }
                        work.push(Work::Body(&mut region.body, context, false));
                    },
                    StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {},
                },
                Work::Expression(expression, context) => {
                    let Context { owner, pending } = context;
                    match &mut expression.kind {
            ExprKind::Name(name) => match env.get(name) {
                Some(Symbol::Function { emitted, id }) => {
                    if let Some(&source) = self.local_owners.get(&owner) {
                        self.capture_plan.dependency(source, *id);
                    }
                    // The value form is chosen once every capture is known.
                    *name = format!("{emitted}{}", crate::local_function_values::VALUE_MARK);
                    self.value_uses = true;
                }
                Some(Symbol::Variable {
                    owner: defining,
                    span,
                }) => {
                    let binding = crate::local_capture_plan::BindingId {
                        owner: *defining,
                        declaration_start: span.start,
                    };
                    self.capture_plan.variable_use(expression.span, binding);
                    if pending && *defining != owner {
                        let function = self.local_owners[&owner];
                        self.capture_plan.capture(
                            function,
                            crate::local_capture_plan::Capture {
                                binding,
                                name: name.clone(),
                                declaration: *span,
                                usage: expression.span,
                            },
                        );
                    }
                }
                _ => {}
            },
                        ExprKind::Closure { params, body, .. } => {
                            work.push(Work::Restore(undo.len()));
                            work.push(Work::Body(body, context, true));
                            for parameter in params.iter_mut().rev() {
                                work.push(Work::Bind(&parameter.name, context));
                                if let ParameterInitializer::Default { value, .. } = &mut parameter.initializer { work.push(Work::Expression(value, context)); }
                            }
                        },
                        ExprKind::Conditional { predicate, consequent, alternate } => {
                            work.push(Work::Expression(alternate, context)); work.push(Work::Expression(consequent, context)); work.push(Work::Expression(predicate, context));
                        },
                        ExprKind::Optional { base: left, body: right, .. } | ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right)
                        | ExprKind::CompoundAssign { target: left, value: right, .. } | ExprKind::Assign { target: left, value: right, .. }
                        | ExprKind::CoalesceAssign { target: left, value: right, .. } | ExprKind::LogicalAssign { target: left, value: right, .. } => {
                            work.push(Work::Expression(right, context)); work.push(Work::Expression(left, context));
                        },
                        ExprKind::OptionalChain(value) | ExprKind::Unary { value, .. } | ExprKind::Update { target: value, .. }
                        | ExprKind::Field(value, _) | ExprKind::Inject { value, .. } | ExprKind::Project { value, .. }
                        | ExprKind::TypeAssertion { value, .. } => work.push(Work::Expression(value, context)),
                        ExprKind::Record(fields) => work.extend(fields.iter_mut().rev().map(|(_, value)| Work::Expression(value, context))),
                        ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _)
                        | ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values) => work.extend(values.iter_mut().rev().map(|value| Work::Expression(value, context))),
                        ExprKind::Call(callee, arguments) => {
                            let called = if let ExprKind::Name(name) = &callee.kind { env.get(name).cloned() } else { None };
                            work.extend(arguments.iter_mut().rev().map(|value| Work::Expression(value, context)));
                            if let Some(Symbol::Function { emitted, id }) = called {
                                if let Some(&source) = self.local_owners.get(&owner) { self.capture_plan.dependency(source, id); }
                                callee.kind = ExprKind::Name(emitted);
                            } else { work.push(Work::Expression(callee, context)); }
                        },
                        ExprKind::SpreadCall(_) | ExprKind::SpreadValue(_) | ExprKind::Selected { .. } => unreachable!("local resolution precedes checked spread lowering"),
                        ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::EnumMember { .. } | ExprKind::Number(_) | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined => {},
                    }
                },
            }
        }
        debug_assert!(env.is_empty() && undo.is_empty());
        Ok(())
    }

    fn declare(&mut self, body: &[Stmt], env: &mut Environment, undo: &mut Undo, owner: usize, function_body: bool) -> Result<(), Diagnostic> {
        let scope = self.next_scope;
        self.next_scope += 1;
        let mut names: HashMap<String, Span> = HashMap::new();
        for statement in body.iter() {
            let name = match &statement.kind {
                StmtKind::LocalFunction(function) => Some(&function.name),
                StmtKind::Let { name, .. } => Some(name),
                _ => None,
            };
            if let Some(name) = name {
                if let Some(previous) = names.insert(name.text.clone(), name.span) {
                    return Err(Diagnostic::new(
                        "duplicate",
                        format!("Name {:?} is already declared in this block.", name.text),
                        name.span,
                    )
                    .related(previous, "Previous declaration here."));
                }
                if function_body
                    && matches!(&statement.kind, StmtKind::LocalFunction(_))
                    && matches!(env.get(&name.text), Some(Symbol::Variable {owner: previous, ..}) if *previous == owner)
                {
                    return Err(Diagnostic::new(
                        "duplicate",
                        "A local function cannot redeclare a parameter.",
                        name.span,
                    ));
                }
                match &statement.kind {
                    StmtKind::LocalFunction(function) => {
                        let id = self.capture_plan.declaration();
                        debug_assert_eq!(id, self.declarations.len());
                        let function_owner = self.next_owner;
                        self.next_owner += 1;
                        self.local_owners.insert(function_owner, id);
                        let emitted = format!("local#{id}#{}", written_name(&name.text));
                        self.capture_plan.owner(function_owner, emitted.clone());
                        // Only calls and value uses create runtime dependencies.
                        // Unused declaration bodies obtain their types separately
                        // from completed lexical contexts.
                        self.declarations.push(Declaration {
                            id,
                            scope,
                            function_owner,
                            enclosing_owner: owner,
                            statement_span: statement.span,
                            source_name: Token { text: written_name(&name.text).to_owned(), ..name.clone() },
                            emitted_name: emitted.clone(),
                            function_scope: function.scope_span,
                        });
                        bind(env, undo, name.text.clone(), Symbol::Function { emitted, id });
                    }
                    _ => {
                        bind(env, undo,
                            name.text.clone(),
                            Symbol::Variable {
                                owner,
                                span: name.span,
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Resolution records DFS completion order independently of storage. Keeping
    /// declarations in their tree during resolution avoids self-referential work
    /// frames; extraction then flattens them without recursion or tree cloning.
    pub(super) fn collect_lifted(&mut self, functions: &mut [Function]) -> Result<(), Diagnostic> {
        fn body(root: &mut [Stmt], pending: &mut Vec<Box<Function>>) {
            let mut work: Vec<&mut Stmt> = root.iter_mut().collect();
            while let Some(stmt) = work.pop() {
                if matches!(stmt.kind, StmtKind::LocalFunction(_)) {
                    let StmtKind::LocalFunction(function) = std::mem::replace(&mut stmt.kind, StmtKind::Block(Vec::new())) else { unreachable!() };
                    pending.push(function);
                    continue;
                }
                match &mut stmt.kind {
                    StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => work.extend(body.iter_mut()),
                    StmtKind::If { then_body, else_body, .. } => { work.extend(then_body.iter_mut()); work.extend(else_body.iter_mut()); },
                    StmtKind::Loop { body, update, .. } => { work.extend(body.iter_mut()); work.extend(update.iter_mut()); },
                    StmtKind::Try(region) => {
                        work.extend(region.body.iter_mut());
                        if let Some(catch) = &mut region.catch { work.extend(catch.body.iter_mut()); }
                        if let Some(finally) = &mut region.finally { work.extend(finally.iter_mut()); }
                    },
                    _ => {},
                }
            }
        }
        fn extract(function: &mut Function, pending: &mut Vec<Box<Function>>) -> Result<(), Diagnostic> {
            body(&mut function.body, pending);
            crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Closure { body: nested, .. } = &mut expression.kind { body(nested, pending); }
                Ok(())
            })
        }
        let mut pending = Vec::new();
        for function in functions { extract(function, &mut pending)?; }
        let mut extracted = HashMap::new();
        while let Some(mut function) = pending.pop() {
            extract(&mut function, &mut pending)?;
            let prior = extracted.insert(function.name.text.clone(), *function);
            debug_assert!(prior.is_none());
        }
        self.lifted.extend(self.lift_order.drain(..).map(|name| extracted.remove(&name).expect("resolved local function")));
        debug_assert!(extracted.is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_named_functions_and_mixed_closure_scopes_resolve_and_drop_on_normal_stack() {
        for (prefix, suffix) in [("function nested(){", "}"), ("{const value=()=>{function nested(){", "}};}")] {
            let depth = 16_384;
            let source = format!("function main(){{{}return 1;{}}}", prefix.repeat(depth), suffix.repeat(depth));
            let mut program = crate::syntax::parse_unbounded(&source).unwrap();
            super::super::resolve(&mut program).unwrap();
            assert_eq!(program.local_declarations.len(), depth);
            assert_eq!(program.functions.len(), depth + 1);
            for (position, function) in program.functions[1..].iter().enumerate() {
                assert_eq!(function.name.text, format!("local#{}#nested", depth - position - 1));
            }
            assert!(program.unresolved_local_function.is_none());
            drop(program);
        }
    }

    #[test]
    fn defaults_and_private_names_keep_lexical_identity_and_postorder_lifting() {
        let source = "function main(){const outer=1;function f(x:int=(()=>{function d(){return outer;}return d();})()){function g(){return x;}return g();}{function f(){return 2;}f();}const value=function privateName(){return privateName();};f();}";
        let mut program = crate::syntax::parse_unbounded(source).unwrap();
        super::super::resolve(&mut program).unwrap();
        let order = program.functions.iter().skip(1).map(|f| f.name.text.as_str()).collect::<Vec<_>>();
        assert_eq!(order, ["local#2#d", "local#3#g", "local#0#f", "local#4#f", "local#1#privateName"]);
        let captures = &program.local_capture_plans[0];
        assert_eq!(captures.direct[2][0].name, "outer");
        assert_eq!(captures.direct[3][0].name, "x");
        assert!(captures.direct[4].is_empty());
        // Private expression names bind recursively only inside their function.
        let private = program.functions.last().unwrap();
        let StmtKind::Return(Some(value)) = &private.body[0].kind else { panic!("private return") };
        let ExprKind::Call(callee, _) = &value.kind else { panic!("private call") };
        assert!(matches!(&callee.kind, ExprKind::Name(name) if name == "local#1#privateName"));
    }
}
