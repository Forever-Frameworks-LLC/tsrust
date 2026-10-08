# Gate 181: truthiness

Status: accepted. D1 in the compiler completion ledger.

Conditions and negation now use scalar, nullable and collection truthiness.
Non-boolean logical assignments test their fixed-path target by borrowing it,
store only on the selected branch, then read the target in place. The earlier
rewrite moved an owned target even when its right operand was skipped; a failing
native witness established the defect before this repair.

Completed verification: 58 affected compiler tests across conditional expressions,
control flow, operators, logical assignment and truthiness; all eight dispatch depth
canaries; the complete editor crate including the new nullable-string narrowing
and member-help witness. The project formatter changed only two existing fixture
files' unary spacing. The owner explicitly authorized normal formatting and keeping
pre-existing fixture changes; those changes must not be discarded.

The matched `truthiness` workload has native and TypeScript sources, an independent
oracle, token-based scenario guards, fingerprint checks, mutation rejection tests
and runner, profiler, oracle and report registrations. All five correctness cases
passed on native and Node-after-tsc. This is untimed evidence; no performance claim
is made. The exact verification record is stored under
`benchmarks/runtime/results/2026-10-08T13-53-12-127Z-verify.json` in
`benchmarks/records.sqlite`; readback matched the original bytes and the loose copy
was removed.

The assertion-based project fixture verifies skipped RHS calls, subsequent stores,
nullable narrowing and scalar/collection truthiness. Its pinned Node reference
passes. Additional rejection witnesses preserve transfer, immutable-target, type
and void-operand diagnostics. All four truthiness tests pass after formatting.
Coverage and the completion ledger now record this implemented, untimed capability.
Computed non-boolean logical-assignment targets remain unsupported.

Scoped aggregates B5 remain next in the owner-approved sequence.
