//! Owning record carrier conversion evaluates its input once and moves leaves.
use super::*;
impl Emitter<'_> {
    /// Empty containers have no payload to convert. Construct them directly in
    /// the checked destination carrier; retain normal record literal emission
    /// for field order, optional defaults and once-only initializer evaluation.
    #[inline(never)]
    pub(super) fn contextual_record_carrier(
        &self,
        fun: &CheckedFunction,
        value: &Expr,
        target: Type,
    ) -> Option<String> {
        let mut value = value;
        while let ExprKind::Unary {
            op: UnaryOp::RecordCarrier,
            value: inner,
            ..
        } = &value.kind
        {
            value = inner;
        }
        if matches!(&value.kind,ExprKind::MapConstructor(arguments,_) if arguments.is_empty())
            && matches!(target, Type::Map(_))
        {
            let mut contextual = value.clone();
            contextual.ty = Some(target);
            return Some(self.expr(fun, &contextual, Mode::Consume));
        }
        if matches!(&value.kind, ExprKind::NativeConstructor(arguments, _) if arguments.is_empty())
            && matches!(target, Type::Record(id) if self.program.records[id].native_element().is_some())
        {
            let mut contextual = value.clone();
            contextual.ty = Some(target);
            return Some(self.expr(fun, &contextual, Mode::Consume));
        }
        let (ExprKind::Record(fields), Type::Record(record)) = (&value.kind, target) else {
            return None;
        };
        let mut contextual = value.clone();
        let ExprKind::Record(contextual_fields) = &mut contextual.kind else {
            unreachable!()
        };
        for ((name, source), (_, child)) in fields.iter().zip(contextual_fields) {
            let index = *self.fields[record]
                .get(name.text.as_str())
                .expect("checked carrier field");
            let destination = self.program.records[record].fields[index]
                .ty
                .expect("checked carrier field type");
            let source_type = self
                .owned_record_source_type(fun, source)
                .unwrap_or_else(|| source.ty.expect("checked initializer"));
            if source_type != destination {
                child.wrap_record_carrier(destination);
            }
        }
        contextual.ty = Some(target);
        Some(self.expr(fun, &contextual, Mode::Consume))
    }
    pub(super) fn record_carrier_conversion(
        &self,
        source: Type,
        target: Type,
        value: &str,
    ) -> String {
        if target.accepts_scalar_value(source) {
            return value.into();
        }
        match (source, target) {
            (Type::Union(source), target) if matches!(target,Type::String|Type::Int|Type::Number|Type::Boolean)
                && self.program.unions[source.index()].members.iter().all(|&member|target.accepts_scalar_value(member))=>{
                let arms=(0..self.program.unions[source.index()].members.len()).map(|tag|format!("Union{}::V{tag}(payload)=>payload",source.index())).collect::<Vec<_>>().join(",");format!("match {value}{{{arms}}}")
            }
            (Type::StoredCallable(source),Type::StoredCallable(target))
                if self.program.stored_contracts.scoped_native_family(source).is_some()
                    && self.program.stored_contracts.scoped_native_family(target).is_some()=> {
                let layout=self.program.stored_contracts.scoped_native_family(source).unwrap();
                let arms=layout.alternatives.iter().map(|alternative|format!("NativeViewCallable{source}::V{0}(environment)=>NativeViewCallable{target}::V{0}(environment)",alternative.origin)).collect::<Vec<_>>().join(",");
                format!("match {value}{{{arms}}}")
            }
            // The written family of a view callback has no environment, so a value
            // of it cannot exist and every arm of the conversion is absent.
            (Type::StoredCallable(source),Type::StoredCallable(target))
                if self.program.stored_contracts.scoped_native_family(target).is_some()
                    && self.program.stored_contracts.origins(source).is_some_and(|origins|origins.is_empty())=>format!("match {value}{{}}"),
            (Type::Union(source), target @ Type::Array(_)) => {
                let arms = self.program.unions[source.index()].members.iter().enumerate()
                    .map(|(tag, &member)| {
                        let payload = self.union_move_payload(source, member, "payload");
                        format!("Union{}::V{tag}(payload)=>{}", source.index(),
                            self.record_carrier_conversion(member, target, &payload))
                    }).collect::<Vec<_>>().join(",");
                format!("{{let array_carrier: {} = {value};match array_carrier{{{arms}}}}}",
                    self.native_type(Type::Union(source)))
            }
            (Type::Array(source), Type::Array(target)) => {
                let converted =
                    self.record_carrier_conversion(source.ty(), target.ty(), "record_array_item");
                format!(
                    "{{let mut record_array_source={value};let mut record_array_result:{}=From::from([]);while let Some(record_array_item)=record_array_source.pop(){{record_array_result.push({converted});}}record_array_result.reverse();record_array_result}}",
                    self.native_type(Type::Array(target))
                )
            }
            (Type::Record(source), Type::Record(target)) => {
                let fields = self.program.records[source]
                    .fields
                    .iter()
                    .zip(&self.program.records[target].fields)
                    .enumerate()
                    .map(|(index, (source, target))| {
                        format!(
                            "field{index}:{}",
                            self.record_carrier_conversion(
                                source.ty.unwrap(),
                                target.ty.unwrap(),
                                &format!("({value}).field{index}")
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let metadata = if self.record_runtime.ordered.contains(&target) {
                    format!(",key_order:({value}).key_order")
                } else {
                    String::new()
                };
                format!(
                    "Record{}{{{fields}{metadata}}}",
                    self.native_record_declaration(target)
                )
            }
            // No value of an absent callback environment exists at run time.
            (Type::Callback(id), _) if self.program.absent_callback_payloads.contains(&id) => {
                "unreachable!(\"absent callback environment\")".into()
            }
            (Type::Union(source), Type::Union(target)) => {
                // A nullable record joins environments; other members keep their value.
                let targets = &self.program.unions[target.index()].members;
                let arms = self.program.unions[source.index()]
                    .members
                    .iter()
                    .enumerate()
                    .map(|(index, &member)| {
                        let callable = |ty| {
                            matches!(
                                ty,
                                Type::Callback(_)
                                    | Type::Closure(_)
                                    | Type::StoredCallable(_)
                                    | Type::BorrowedCallable(_)
                            )
                        };
                        let to = targets
                            .iter()
                            .position(|&candidate| candidate.accepts_scalar_value(member))
                            .or_else(|| targets.iter().position(|&candidate| {
                                matches!((candidate, member), (Type::Array(_), Type::Array(_)))
                                    || matches!(
                                        (candidate, member),
                                        (Type::Record(_), Type::Record(_))
                                    )
                                    || (callable(candidate) && callable(member))
                            }))
                            .expect("checked nullable carrier member");
                        format!(
                            "Union{}::V{index}(payload)=>Union{}::V{to}({})",
                            source.index(),
                            target.index(),
                            if targets[to].accepts_scalar_value(member) {
                                self.union_transfer_payload(source, target, member, "payload")
                            } else { self.record_carrier_conversion(member, targets[to], "payload") }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                // Name the source type: an absent payload leaves its parameter unconstrained.
                format!(
                    "{{let nullable_carrier: {} = {value}; match nullable_carrier{{{arms}}}}}",
                    self.native_type(Type::Union(source))
                )
            }
            (source, Type::Union(target)) => {
                let targets = &self.program.unions[target.index()].members;
                let tag = targets.iter().position(|&member| member.accepts_scalar_value(source))
                    .or_else(|| targets.iter().position(|member| matches!((source, member), (Type::Array(_), Type::Array(_)))))
                    .expect("checked array element alternative");
                let converted = self.record_carrier_conversion(source, targets[tag], value);
                format!("Union{}::V{tag}({})", target.index(), self.union_store_payload(target, targets[tag], &converted))
            }
            (Type::Closure(_) | Type::BorrowedCallable(_), Type::Callback(id)) => {
                format!("C{id}::from({value})")
            }
            (Type::Closure(origin), Type::BorrowedCallable(family)) => {
                self.stored_callable_constructor(family, origin, value)
            }
            (Type::BorrowedCallable(source), Type::BorrowedCallable(target)) => {
                let payloads = self.borrowed_payloads(source);
                let arms = payloads
                    .iter()
                    .enumerate()
                    .map(|(index, payload)| {
                        format!(
                            "BorrowedCallable{source}::{}(payload)=>{}",
                            crate::borrowed_callable_dispatch::variant(index, payloads.len()),
                            self.stored_callable_constructor(target, payload.origin, "payload")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("match {value}{{{arms}}}")
            }
            _ => panic!("unchecked record carrier conversion {source:?} to {target:?}"),
        }
    }
}
