//! Bounded channels move owned payloads; operations borrow their endpoint.
use super::*;
use crate::native_signatures::NativeTypeId;

impl Analyzer {
    pub(super) fn channel_constructor(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::NativeConstructor(arguments, Some(annotation)) = &mut expression.kind else { return Ok(None); };
        let ty = self.resolve_value_annotation(annotation, false)?;
        let Type::Record(record) = ty else { return Ok(None); };
        if self.program.records[record].native_type != Some(NativeTypeId::Channel) { return Ok(None); }
        if arguments.len() != 1 { return Err(Diagnostic::new("arity", "new Channel<T> requires one integer capacity (1..1000000).", expression.span)); }
        let payload = self.program.records[record].application.as_ref().unwrap().resolved_arguments[0];
        thread_transfer::require_channel_payload(payload, &self.program.records, &|id| self.unions.members(id))
            .map_err(|_| Diagnostic::new("channel-transfer", "Channel payloads must be owned transferable data without channel endpoints.", expression.span))?;
        let capacity = self.expression(context, &mut arguments[0], Mode::Read, loans, Some(Type::Int))?;
        self.require(capacity, Some(Type::Int), arguments[0].span)?;
        self.require(ty, expected, expression.span)?;
        expression.target = Some(CallTarget::NativeBinding(crate::native_bindings::by_symbol("@native:std:threads#__channelCreate").unwrap().0));
        Ok(Some(ty))
    }

    pub(super) fn channel_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { return Ok(None); };
        if let ExprKind::Field(base, member) = &callee.kind {
            let Some(Type::Record(record)) = self.expression_type_hint(context, base) else { return Ok(None); };
            let identity = self.program.records[record].native_type;
            let name = match (identity, member.as_str()) {
                (Some(NativeTypeId::Channel), "sender") => "__channelSender",
                (Some(NativeTypeId::Channel), "receive") => "__channelReceive",
                (Some(NativeTypeId::Channel), "tryReceive") => "__channelTryReceive",
                (Some(NativeTypeId::Channel), "close") => "__channelClose",
                (Some(NativeTypeId::Sender), "send") => "__senderSend",
                (Some(NativeTypeId::Sender), "trySend") => "__senderTrySend",
                (Some(NativeTypeId::Sender), "close") => "__senderClose",
                (Some(NativeTypeId::Channel | NativeTypeId::Sender), _) => return Err(Diagnostic::new("channel-member", "Unknown channel endpoint operation.", callee.span)),
                _ => return Ok(None),
            };
            arguments.insert(0, (**base).clone());
            callee.kind = ExprKind::Name(format!("@native:std:threads#{name}"));
        }
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        let Some((id, binding)) = crate::native_bindings::by_symbol(name).filter(|(_, binding)| binding.module == "std:threads" && (binding.name.starts_with("__channel") || binding.name.starts_with("__sender"))) else { return Ok(None); };
        if !binding.accepts_arity(arguments.len()) { return Err(Diagnostic::new("arity", format!("This channel operation expects {} arguments.", binding.parameters.len() - 1), expression.span)); }
        let access = if binding.name == "__senderClose" { Mode::Mutate } else { Mode::Read };
        let receiver = self.expression(context, &mut arguments[0], access, loans, None)?;
        let Type::Record(record) = receiver else { unreachable!("checked endpoint"); };
        let payload = self.program.records[record].application.as_ref().unwrap().resolved_arguments[0];
        let result = match binding.name {
            "__channelSender" => self.thread_application(NativeTypeId::Sender, &[payload], expression.span)?,
            "__channelClose" | "__senderClose" => Type::Void,
            "__senderSend" | "__senderTrySend" => {
                let mut active = loans.clone();
                for root in Self::loan_roots(&arguments[0], access) { active.entry(root).or_default().push((access, arguments[0].span)); }
                let actual = self.expression(context, &mut arguments[1], Mode::Consume, &active, Some(payload))?;
                self.require(actual, Some(payload), arguments[1].span)?;
                let failure = self.thread_application(NativeTypeId::SendError, &[payload], expression.span)?;
                self.thread_result(Type::Void, failure, expression.span)?
            },
            _ => {
                let failure = Type::Record(self.schema_native_dependency(NativeTypeId::ChannelError, expression.span)?);
                self.thread_result(payload, failure, expression.span)?
            },
        };
        self.require(result, expected, expression.span)?;
        expression.ty = Some(result);
        expression.target = Some(CallTarget::NativeBinding(id));
        Ok(Some(result))
    }
}
