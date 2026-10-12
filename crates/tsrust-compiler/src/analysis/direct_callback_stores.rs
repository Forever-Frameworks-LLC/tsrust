//! Direct borrowed callback stores use the ordinary capture transport machinery.
use super::*;

impl Analyzer {
    pub(super) fn prepare_direct_callback_store(
        &mut self,
        context: &Context,
        target: &Expr,
        value: &mut Expr,
    ) -> Result<(), Diagnostic> {
        let Some(root) = Self::root_binding(target) else {
            return Ok(());
        };
        if root >= context.bindings.len()
            || !context.bindings[root].parameter
            || context.bindings[root].mode != Mode::Mutate
            || !matches!(context.bindings[root].ty, Type::Record(_))
        {
            return Ok(());
        }
        let ExprKind::Name(name) = &value.kind else {
            return Ok(());
        };
        let Some(&binding) = context.names.get(name) else {
            return Ok(());
        };
        let Type::Callback(contract) = context.bindings[binding].ty else {
            return Ok(());
        };
        let span = value.span;
        let annotation = self.checked_value_annotation(Type::Callback(contract), span)?;
        let signature = crate::callable_annotations::callback(&annotation)
            .expect("checked callback annotation");
        let mut parameters = signature.params;
        let arguments = parameters
            .iter_mut()
            .enumerate()
            .map(|(index, parameter)| {
                let name = format!("#stored-callback-argument-{index}");
                parameter.name.text = name.clone();
                parameter.name.span = span;
                let argument = Expr::new(ExprKind::Name(name), span);
                if parameter.rest {
                    Expr::new(
                        ExprKind::Unary {
                            op: UnaryOp::RestPack,
                            operator: span,
                            value: Box::new(argument),
                        },
                        span,
                    )
                } else {
                    argument
                }
            })
            .collect();
        let call = Expr::new(ExprKind::Call(Box::new(value.clone()), arguments), span);
        *value = Expr::new(
            ExprKind::Closure {
                id: usize::MAX / 3 + span.start,
                params: parameters,
                result: *signature.result,
                body: vec![Stmt {
                    kind: StmtKind::Return(Some(call)),
                    span,
                    control_target: None,
                    labels: Vec::new(),
                }],
            },
            span,
        );
        Ok(())
    }
}
