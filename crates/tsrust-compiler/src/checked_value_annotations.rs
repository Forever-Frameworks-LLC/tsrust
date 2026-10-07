//! Reconstruct concrete callable contracts without discarding parameter metadata.
use super::*;
use crate::type_syntax::{CallableOwnership, CallableParameter as StructuredParameter, ParameterAccess, TypeName, ValueAnnotation, ValueAnnotationKind as Kind};

impl Analyzer {
    pub(super) fn checked_inference_callable_annotations(&self, roots: impl IntoIterator<Item = Type>) -> Result<HashMap<Type, ValueAnnotation>, Diagnostic> {
        self.checked_inference_callable_shapes(roots).keys().copied().map(|ty| {
            Ok((ty, self.checked_value_annotation(ty, Span::default())?))
        }).collect()
    }

    pub(super) fn checked_inference_callable_shapes(&self, roots: impl IntoIterator<Item = Type>) -> crate::generic_inference::CallableShapes {
        let mut shapes = crate::generic_inference::CallableShapes::new();
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        let mut visited = HashSet::new();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) { continue; }
            if let Some(id) = ty.union() { pending.extend(self.unions.members(id)); }
            if let Some((parameters, result)) = self.checked_generic_callable_shape(ty) {
                pending.extend(parameters.iter().copied());
                pending.push(result);
                shapes.insert(ty, (parameters, result));
            }
        }
        shapes
    }

    pub(super) fn checked_value_annotation(&self, ty: Type, span: Span) -> Result<ValueAnnotation, Diagnostic> {
        self.checked_value_annotation_at(ty, span, 0)
    }

    fn checked_value_annotation_at(&self, ty: Type, span: Span, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        if depth >= 128 {
            return Err(Diagnostic::new("unsupported", "Checked callable annotation nesting exceeds the limit of 128.", span));
        }
        let contract = match ty {
            Type::Callable(id) => {
                let contract = &self.callables.results[id];
                Some((contract.params.clone(), Vec::new(), contract.result,
                    self.callables.shapes.get(contract.structural).kind.unwrap_or(CallbackKind::Owned)))
            }
            Type::Callback(id) => {
                let contract = &self.callables.callbacks[id];
                Some((contract.params.clone(), contract.parameter_names.clone(), contract.result, contract.kind))
            }
            Type::StoredCallable(family) => {
                let id = self.callables.storage.declaration(family).ok_or_else(|| Diagnostic::new("callable-layout", "Unknown callable storage signature.", span))?;
                let contract = &self.callables.callbacks[id];
                Some((contract.params.clone(), contract.parameter_names.clone(), contract.result, contract.kind))
            }
            Type::Closure(id) if !self.deferred_generic_values.contains_key(&id) => {
                let closure = &self.closures[id];
                let bindings = closure.parameters.iter().map(|id| &closure.function.bindings[*id]).collect::<Vec<_>>();
                Some((bindings.iter().map(|binding| ArgumentContract::from(*binding)).collect(),
                    bindings.iter().map(|binding| binding.name.clone()).collect(), closure.function.result,
                    if closure.borrowed {
                        match closure.invocation {
                            Mode::Consume => CallbackKind::Once,
                            Mode::Mutate => CallbackKind::Reusable,
                            Mode::Read | Mode::Copy => CallbackKind::Read,
                        }
                    } else if closure.invocation == Mode::Consume { CallbackKind::OwnedOnce } else { CallbackKind::Owned }))
            }
            _ => None,
        };
        let kind = if let Some((parameters, names, result, kind)) = contract {
            let ownership = match kind {
                CallbackKind::Read => CallableOwnership::Read,
                CallbackKind::Reusable => CallableOwnership::Reusable,
                CallbackKind::Once => CallableOwnership::Once,
                CallbackKind::Owned => CallableOwnership::Owned,
                CallbackKind::OwnedOnce => CallableOwnership::OwnedOnce,
                CallbackKind::SharedRead => CallableOwnership::SharedRead,
            };
            let parameters = parameters.iter().enumerate().map(|(index, parameter)| Ok(StructuredParameter {
                rest: parameter.rest,
                name: TypeName { text: names.get(index).cloned().unwrap_or_else(|| format!("argument{index}")), span },
                access: (parameter.mode != Mode::Copy && self.checked_generic_callable_shape(self.unions.payload(parameter.ty).unwrap_or(parameter.ty)).is_none()).then_some(ParameterAccess { mode: parameter.mode, span }),
                omission: parameter.omission,
                annotation: self.checked_value_annotation_at(parameter.ty, span, depth + 1)?,
                span,
            })).collect::<Result<Vec<_>, Diagnostic>>()?;
            Kind::Callable { ownership, parameters, result: Box::new(self.checked_value_annotation_at(result, span, depth + 1)?) }
        } else {
            match ty {
                Type::Void => Kind::Named(TypeName { text: "void".into(), span }),
                Type::Array(element) => Kind::Array(Box::new(self.checked_value_annotation_at(element.ty(), span, depth + 1)?)),
                Type::Union(id) => Kind::Union(self.unions.members(id).into_iter().map(|ty| self.checked_value_annotation_at(ty, span, depth + 1)).collect::<Result<Vec<_>, _>>()?),
                _ => {
                    let types = crate::generic_inference::Types { callable_annotations: None, callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
                    return types.annotation(ty, span);
                }
            }
        };
        Ok(ValueAnnotation { kind, span })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concrete_borrowed_closure_reconstruction_retains_borrowed_invocation() {
        let source = "function main(){let count=0;const action=(value:int):int=>{count+=value;return count;};console.log(action(2));}";
        let checked = crate::check(source).unwrap();
        let mut analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        analyzer.closures = checked.program.closures;
        let closure = &analyzer.closures[0];
        assert!(closure.borrowed);
        assert_eq!(closure.invocation, Mode::Mutate);
        let annotation = analyzer.checked_value_annotation(Type::Closure(0), Span::default()).unwrap();
        assert_eq!(crate::callable_annotations::ownership(&annotation), Some(CallbackKind::Reusable));
    }

    #[test]
    fn checked_callable_reconstruction_preserves_shared_and_parameter_metadata() {
        let source = "function inspect(action:Function<int,[value?:int,...tail:int[]]>):void{}function main(){}";
        let mut analyzer = Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
        let mut declaration = analyzer.program.functions[0].params[0].2.clone().unwrap();
        for kind in [CallbackKind::Read, CallbackKind::Reusable, CallbackKind::Once, CallbackKind::Owned, CallbackKind::OwnedOnce, CallbackKind::SharedRead] {
            declaration.kind = kind;
            let Type::Callback(original) = analyzer.resolve_callback(&declaration).unwrap() else { panic!("callback") };
            let annotation = analyzer.checked_value_annotation(Type::Callback(original), declaration.span).unwrap();
            let restored = crate::callable_annotations::callback(&annotation).unwrap();
            assert_eq!(restored.kind, kind);
            assert_eq!(restored.params[0].name.text, "value");
            assert!(restored.params[0].initializer.omission().is_omittable());
            assert!(restored.params[1].rest);
            assert_eq!(restored.params[1].name.text, "tail");
            let Type::Callback(round_trip) = analyzer.resolve_callback(&restored).unwrap() else { panic!("callback") };
            assert_eq!(analyzer.callables.callbacks[original].structural, analyzer.callables.callbacks[round_trip].structural);
        }
    }
}
