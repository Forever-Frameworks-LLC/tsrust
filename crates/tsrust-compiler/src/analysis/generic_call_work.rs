//! Pure generic publication and argument inference; the driver schedules bodies.
use super::*;
pub(super) struct Arguments {
    pub inline_shapes: HashMap<(usize, usize), (Vec<Type>, Type)>,
    pub prepared: Vec<bool>,
}
pub(super) enum Prepared {
    Spread(Box<Expr>),
    Done,
    Function { index: usize, restore_name: Option<String> },
    Arguments(Arguments),
}
pub(super) fn call(mut expr: &mut Expr) -> &mut Expr {
    while matches!(&expr.kind, ExprKind::OptionalChain(_) | ExprKind::Unary { op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected, .. }) {
        expr = match &mut expr.kind { ExprKind::OptionalChain(value) | ExprKind::Unary { value, .. } => value, _ => unreachable!() };
    }
    expr
}
impl Analyzer {
    pub(super) fn request_named_result_hint(&mut self, context: &Context, index: usize) -> Result<Prepared, Diagnostic> {
        if self.signatures[index].inferred_result && !self.discovering_results {
            self.named_captures.prepare(index, context, &self.program)?;
            return Ok(Prepared::Function { index, restore_name: None });
        }
        Ok(Prepared::Done)
    }
    fn publish_generic_preparation(&mut self, context: &Context, name: &mut String, concrete: String) -> Result<Prepared, Diagnostic> {
        let concrete = self.publish_generic_call_work(concrete)?;
        let prepared = self.request_named_result_hint(context, self.function_names[&concrete])?;
        let original = std::mem::replace(name, concrete);
        Ok(match prepared { Prepared::Function { index, .. } => Prepared::Function { index, restore_name: Some(original) }, other => other })
    }
    pub(super) fn start_generic_preparation(&mut self, context: &Context, expr: &mut Expr, expected: Option<Type>) -> Result<Prepared, Diagnostic> {
        let expr = call(expr);
        if matches!(&expr.kind, ExprKind::Call(_, args) if args.iter().any(|arg| matches!(arg.kind, ExprKind::Unary { op: UnaryOp::Spread, .. }))) {
            return Ok(self.generic_spread_prototype(context, expr)?.map_or(Prepared::Done, Prepared::Spread));
        }
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(Prepared::Done); };
        if matches!(&callee.kind,ExprKind::Name(name) if name=="@json:typedreadJsonl")
            && args.first().and_then(|arg|self.expression_type_hint(context,arg)).is_some_and(|ty|matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonlReaderView))) {
            callee.kind=ExprKind::Name("@json:typedreadJsonlView".into());
        }
        if matches!(&callee.kind,ExprKind::Name(name) if name=="@json:writeJsonl")
            && args.first().and_then(|arg|self.expression_type_hint(context,arg)).is_some_and(|ty|matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonlWriterView))) {
            callee.kind=ExprKind::Name("@json:writeJsonlView".into());
        }
        let Some((name, _)) = Self::generic_call_name(callee) else { return Ok(Prepared::Done); };
        if context.names.contains_key(name) || context.pending_names.contains(name) {
            if self.generic_functions.value_arguments(&context.function_name, expr.span).is_some()
                && let Some(&binding) = context.names.get(name)
                && !matches!(self.unions.payload(context.bindings[binding].ty).unwrap_or(context.bindings[binding].ty), Type::Closure(origin) if self.deferred_generic_values.contains_key(&origin)) {
                return Err(Diagnostic::new("type", "Explicit type arguments require a generic function value.", callee.span));
            }
            return Ok(Prepared::Done);
        }
        if self.generic_functions.template(name).is_none() {
            return if let Some(&index) = self.function_names.get(name) { self.request_named_result_hint(context, index) } else { Ok(Prepared::Done) };
        }
        self.start_named_generic_preparation(context, expr, expected)
    }
    fn start_named_generic_preparation(&mut self, context: &Context, expr: &mut Expr, _expected: Option<Type>) -> Result<Prepared, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("generic call preparation");
        };
        let (name, _) = Self::generic_call_name(callee)
            .expect("generic call name checked by dispatch");
        let template=self.generic_functions.template(name).expect("generic template");
        if template.rest_parameter {
            let fixed=template.params.len()-1;
            let minimum=crate::parameter_arity::minimum_arguments(&template.parameter_initializers[..fixed],|initializer|initializer.omission().is_omittable());
            if args.len()<minimum {return Err(Diagnostic::new("arity",format!("Expected at least {minimum} arguments, found {}.",args.len()),expr.span));}
            Self::pack_rest_arguments(args,fixed,expr.span);
        }
        if let Some(arguments) = self.generic_functions.value_arguments(&context.function_name, expr.span) {
            let captures = self.named_captures.source_context(name, context, &self.program)?
                .map(|(context, _)| context.bindings.iter().map(|binding| binding.ty).collect()).unwrap_or_default();
            let concrete = self.generic_functions.request_explicit_value(name, arguments, captures, expr.span, self.function_stack.len())?;
            return self.publish_generic_preparation(context, name, concrete);
        }
        let polymorphic_slots=self.generic_functions.template(name).expect("generic template").params.iter().map(|(_,annotation,_)| {
            matches!(&annotation.kind,crate::type_syntax::ValueAnnotationKind::Named(variable) if self.generic_functions.template(name).unwrap().type_parameters.iter().any(|parameter|parameter.text==variable.text))
        }).collect::<Vec<_>>();
        for (argument,polymorphic) in args.iter_mut().zip(polymorphic_slots) {
            if polymorphic {self.prepare_polymorphic_argument_hint(context,argument)?;}
        }
        let mut inline_shapes = HashMap::new();
        for ((_, pattern, _), argument) in self
            .generic_functions
            .template(name)
            .expect("generic template")
            .params
            .iter()
            .zip(args.iter())
        {
            if let Some(shape) = self.inline_generic_callable_shape(argument) {
                inline_shapes.insert((pattern.span.start, pattern.span.end), shape);
            }
        }
        Ok(Prepared::Arguments(Arguments { inline_shapes, prepared: vec![false; args.len()] }))
    }
    pub(super) fn generic_preparation_contexts(&mut self, context: &Context, expr: &mut Expr, expected: Option<Type>, frame: &Arguments) -> Result<Vec<Option<Type>>, Diagnostic> {
        let expr = call(expr);
        let span = expr.span;
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        let (name, _) = Self::generic_call_name(callee).expect("generic name");
            let callable_shapes = self.generic_callable_shapes(context, args);
            let mut annotations = self.checked_inference_callable_annotations(callable_shapes.keys().copied().chain(expected))?;
            annotations.extend(self.polymorphic_argument_annotations(context,args)?);
            let types = crate::generic_inference::Types { callable_annotations: Some(&annotations),
                callable_shapes: Some(&callable_shapes),
                inline_shapes: Some(&frame.inline_shapes),
                records: &self.program.records,
                names: &self.record_names,
                unions: &self.unions,
            };
            let seed = self
                .generic_functions
                .parameter_contexts(name, expected, &types, None)?;
            let template=self.generic_functions.template(name).expect("generic template");
            let fixed=template.params.len()-usize::from(template.rest_parameter);
            let minimum=crate::parameter_arity::minimum_arguments(&template.parameter_initializers[..fixed],|initializer|initializer.omission().is_omittable());
            if args.len()<minimum||args.len()>seed.len(){return Err(Diagnostic::new("arity",format!("Expected {minimum}..={} arguments, found {}.",seed.len(),args.len()),span));}
            let hints = args
                .iter()
                .zip(&seed)
                .map(|(argument, expected)| {
                    self.generic_argument_hint(context, argument, *expected)
                })
                .collect::<Vec<_>>();
            let contexts =
                self.generic_functions
                    .parameter_contexts(name, expected, &types, Some(&hints))?;
        Ok(contexts)
    }
    pub(super) fn finish_generic_preparation(&mut self, context: &Context, expr: &mut Expr, expected: Option<Type>, frame: &Arguments) -> Result<Prepared, Diagnostic> {
        let expr = call(expr);
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        let (name, callee_span) = Self::generic_call_name(callee).expect("generic name");
        let callable_shapes = self.generic_callable_shapes(context, args);
            let mut annotations = self.checked_inference_callable_annotations(callable_shapes.keys().copied().chain(expected))?;
            annotations.extend(self.polymorphic_argument_annotations(context,args)?);
        let types = crate::generic_inference::Types { callable_annotations: Some(&annotations),
            callable_shapes: Some(&callable_shapes),
            inline_shapes: Some(&frame.inline_shapes),
            records: &self.program.records,
            names: &self.record_names,
            unions: &self.unions,
        };
        let seed = self
            .generic_functions
            .parameter_contexts(name, expected, &types, None)?;
        let hints = args
            .iter()
            .zip(&seed)
            .map(|(argument, expected)| self.generic_argument_hint(context, argument, *expected))
            .collect::<Vec<_>>();
        let contexts =
            self.generic_functions
                .parameter_contexts(name, expected, &types, Some(&hints))?;
        let hints = args
            .iter()
            .zip(&contexts)
            .map(|(argument, expected)| self.generic_argument_hint(context, argument, *expected))
            .collect::<Vec<_>>();
        let arguments =
            self.generic_functions
                .infer(name, &hints, expected, &types, callee_span)?;
        let captures = self.named_captures.source_context(name, context, &self.program)?
            .map(|(context, _)| context.bindings.iter().map(|binding| binding.ty).collect())
            .unwrap_or_default();
        let concrete = self.generic_functions.request_captured_checked(
            name,
            arguments,
            captures,
            &types,
            expr.span,
            self.function_stack.len(),
        )?;
        self.publish_generic_preparation(context, name, concrete)
    }
}
