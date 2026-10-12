//! Invoke only origins permitted at the source point inside a wider native carrier.
use super::*;

impl Emitter<'_> {
    pub(super) fn record_point_call(
        &self,
        fun: &CheckedFunction,
        callee: &Expr,
        source: Type,
        receiver: &str,
        arguments: &str,
        count: CallArgumentCount,
    ) -> Option<String> {
        let family = self.record_callee_family(fun, callee)?;
        let (allowed, result_family) = match source {
            Type::Closure(origin) => (vec![origin], None),
            Type::BorrowedCallable(source) if source != family => (
                self.program.stored_contracts.origins(source)?.to_vec(),
                Some(source),
            ),
            _ => return None,
        };
        let payloads = self.borrowed_payloads(family);
        let arms = payloads
            .iter()
            .enumerate()
            .filter(|(_, payload)| allowed.contains(&payload.origin))
            .map(|(position, payload)| {
                let variant = crate::borrowed_callable_dispatch::variant(position, payloads.len());
                let call = self.counted_call("point_payload", arguments, count);
                let call = result_family.map_or(call.clone(), |source| {
                    self.stored_origin_result(source, payload.origin, call)
                });
                format!("BorrowedCallable{family}::{variant}(point_payload)=>{call}")
            })
            .collect::<Vec<_>>();
        assert!(
            !arms.is_empty(),
            "native carrier includes the checked point origin"
        );
        let fallback = if payloads
            .iter()
            .all(|payload| allowed.contains(&payload.origin))
        {
            String::new()
        } else {
            ",_=>unreachable!(\"checked record point origin\")".to_owned()
        };
        Some(format!("match {receiver} {{{}{fallback}}}", arms.join(",")))
    }
}
