//! Lexical named declarations resolve before template, module and ownership passes.
use crate::syntax::*;
use crate::{Diagnostic, Span};
use std::collections::HashMap;

/// Separates the written name of a named function expression from the
/// source position that makes its block-level declaration name unique.
pub(crate) const EXPRESSION_MARK: &str = "#expression";

/// The name as written, without the expression marker.
pub(crate) fn written_name(name: &str) -> &str {
    name.split_once(EXPRESSION_MARK).map_or(name, |(written, _)| written)
}

#[derive(Clone, Debug)]
pub(crate) struct Declaration {
    pub id: usize,
    pub scope: usize,
    pub function_owner: usize,
    pub enclosing_owner: usize,
    pub statement_span: Span,
    pub source_name: Token,
    pub emitted_name: String,
    pub function_scope: Span,
}
#[derive(Clone)]
enum Symbol {
    Function { emitted: String, id: usize },
    Variable { owner: usize, span: Span },
}
type Environment = HashMap<String, Symbol>;
struct Resolver {
    next_owner: usize,
    next_scope: usize,
    declarations: Vec<Declaration>,
    lifted: Vec<Function>,
    capture_plan: crate::local_capture_plan::Planner,
    local_owners: HashMap<usize, usize>,
    /// A local function name occurs outside the callee position of a call.
    value_uses: bool,
}

pub(crate) fn resolve(program: &mut Program) -> Result<(), Diagnostic> {
    if program.unresolved_local_function.is_none() {
        return Ok(());
    }
    let mut resolver = Resolver {
        next_owner: program.functions.len(),
        next_scope: 0,
        declarations: Vec::new(),
        lifted: Vec::new(),
        capture_plan: Default::default(),
        local_owners: HashMap::new(),
        value_uses: false,
    };
    for (owner, function) in program.functions.iter_mut().enumerate() {
        resolver
            .capture_plan
            .owner(owner, function.name.text.clone());
        resolver.function(function, &Environment::new(), owner, false)?;
    }
    let plan = resolver
        .capture_plan
        .has_captures()
        .then(|| resolver.capture_plan.finish());
    if resolver.value_uses {
        let ids: HashMap<&str, usize> = resolver
            .declarations
            .iter()
            .map(|declaration| (declaration.emitted_name.as_str(), declaration.id))
            .collect();
        crate::local_function_values::lower(
            &mut program.functions,
            &mut resolver.lifted,
            &|name| {
                plan.as_ref()
                    .is_some_and(|plan| !plan.requirements(ids[name]).is_empty())
            },
        )?;
    }
    program.local_capture_plans.extend(plan);
    for function in program.functions.iter_mut().chain(resolver.lifted.iter_mut()) {
        remove_lifted_placeholders(&mut function.body)?;
    }
    program.functions.extend(resolver.lifted);
    program.local_declarations = resolver.declarations;
    program.unresolved_local_function = None;
    Ok(())
}
/// A lifted declaration leaves an empty block where it stood. Remove those:
/// a declaration is hoisted, so it may follow a `return` and must not count
/// as an unreachable statement or as the value of the block.
fn remove_lifted_placeholders(body: &mut Vec<Stmt>) -> Result<(), Diagnostic> {
    fn statements(body: &mut Vec<Stmt>) {
        body.retain(|statement| !matches!(&statement.kind, StmtKind::Block(inner) if inner.is_empty()));
        for statement in body {
            match &mut statement.kind {
                StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => statements(body),
                StmtKind::If { then_body, else_body, .. } => {
                    statements(then_body);
                    statements(else_body);
                }
                StmtKind::Loop { body, update, .. } => {
                    statements(body);
                    statements(update);
                }
                StmtKind::Try(region) => {
                    statements(&mut region.body);
                    if let Some(catch) = &mut region.catch {
                        statements(&mut catch.body);
                    }
                    if let Some(finally) = &mut region.finally {
                        statements(finally);
                    }
                }
                _ => {}
            }
        }
    }
    statements(body);
    crate::source_walk::body(body, &mut |_| Ok(()), &mut |expression| {
        if let ExprKind::Closure { body, .. } = &mut expression.kind {
            statements(body);
        }
        Ok(())
    })
}

impl Resolver {
    fn function(
        &mut self,
        function: &mut Function,
        outer: &Environment,
        owner: usize,
        captures_pending: bool,
    ) -> Result<(), Diagnostic> {
        let mut env = outer.clone();
        for ((name, _, _), initializer) in function
            .params
            .iter()
            .zip(&mut function.parameter_initializers)
        {
            self.initializer(initializer, &env, owner, captures_pending)?;
            env.insert(
                name.text.clone(),
                Symbol::Variable {
                    owner,
                    span: name.span,
                },
            );
        }
        self.body(&mut function.body, &env, owner, captures_pending, true)
    }
    #[inline(never)]
    fn body(
        &mut self,
        body: &mut [Stmt],
        outer: &Environment,
        owner: usize,
        captures_pending: bool,
        function_body: bool,
    ) -> Result<(), Diagnostic> {
        let scope = self.next_scope;
        self.next_scope += 1;
        let mut env = outer.clone();
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
                    && matches!(outer.get(&name.text), Some(Symbol::Variable {owner: previous, ..}) if *previous == owner)
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
                        env.insert(name.text.clone(), Symbol::Function { emitted, id });
                    }
                    _ => {
                        env.insert(
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
        for statement in body {
            match &mut statement.kind {
                StmtKind::LocalFunction(_) => {
                    let StmtKind::LocalFunction(mut function) =
                        std::mem::replace(&mut statement.kind, StmtKind::Block(Vec::new()))
                    else {
                        unreachable!()
                    };
                    let symbol = env[&function.name.text].clone();
                    let Symbol::Function { emitted, id } = &symbol else {
                        unreachable!()
                    };
                    // The written name of a function expression is visible in its
                    // own body only.
                    let written = written_name(&function.name.text).to_owned();
                    let private = (written != function.name.text).then(|| {
                        let mut inner = env.clone();
                        inner.insert(written, symbol.clone());
                        inner
                    });
                    function.name.text = emitted.clone();
                    let local_owner = self.declarations[*id].function_owner;
                    self.function(&mut function, private.as_ref().unwrap_or(&env), local_owner, true)?;
                    self.lifted.push(*function);
                }
                StmtKind::Let { value, .. }
                | StmtKind::Throw(value)
                | StmtKind::Expr(value)
                | StmtKind::Return(Some(value)) => {
                    self.expression(value, &env, owner, captures_pending)?
                }
                StmtKind::Assign { target, value, .. } => {
                    self.expression(target, &env, owner, captures_pending)?;
                    self.expression(value, &env, owner, captures_pending)?;
                }
                StmtKind::Block(body) => self.body(body, &env, owner, captures_pending, false)?,
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    self.expression(condition, &env, owner, captures_pending)?;
                    self.body(then_body, &env, owner, captures_pending, false)?;
                    self.body(else_body, &env, owner, captures_pending, false)?;
                }
                StmtKind::While { condition, body } => {
                    self.expression(condition, &env, owner, captures_pending)?;
                    self.body(body, &env, owner, captures_pending, false)?;
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    ..
                } => {
                    if let Some(condition) = condition {
                        self.expression(condition, &env, owner, captures_pending)?;
                    }
                    self.body(body, &env, owner, captures_pending, false)?;
                    self.body(update, &env, owner, captures_pending, false)?;
                }
                StmtKind::ForOf {
                    name,
                    iterable,
                    body,
                    ..
                } => {
                    self.expression(iterable, &env, owner, captures_pending)?;
                    let mut loop_env = env.clone();
                    loop_env.insert(
                        name.text.clone(),
                        Symbol::Variable {
                            owner,
                            span: name.span,
                        },
                    );
                    self.body(body, &loop_env, owner, captures_pending, false)?;
                }
                StmtKind::Try(region) => {
                    self.body(&mut region.body, &env, owner, captures_pending, false)?;
                    if let Some(catch) = &mut region.catch {
                        let mut catch_env = env.clone();
                        if let Some(name) = &catch.name {
                            catch_env.insert(
                                name.text.clone(),
                                Symbol::Variable {
                                    owner,
                                    span: name.span,
                                },
                            );
                        }
                        self.body(&mut catch.body, &catch_env, owner, captures_pending, false)?;
                    }
                    if let Some(finally) = &mut region.finally {
                        self.body(finally, &env, owner, captures_pending, false)?;
                    }
                }
                StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) => {}
            }
        }
        Ok(())
    }
    fn initializer(
        &mut self,
        value: &mut ParameterInitializer,
        env: &Environment,
        owner: usize,
        pending: bool,
    ) -> Result<(), Diagnostic> {
        if let ParameterInitializer::Default { value, .. } = value {
            self.expression(value, env, owner, pending)?;
        }
        Ok(())
    }
    #[inline(never)]
    fn expression(
        &mut self,
        expression: &mut Expr,
        env: &Environment,
        owner: usize,
        pending: bool,
    ) -> Result<(), Diagnostic> {
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
                let mut nested = env.clone();
                for parameter in params {
                    self.initializer(&mut parameter.initializer, &nested, owner, pending)?;
                    nested.insert(
                        parameter.name.text.clone(),
                        Symbol::Variable {
                            owner,
                            span: parameter.name.span,
                        },
                    );
                }
                self.body(body, &nested, owner, pending, true)?;
            }
            ExprKind::Conditional {
                predicate,
                consequent,
                alternate,
            } => {
                self.expression(predicate, env, owner, pending)?;
                self.expression(consequent, env, owner, pending)?;
                self.expression(alternate, env, owner, pending)?;
            }
            ExprKind::Optional {
                base: left,
                body: right,
                ..
            }
            | ExprKind::Binary { left, right, .. }
            | ExprKind::Index(left, right)
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
                self.expression(left, env, owner, pending)?;
                self.expression(right, env, owner, pending)?;
            }
            ExprKind::OptionalChain(value)
            | ExprKind::Unary { value, .. }
            | ExprKind::Update { target: value, .. }
            | ExprKind::Field(value, _)
            | ExprKind::Inject { value, .. }
            | ExprKind::Project { value, .. }
            | ExprKind::TypeAssertion { value, .. } => {
                self.expression(value, env, owner, pending)?
            }
            ExprKind::Record(fields) => {
                for (_, value) in fields {
                    self.expression(value, env, owner, pending)?;
                }
            }
            ExprKind::ArrayOf(values, _)
            | ExprKind::SetConstructor(values, _)
            | ExprKind::MapConstructor(values, _)
            | ExprKind::Array(values)
            | ExprKind::Template(values)
            | ExprKind::Sequence(values) => {
                for value in values {
                    self.expression(value, env, owner, pending)?;
                }
            }
            ExprKind::Call(callee, arguments) => {
                let called = match &callee.kind {
                    ExprKind::Name(name) => match env.get(name) {
                        Some(Symbol::Function { emitted, id }) => Some((emitted.clone(), *id)),
                        _ => None,
                    },
                    _ => None,
                };
                if let Some((emitted, id)) = called {
                    if let Some(&source) = self.local_owners.get(&owner) {
                        self.capture_plan.dependency(source, id);
                    }
                    callee.kind = ExprKind::Name(emitted);
                } else {
                    self.expression(callee, env, owner, pending)?;
                }
                for value in arguments {
                    self.expression(value, env, owner, pending)?;
                }
            }
            ExprKind::SpreadCall(_) | ExprKind::SpreadValue(_) | ExprKind::Selected { .. } => {
                unreachable!("local resolution precedes checked spread lowering")
            }
            ExprKind::String(_)
            | ExprKind::Integer(_)
            | ExprKind::EnumMember { .. }
            | ExprKind::Number(_)
            | ExprKind::Boolean(_)
            | ExprKind::Null
            | ExprKind::Undefined => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn lexical_declaration_identity_survives_checked_projection() {
        let source = "function main(){function value():int{return 1;} {function value():int{return 2;} console.log(value());} console.log(value());}";
        let checked = crate::check(source).unwrap();
        let declarations = &checked.program.local_declarations;
        assert_eq!(declarations.len(), 2);
        assert_ne!(declarations[0].id, declarations[1].id);
        assert_ne!(declarations[0].scope, declarations[1].scope);
        assert_ne!(declarations[0].emitted_name, declarations[1].emitted_name);
        for declaration in declarations {
            assert_eq!(declaration.source_name.text, "value");
            assert_eq!(
                &source[declaration.source_name.span.start..declaration.source_name.span.end],
                "value"
            );
            assert!(
                source[declaration.function_scope.start..declaration.function_scope.end]
                    .starts_with("value():int")
            );
        }
    }
    #[test]
    fn transitive_capture_diagnostic_points_to_the_owner_not_a_sibling_function() {
        let source = "function main(){function forward():int{return read();}const data=[1];function read():int{return data[0];}console.log(forward());}";
        // The capture is planned for its owner; the program checks now that
        // captures lower to hidden arguments.
        let parsed = crate::syntax::parse(source).unwrap();
        let plan = &parsed.local_capture_plans[0];
        assert!(plan.requirements(0).iter().all(|capture| capture.name == "data"));
        assert!(plan.requirements(1).iter().all(|capture| capture.name == "data"));
        assert!(crate::check(source).is_ok());
        // The diagnostic for uses that are still pending names the captured
        // owner binding, not the sibling function.
        let error = plan.pending_diagnostic().expect("captured owner");
        assert_eq!(error.code, "unsupported");
        assert_eq!(&source[error.span.start..error.span.end], "data");
        let binding = error.related[0].0;
        assert_eq!(&source[binding.start..binding.end], "data");
    }
    #[test]
    fn captured_declaration_syntax_reaches_the_checked_context_boundary() {
        let source = "function main(){function read():int{return data[0];}const data=[1];console.log(read());}";
        let parsed = crate::syntax::parse(source).expect("capture syntax must survive parsing");
        assert_eq!(parsed.local_declarations.len(), 1);
        assert_eq!(parsed.local_capture_plans.len(), 1);
        let plan = &parsed.local_capture_plans[0];
        assert_eq!(plan.requirements(0)[0].name, "data");
        let declaration = &parsed.local_declarations[0];
        assert_eq!(declaration.source_name.text, "read");
        assert_eq!(declaration.enclosing_owner, 0);
        assert_ne!(declaration.function_owner, declaration.enclosing_owner);
        assert!(
            source[declaration.statement_span.start..declaration.statement_span.end]
                .starts_with("function read")
        );
        assert!(
            parsed
                .functions
                .iter()
                .any(|function| function.name.text == declaration.emitted_name)
        );
        // The retained plan reaches checking, which lowers the capture.
        assert!(crate::analysis::analyze(parsed).is_ok());
    }
}
