//! Structural callable identity, separate from environment and ownership-path IDs.
use crate::analysis::Mode;
use crate::syntax::{CallbackKind, Type};
use crate::type_syntax::ParameterAccess;
use crate::{Diagnostic, Span};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) fn parameter_name<'a>(
    name: &'a str,
    span: Span,
    names: &mut HashSet<&'a str>,
) -> Result<(), Diagnostic> {
    if !names.insert(name)
        || [
            "clone", "String", "Boolean", "Array", "console", "Number", "Math", "null", "typeof",
            "void",
        ]
        .contains(&name)
    {
        return Err(Diagnostic::new(
            "duplicate",
            "Duplicate or reserved callable parameter name.",
            span,
        ));
    }
    Ok(())
}

pub(crate) fn parameter_mode(
    ty: Type,
    callback: Option<CallbackKind>,
    explicit: bool,
    access: Option<&ParameterAccess>,
    span: Span,
) -> Result<Mode, Diagnostic> {
    if ty == Type::Void {
        return Err(Diagnostic::new(
            "unsupported",
            "Closure arguments cannot have type void.",
            span,
        ));
    }
    if let Some(kind) = callback {
        if let Some(access) = access {
            return Err(Diagnostic::new(
                "callback-access",
                "A Function argument states its ownership with ReadOnly<Function<...>>, Mutable<Function<...>> or Once<Function<...>>, not a data access wrapper.",
                access.span,
            ));
        }
        return Ok(if kind.is_owned() || kind.is_once() {
            Mode::Consume
        } else {
            kind.invocation()
        });
    }
    if ty.is_copy() || !explicit {
        if let Some(access) = access {
            return Err(Diagnostic::new(
                "callback-access",
                "ReadOnly, Mutable and Owned apply only to data-owning arguments in a Function type; arrow bodies infer access and scalars copy.",
                access.span,
            ));
        }
        return Ok(if ty.is_copy() { Mode::Copy } else { Mode::Read });
    }
    let access = access
        .filter(|access| access.mode != Mode::Copy)
        .ok_or_else(|| {
            Diagnostic::new(
                "callback-access",
                "A Function argument that owns data needs ReadOnly<T>, Mutable<T> or Owned<T>, such as Function<int, [ReadOnly<string>]>.",
                span,
            )
        })?;
    Ok(access.mode)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ContractId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ContractType {
    Data(Type),
    Callable(ContractId),
}

impl From<Type> for ContractType {
    fn from(ty: Type) -> Self {
        Self::Data(ty)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Shape {
    parameters: Vec<(ContractType, Mode)>,
    minimum: usize,
    rest: bool,
    result: ContractType,
    // None denotes a result annotation, whose invocation mode is checked later.
    pub(crate) kind: Option<CallbackKind>,
}

#[derive(Default)]
pub(crate) struct Contracts {
    shapes: HashMap<Arc<Shape>, ContractId>,
    ordered: Vec<Arc<Shape>>,
}

impl Contracts {
    #[cfg(test)]
    pub(crate) fn intern<T: Into<ContractType>>(
        &mut self,
        parameters: impl IntoIterator<Item = (T, Mode)>,
        result: T,
        kind: Option<CallbackKind>,
    ) -> ContractId {
        let parameters = parameters.into_iter().map(|(ty,mode)|(ty.into(),mode)).collect::<Vec<_>>();
        let minimum = parameters.len();
        self.intern_arity(parameters, result.into(), kind, minimum)
    }
    #[cfg(test)]
    pub(crate) fn intern_arity(&mut self, parameters: Vec<(ContractType, Mode)>, result: ContractType, kind: Option<CallbackKind>, minimum: usize) -> ContractId {
        self.intern_rest(parameters,result,kind,minimum,false)
    }
    pub(crate) fn intern_rest(&mut self, parameters: Vec<(ContractType, Mode)>, result: ContractType, kind: Option<CallbackKind>, minimum: usize, rest: bool) -> ContractId {
        let shape = Shape { parameters, minimum, rest, result, kind };
        if let Some(id) = self.shapes.get(&shape) {
            return *id;
        }
        let id = ContractId(self.ordered.len());
        let shape = Arc::new(shape);
        self.ordered.push(Arc::clone(&shape));
        self.shapes.insert(shape, id);
        id
    }

    pub(crate) fn get(&self, id: ContractId) -> &Shape {
        &self.ordered[id.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_names_preserve_order_case_and_original_error_ranges() {
        let mut names = HashSet::new();
        let first = Span::new(3, 8);
        let second = Span::new(20, 25);
        parameter_name("value", first, &mut names).unwrap();
        parameter_name("Value", second, &mut names).unwrap();
        parameter_name("entrée", second, &mut names).unwrap();
        let error = parameter_name("value", second, &mut names).unwrap_err();
        assert_eq!(error.code, "duplicate");
        assert_eq!(error.span, second);
        for name in [
            "clone", "String", "Boolean", "Array", "console", "Number", "Math", "null", "typeof",
            "void",
        ] {
            let error = parameter_name(name, first, &mut HashSet::new()).unwrap_err();
            assert_eq!(error.code, "duplicate");
            assert_eq!(error.span, first);
        }
    }

    #[test]
    fn ordinary_checker_keeps_parameter_error_precedence() {
        let source = "function apply(action: Function<int, [value: void, value: int]>): int { return action(1); }";
        let error = crate::check(source).err().expect("void parameter rejected");
        assert_eq!(error.code, "unsupported");
        let start = source.find("void").unwrap();
        assert_eq!(error.span, Span::new(start, start + 4));

        let source = "function apply(action: Function<int, [value: int, value: int]>): int { return action(1, 2); }";
        let error = crate::check(source).err().expect("duplicate parameter rejected");
        assert_eq!(error.code, "duplicate");
        let start = source.rfind("value:").unwrap();
        assert_eq!(error.span, Span::new(start, start + 5));
    }

    #[test]
    fn parameter_access_uses_one_rule_for_data_and_nested_contracts() {
        let span = Span::new(5, 11);
        assert_eq!(
            parameter_mode(Type::Int, None, true, None, span).unwrap(),
            Mode::Copy
        );
        assert_eq!(
            parameter_mode(Type::String, None, false, None, span).unwrap(),
            Mode::Read
        );
        for mode in [Mode::Read, Mode::Mutate, Mode::Consume] {
            let access = ParameterAccess { mode, span };
            assert_eq!(
                parameter_mode(Type::String, None, true, Some(&access), span).unwrap(),
                mode
            );
            let error = parameter_mode(Type::Int, None, true, Some(&access), span).unwrap_err();
            assert_eq!(error.code, "callback-access");
            assert_eq!(error.span, span);
        }
        assert_eq!(
            parameter_mode(Type::String, None, true, None, span)
                .unwrap_err()
                .code,
            "callback-access"
        );
        assert_eq!(
            parameter_mode(Type::Void, None, true, None, span)
                .unwrap_err()
                .code,
            "unsupported"
        );
        for (kind, mode) in [
            (CallbackKind::Reusable, Mode::Mutate),
            (CallbackKind::Once, Mode::Consume),
            (CallbackKind::Owned, Mode::Consume),
            (CallbackKind::OwnedOnce, Mode::Consume),
        ] {
            assert_eq!(
                parameter_mode(Type::Callback(0), Some(kind), true, None, span).unwrap(),
                mode
            );
        }
    }

    #[test]
    fn ordered_lookup_preserves_ids_and_shares_the_signature_allocation() {
        let mut contracts = Contracts::default();
        let original = contracts.intern(
            [(Type::Int, Mode::Copy)],
            Type::Int,
            Some(CallbackKind::Owned),
        );
        let expected = contracts.get(original).clone();
        for id in 0..64 {
            contracts.intern(
                [(Type::Record(id), Mode::Read)],
                Type::String,
                Some(CallbackKind::Reusable),
            );
        }
        assert_eq!(contracts.get(original), &expected);
        assert_eq!(
            original,
            contracts.intern(
                [(Type::Int, Mode::Copy)],
                Type::Int,
                Some(CallbackKind::Owned)
            )
        );
        let (key, id) = contracts.shapes.get_key_value(&expected).unwrap();
        assert!(Arc::ptr_eq(key, &contracts.ordered[id.0]));
    }

    #[test]
    fn nested_contracts_reuse_child_structure_and_preserve_child_ownership() {
        let mut contracts = Contracts::default();
        let child = contracts.intern(
            [(Type::Int, Mode::Copy)],
            Type::Int,
            Some(CallbackKind::Owned),
        );
        let repeated = contracts.intern(
            [(Type::Int, Mode::Copy)],
            Type::Int,
            Some(CallbackKind::Owned),
        );
        let once = contracts.intern(
            [(Type::Int, Mode::Copy)],
            Type::Int,
            Some(CallbackKind::OwnedOnce),
        );
        let outer = contracts.intern(
            [(ContractType::Callable(child), Mode::Consume)],
            ContractType::Callable(child),
            None,
        );
        assert_eq!(
            outer,
            contracts.intern(
                [(ContractType::Callable(repeated), Mode::Consume)],
                ContractType::Callable(repeated),
                None
            )
        );
        assert_ne!(
            outer,
            contracts.intern(
                [(ContractType::Callable(once), Mode::Consume)],
                ContractType::Callable(once),
                None
            )
        );
    }

    #[test]
    fn contracts_reuse_structure_but_preserve_nominals_modes_order_and_ownership() {
        let mut contracts = Contracts::default();
        let parameters = [(Type::Record(0), Mode::Read), (Type::Int, Mode::Copy)];
        let original = contracts.intern(parameters, Type::String, Some(CallbackKind::Owned));
        assert_eq!(
            original,
            contracts.intern(parameters, Type::String, Some(CallbackKind::Owned))
        );
        for (parameters, result, kind) in [
            (
                [(Type::Record(1), Mode::Read), (Type::Int, Mode::Copy)],
                Type::String,
                Some(CallbackKind::Owned),
            ),
            (
                [(Type::Record(0), Mode::Mutate), (Type::Int, Mode::Copy)],
                Type::String,
                Some(CallbackKind::Owned),
            ),
            (
                [(Type::Int, Mode::Copy), (Type::Record(0), Mode::Read)],
                Type::String,
                Some(CallbackKind::Owned),
            ),
            (parameters, Type::Int, Some(CallbackKind::Owned)),
            (parameters, Type::String, Some(CallbackKind::OwnedOnce)),
            (parameters, Type::String, None),
        ] {
            assert_ne!(original, contracts.intern(parameters, result, kind));
        }
        assert_eq!(contracts.shapes.len(), 7);
        assert_eq!(
            original,
            contracts.intern(parameters, Type::String, Some(CallbackKind::Owned))
        );
    }
}
