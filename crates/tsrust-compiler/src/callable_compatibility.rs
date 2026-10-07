//! Borrowed callable signatures and environment eligibility, without argument copies.
use crate::analysis::{ArgumentContract, CallbackContract, CheckedClosure};
use crate::callable_resolution::ResultContract;
use crate::syntax::Type;
use crate::{Diagnostic, Mode, Span};

#[derive(Clone, Copy)]
pub(crate) enum Parameters<'a> {
    Declared(&'a [ArgumentContract]),
    Closure(&'a CheckedClosure),
}
impl Parameters<'_> {
    pub(crate) fn len(self) -> usize {
        match self {
            Self::Declared(parameters) => parameters.len(),
            Self::Closure(closure) => closure.parameters.len(),
        }
    }
    pub(crate) fn iter(self) -> impl Iterator<Item = ArgumentContract> {
        (0..self.len()).map(move |index| match self {
            Self::Declared(parameters) => parameters[index],
            Self::Closure(closure) => {
                ArgumentContract::from(&closure.function.bindings[closure.parameters[index]])
            }
        })
    }
}

pub(crate) struct Match<'a> {
    pub(crate) actual: Parameters<'a>,
    pub(crate) expected: &'a [ArgumentContract],
    pub(crate) result: Type,
    pub(crate) expected_result: Type,
}

pub(crate) struct Contracts<'a> {
    pub(crate) storage: &'a crate::callable_storage::Registry,
    pub(crate) signatures: &'a crate::callable_contracts::Contracts,
    pub(crate) callbacks: &'a [CallbackContract],
    pub(crate) results: &'a [ResultContract],
    pub(crate) closures: &'a [CheckedClosure],
    pub(crate) effects_only: bool,
}
impl<'a> Contracts<'a> {
    pub(crate) fn matching(
        &self,
        actual: Type,
        expected: Type,
        span: Span,
    ) -> Result<Option<Match<'a>>, Diagnostic> {
        if let Type::Callable(wanted) = expected {
            let contract = &self.results[wanted];
            let signature = match actual {
                Type::Callable(id) => Some((
                    Parameters::Declared(&self.results[id].params),
                    self.results[id].result,
                )),
                Type::Callback(id) if self.callbacks[id].kind.is_owned() => Some((
                    Parameters::Declared(&self.callbacks[id].params),
                    self.callbacks[id].result,
                )),
                Type::StoredCallable(family) => {
                    let declaration=self.storage.declaration(family).expect("checked owning family");
                    let actual=&self.callbacks[declaration];
                    Some((Parameters::Declared(&actual.params),actual.result))
                }
                Type::Closure(id) => {
                    let closure = &self.closures[id];
                    if !self.effects_only && closure.borrowed {
                        return Err(Diagnostic::new(
                            "closure-escape",
                            "A callable result cannot retain borrowed capture storage.",
                            span,
                        )
                        .related(closure.span, "Borrowed environment is created here."));
                    }
                    Some((Parameters::Closure(closure), closure.function.result))
                }
                _ => None,
            };
            return Ok(signature.map(|(actual, result)| Match {
                actual,
                expected: &contract.params,
                result,
                expected_result: contract.result,
            }));
        }
        if let Type::Callback(wanted) = expected {
            let contract = &self.callbacks[wanted];
            let signature = match actual {
                Type::Closure(id) => {
                    let closure = &self.closures[id];
                    let closure = if !self.effects_only && contract.kind.is_owned() {
                        if contract.kind == crate::syntax::CallbackKind::SharedRead {
                            crate::callable_storage::StorageContract::checked_shared(self.signatures, contract.structural)
                        } else {
                            crate::callable_storage::StorageContract::checked(self.signatures, contract.structural)
                        }
                        .expect("resolved owned callback contract")
                        .environment(self.signatures, closure, span)?
                        .closure()
                    } else {
                        if !self.effects_only && contract.kind == crate::syntax::CallbackKind::Read && closure.invocation != crate::Mode::Read {
                            return Err(Diagnostic::new("callback-access", "A ReadOnly<Function<...>> callback cannot mutate or consume its captured environment.", span).related(closure.span, "Callback environment is created here."));
                        }
                        if !self.effects_only
                            && !contract.kind.is_once()
                            && closure.invocation == Mode::Consume
                        {
                            return Err(Diagnostic::new(
                                "callback-once",
                                "A reusable callback cannot consume its captures when invoked.",
                                span,
                            )
                            .related(closure.span, "Single-use closure is created here."));
                        }
                        closure
                    };
                    Some((Parameters::Closure(closure), closure.function.result))
                }
                Type::Callable(id) => {
                    let actual = &self.results[id];
                    let kind = self.signatures.get(actual.structural).kind
                        .unwrap_or(crate::syntax::CallbackKind::Owned);
                    if !kind.can_pass_to(contract.kind) {
                        return Err(Diagnostic::new("callback-contract", "Callable result ownership and invocation contracts cannot be interchanged.", span));
                    }
                    Some((Parameters::Declared(&actual.params), actual.result))
                }
                Type::Callback(id) => {
                    let actual = &self.callbacks[id];
                    if !actual.kind.can_pass_to(contract.kind) {
                        return Err(Diagnostic::new(
                            "callback-contract",
                            "Callback ownership and invocation contracts cannot be interchanged.",
                            span,
                        ));
                    }
                    Some((Parameters::Declared(&actual.params), actual.result))
                }
                Type::StoredCallable(family) => {
                    let actual = &self.callbacks[self.storage.declaration(family).expect("checked stored family")];
                    if actual.kind != crate::syntax::CallbackKind::Owned
                        || !matches!(contract.kind,crate::syntax::CallbackKind::Reusable|crate::syntax::CallbackKind::Owned)
                    {
                        return Err(Diagnostic::new("callback-contract", "Stored reusable callbacks require an Owned<Function<...>> transfer or a Mutable<Function<...>> loan; single-use environments cannot be used as reusable callbacks.", span));
                    }
                    Some((Parameters::Declared(&actual.params), actual.result))
                }
                _ => None,
            };
            return Ok(signature.map(|(actual, result)| Match {
                actual,
                expected: &contract.params,
                result,
                expected_result: contract.result,
            }));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::CallbackKind;

    fn callback(
        shapes: &mut crate::callable_contracts::Contracts,
        kind: CallbackKind,
    ) -> CallbackContract {
        CallbackContract {
            structural: shapes.intern([], Type::Int, Some(kind)),
            parameter_names: Vec::new(),
            params: Vec::new(),
            result: Type::Int,
            kind,
        }
    }

    #[test]
    fn closure_parameter_view_skips_capture_binding_prefix_without_copying() {
        let analysis = crate::check("function main() { const offset = 3; const action = (value: int): int => { return value + offset; }; console.log(action(4)); }").unwrap_or_else(|error| panic!("{error:?}"));
        let closure = &analysis.program.closures[0];
        assert!(closure.parameters[0] > 0);
        let view = Parameters::Closure(closure);
        assert_eq!(view.len(), 1);
        let parameters = view.iter().collect::<Vec<_>>();
        assert_eq!(parameters[0].ty, Type::Int);
        assert_eq!(parameters[0].mode, Mode::Copy);
    }

    #[test]
    fn owned_environment_rejection_retains_use_and_creation_ranges() {
        let analysis = crate::check("function main() { const text = \"Pat\"; const action = (): int => { return text.length; }; console.log(action()); }").unwrap_or_else(|error| panic!("{error:?}"));
        let closure = &analysis.program.closures[0];
        assert!(closure.borrowed);
        let mut shapes = crate::callable_contracts::Contracts::default();
        let callbacks = [callback(&mut shapes, CallbackKind::Owned)];
        let results = [ResultContract {
            structural: shapes.intern([], Type::Int, None),
            params: Vec::new(),
            result: Type::Int,
        }];
        let span = Span::new(1, 3);
        for (expected, code) in [
            (Type::Callback(0), "callback-escape"),
            (Type::Callable(0), "closure-escape"),
        ] {
            let contracts = Contracts {
                storage: &crate::callable_storage::Registry::default(),
                signatures: &shapes,
                callbacks: &callbacks,
                results: &results,
                closures: &analysis.program.closures,
                effects_only: false,
            };
            let error = contracts
                .matching(Type::Closure(0), expected, span)
                .err()
                .expect("borrowed capture rejected");
            assert_eq!(error.code, code);
            assert_eq!(error.span, span);
            assert_eq!(error.related[0].0, closure.span);
            let contracts = Contracts {
                effects_only: true,
                ..contracts
            };
            assert!(
                contracts
                    .matching(Type::Closure(0), expected, span)
                    .unwrap()
                    .is_some()
            );
        }
    }

    #[test]
    fn consuming_environments_require_once_but_callback_kinds_still_match_exactly() {
        let analysis = crate::check("type User = { name: string; }; function main() { const user: User = { name: \"Pat\" }; const action = (): User => { return user; }; const saved = action(); console.log(saved.name); }").unwrap_or_else(|error| panic!("{error:?}"));
        assert_eq!(analysis.program.closures[0].invocation, Mode::Consume);
        let mut shapes = crate::callable_contracts::Contracts::default();
        let callbacks = [
            callback(&mut shapes, CallbackKind::Owned),
            callback(&mut shapes, CallbackKind::OwnedOnce),
        ];
        let contracts = Contracts {
            storage: &crate::callable_storage::Registry::default(),
            signatures: &shapes,
            callbacks: &callbacks,
            results: &[],
            closures: &analysis.program.closures,
            effects_only: false,
        };
        let span = Span::new(1, 3);
        assert_eq!(
            contracts
                .matching(Type::Closure(0), Type::Callback(0), span)
                .err()
                .unwrap()
                .code,
            "callback-once"
        );
        assert!(
            contracts
                .matching(Type::Closure(0), Type::Callback(1), span)
                .unwrap()
                .is_some()
        );
        for effects_only in [false, true] {
            let contracts = Contracts {
                effects_only,
                ..contracts
            };
            assert_eq!(
                contracts
                    .matching(Type::Callback(0), Type::Callback(1), span)
                    .err()
                    .unwrap()
                    .code,
                "callback-contract"
            );
            assert!(
                contracts
                    .matching(Type::Int, Type::String, span)
                    .unwrap()
                    .is_none()
            );
        }
    }
}
