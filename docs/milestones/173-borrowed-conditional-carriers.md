# Scoped borrowed conditional callable carriers

This is an accepted prerequisite within Gate173, not acceptance of the full gate.
Concrete monomorphic alternatives with readonly or mutable local captures now use
Type::BorrowedCallable, distinct from owned storage. Capture lifetimes and origin
identity survive selection, repeated calls, argument defaults, direct invocation,
scoped callback parameters and callback failures. Original captured variables
retain their updates. Scope escape and owning callback transfers remain errors.

The implementation keeps contract validation, conditional registration, native
sum emission and callback error adaptation in separate modules. Family identity
lookup is constant time. Native sums introduce no allocation or implicit cloning.
They preserve the original concrete environment types and capture references.

The compiler batch passes 116 tests, including 18 conditional tests, 35 callable
field controls and all eight depth canaries. An additional mixed read/mutate and
owning-alternative native witness passes. Explicit throw and owning-transfer
rejection controls pass. Twenty-two focused unit tests and all 332 editor tests
pass. Original record owners dispose exactly once at optimization levels 0 and 3.
The guide executes with output `12 14` followed by `0 14`.

The generic adapter workload now retains 131 observations. Native and Node after
TypeScript compilation pass all five cases against the independent oracle.
Registration and mutation guards pass. Formatting changes one workload without
changing its tokens, refreshes that fingerprint, then changes zero files or
fingerprints on the checkpoint pass. No timing was run; measurement status remains
pending. Closed evidence is preserved byte for byte in benchmarks/records.sqlite.

Consuming borrowed sums, polymorphic selection/replacement, scoped aggregate and
higher-order builtin transport, recursive environments and point-sensitive mutable
capture restoration remain open. See docs/conditional-callable-carriers-plan.md
and the current handoff for the next integration gates.
