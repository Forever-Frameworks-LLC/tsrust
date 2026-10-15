//! Async declarations reuse concrete environment ownership, with a distinct task contract.
use super::*;
impl Parser<'_> {
    pub(super) fn finish_async_function(&mut self, function: &mut Function) -> Result<(), Diagnostic> {
        let asynchronous = function.name.text.starts_with("async#");
        if asynchronous { function.name.text = function.name.text[6..].into(); }
        if !asynchronous && !self.has_await {return Ok(()); }
        for initializer in &mut function.parameter_initializers {
            if let ParameterInitializer::Default{value,..}=initializer {
                crate::source_walk::expression(value,&mut |expression|{
                    if let ExprKind::Unary{op:UnaryOp::Await,operator,..}=expression.kind {return Err(Diagnostic::new("await-context","Default arguments run at construction; await is not allowed here.",operator));}Ok(())
                })?;
            }
        }
        let mut task_bodies = std::collections::HashSet::new();
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |value| {
            if let ExprKind::Unary { op: UnaryOp::TaskCreate, value, .. } = &value.kind
                && let ExprKind::Closure { id, .. } = value.kind { task_bodies.insert(id); }
            Ok(())
        })?;
        let mut scopes = Vec::new();
        let mut awaits = Vec::new();
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |value| {
            match &value.kind {
                ExprKind::Closure { id, .. } => scopes.push((value.span, task_bodies.contains(id))),
                ExprKind::Unary { op: UnaryOp::Await, operator, .. } => awaits.push(*operator),
                _ => {}
            }
            Ok(())
        })?;
        for at in awaits {
            let allowed = scopes.iter().filter(|(span,_)|span.start<=at.start && span.end>=at.end)
                .min_by_key(|(span,_)|span.end-span.start).map_or(asynchronous,|(_,allowed)|*allowed);
            if !allowed { return Err(Diagnostic::new("await-context", "await is only valid inside an async function.", at)); }
        }
        if !asynchronous { return Ok(()); }
        let Some(ResultAnnotation::Callable { kind: Some(CallbackKind::Task), params, result, .. }) = &function.result else {
            return Err(Diagnostic::new("async-result", "An async function declares Task<T>; its body returns T.", function.name.span));
        };
        if !params.is_empty() { unreachable!("Task has no parameters"); }
        let result = (**result).clone();
        let id = self.next_closure; self.next_closure += 1;
        let span = function.scope_span;
        let mut body=Vec::new();
        for (name,_,_) in &function.params {
            let value=Expr::new(ExprKind::Name(name.text.clone()),name.span);
            body.push(Stmt{labels:Vec::new(),control_target:None,span:name.span,kind:StmtKind::Expr(Expr::new(ExprKind::Unary{op:UnaryOp::Void,value:Box::new(value),operator:name.span},name.span))});
        }
        body.append(&mut function.body);
        let closure = Expr::new(ExprKind::Closure { id, params: Vec::new(), result, body }, span);
        let task = Expr::new(ExprKind::Unary { op: UnaryOp::TaskCreate, value: Box::new(closure), operator: function.name.span }, span);
        function.body = vec![Stmt { labels: Vec::new(), control_target: None, span, kind: StmtKind::Return(Some(task)) }];
        Ok(())
    }
}
