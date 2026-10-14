//! Lexically resolved source references used only to order body checking.
//! This pass never rewrites syntax, evaluates arguments, or infers access modes.
use super::*;

pub(super) fn references(function: &Function, names: &HashMap<String, usize>) -> Vec<usize> {
    enum Work<'a> {
        Body(&'a [Stmt], Vec<&'a str>),
        Statement(&'a Stmt),
        Expression(&'a Expr),
        Leave(Vec<&'a str>),
    }
    let mut bound = HashMap::<&str, usize>::new();
    for (name, _, _) in &function.params {
        *bound.entry(&name.text).or_default() += 1;
    }
    let mut pending = vec![Work::Body(&function.body, Vec::new())];
    for initializer in function.parameter_initializers.iter().rev() {
        if let ParameterInitializer::Default { value, .. } = initializer {
            pending.push(Work::Expression(value));
        }
    }
    let mut references = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Body(body, mut locals) => {
                locals.extend(body.iter().filter_map(|statement| match &statement.kind {
                    StmtKind::Let { name, .. } => Some(name.text.as_str()),
                    _ => None,
                }));
                for &name in &locals { *bound.entry(name).or_default() += 1; }
                pending.push(Work::Leave(locals));
                pending.extend(body.iter().rev().map(Work::Statement));
            }
            Work::Leave(locals) => {
                for name in locals {
                    let count = bound.get_mut(name).expect("entered source scope");
                    *count -= 1;
                    if *count == 0 { bound.remove(name); }
                }
            }
            Work::Statement(statement) => match &statement.kind {
                StmtKind::LocalFunction(_) => unreachable!("local declarations are resolved before analysis"),
                StmtKind::Try(region) => {
                    if let Some(body) = &region.finally { pending.push(Work::Body(body, Vec::new())); }
                    if let Some(handler) = &region.catch {
                        pending.push(Work::Body(&handler.body, handler.name.iter().map(|name| name.text.as_str()).collect()));
                    }
                    pending.push(Work::Body(&region.body, Vec::new()));
                }
                StmtKind::Let { value, .. } | StmtKind::Throw(value)
                | StmtKind::Expr(value) | StmtKind::Return(Some(value)) => pending.push(Work::Expression(value)),
                StmtKind::Block(body) => pending.push(Work::Body(body, Vec::new())),
                StmtKind::If { condition, then_body, else_body } => {
                    pending.push(Work::Body(else_body, Vec::new()));
                    pending.push(Work::Body(then_body, Vec::new()));
                    pending.push(Work::Expression(condition));
                }
                StmtKind::While { condition, body } => {
                    pending.push(Work::Body(body, Vec::new()));
                    pending.push(Work::Expression(condition));
                }
                StmtKind::Loop { condition, body, update, .. } => {
                    pending.push(Work::Body(update, Vec::new()));
                    pending.push(Work::Body(body, Vec::new()));
                    if let Some(condition) = condition { pending.push(Work::Expression(condition)); }
                }
                StmtKind::ForOf { name, iterable, body, .. } => {
                    pending.push(Work::Body(body, vec![&name.text]));
                    pending.push(Work::Expression(iterable));
                }
                StmtKind::Assign { target, value, .. } => {
                    pending.push(Work::Expression(value));
                    pending.push(Work::Expression(target));
                }
                StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
            },
            Work::Expression(expression) => match &expression.kind {
                ExprKind::Name(name) => {
                    if !bound.contains_key(name.as_str()) && let Some(&id) = names.get(name) {
                        references.push(id);
                    }
                }
                ExprKind::Match { body, .. } => pending.push(Work::Body(body, Vec::new())),
                    ExprKind::Closure { params, body, .. } => {
                    let locals = params.iter().map(|parameter| parameter.name.text.as_str()).collect::<Vec<_>>();
                    // Defaults have the callable parameter scope, but not body locals.
                    for &name in &locals { *bound.entry(name).or_default() += 1; }
                    pending.push(Work::Leave(locals));
                    pending.push(Work::Body(body, Vec::new()));
                    for parameter in params.iter().rev() {
                        if let ParameterInitializer::Default { value, .. } = &parameter.initializer {
                            pending.push(Work::Expression(value));
                        }
                    }
                }
                ExprKind::SpreadCall(plan) => pending.extend(plan.expressions().map(Work::Expression)),
                ExprKind::OptionalChain(value) | ExprKind::Unary { value, .. }
                | ExprKind::Update { target: value, .. } | ExprKind::Field(value, _)
                | ExprKind::Inject { value, .. } | ExprKind::Project { value, .. }
                | ExprKind::TypeAssertion { value, .. } => pending.push(Work::Expression(value)),
                ExprKind::Optional { base: left, body: right, .. }
                | ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right)
                | ExprKind::Assign { target: left, value: right, .. }
                | ExprKind::CompoundAssign { target: left, value: right, .. }
                | ExprKind::CoalesceAssign { target: left, value: right, .. }
                | ExprKind::LogicalAssign { target: left, value: right, .. } => {
                    pending.push(Work::Expression(right));
                    pending.push(Work::Expression(left));
                }
                ExprKind::Conditional { predicate, consequent, alternate } => {
                    pending.push(Work::Expression(alternate));
                    pending.push(Work::Expression(consequent));
                    pending.push(Work::Expression(predicate));
                }
                ExprKind::Record(fields) => pending.extend(fields.iter().rev().map(|(_, value)| Work::Expression(value))),
                ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::Template(values)
                | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _)
                | ExprKind::NativeConstructor(values, _) | ExprKind::Sequence(values) => {
                    pending.extend(values.iter().rev().map(Work::Expression));
                }
                ExprKind::Call(callee, arguments) => {
                    pending.extend(arguments.iter().rev().map(Work::Expression));
                    pending.push(Work::Expression(callee));
                }
                ExprKind::SpreadValue(_) | ExprKind::Selected { .. }
                | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::Number(_)
                | ExprKind::EnumMember { .. } | ExprKind::Boolean(_) | ExprKind::Null
                | ExprKind::Undefined => {}
            },
        }
    }
    let mut seen = HashSet::new();
    references.retain(|id| seen.insert(*id));
    references
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_dependencies_respect_parameters_blocks_closures_loops_and_handlers() {
        let source = crate::format(r#"
function target(): int { return 1; }
function other(): int { return 2; }
function outer(target: int) {
    { const other = 3; console.log(other); }
    const action = other;
    const closure = (other: int): int => other;
    for (const other of [1]) { console.log(other); }
    try { throw 1; } catch (other) { console.log(other); }
    console.log(other());
}
function main() {}
"#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let names = program.functions.iter().enumerate().map(|(id, function)| (function.name.text.clone(), id)).collect::<HashMap<_, _>>();
        let outer = &program.functions[names["outer"]];
        assert_eq!(references(outer, &names), [names["other"]]);
    }

    #[test]
    fn defaults_and_closure_bodies_keep_function_dependencies() {
        let source = crate::format("function target():int{return 1;}function outer(value:int=target()){const action=():int=>target();}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let names = program.functions.iter().enumerate().map(|(id, function)| (function.name.text.clone(), id)).collect::<HashMap<_, _>>();
        assert_eq!(references(&program.functions[names["outer"]], &names), [names["target"]]);
    }
}
