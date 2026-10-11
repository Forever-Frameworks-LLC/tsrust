//! Checked Map operations stay outside recursive expression dispatch.
use super::*;
impl Analyzer {
    pub(super) fn map_receiver_hint(&self, context: &Context, base: &Expr) -> bool {
        matches!(self.expression_type_hint(context, base), Some(Type::Map(_))) || matches!(base.kind, ExprKind::MapConstructor(_, _)) || Self::map_group_by_callee(context,base)
    }
    pub(super) fn map_method_receiver(&self, context: &Context, expression: &Expr) -> bool {
        matches!(&expression.kind, ExprKind::Call(callee, _) if matches!(&callee.kind, ExprKind::Field(base, _) if self.map_receiver_hint(context, base)))
    }
    pub(super) fn map_method_type(&self, id: usize, name: &str, span: Span) -> Result<Type, Diagnostic> {
        match name {
            "set" | "clear" | "forEach" => Ok(Type::Void),
            "has" | "delete" => Ok(Type::Boolean),
            "get" => self.unions.intern([self.program.records[id].map_definition.as_ref().unwrap().value, Type::Undefined], span),
            "keys" | "values" => {
                let map=self.program.records[id].map_definition.as_ref().unwrap();
                ArrayElement::from_value_type(if name=="keys"{map.key}else{map.value}).map(Type::Array)
                    .ok_or_else(||Diagnostic::new("type","Map snapshots require array-compatible values.",span))
            },
            "entries" => {
                let map=self.program.records[id].map_definition.as_ref().unwrap();
                let label=format!("[{},{}]",self.type_name(map.key),self.type_name(map.value));
                self.record_names.get(&label).copied().map(|pair|Type::Array(ArrayElement::Record(pair,0)))
                    .ok_or_else(||Diagnostic::new("type","Map entry tuple needs checked receiver context.",span))
            },
            _ => Err(Diagnostic::new("map-member", "Unsupported Map method.", span)),
        }
    }
    #[inline(never)]
    pub(super) fn map_size(&mut self, context: &mut Context, base: &mut Expr, name: &str, mode: Mode, loans: &Loans, span: Span) -> Result<Type, Diagnostic> {
        self.expression(context, base, Mode::Read, loans, None)?;
        if name != "size" { return Err(Diagnostic::new("map-member", "Map has only the size value property; call its methods with parentheses.", span)); }
        if mode == Mode::Mutate { return Err(Diagnostic::new("readonly", "Map size is read-only.", span)); }
        Ok(Type::Int)
    }
    #[inline(never)]
    pub(super) fn map_method_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { unreachable!() };
        let ExprKind::Field(base, name) = &mut callee.kind else { unreachable!() };
        if name=="forEach" {
            let result=self.array_transform(context,base,arguments,loans,expression.span,(false,true,Mode::Read,false,false,false,false))?;
            self.require(result,expected,expression.span)?;
            expression.target=Some(CallTarget::ArrayForEach);
            return Ok(result);
        }
        if matches!(name.as_str(),"keys"|"values"|"entries") {
            return self.map_snapshot_call(context,expression,loans,expected);
        }
        let method = match name.as_str() {
            "set" => MapMethod::Set, "get" => MapMethod::Get, "has" => MapMethod::Has,
            "delete" => MapMethod::Delete, "clear" => MapMethod::Clear,
            _ => return Err(Diagnostic::new("map-member", "Unsupported Map method.", callee.span)),
        };
        Self::arity(arguments.len(), match method { MapMethod::Set => 2, MapMethod::Clear => 0, _ => 1 }, expression.span)?;
        let named = Self::stable_at_receiver(base);
        let receiver_mode = if method == MapMethod::Get && !named { Mode::Consume } else { Mode::Read };
        let Type::Map(id) = self.expression(context, base, receiver_mode, loans, None)? else { unreachable!("Map receiver hint") };
        if matches!(method, MapMethod::Set | MapMethod::Delete | MapMethod::Clear) { Self::storage_access(context, base, Mode::Mutate, expression.span, loans)?; }
        let definition = self.program.records[id].map_definition.as_ref().unwrap().clone();
        let mut active = loans.clone();
        let mut entry_place = None;
        let mut replaced_environment = false;
        let previous_loans = if matches!(method,MapMethod::Set|MapMethod::Delete)
            && self.aggregate_callable_facts(definition.value).scoped
        {context.capture_loan_owners.live()} else {Default::default()};
        for root in Self::loan_roots(base, Mode::Read) { active.entry(root).or_default().push((Mode::Read, base.span)); }
        for (index, argument) in arguments.iter_mut().enumerate() {
            let ty = if index == 0 { definition.key } else if matches!(definition.value, Type::Record(_) if self.aggregate_callable_facts(definition.value).scoped && Self::scoped_array_storage_path(base).is_some()) {
                let Type::Record(record) = definition.value else { unreachable!() };
                Type::Record(self.scoped_records.declaration(record))
            } else { definition.value };
            let mode = if ty.is_copy() { Mode::Copy } else if method == MapMethod::Set { Mode::Consume } else { Mode::Read };
            let actual = self.expression(context, argument, mode, &active, Some(ty))?;
            self.require(actual, Some(ty), argument.span)?;
            if index == 0 { entry_place = Self::map_entry_capture_place(base, argument); }
            if method == MapMethod::Set && index == 1
                && let (Type::Record(previous), Type::Record(incoming)) = (definition.value, actual)
                && self.aggregate_callable_facts(actual).scoped
                && Self::scoped_array_storage_path(base).is_some()
            {
                context.storage_exits.record_map_entry_write(expression.span,actual);
                let value = if previous == self.scoped_records.declaration(previous) { actual }
                    else if previous == incoming { actual }
                    else { self.join_record_types(previous, incoming, argument.span)?.ok_or_else(|| Diagnostic::new("type", "Map callback values require compatible environment contracts.", argument.span))? };
                let instance = self.scoped_map_value_instance(id, value);
                let binding = Self::root_binding(base).expect("stable Map root");
                let (_, path) = Self::scoped_array_storage_path(base).unwrap();
                if let Some(root) = self.scoped_array_replace_field(context.bindings[binding].ty, &path, Type::Map(instance)) { context.bindings[binding].ty = root; }
                if let Some(place) = entry_place.clone() {
                    let previous=context.capture_loan_owners.map_payloads(&place);
                    replaced_environment=!previous.is_empty() && previous.iter().any(|ty|*ty!=actual);
                    self.replace_map_entry_capture_loans(context, place, argument);
                } else {
                    self.append_target_capture_loans(context, base, argument);
                    if let Some((_,fields))=Self::scoped_array_storage_path(base) {
                        context.capture_loan_owners.attach_map_payload(crate::capture_loan_owners::Place{binding,fields},actual);
                    }
                }
            }
            if mode != Mode::Copy {
                for root in Self::loan_roots(argument, mode) { active.entry(root).or_default().push((mode, argument.span)); }
            }
        }
        if (method==MapMethod::Delete || replaced_environment)
            && self.aggregate_callable_facts(definition.value).scoped
            && let Some(place) = entry_place {
            if method == MapMethod::Delete {Self::discard_map_entry_capture_loans(context, &place);}
            if !context.bindings[place.binding].parameter && !context.bindings[place.binding].capture_reference {
                let mut root=place;
                root.fields.pop();
                let payloads=context.capture_loan_owners.map_payloads(&root);
                let live=context.capture_loan_owners.live();
                let released=previous_loans.difference(&live).filter_map(|id|context.capture_loan_ids.loan(*id).map(|loan|loan.0)).collect();
                context.storage_exits.record_map_entry_survivors(expression.span,root,Type::Map(id),payloads,released);
            }
        }
        if method == MapMethod::Clear
            && let Some((_, fields)) = Self::direct_fields(base)
            && let Some(binding) = Self::root_binding(base)
            && !context.bindings[binding].parameter
            && !context.bindings[binding].capture_reference
        {
            context.capture_loan_owners.discard(&crate::capture_loan_owners::Place { binding, fields });
            Self::restore_live_capture_loans(context);
        }
        if method == MapMethod::Set {
            Self::append_native_view_element(context, base, &arguments[1])?;
            self.record_native_element_store(context, base, &arguments[1])?;
        }
        let mut result = self.map_method_type(id, name, expression.span)?;
        if method == MapMethod::Get && named && !result.is_copy() { result = Type::BorrowedUnion(result.union().unwrap()); }
        self.require(result, expected, expression.span)?;
        expression.target = Some(CallTarget::MapMethod(method));
        Ok(result)
    }
}

impl Analyzer {
    #[inline(never)]
    fn map_snapshot_call(&mut self, context:&mut Context, expression:&mut Expr, loans:&Loans, expected:Option<Type>) -> Result<Type,Diagnostic> {
        let ExprKind::Call(callee,arguments)=&mut expression.kind else {unreachable!()};
        let ExprKind::Field(base,name)=&mut callee.kind else {unreachable!()};
        Self::arity(arguments.len(),0,expression.span)?;
        let Type::Map(id)=self.expression(context,base,Mode::Read,loans,None)? else {unreachable!("checked Map snapshot receiver")};
        let map=self.program.records[id].map_definition.as_ref().unwrap().clone();
        if name!="values" {self.require_cloneable(map.key,base.span)?;}
        if name!="keys" {self.require_cloneable(map.value,base.span)?;}
        let (result,method)=if name=="entries" {
            let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
            let mut annotation=crate::type_syntax::ValueAnnotation{span:expression.span,kind:crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(map.key,expression.span)?,types.annotation(map.value,expression.span)?])};
            self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
            append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
            let Type::Record(pair)=self.resolve_value_annotation(&annotation,false)? else {unreachable!("Map entry tuple")};
            (Type::Array(ArrayElement::Record(pair,0)),MapMethod::Entries(pair))
        } else {
            (self.map_method_type(id,name,expression.span)?,if name=="keys"{MapMethod::Keys}else{MapMethod::Values})
        };
        self.require(result,expected,expression.span)?;
        expression.target=Some(CallTarget::MapMethod(method));
        Ok(result)
    }
}

impl Analyzer {
    #[inline(never)]
    fn map_literal_types(&self, context: &Context, source: &Expr) -> Result<Option<(Type, Type)>, Diagnostic> {
        let ExprKind::Array(entries) = &source.kind else { return Ok(None) };
        if entries.is_empty() { return Ok(None); }
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for entry in entries {
            let ExprKind::Array(pair) = &entry.kind else { return Ok(None) };
            if pair.len() != 2 { return Ok(None); }
            let Some(key) = self.expression_type_hint(context, &pair[0]) else { return Ok(None) };
            let Some(value) = self.expression_type_hint(context, &pair[1]) else { return Ok(None) };
            keys.push(key.owning());
            values.push(value.owning());
        }
        // Match ordinary numeric literal inference: int beside number widens.
        for types in [&mut keys, &mut values] {
            if types.contains(&Type::Number) { types.retain(|ty| *ty != Type::Int); }
        }
        Ok(Some((self.unions.intern(keys, source.span)?, self.unions.intern(values, source.span)?)))
    }
    #[inline(never)]
    pub(super) fn map_constructor(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Type, Diagnostic> {
        let ExprKind::MapConstructor(arguments, annotation) = &mut expression.kind else { unreachable!() };
        if arguments.len() > 1 { return Err(Diagnostic::new("arity", "Map accepts zero or one entry source.", expression.span)); }
        if context.names.contains_key("Map") || context.pending_names.contains("Map") || context.lexical_scopes.iter().any(|scope| scope.contains("Map")) || self.function_names.contains_key("Map") {
            return Err(Diagnostic::new("unsupported", "A shadowed Map name cannot construct the built-in collection.", expression.span));
        }
        let declared = annotation.as_ref().map(|annotation| self.resolve_value_annotation(annotation, false)).transpose()?;
        let empty_scoped = arguments.is_empty() && declared.zip(expected).is_some_and(|(declared, expected)| self.scoped_map_contract(expected, declared));
        if let Some(declared) = declared && !empty_scoped { self.require(declared, expected, expression.span)?; }
        let target = if empty_scoped { expected } else { declared.or(expected) };
        let target_types = if let Some(Type::Map(id)) = target {
            let map = self.program.records[id].map_definition.as_ref().unwrap(); Some((map.key, map.value))
        } else if let Some(source) = arguments.first() {
            self.map_literal_types(context, source)?
        } else { None };
        let source_types = if let Some(source) = arguments.first_mut() {
            let mut source_context = None;
            if matches!(source.kind, ExprKind::Array(_)) {
                if let Some((key, value)) = target_types {
                    let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
                    let mut pair = crate::type_syntax::ValueAnnotation { span: source.span, kind: crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(key, source.span)?, types.annotation(value, source.span)?]) };
                    self.generic_records.materialize(&mut pair, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
                    append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
                    let pair = self.resolve_value_annotation(&pair, false)?;
                    source_context = ArrayElement::from_value_type(pair).map(Type::Array);
                }
            }
            let actual = self.expression(context, source, Mode::Read, loans, source_context)?;
            let types = match actual {
                Type::Map(id) => { let map = self.program.records[id].map_definition.as_ref().unwrap(); Some((map.key, map.value)) },
                Type::Array(ArrayElement::Record(id, 0)) if self.program.records[id].is_tuple() && self.program.records[id].fields.len() == 2 => {
                    let fields = &self.program.records[id].fields; Some((fields[0].ty.unwrap(), fields[1].ty.unwrap()))
                },
                Type::Null | Type::Undefined => None,
                _ => return Err(Diagnostic::new("type", "Map construction requires an array of key/value tuples or another Map.", source.span)),
            };
            if let Some((key, value)) = types {
                if let Some((target_key, target_value)) = target_types { self.require(key, Some(target_key), source.span)?; self.require(value, Some(target_value), source.span)?; }
                self.require_cloneable(value, source.span)?;
            }
            types.or(target_types)
        } else { target_types };
        let Some((key, value)) = source_types else { return Err(Diagnostic::new("map-type", "An empty Map needs key/value types or a Map annotation.", expression.span)); };
        if let Some(Type::Map(id)) = target { return Ok(Type::Map(id)); }
        let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
        let mut annotation = crate::type_syntax::ValueAnnotation { span: expression.span, kind: crate::type_syntax::ValueAnnotationKind::Applied {
            name: crate::type_syntax::TypeName { text: "Map".into(), span: expression.span }, arguments: vec![types.annotation(key, expression.span)?, types.annotation(value, expression.span)?],
        } };
        self.generic_records.materialize(&mut annotation, &mut self.program.records, &mut self.record_names, &self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        let result = self.resolve_value_annotation(&annotation, false)?;
        self.require(result, expected, expression.span)?;
        Ok(result)
    }
}

impl Analyzer {
    #[inline(never)]
    pub(super) fn map_iteration_entry(&mut self,id:usize,span:Span)->Result<usize,Diagnostic>{
        let map=self.program.records[id].map_definition.as_ref().unwrap().clone();
        if let Some(pair)=map.entry{return Ok(pair);}
        let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        let mut annotation=crate::type_syntax::ValueAnnotation{span,kind:crate::type_syntax::ValueAnnotationKind::Tuple(vec![types.annotation(map.key,span)?,types.annotation(map.value,span)?])};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let Type::Record(declaration)=self.resolve_value_annotation(&annotation,false)?else{unreachable!("Map iteration tuple")};
        // Annotation round trips retain the tuple's nominal identity, but not
        // the concrete callback environment carried by a Map value parameter.
        let pair=if self.program.records[declaration].fields.iter().filter_map(|field|field.ty).eq([map.key,map.value]) {declaration} else {
            let pair=self.scoped_records.materialize(declaration,vec![map.key,map.value],&mut self.program.records);
            if self.fields.len()<=pair {self.fields.push(self.program.records[pair].fields.iter().map(|field|(field.name.text.clone(),field.ty.unwrap())).collect());}
            pair
        };
        self.program.records[id].map_definition.as_mut().unwrap().entry=Some(pair);
        Ok(pair)
    }
}

impl Analyzer {
    pub(super) fn map_group_by_callee(context:&Context,expression:&Expr)->bool {
        matches!(&expression.kind,ExprKind::Call(callee,_) if matches!(&callee.kind,ExprKind::Field(base,name) if name=="groupBy" && matches!(&base.kind,ExprKind::Name(name) if name=="Map" && !context.names.contains_key(name))))
    }
    #[inline(never)]
    pub(super) fn map_group_by(&mut self,context:&mut Context,expression:&mut Expr,loans:&Loans,expected:Option<Type>)->Result<Type,Diagnostic>{
        let span=expression.span;
        if self.function_names.contains_key("Map")||context.pending_names.contains("Map")||context.lexical_scopes.iter().any(|scope|scope.contains("Map")){return Err(Diagnostic::new("unsupported","A shadowed Map name cannot invoke Map.groupBy.",span));}
        let ExprKind::Call(_,arguments)=&mut expression.kind else{unreachable!()};
        Self::arity(arguments.len(),2,span)?;
        if arguments.iter().any(|argument|matches!(argument.kind,ExprKind::Unary{op:UnaryOp::Spread,..})){return Err(Diagnostic::new("unsupported","Map.groupBy requires a source and callback without spread arguments.",span));}
        let mut arguments=std::mem::take(arguments);let callback=arguments.pop().unwrap();let mut source=arguments.pop().unwrap();
        if self.expression_type_hint(context,&source)==Some(Type::String){
            let source_span=source.span;
            let callee=Expr::new(ExprKind::Field(Box::new(source),"split".into()),source_span);
            source=Expr::new(ExprKind::Call(Box::new(callee),vec![Expr::new(ExprKind::String(String::new()),source_span)]),source_span);
        }
        let mut callbacks=vec![callback];
        let keys=self.array_transform(context,&mut source,&mut callbacks,loans,span,(false,false,Mode::Read,false,false,false,true))?;
        let Type::Array(key)=keys else{unreachable!("group key results")};
        let element=self.iterable_element(source.ty.unwrap()).ok_or_else(||Diagnostic::new("type","Map.groupBy requires an array, string, Set or Map source.",source.span))?.ty();
        self.require_cloneable(element,source.span)?;
        let groups=self.constructor_array_type(element,span)?;
        let types=crate::generic_inference::Types{ callable_annotations: None,callable_shapes:None,inline_shapes:None,records:&self.program.records,names:&self.record_names,unions:&self.unions};
        let mut annotation=crate::type_syntax::ValueAnnotation{span,kind:crate::type_syntax::ValueAnnotationKind::Applied{name:crate::type_syntax::TypeName{text:"Map".into(),span},arguments:vec![types.annotation(key.ty(),span)?,types.annotation(groups,span)?]}};
        self.generic_records.materialize(&mut annotation,&mut self.program.records,&mut self.record_names,&self.unions, &mut self.callables)?;
        append_checked_record_fields(&mut self.program.records,&self.record_names,&self.unions,&mut self.fields,&mut self.callables)?;
        let result=self.resolve_value_annotation(&annotation,false)?;self.require(result,expected,span)?;
        let callee=Expr::new(ExprKind::Field(Box::new(source),"groupBy#map".into()),span);
        expression.kind=ExprKind::Call(Box::new(callee),callbacks);expression.target=Some(CallTarget::MapGroupBy);Ok(result)
    }
}
