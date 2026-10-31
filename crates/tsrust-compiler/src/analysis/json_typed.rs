//! Explicit JSON decoding validates the requested owned shape before emission.
use super::*;
impl Analyzer {
    pub(super) fn json_typed_decode(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
        expected: Option<Type>,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            return Ok(None);
        };
        if !matches!(&callee.kind,ExprKind::Name(name) if matches!(name.as_str(),"@json:decode"|"@json:decodeJsonc"|"@json:decodeJsonl")) {
            return Ok(None);
        }
        let jsonl=matches!(&callee.kind,ExprKind::Name(name) if name=="@json:decodeJsonl");
        let jsonc=matches!(&callee.kind,ExprKind::Name(name) if name=="@json:decodeJsonc");
        let span = self
            .program
            .functions
            .iter()
            .find(|function| function.name.text == context.function_name)
            .and_then(|function| function.application.as_ref())
            .and_then(|application| application.arguments.first())
            .map_or(expr.span, |argument| argument.span);
        let result = expected.ok_or_else(|| {
            Diagnostic::new(
                "json-type",
                "Typed JSON decoding requires a concrete result type.",
                span,
            )
        })?;
        let (_, variants) = self.result_variant_records(result).ok_or_else(|| {
            Diagnostic::new(
                "json-type",
                "Typed JSON decoding requires a Result context.",
                span,
            )
        })?;
        let ty = self.program.records[variants[0]]
            .fields
            .first()
            .and_then(|field| field.ty)
            .ok_or_else(|| {
                Diagnostic::new(
                    "json-type",
                    "Typed JSON decoding requires a value target; void is not a JSON value.",
                    span,
                )
            })?;
        let mut borrowed_raw = false;
        let mut pending = vec![ty];
        let mut seen = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match ty {
                Type::Int | Type::Number | Type::Boolean | Type::String | Type::Null | Type::Literal(_,_) => {}
                Type::Array(element) => pending.push(element.ty()),
                Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonRaw) && !jsonc && !jsonl => { borrowed_raw=true; }
                Type::Record(id)
                    if self.program.records[id].native_type.is_none()
                        && self.program.records[id].map_definition.is_none()
                        && self.program.records[id].set_definition.is_none() =>
                {
                    pending.extend(
                        self.program.records[id]
                            .fields
                            .iter()
                            .filter_map(|field| field.ty),
                    )
                }
                Type::Union(id) => {
                    let members = self.unions.members(id);
                    let has_null=members.contains(&Type::Null);
                    let values: Vec<_> = members
                        .into_iter()
                        .filter(|ty| !matches!(ty, Type::Null | Type::Undefined))
                        .collect();
                    if values.len() != 1 && !(values.is_empty() && has_null) && crate::literal_types::family(&values).is_none() {
                        return Err(Diagnostic::new(
                            "json-type",
                            "Typed JSON decoding supports nullable/optional values and closed same-kind literal unions; other multiple-shape unions are unsupported.",
                            span,
                        ));
                    }
                    pending.extend(values);
                }
                _ => {
                    return Err(Diagnostic::new(
                        "json-type",
                        format!(
                            "Typed JSON decoding does not support {}.",
                            self.type_name(ty)
                        ),
                        span,
                    ));
                }
            }
        }
        // `JSON.parse<T>(text)` may omit limits; JSONC and JSONL arities are fixed.
        let plain = !jsonl && !jsonc && args.len() == 1;
        if !plain && args.len() != if jsonl {1}else if jsonc {3}else{2} {
            return Err(Diagnostic::new(
                "arity",
                "Typed JSON decoding expects text and limits.",
                span,
            ));
        }
        let limits = self
            .program
            .records
            .iter()
            .position(|record| {
                record.native_type == Some(crate::native_signatures::NativeTypeId::JsonLimits)
            })
            .expect("prepared JSON limits");
        let mut contracts = vec![
            ArgumentContract {
                ty: Type::String,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
                rest: false,
            },
            ArgumentContract {
                ty: Type::Record(limits),
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
                rest: false,
            },
        ];
        if plain {contracts.truncate(1);}
        if jsonc {contracts.insert(1,ArgumentContract{ty:Type::Boolean,mode:Mode::Copy,omission:crate::ParameterOmission::Required,rest:false});}
        if jsonl {
            let identity=if args.first().and_then(|arg|self.expression_type_hint(context,arg)).is_some_and(|ty|matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonlReaderView))) {crate::native_signatures::NativeTypeId::JsonlReaderView}else{crate::native_signatures::NativeTypeId::JsonlReader};
            let reader=self.program.records.iter().position(|record|record.native_type==Some(identity)).unwrap();
            contracts=vec![ArgumentContract{ty:Type::Record(reader),mode:Mode::Mutate,omission:crate::ParameterOmission::Required,rest:false}];
        }
        self.check_call_arguments(context, args, &contracts, loans)?;
        if borrowed_raw {
            let contract=crate::native_return_borrows::ReturnedBorrow{source_parameter:0,access:crate::native_return_borrows::BorrowAccess::ReadOnly,payload:crate::native_signatures::NativeTypeId::JsonRaw,slot:crate::native_return_borrows::ReturnSlot::ResultOk};
            Self::native_view_source_loans(context,&args[0],contract)?;
            context.native_view_calls=true;
            context.native_view_results.get_or_insert_with(Default::default).insert(result.union().unwrap(),(variants[0],None));
        }
        expr.target = Some(CallTarget::JsonDecode);
        expr.ty = Some(result);
        Ok(Some(result))
    }
}

impl Analyzer {
    pub(super) fn json_typed_encode(
        &mut self,
        context: &mut Context,
        expr: &mut Expr,
        loans: &Loans,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            return Ok(None);
        };
        if !matches!(&callee.kind,ExprKind::Name(name) if matches!(name.as_str(),"@json:encode"|"@json:write")) {
            return Ok(None);
        }
        let writing=matches!(&callee.kind,ExprKind::Name(name) if name=="@json:write");
        let span = self
            .program
            .functions
            .iter()
            .find(|function| function.name.text == context.function_name)
            .and_then(|function| function.application.as_ref())
            .and_then(|application| application.arguments.first())
            .map_or(expr.span, |argument| argument.span);
        if args.len() != 2 {
            return Err(Diagnostic::new(
                "arity",
                "JSON encoding expects a value and byte limit.",
                span,
            ));
        }
        let ty = self
            .expression(context, &mut args[usize::from(writing)], Mode::Read, loans, None)
            .map_err(|mut error| {
                // Compiler-owned adapter expressions must never escape into source diagnostics.
                error.span = span;
                if error.code == "callback-escape" {
                    error.code = "json-type";
                    error.message = "JSON encoding does not support callbacks.".into();
                }
                error
            })?;
        if writing {
            let identity=if args.first().and_then(|arg|self.expression_type_hint(context,arg)).is_some_and(|ty|matches!(ty,Type::Record(id) if self.program.records[id].native_type==Some(crate::native_signatures::NativeTypeId::JsonlWriterView))) {crate::native_signatures::NativeTypeId::JsonlWriterView}else{crate::native_signatures::NativeTypeId::JsonlWriter};
            let writer=self.program.records.iter().position(|record|record.native_type==Some(identity)).unwrap();
            self.expression(context,&mut args[0],Mode::Mutate,loans,Some(Type::Record(writer)))?;
        }else{self.expression(context, &mut args[1], Mode::Copy, loans, Some(Type::Int))?;}

        let mut pending = vec![ty];
        let mut seen = std::collections::HashSet::new();
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match ty {
                Type::Int
                | Type::Number
                | Type::Boolean
                | Type::String
                | Type::Null
                | Type::Undefined | Type::Literal(_,_) | Type::Enum(_,_) => {}
                Type::Array(element) => pending.push(element.ty()),
                Type::Union(id) | Type::BorrowedUnion(id) => {
                    pending.extend(self.unions.members(id))
                }
                Type::Record(id) => {
                    let record = &self.program.records[id];
                    match record.native_type {
                        Some(crate::native_signatures::NativeTypeId::JsonRaw) => {}
                        Some(
                            crate::native_signatures::NativeTypeId::JsonValue
                            | crate::native_signatures::NativeTypeId::JsonView
                            | crate::native_signatures::NativeTypeId::StringView,
                        ) => {}
                        Some(identity) if crate::native_types::declaration(identity).source_constructible()=>{
                            pending.extend(record.fields.iter().filter_map(|field|field.ty))
                        }
                        None if record.map_definition.is_none()
                            && record.set_definition.is_none()
                            && !record.application.as_ref().is_some_and(|application| {
                                matches!(
                                    application.declaration.text.as_str(),
                                    "Result#Ok" | "Result#Err"
                                )
                            }) =>
                        {
                            pending.extend(record.fields.iter().filter_map(|field| field.ty))
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                "json-type",
                                format!("JSON encoding does not support {}.", self.type_name(ty)),
                                span,
                            ));
                        }
                    }
                }
                _ => {
                    return Err(Diagnostic::new(
                        "json-type",
                        format!("JSON encoding does not support {}.", self.type_name(ty)),
                        span,
                    ));
                }
            }
        }
        let binding = crate::native_bindings::lookup("std:json", if writing {"writeJsonl"}else{"stringify"}).unwrap();
        let result = native_type(binding.result, &self.program.records);
        expr.ty = Some(result);
        expr.target = Some(CallTarget::JsonEncode);
        Ok(Some(result))
    }
}
