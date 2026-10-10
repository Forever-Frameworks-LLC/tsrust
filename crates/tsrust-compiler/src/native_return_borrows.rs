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
}

impl NativeBinding {
    /// Validate identities and access before a declaration reaches type projection.
    /// This checks metadata, not whether the backend supports its lifetime ABI.
    pub fn validate_returned_borrow(&self) -> Result<(), ContractError> {
        let Some(contract) = self.returned_borrow else {
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
            (ReturnSlot::ResultOk, Signature::Result(value, _)) => value,
            (ReturnSlot::ResultOk, _) => return Err(ContractError::ExpectedResult),
        };
        if *payload != Signature::Named(contract.payload) {
            return Err(ContractError::PayloadMismatch);
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
    static RESULT: Signature =
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
                payload: NativeTypeId::Bytes,
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
        probe.returned_borrow.as_mut().unwrap().payload = NativeTypeId::Bytes;
        probe.returned_borrow.as_mut().unwrap().slot = ReturnSlot::Direct;
        assert_eq!(
            probe.validate_returned_borrow(),
            Err(ContractError::PayloadMismatch)
        );
        probe.result = NativeType::Structured(&OWNER);
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
    fn owned_registry_contracts_remain_unborrowed() {
        for binding in crate::native_bindings::BINDINGS {
            assert_eq!(binding.returned_borrow, None);
            assert_eq!(binding.validate_returned_borrow(), Ok(()));
        }
    }
}
