# Scoped traversal and nullable callable transport

This is focused acceptance within Gate173. Full Gate173, general scoped aggregate
storage, polymorphic replacement and mutable environment restoration remain open.

Scoped monomorphic conditional families now pass the existing reusable traversal
contracts. Coverage includes map/forEach, reduction in both directions, predicates,
search, filter, sorting and copying sort, flatMap, Array.from and Set/Map forEach.
The native emitter selects readonly or mutable borrowing from checked callback
metadata. Selected defaults and rest packs retain their original argument behavior.
Consuming callbacks and mutations conflicting with traversal are rejected.

Nullable families preserve optional invocation, result coalescing and typeof. They
cross readonly/mutable nullable callback parameter boundaries without erasing the
concrete environment. Native unions use generic callable payloads instead of data
payload defaults containing invalid unnamed lifetimes. Borrowed families inside
nullable unions cannot escape through returns or throws. Capture loans still reject
owner mutation while a handle is live and release when its scope ends.

The original two source witnesses fail before implementation. Additional witnesses
expose a typeof emitter panic, a nullable parameter injection panic and an escape
check gap. Their repaired source/native controls pass. A matched workload exposes
ambiguous Result trait selection when multiple checked error bridges exist; traversal
now explicitly selects the family's storage Result ABI. Original failures remain
in SQLite. An invalid test reuse of a transferred handle is corrected without
weakening transfer checking.

Six project tests pass using std:testing and formatting round trips, including five
native assertion cases and five rejection controls. The five positive cases also
pass after TypeScript compilation with independent Node assertions. The affected
compiler batch passes 55 tests. Five union-domain unit controls also pass. All eight depth canaries pass, 64 tests. The complete
current editor crate passes 335 tests, including hover, optional-call signature help
and result-chain documentation. The guide executes successfully with no output.

Matched generic adapters now retain 136 observations, adding mapped values, optional
results and mutation of an original scoped capture. Five native and compiled-TypeScript
cases pass against the independent oracle. Source guards, mutation controls and
registration pass. The observer's nonnegative domain is preserved by using zero
for the absent observation. Operation counts, protocols and measurement flags remain
unchanged. No timing measurements have run. Checkpoint formatting preserves tokens
and refreshes one workload fingerprint; the final pass is idempotent.

Byte-exact evidence is stored under runtime/diagnostics/scoped-callable-transport/
in benchmarks/records.sqlite. See scoped-callable-transport.md and the current
handoff for remaining work. This acceptance does not claim general record/array
storage of scoped callable families or joins of distinct polymorphic carriers.
