//! Returned native borrows identify their source argument and exact result slot.
//! No source spelling or checked-arena index participates in the contract.

use crate::{
    Mode,
    native_bindings::NativeBinding,
    native_signatures::{NativeTypeId, Signature},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowAccess {
    ReadOnly,
    Exclusive,
}

/// Identifies the payload that carries the loan. Err never carries this loan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnSlot {
    Direct,
    ResultOk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReturnedBorrow {
    pub source_parameter: usize,
    pub access: BorrowAccess,
    pub payload: NativeTypeId,
    pub slot: ReturnSlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError {
    MissingSource,
    OptionalSource,
    RestSource,
    SourceNotBorrowed,
    SourceWithoutStorage,
    ExclusiveSourceRequired,
    ExpectedResult,
    PayloadMismatch,
    MissingReturnContract,
    SignatureLimit,
    PayloadNotBorrowed,
    PayloadAccessMismatch,
    ReadOnlySource,
    BorrowedErrorPayload,
}

impl NativeBinding {
    pub fn contains_borrowed_signatures(
        &self,
    ) -> Result<bool, crate::native_signatures::RenderError> {
        for ty in self
            .parameters
            .iter()
            .map(|parameter| parameter.ty)
            .chain(std::iter::once(self.result))
        {
            if ty.signature().contains_borrowed_native()? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Validate identities and access before a declaration reaches type projection.
    /// This checks metadata, not whether the backend supports its lifetime ABI.
    pub fn validate_returned_borrow(&self) -> Result<(), ContractError> {
        let Some(contract) = self.returned_borrow else {
            if self
                .result
                .signature()
                .contains_borrowed_native()
                .map_err(|_| ContractError::SignatureLimit)?
            {
                return Err(ContractError::MissingReturnContract);
            }
            return Ok(());
        };
        let source = self
            .parameters
            .get(contract.source_parameter)
            .ok_or(ContractError::MissingSource)?;
        // Arity checking stops at the first optional/rest parameter. A later
        // required-looking parameter therefore cannot be a guaranteed source.
        let required_prefix = &self.parameters[..=contract.source_parameter];
        if required_prefix.iter().any(|parameter| parameter.optional) {
            return Err(ContractError::OptionalSource);
        }
        if required_prefix.iter().any(|parameter| parameter.rest) {
            return Err(ContractError::RestSource);
        }
        if !matches!(source.access, Mode::Read | Mode::Mutate) {
            return Err(ContractError::SourceNotBorrowed);
        }
        if contract.access == BorrowAccess::Exclusive && source.access != Mode::Mutate {
            return Err(ContractError::ExclusiveSourceRequired);
        }
        if !matches!(
            source.ty.signature(),
            Signature::Named(_) | Signature::Array(_) | Signature::String
        ) {
            return Err(ContractError::SourceWithoutStorage);
        }
        let signature = self.result.signature();
        let payload = match (contract.slot, &signature) {
            (ReturnSlot::Direct, signature) => signature,
            (ReturnSlot::ResultOk, Signature::Result(value, error)) => {
                if error
                    .contains_borrowed_native()
                    .map_err(|_| ContractError::SignatureLimit)?
                {
                    return Err(ContractError::BorrowedErrorPayload);
                }
                value
            }
            (ReturnSlot::ResultOk, _) => return Err(ContractError::ExpectedResult),
        };
        if *payload != Signature::Named(contract.payload) {
            return Err(ContractError::PayloadMismatch);
        }
        let access = crate::native_types::declaration(contract.payload)
            .borrow_access()
            .ok_or(ContractError::PayloadNotBorrowed)?;
        if access != contract.access {
            return Err(ContractError::PayloadAccessMismatch);
        }
        if contract.access == BorrowAccess::Exclusive {
            if let Signature::Named(id) = source.ty.signature() {
                if crate::native_types::declaration(id).borrow_access()
                    == Some(BorrowAccess::ReadOnly)
                {
                    return Err(ContractError::ReadOnlySource);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Mode,
        native_bindings::{NativeBinding, NativeParameter, NativeType},
        native_signatures::{NativeTypeId, Signature},
    };

    static OWNER: Signature = Signature::Named(NativeTypeId::Bytes);
    static VIEW: Signature = Signature::Named(NativeTypeId::ByteView);
    static MUTABLE_VIEW: Signature = Signature::Named(NativeTypeId::ByteViewMut);
    static RESULT: Signature = Signature::Result(&VIEW, &Signature::Named(NativeTypeId::ByteError));
    static MUTABLE_RESULT: Signature =
        Signature::Result(&MUTABLE_VIEW, &Signature::Named(NativeTypeId::ByteError));
    static OWNED_RESULT: Signature =
        Signature::Result(&OWNER, &Signature::Named(NativeTypeId::ByteError));

    fn binding(access: Mode, optional: bool, rest: bool) -> NativeBinding {
        NativeBinding {
            parameters: Box::leak(Box::new([NativeParameter {
                name: "source",
                ty: NativeType::Structured(&OWNER),
                access,
                optional,
                rest,
            }])),
            result: NativeType::Structured(&RESULT),
            returned_borrow: Some(ReturnedBorrow {
                source_parameter: 0,
                access: BorrowAccess::ReadOnly,
                payload: NativeTypeId::ByteView,
                slot: ReturnSlot::ResultOk,
            }),
            ..crate::stdlib::bytes::bindings::COPY
        }
    }

    #[test]
    fn returned_borrow_requires_a_required_borrowed_source() {
        assert_eq!(
            binding(Mode::Read, false, false).validate_returned_borrow(),
            Ok(())
        );
        assert_eq!(
            binding(Mode::Mutate, false, false).validate_returned_borrow(),
            Ok(())
        );
        for mode in [Mode::Consume, Mode::Copy] {
            assert_eq!(
                binding(mode, false, false).validate_returned_borrow(),
                Err(ContractError::SourceNotBorrowed)
            );
        }
        assert_eq!(
            binding(Mode::Read, true, false).validate_returned_borrow(),
            Err(ContractError::OptionalSource)
        );
        assert_eq!(
            binding(Mode::Read, false, true).validate_returned_borrow(),
            Err(ContractError::RestSource)
        );
        let mut probe = binding(Mode::Read, false, false);
        let source = probe.parameters[0];
        probe.parameters = Box::leak(Box::new([
            NativeParameter {
                optional: true,
                ..source
            },
            source,
        ]));
        probe.returned_borrow.as_mut().unwrap().source_parameter = 1;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::OptionalSource)
        );
        probe.parameters = Box::leak(Box::new([
            NativeParameter {
                rest: true,
                ..source
            },
            source,
        ]));
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::RestSource)
        );
        probe.parameters = binding(Mode::Read, false, false).parameters;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::MissingSource)
        );
    }

    #[test]
    fn exclusive_return_requires_exclusive_source_and_exact_payload_slot() {
        let mut probe = binding(Mode::Read, false, false);
        probe.returned_borrow.as_mut().unwrap().access = BorrowAccess::Exclusive;
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::ByteViewMut;
        probe.result = NativeType::Structured(&MUTABLE_RESULT);
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::ExclusiveSourceRequired)
        );
        probe.parameters = binding(Mode::Mutate, false, false).parameters;
        assert_eq!(probe.validate_returned_borrow(), Ok(()));
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::ByteError;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::PayloadMismatch)
        );
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::ByteViewMut;
        probe.returned_borrow.as_mut().unwrap().slot = ReturnSlot::Direct;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::PayloadMismatch)
        );
        probe.result = NativeType::Structured(&MUTABLE_VIEW);
        assert_eq!(probe.validate_returned_borrow(), Ok(()));
        probe.returned_borrow.as_mut().unwrap().slot = ReturnSlot::ResultOk;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::ExpectedResult)
        );
    }

    #[test]
    fn borrowed_source_must_have_a_storage_signature() {
        let mut probe = binding(Mode::Read, false, false);
        for ty in [
            NativeType::Int,
            NativeType::Boolean,
            NativeType::Void,
            NativeType::ObservedValue,
        ] {
            probe.parameters = Box::leak(Box::new([NativeParameter {
                ty,
                ..probe.parameters[0]
            }]));
            assert_eq!(
                probe.validate_returned_borrow(),
                Err(ContractError::SourceWithoutStorage)
            );
        }
    }

    #[test]
    fn projection_rejects_invalid_or_unimplemented_borrow_contracts_before_interning() {
        let mut probe = binding(Mode::Read, false, false);
        let mut records = Vec::new();
        probe.returned_borrow.as_mut().unwrap().source_parameter = 1;
        let error =
            crate::native_binding_types::prepare(&mut records, &probe, crate::Span::default())
                .unwrap_err();
        assert_eq!(error.code, "native-return-borrow-contract");
        assert!(records.is_empty());
        probe.returned_borrow.as_mut().unwrap().source_parameter = 0;
        let error =
            crate::native_binding_types::prepare(&mut records, &probe, crate::Span::default())
                .unwrap_err();
        assert_eq!(error.code, "native-return-borrow");
        assert!(records.is_empty());
    }

    #[test]
    fn returned_borrow_payload_must_be_borrowed_storage_with_matching_access() {
        let mut probe = binding(Mode::Read, false, false);
        probe.result = NativeType::Structured(&OWNED_RESULT);
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::Bytes;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::PayloadNotBorrowed)
        );
        probe.result = NativeType::Structured(&RESULT);
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::ByteView;
        probe.returned_borrow.as_mut().unwrap().access = BorrowAccess::Exclusive;
        probe.parameters = binding(Mode::Mutate, false, false).parameters;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::PayloadAccessMismatch)
        );
        probe.result = NativeType::Structured(&MUTABLE_RESULT);
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::ByteViewMut;
        probe.parameters = Box::leak(Box::new([NativeParameter {
            ty: NativeType::Structured(&VIEW),
            ..probe.parameters[0]
        }]));
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::ReadOnlySource)
        );
    }

    #[test]
    fn borrowed_results_cannot_omit_a_contract_and_observers_cannot_bypass_the_abi_guard() {
        let mut probe = binding(Mode::Read, false, false);
        probe.returned_borrow = None;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::MissingReturnContract)
        );
        probe.result =
            NativeType::Structured(&Signature::Array(&Signature::Named(NativeTypeId::ByteView)));
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::MissingReturnContract)
        );
        probe.result = NativeType::Int;
        probe.parameters = Box::leak(Box::new([NativeParameter {
            ty: NativeType::Structured(&Signature::Array(&Signature::Named(
                NativeTypeId::ByteViewMut,
            ))),
            ..probe.parameters[0]
        }]));
        assert_eq!(probe.validate_returned_borrow(), Ok(()));
        let mut records = Vec::new();
        let error =
            crate::native_binding_types::prepare(&mut records, &probe, crate::Span::default())
                .unwrap_err();
        assert_eq!(error.code, "native-return-borrow");
        assert!(records.is_empty());
    }

    #[test]
    fn result_error_cannot_hide_an_untracked_borrow() {
        let mut probe = binding(Mode::Read, false, false);
        static BORROWED_ERROR_RESULT: Signature = Signature::Result(&VIEW, &MUTABLE_VIEW);
        probe.result = NativeType::Structured(&BORROWED_ERROR_RESULT);
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::BorrowedErrorPayload)
        );
    }

    #[test]
    fn registry_contracts_keep_owned_values_unborrowed_and_views_guarded() {
        for binding in crate::native_bindings::BINDINGS {
            assert_eq!(binding.validate_returned_borrow(), Ok(()));
            if binding.contains_borrowed_signatures().unwrap() {
                assert!(binding.returned_borrow.is_some());
                let mut records=Vec::new();
                assert_eq!(crate::native_binding_types::prepare(&mut records,binding,crate::Span::default()).unwrap_err().code,"native-return-borrow");
                assert!(records.is_empty());
            } else {
                assert_eq!(binding.returned_borrow,None);
            }
        }
    }
}
