//! Schema callbacks retain independent input/output contracts and unique storage.
use super::*;
use crate::stdlib::schema::pipelines::{Operation, Stage};

impl Analyzer {
    pub(super) fn schema_pipeline_array(&mut self, expr: &mut Expr, element: usize, child: Expr, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let (input, output, json_input) = self.schema_pipelines.types(element).expect("checked child pipeline");
        let array = |ty| ArrayElement::from_value_type(ty).map(Type::Array)
            .ok_or_else(|| Diagnostic::new("schema-type", "Schema output cannot be stored in an array.", expr.span));
        let plan = crate::stdlib::schema::pipelines::ArrayPlan { element, input: array(input)?, output: array(output)?, json_input };
        let id = if let Some(id) = self.schema_pipelines.array_record(&plan) { id } else {
            let id = self.program.records.len();
            let name = format!("@schema:pipeline-array:{id}");
            let field = RecordField {
                name: Token { text: "@element".into(), span: expr.span, kind: TokenKind::Name },
                annotation: self.checked_value_annotation(Type::Record(element), expr.span)?,
                ty: Some(Type::Record(element)), callable_declaration: None, optional: false,
            };
            self.record_names.insert(name.clone(), id);
            self.fields.push([(field.name.text.clone(), Type::Record(element))].into_iter().collect());
            self.program.records.push(Record {
                native_type: None, map_definition: None, set_definition: None, enum_definition: None,
                literal_definition: None, application: None,
                name: Token { text: name, span: expr.span, kind: TokenKind::Name },
                type_parameters: Vec::new(), fields: vec![field],
            });
            self.schema_pipelines.insert_array(id, plan);
            id
        };
        self.require(Type::Record(id), expected, expr.span)?;
        expr.kind = ExprKind::Record(vec![(Token { text: "@element".into(), span: expr.span, kind: TokenKind::Name }, child)]);
        Ok(Type::Record(id))
    }

    pub(super) fn schema_pipeline_validation(
        &mut self, context: &mut Context, expr: &mut Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, _) = &mut expr.kind else { return Ok(None); };
        let ExprKind::Field(base, method) = &callee.kind else { return Ok(None); };
        if method != "safeParse" { return Ok(None); }
        let hint = self.expression_type_hint(context, base);
        let constructing = matches!(&base.kind, ExprKind::Call(callee, _)
            if matches!(&callee.kind, ExprKind::Field(_, method)
                if matches!(method.as_str(), "refine" | "superRefine" | "transform" | "coerce")));
        if !constructing && !matches!(hint, Some(Type::Record(id)) if self.schema_pipelines.types(id).is_some()) {
            return Ok(None);
        }
        let mut receiver = (**base).clone();
        self.expression(context, &mut receiver, Mode::Read, loans, None)?;
        self.schema_pipeline_validation_receiver(context, expr, receiver, loans, expected)
    }

    pub(super) fn schema_pipeline_validation_receiver(
        &mut self, context: &mut Context, expr: &mut Expr, receiver: Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let Some(Type::Record(root)) = receiver.ty else { return Ok(None); };
        let Some((_, output, json_input)) = self.schema_pipelines.types(root) else { return Ok(None); };
        let ExprKind::Call(callee, args) = &mut expr.kind else { unreachable!() };
        if args.len() != 1 { return Err(Diagnostic::new("arity", "safeParse expects one input value.", expr.span)); }
        let (mut families, leaves) = self.schema_pipelines.dependencies(root);
        let hint = self.expression_type_hint(context, &args[0]);
        let json = matches!(hint, Some(Type::Record(id)) if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::JsonValue));
        let input = if json && json_input { hint.unwrap() }
            else { self.schema_pipeline_input_type(context,root,&args[0])? };
        let actual = self.expression(context, &mut args[0], Mode::Consume, loans, Some(input))?;
        self.require(actual, Some(input), args[0].span)?;
        let mut pending = leaves.into_iter().map(|source| self.program.records[source].application.as_ref().unwrap().resolved_arguments[0]).collect::<Vec<_>>();
        let mut seen = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) { continue; }
            if let Some(&family) = self.schema_factories.get(&ty) { families.push(family); }
            pending.extend(self.schema_choices.for_output(ty).flat_map(|(_, choice)| choice.branches.iter().copied()));
            families.extend(crate::stdlib::schema::defaults::inherited_factories(ty, &self.schema_factories,
                |id| self.unions.members(id).to_vec()).into_iter().map(|(_, family)| family));
            match ty {
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) => pending.extend(self.unions.members(id)),
                Type::Record(id) => pending.extend(self.program.records[id].fields.iter().filter_map(|field| field.ty)),
                _ => {}
            }
        }
        let error = self.program.records.iter().position(|record|
            record.native_type == Some(crate::native_signatures::NativeTypeId::ValidationError)).unwrap();
        use crate::type_syntax::{ValueAnnotation, ValueAnnotationKind, TypeName};
        let mut annotation = ValueAnnotation { span: expr.span, kind: ValueAnnotationKind::Applied {
            name: TypeName { text: "Result".into(), span: expr.span },
            arguments: vec![self.checked_value_annotation(output, expr.span)?, self.checked_value_annotation(Type::Record(error), expr.span)?],
        }};
        self.generic_records.materialize(&mut annotation, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        let result = self.resolve_value_annotation(&annotation, false)?;
        self.require(result, expected, expr.span)?;
        families.sort_unstable(); families.dedup();
        expr.schema_invocations = Some(families.into_boxed_slice());
        args.insert(0, receiver);
        callee.kind = ExprKind::Name("@schema:validate".into());
        expr.target = Some(CallTarget::SchemaValidate);
        Ok(Some(result))
    }

    pub(super) fn schema_pipeline_constructor(
        &mut self, context: &mut Context, expr: &mut Expr, loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else { return Ok(None); };
        let ExprKind::Field(base, method) = &callee.kind else { return Ok(None); };
        let operation = match method.as_str() {
            "refine" | "superRefine" => Operation::Refine,
            "transform" => Operation::Transform,
            "coerce" => Operation::Coerce,
            _ => return Ok(None),
        };
        let multiple_issues = method == "superRefine";
        let hint = self.expression_type_hint(context, base);
        let mut root = &**base;
        while let ExprKind::Call(callee, _) = &root.kind {
            match &callee.kind {
                ExprKind::Field(receiver, _) => root = receiver,
                _ => { root = callee; break; }
            }
        }
        let constructing = matches!(&root.kind, ExprKind::Name(name)
            if name.starts_with("@schema:") || name.starts_with("@native:std:schema#"));
        if !constructing && !matches!(hint, Some(Type::Record(id))
            if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::Schema)
                || self.schema_pipelines.types(id).is_some()) { return Ok(None); }
        if args.len() != 1 && !(operation == Operation::Refine && !multiple_issues && args.len() == 2) {
            return Err(Diagnostic::new("arity", "Schema pipeline methods expect one reusable callback; refine also accepts diagnostic options.", expr.span));
        }
        let mut receiver = (**base).clone();
        let actual = self.expression(context, &mut receiver, Mode::Consume, loans, None)?;
        let Type::Record(source) = actual else { return Ok(None); };
        let (input, output, json_input) = if let Some(types) = self.schema_pipelines.types(source) {
            types
        } else if self.program.records[source].native_type == Some(crate::native_signatures::NativeTypeId::Schema) {
            let output = self.program.records[source].application.as_ref().unwrap().resolved_arguments[0];
            (output, output, true)
        } else { return Ok(None); };

        let mut diagnostic_path = Vec::new();
        let mut message = None;
        if args.len() == 2 {
            let options = args.pop().unwrap();
            let ExprKind::Record(fields) = options.kind else {
                return Err(Diagnostic::new("schema-options", "Refinement options require an object with path and/or message.", options.span));
            };
            let mut names = HashSet::new();
            for (name, mut value) in fields {
                if !names.insert(name.text.clone()) {
                    return Err(Diagnostic::new("schema-options", "Duplicate refinement option.", name.span));
                }
                match name.text.as_str() {
                    "path" => diagnostic_path = self.schema_diagnostic_path(output, &value)?,
                    "message" => {
                        let message_type = self.expression(context, &mut value, Mode::Consume, loans, Some(Type::String))?;
                        self.require(message_type, Some(Type::String), value.span)?;
                        message = Some(value);
                    }
                    _ => return Err(Diagnostic::new("schema-options", "Unknown refinement option; use path or message.", name.span)),
                }
            }
        }

        let context_type = if multiple_issues { Some(self.schema_context_type(output, expr.span)?) } else { None };
        let callback = &mut args[0];
        if let ExprKind::Closure { params, result, .. } = &mut callback.kind {
            if operation != Operation::Coerce && params.len() == (if multiple_issues { 2 } else { 1 })
                && matches!(params[0].ty, ParameterAnnotation::Inferred(_)) {
                params[0].ty = ParameterAnnotation::Named(Box::new(self.checked_value_annotation(output, callback.span)?));
            }
            if let Some(context_type) = context_type {
                if params.len() == 2 && matches!(params[1].ty, ParameterAnnotation::Inferred(_)) {
                    params[1].ty = ParameterAnnotation::Named(Box::new(self.checked_value_annotation(context_type, callback.span)?));
                }
            }
            if operation == Operation::Refine && matches!(result, ResultAnnotation::Inferred(_)) {
                *result = ResultAnnotation::Named(Box::new(self.checked_value_annotation(if multiple_issues { Type::Void } else { Type::Boolean }, callback.span)?));
            }
        }
        if let ExprKind::Closure { id, .. } = &callback.kind { self.escaping.insert(*id); }
        let actual = self.expression(context, callback, Mode::Consume, loans, None)?;
        let Type::Closure(origin) = actual else {
            return Err(Diagnostic::new("schema-callback", "A schema pipeline requires a concrete reusable callback environment.", callback.span));
        };
        let closure = &self.closures[origin];
        let parameters = closure.parameters.iter()
            .map(|&id| ArgumentContract::from(&closure.function.bindings[id])).collect::<Vec<_>>();
        if parameters.len() != (if multiple_issues { 2 } else { 1 }) || parameters.iter().any(|parameter| parameter.rest
            || parameter.omission != crate::ParameterOmission::Required) {
            return Err(Diagnostic::new("schema-callback", "A schema callback requires one value parameter; superRefine also requires an issue context.", callback.span));
        }
        let callback_input = parameters[0].ty;
        let callback_output = closure.function.result;
        let parameter_names = closure.parameters.iter()
            .map(|&id| closure.function.bindings[id].name.clone()).collect();
        match operation {
            Operation::Refine => {
                self.require(output, Some(callback_input), callback.span)?;
                self.require(callback_output, Some(if multiple_issues { Type::Void } else { Type::Boolean }), callback.span)?;
                if let Some(context_type) = context_type {
                    self.require(parameters[1].ty, Some(context_type), callback.span)?;
                    if parameters[1].mode == Mode::Consume {
                        return Err(Diagnostic::new("schema-context-escape", "Validation context may only be borrowed during superRefine; it cannot be consumed or retained.", callback.span));
                    }
                }
                if matches!(parameters[0].mode, Mode::Mutate | Mode::Consume) {
                    return Err(Diagnostic::new("schema-refinement", "Refinements may only read their input. Use transform for mutation or consumption.", callback.span));
                }
            }
            Operation::Transform => { self.require(output, Some(callback_input), callback.span)?; }
            Operation::Coerce => {
                let json = matches!(callback_output, Type::Record(id)
                    if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::JsonValue));
                let accepts = if self.schema_pipelines.types(source).is_some() {
                    input == callback_output || (json && json_input)
                } else {
                    json || self.schema_owned_input_compatible(input, callback_output)
                        || (input == Type::Number && callback_output.scalar_base() == Type::Int)
                };
                if !accepts { self.require(callback_output, Some(input), callback.span)?; }
            }
        }
        if callback_output == Type::Void && !multiple_issues {
            return Err(Diagnostic::new("schema-callback", "Schema conversion callbacks must return a value.", callback.span));
        }
        let structural = self.callables.shapes.intern_rest(
            parameters.iter().map(|p| (p.ty.into(), p.mode)).collect(),
            callback_output.into(), Some(CallbackKind::Read), parameters.len(), false,
        );
        let declaration = self.callables.callbacks.len();
        let contract = CallbackContract {
            structural, parameter_names, params: parameters,
            result: callback_output, kind: CallbackKind::Read,
        };
        let Type::StoredCallable(family) = self.callables.storage
            .intern_owned_read(&self.callables.shapes, &contract, declaration).unwrap() else { unreachable!() };
        self.callables.callbacks.push(contract);
        if !context.template && !self.discovering_results && !self.effects_only {
            self.callables.storage.register_origin(family, origin, &self.callables.callbacks,
                &self.callables.shapes, &self.closures, callback.span)?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        callback.wrap_storage(family);
        let stage = Stage {
            source, diagnostic_path, custom_message: message.is_some(), operation, callback: family, callback_input, callback_output,
            input: if operation == Operation::Coerce { callback_input } else { input },
            output: if operation == Operation::Transform { callback_output } else { output },
            json_input: if operation == Operation::Coerce {
                matches!(callback_input, Type::Record(id) if self.program.records[id].native_type == Some(crate::native_signatures::NativeTypeId::JsonValue))
            } else { json_input },
        };
        let record = if let Some(record) = self.schema_pipelines.record_for(&stage) { record } else {
            let record = self.program.records.len();
            let name = format!("@schema:pipeline:{record}");
            let mut stored = vec![("@schema", Type::Record(source)), ("@callback", Type::StoredCallable(family))];
            if message.is_some() { stored.push(("@message", Type::String)); }
            let fields = stored.into_iter().map(|(name, ty)| Ok(RecordField {
                    name: Token { text: name.into(), span: expr.span, kind: TokenKind::Name },
                    annotation: self.checked_value_annotation(ty, expr.span)?,
                    ty: Some(ty), callable_declaration: None, optional: false,
                })).collect::<Result<Vec<_>, Diagnostic>>()?;
            self.record_names.insert(name.clone(), record);
            self.fields.push(fields.iter().map(|field| (field.name.text.clone(), field.ty.unwrap())).collect());
            self.program.records.push(Record {
                native_type: None, map_definition: None, set_definition: None,
                enum_definition: None, literal_definition: None, application: None,
                name: Token { text: name, span: expr.span, kind: TokenKind::Name },
                type_parameters: Vec::new(), fields,
            });
            self.schema_pipelines.insert(record, stage);
            record
        };
        self.require(Type::Record(record), expected, expr.span)?;
        let callback = args.remove(0);
        let mut values = vec![
            (Token { text: "@schema".into(), span: expr.span, kind: TokenKind::Name }, receiver),
            (Token { text: "@callback".into(), span: expr.span, kind: TokenKind::Name }, callback),
        ];
        if let Some(message) = message { values.push((Token { text: "@message".into(), span: expr.span, kind: TokenKind::Name }, message)); }
        expr.kind = ExprKind::Record(values);
        Ok(Some(Type::Record(record)))
    }
}
