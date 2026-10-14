//! Adapter checking owns hidden-name and capture cleanup across suspension.
use super::*;
use super::super::adapters::{Request, Cleanup};
pub(super) struct AdapterTask<'a> {
    request: Option<Request>, cleanup: Option<Cleanup>, original: Option<Box<Expr>>,
    pub(super) context: ContextId, loans: ActiveLoans<'a>,
}
impl<'a> AdapterTask<'a> {
    pub(super) fn new(original: Box<Expr>, request: Request, context: ContextId, loans: ActiveLoans<'a>) -> Self {
        let original = request.restore_on_error.then_some(original);
        Self { request: Some(request), cleanup: None, original, context, loans }
    }
}
pub(super) fn step<'a>(analyzer: &mut Analyzer, context: &mut Context, mut task: Box<AdapterTask<'a>>, completion: Option<Completion<'a>>) -> Step<'a> {
    if let Some(request) = task.request.take() {
        task.cleanup = Some(request.cleanup);
        let child = Task::Expression(ExpressionTask::new(Expression::Child(request.value), task.context, request.mode, task.loans.clone(), request.expected));
        Step::Push(Task::Adapter(task), child)
    } else {
        let Completion::Expression(value, checked) = completion.expect("adapter expression") else { unreachable!("adapter completion") };
        let mut value = value.into_child();
        let checked = analyzer.finish_adapter(context, &mut value, task.cleanup.take().expect("adapter cleanup"), checked);
        if checked.is_err() && let Some(original) = task.original.take() { value = original; }
        Step::Complete(Completion::Expression(Expression::Child(value), checked))
    }
}
pub(super) fn schedule<'a>(mut task: ExpressionTask<'a>, request: Request) -> Step<'a> {
    let expr = task.expression.value();
    let span = expr.span;
    let original = Box::new(std::mem::replace(expr, Expr::new(ExprKind::Undefined, span)));
    let child = Task::Adapter(Box::new(AdapterTask::new(original, request, task.context, task.loans.clone())));
    task.state = ExpressionState::Adapter;
    Step::Push(Task::Expression(task), child)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(source: &str) -> crate::syntax::Program {
        let mut program = crate::syntax::parse_unbounded(source).unwrap();
        crate::callables::module_state::lower(&mut program).unwrap();
        crate::local_functions::resolve(&mut program).unwrap();
        program
    }
    #[test]
    fn fresh_factory_adapter_wrappers_share_checker_stack() {
        let count = 128;
        let source = format!("function make(seed:int){{function identity<T>(value:T):T{{if(seed===0){{return value;}}return value;}}return identity;}}function main(){{console.log({}7{});}}", "[7].map(make(".repeat(count), "))[0]".repeat(count));
        let checked = crate::analysis::analyze_recursive(parse(&source)).unwrap();
        assert!(checked.closures.len() >= count);
        drop(checked);
    }
    #[test]
    fn failed_named_adapter_restores_hidden_binding_escape_and_source() {
        let mut analyzer = Analyzer::new(crate::syntax::parse("function fail<T>(value:T):T{return true;}function use(f:Function<int,[value:int]>):void{}function main(){}").unwrap()).unwrap();
        let mut context = Context::new("main".into());
        let span = Span::new(400, 406);
        let ty = analyzer.deferred_captured_generic_value(&mut context, "fail", 1, span, &Loans::new()).unwrap();
        let token = Token { text: "action".into(), kind: TokenKind::Name, span };
        let binding = Analyzer::bind(&mut context, &token, ty, false).unwrap();
        let id = usize::MAX - span.start;
        let hidden = format!("#generic-adapter-{id}");
        context.names.insert(hidden, binding);
        let before = context.names.clone();
        let expected = analyzer.signatures[analyzer.function_names["use"]].params[0];
        let mut value = Expr::new(ExprKind::Name("action".into()), span);
        let error = analyzer.expression(&mut context, &mut value, Mode::Consume, &Loans::new(), Some(expected)).expect_err("wrong generic body result");
        assert_eq!(error.code, "type");
        assert_eq!(context.names, before);
        assert!(!analyzer.escaping.contains(&id));
        assert!(matches!(&value.kind, ExprKind::Name(name) if name == "action"));
    }
}
