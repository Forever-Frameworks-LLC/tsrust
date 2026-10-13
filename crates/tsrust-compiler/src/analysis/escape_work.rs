//! Escape discovery uses lexical rollback and heap continuations across bodies/defaults.
use super::*;
type Names = HashMap<String, usize>;
type Memo = HashMap<*const Expr, Option<usize>>;

fn origin(root: &Expr, names: &Names, memo: &mut Memo) -> Option<usize> {
    enum Work<'a> { Visit(&'a Expr), Publish(&'a Expr), Fallback(&'a Expr), Alternate(&'a Expr), Compare(usize) }
    let mut work = vec![Work::Visit(root)];
    let mut result = None;
    while let Some(next) = work.pop() {
        match next {
            Work::Publish(expr) => { memo.insert(expr as *const Expr, result); },
            Work::Fallback(right) => if result.is_none() { work.push(Work::Visit(right)); },
            Work::Alternate(right) => if let Some(left) = result { work.push(Work::Compare(left)); work.push(Work::Visit(right)); },
            Work::Compare(left) => result = (result == Some(left)).then_some(left),
            Work::Visit(expr) => {
                if let Some(found) = memo.get(&(expr as *const Expr)) { result = *found; continue; }
                work.push(Work::Publish(expr));
                match &expr.kind {
                    ExprKind::TypeAssertion { value, .. }
                    | ExprKind::Unary { op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull, value, .. } => work.push(Work::Visit(value)),
                    ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, left, right, .. } => { work.push(Work::Fallback(right)); work.push(Work::Visit(left)); },
                    ExprKind::Conditional { consequent, alternate, .. } => { work.push(Work::Alternate(alternate)); work.push(Work::Visit(consequent)); },
                    ExprKind::Closure { id, .. } => result = Some(*id),
                    ExprKind::Name(name) => result = names.get(name).copied(),
                    _ => result = None,
                }
            },
        }
    }
    result
}

fn mark(root: &Expr, names: &Names, escaping: &mut HashSet<usize>) {
    let mut work = vec![root];
    // This memo lasts only while the lexical map is immutable.
    let mut memo = Memo::new();
    while let Some(expr) = work.pop() {
        if let Some(id) = origin(expr, names, &mut memo) { escaping.insert(id); }
        match &expr.kind {
            ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, left, right, .. } => { work.push(right); work.push(left); },
            ExprKind::Unary { op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull, value, .. } => work.push(value),
            ExprKind::Conditional { consequent, alternate, .. } => { work.push(alternate); work.push(consequent); },
            _ => {},
        }
    }
}

enum Work<'a> {
    Body(&'a [Stmt], Option<&'a str>), Statement(&'a Stmt), After(&'a Stmt),
    Expression(&'a Expr), Parameter(&'a CallableParameter), Bind(&'a str, Option<&'a Expr>), Restore(usize),
}

fn bind(names: &mut Names, undo: &mut Vec<(String, Option<usize>)>, name: &str, value: Option<usize>) {
    let prior = names.remove(name);
    undo.push((name.to_owned(), prior));
    if let Some(id) = value { names.insert(name.to_owned(), id); }
}

fn walk(root: Work<'_>, outer: &Names, escaping: &mut HashSet<usize>) {
    let mut names = outer.clone();
    let mut undo = Vec::new();
    let mut work = vec![root];
    while let Some(next) = work.pop() {
        match next {
            Work::Restore(mark) => while undo.len() > mark {
                let (name, value) = undo.pop().unwrap();
                names.remove(&name);
                if let Some(id) = value { names.insert(name, id); }
            },
            Work::Body(body, shadow) => {
                work.push(Work::Restore(undo.len()));
                if let Some(name) = shadow { bind(&mut names, &mut undo, name, None); }
                work.extend(body.iter().rev().map(Work::Statement));
            },
            Work::Bind(name, value) => {
                let value = value.and_then(|value| origin(value, &names, &mut Memo::new()));
                bind(&mut names, &mut undo, name, value);
            },
            Work::Parameter(parameter) => {
                let value = match &parameter.initializer { ParameterInitializer::Default { value, .. } => Some(&**value), _ => None };
                work.push(Work::Bind(&parameter.name.text, value));
                if let Some(value) = value { work.push(Work::Expression(value)); }
            },
            Work::Statement(stmt) => {
                work.push(Work::After(stmt));
                match &stmt.kind {
                    StmtKind::Let { value, .. } | StmtKind::Expr(value) | StmtKind::Throw(value) | StmtKind::Return(Some(value)) => work.push(Work::Expression(value)),
                    StmtKind::Assign { target, value, .. } => { work.push(Work::Expression(value)); work.push(Work::Expression(target)); },
                    StmtKind::If { condition, .. } | StmtKind::While { condition, .. } => work.push(Work::Expression(condition)),
                    StmtKind::ForOf { iterable, .. } => work.push(Work::Expression(iterable)),
                    StmtKind::Loop { condition: Some(condition), .. } => work.push(Work::Expression(condition)),
                    _ => {},
                }
            },
            Work::After(stmt) => match &stmt.kind {
                StmtKind::Let { name, annotation, value, .. } => {
                    if annotation.as_ref().and_then(crate::callable_annotations::ownership).is_some_and(CallbackKind::is_owned) { mark(value, &names, escaping); }
                    work.push(Work::Bind(&name.text, Some(value)));
                },
                StmtKind::Return(Some(value)) | StmtKind::Throw(value) => mark(value, &names, escaping),
                StmtKind::Try(region) => {
                    if let Some(body) = &region.finally { work.push(Work::Body(body, None)); }
                    if let Some(handler) = &region.catch { work.push(Work::Body(&handler.body, handler.name.as_ref().map(|name| name.text.as_str()))); }
                    work.push(Work::Body(&region.body, None));
                },
                StmtKind::While { body, .. } | StmtKind::Block(body) => work.push(Work::Body(body, None)),
                StmtKind::Loop { body, update, .. } => { work.push(Work::Body(update, None)); work.push(Work::Body(body, None)); },
                StmtKind::ForOf { name, body, .. } => work.push(Work::Body(body, Some(&name.text))),
                StmtKind::If { then_body, else_body, .. } => { work.push(Work::Body(else_body, None)); work.push(Work::Body(then_body, None)); },
                _ => {},
            },
            Work::Expression(expr) => match &expr.kind {
                ExprKind::Closure { params, body, .. } => {
                    work.push(Work::Restore(undo.len()));
                    work.push(Work::Body(body, None));
                    work.extend(params.iter().rev().map(Work::Parameter));
                },
                ExprKind::Call(callee, args) => { work.extend(args.iter().rev().map(Work::Expression)); work.push(Work::Expression(callee)); },
                ExprKind::Project { value, .. } | ExprKind::Inject { value, .. } | ExprKind::Unary { value, .. } | ExprKind::Update { target: value, .. } | ExprKind::Field(value, _) => work.push(Work::Expression(value)),
                ExprKind::Conditional { predicate, consequent, alternate } => { work.push(Work::Expression(alternate)); work.push(Work::Expression(consequent)); work.push(Work::Expression(predicate)); },
                ExprKind::Binary { left, right, .. } | ExprKind::CompoundAssign { target: left, value: right, .. } | ExprKind::Assign { target: left, value: right, .. } | ExprKind::CoalesceAssign { target: left, value: right, .. } | ExprKind::LogicalAssign { target: left, value: right, .. } | ExprKind::Index(left, right) => { work.push(Work::Expression(right)); work.push(Work::Expression(left)); },
                ExprKind::Array(values) | ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Template(values) | ExprKind::Sequence(values) => work.extend(values.iter().rev().map(Work::Expression)),
                ExprKind::Record(fields) => work.extend(fields.iter().rev().map(|(_, value)| Work::Expression(value))),
                _ => {},
            },
        }
    }
}

impl Analyzer {
    pub(super) fn escape_origins(body: &[Stmt], outer: &Names, escaping: &mut HashSet<usize>) { walk(Work::Body(body, None), outer, escaping); }
    pub(super) fn parameter_escape_origins(name: &str, initializer: &ParameterInitializer, names: &mut Names, escaping: &mut HashSet<usize>) {
        let value = if let ParameterInitializer::Default { value, .. } = initializer {
            walk(Work::Expression(value), names, escaping);
            origin(value, names, &mut Memo::new())
        } else { None };
        names.remove(name);
        if let Some(id) = value { names.insert(name.to_owned(), id); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deep_closure_bodies_and_origin_wrappers_use_heap_work() {
        let depth = 16_384;
        let source = format!("function main(){{const value={}()=>1{};}}", "()=>{return ".repeat(depth), ";}".repeat(depth));
        let program = crate::syntax::parse_unbounded(&source).unwrap();
        let mut escaping = HashSet::new();
        Analyzer::escape_origins(&program.functions[0].body, &Names::new(), &mut escaping);
        assert_eq!(escaping.len(), depth);
        assert!(!escaping.contains(&0));
        assert!((1..=depth).all(|id| escaping.contains(&id)));
        let span = Span::new(0, 1);
        let mut value = Expr::new(ExprKind::Name("origin".into()), span);
        for _ in 0..depth {
            value = Expr::new(ExprKind::Binary { op: BinaryOp::Coalesce, operator: span, left: Box::new(Expr::new(ExprKind::Null, span)), right: Box::new(value) }, span);
        }
        escaping.clear();
        mark(&value, &Names::from([("origin".into(), 77)]), &mut escaping);
        assert_eq!(escaping, HashSet::from([77]));
        drop(value);
        drop(program);
    }

    #[test]
    fn lexical_rollback_preserves_defaults_shadowing_and_sibling_origins() {
        let source = "function main(){const outer=()=>1;const other=()=>2;{const outer=other;return outer;}for(const outer of [1]){return outer;}try{}catch(outer){return outer;}const f=(outer:int=outer,other:int=outer)=>{return other;};return outer;}";
        let program = crate::syntax::parse_unbounded(source).unwrap();
        let mut escaping = HashSet::new();
        Analyzer::escape_origins(&program.functions[0].body, &Names::new(), &mut escaping);
        assert_eq!(escaping, HashSet::from([0, 1]));
        // Query caches never escape a lexical-map version.
        let value = Expr::new(ExprKind::Name("outer".into()), Span::new(0, 5));
        assert_eq!(origin(&value, &Names::from([("outer".into(), 7)]), &mut Memo::new()), Some(7));
        assert_eq!(origin(&value, &Names::from([("outer".into(), 9)]), &mut Memo::new()), Some(9));
    }
}
