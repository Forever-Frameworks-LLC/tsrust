//! Constructor messages wrap the exact child plan without copying it.
use super::*;
use crate::stdlib::schema::pipelines::WrapperKind;

impl Analyzer {
    pub(super) fn schema_message_policy(
        &mut self, expr: &mut Expr, id: usize, receiver: Expr, retaining: bool,
        expected: Option<Type>,
    ) -> Result<Type, Diagnostic> {
        let source = self.schema_pipelines.wrapper(id).unwrap().source;
        let mut child = Expr::new(ExprKind::Field(Box::new(receiver.clone()), "@schema".into()), expr.span);
        child.ty = Some(Type::Record(source));
        let changed = if self.schema_pipelines.object(source).is_some() {
            let mut result = child.clone();
            self.schema_pipeline_object_policy(&mut result, source, child.clone(), retaining, None)?
        } else {
            let output = self.program.records[source].application.as_ref()
                .and_then(|application| application.resolved_arguments.first()).copied();
            let Some(output @ Type::Record(object)) = output else {
                return Err(Diagnostic::new("schema-member", "Object policy methods require an object schema.", expr.span));
            };
            if self.program.records[object].name.text.starts_with("@schema:retained:") {
                return Err(Diagnostic::new("schema-member", "Object policy is already retained.", expr.span));
            }
            if retaining { self.schema_retained_type(output, expr.span)? } else { Type::Record(source) }
        };
        let Type::Record(changed) = changed else { unreachable!() };
        let mut message = Expr::new(ExprKind::Field(Box::new(receiver.clone()), "@message".into()), expr.span);
        message.ty = Some(Type::String);
        let result = self.schema_pipeline_wrapper(expr, changed, child, WrapperKind::Message, Some(message), expected)?;
        // The final lowering evaluates the owned receiver once and moves its two
        // fields separately. The temporary expressions above only build types.
        let callee = Expr::new(ExprKind::Name("@schema:message-policy".into()), expr.span);
        expr.kind = ExprKind::Call(Box::new(callee), vec![receiver]);
        expr.target = Some(CallTarget::SchemaMessagePolicy(retaining));
        Ok(result)
    }

    pub(super) fn schema_message_constraint(
        &mut self, context: &mut Context, expr: &mut Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &expr.kind else { return Ok(None); };
        let ExprKind::Field(base, method) = &callee.kind else { return Ok(None); };
        if !matches!(method.as_str(), "min" | "max" | "gt" | "lt") { return Ok(None); }
        fn constructing(value: &Expr) -> bool {
            match &value.kind {
                ExprKind::Call(callee, args) => match &callee.kind {
                    ExprKind::Name(name) => matches!(name.as_str(),
                        "@native:std:schema#__int" | "@native:std:schema#__number" |
                        "@native:std:schema#__string") && args.len() == 1,
                    ExprKind::Field(base, method) if matches!(method.as_str(), "min" | "max" | "gt" | "lt") => constructing(base),
                    _ => false,
                },
                _ => false,
            }
        }
        let hinted = self.expression_type_hint(context, base);
        if !constructing(base) && !matches!(hinted, Some(Type::Record(id))
            if self.schema_pipelines.wrapper(id).is_some_and(|plan| plan.kind == WrapperKind::Message)) {
            return Ok(None);
        }
        if args.len() != 1 { return Err(Diagnostic::new("arity", "Constraints expect one numeric limit.", expr.span)); }
        let method = method.clone();
        let mut receiver = (**base).clone();
        let mut limit = args[0].clone();
        let Type::Record(id) = self.expression(context, &mut receiver, Mode::Consume, loans, None)? else { unreachable!() };
        let plan = self.schema_pipelines.wrapper(id).ok_or_else(|| Diagnostic::new("schema-member", "This schema does not support scalar constraints.", expr.span))?;
        let source = &self.program.records[plan.source];
        let output = source.application.as_ref().and_then(|application| application.resolved_arguments.first()).copied();
        let (kind, limit_type) = match output {
            Some(Type::Int) => ("Int", Type::Int),
            Some(Type::Number) => ("Number", Type::Number),
            Some(Type::String) if matches!(method.as_str(), "min" | "max") => ("String", Type::Int),
            _ => return Err(Diagnostic::new("schema-member", "This schema does not support this scalar constraint.", expr.span)),
        };
        let name = format!("@schema:__{method}{kind}");
        let target = self.function_names[&name];
        let actual_limit = self.expression(context, &mut limit, Mode::Copy, loans, Some(limit_type))?;
        self.require(actual_limit, Some(limit_type), limit.span)?;
        self.require(Type::Record(id), expected, expr.span)?;
        let mut callee = (**callee).clone();
        callee.kind = ExprKind::Name(name);
        expr.kind = ExprKind::Call(Box::new(callee), vec![receiver, limit]);
        expr.target = Some(CallTarget::SchemaConstraint(target));
        Ok(Some(Type::Record(id)))
    }

    pub(super) fn schema_constructor_options(
        &mut self, context: &mut Context, expr: &mut Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &expr.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        let required = match name.as_str() {
            "@native:std:schema#__string" | "@native:std:schema#__int" |
            "@native:std:schema#__number" | "@native:std:schema#__boolean" |
            "@native:std:schema#__null" => 0,
            "@schema:array" | "@schema:object" | "@schema:record" | "@schema:set" |
            "@schema:literal" | "@schema:enum" | "@schema:union" => 1,
            "@schema:map" | "@schema:discriminatedUnion" => 2,
            _ => return Ok(None),
        };
        if args.len() != required + 1 { return Ok(None); }
        let mut receiver = expr.clone();
        let ExprKind::Call(_, args) = &mut receiver.kind else { unreachable!() };
        let options = args.pop().unwrap();
        let ExprKind::Record(fields) = options.kind else {
            return Err(Diagnostic::new("schema-options", "Schema options require an object containing a string message.", options.span));
        };
        let mut message = None;
        for (name, value) in fields {
            if name.text != "message" || message.is_some() {
                return Err(Diagnostic::new("schema-options", "Constructor options accept one message field.", name.span));
            }
            message = Some(value);
        }
        let ty = self.expression(context, &mut receiver, Mode::Consume, loans, None)?;
        let Type::Record(source) = ty else { unreachable!("schema constructor result") };
        let Some(mut message) = message else {
            self.require(ty, expected, expr.span)?;
            *expr = receiver;
            return Ok(Some(ty));
        };
        let message_type = self.expression(context, &mut message, Mode::Consume, loans, Some(Type::String))?;
        self.require(message_type, Some(Type::String), message.span)?;
        self.schema_pipeline_wrapper(expr, source, receiver, WrapperKind::Message, Some(message), expected).map(Some)
    }
}
