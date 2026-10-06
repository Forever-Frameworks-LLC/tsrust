# Handoff: capture-free deferred generic methods

Read AGENTS.md, docs/development-workflow.md and
docs/generic-function-values-plan.md first. Previous checkpoint: 19b3e98.
This checkpoint implements capture-free polymorphic values; full generic values,
gate173 and the named-declaration family milestone remain open.

The deferred_generic_values module owns explicit declaration/span origins and
concrete call preparation. A source origin retains one empty environment. Calls
prepare the original generic template without evaluating arguments, check its
concrete argument contracts and reuse bounded methods keyed by checked argument
types. Methods currently reuse checked concrete function bodies. Captured replay
through closure_methods.rs remains the next implementation boundary. No unresolved
data binding or ordinary void wildcard is introduced. Empty prototypes are hidden
from result hints and editor type facts. Template and concrete origins stay distinct.

Native witnesses verify int/string/array calls, borrowed arrays, transferred owners,
rest packs, one default evaluation and caught failure. Eight invalid controls reject
wrong same-T relations, missing arguments, wrong results, reused owners and false
fixed-contract promises through annotations, unused callback arguments and record
fields. A real false-compatibility defect with () => void was found and fixed.
Until real adaptation exists, an unannotated generic value cannot be coerced to a
different callable contract; the original declaration can still be assigned directly
to a concrete context. Same-origin nullable admission is not blocked by that guard.

Evidence: nine focused native tests, one structural origin/cache test, all 63 depth
canaries, whole editor crate 284 tests and affected supplements. Canary logs include
the nine focused tests, so their aggregate is 72. Strict TypeScript/Node reference
passes with exactly one default line. Concrete call hover and source navigation pass;
generic alias parameter help remains pending.

Matched generic_function_deferred support is appended at manifest end: seven-value
independent oracle, native/TS pair, source hashes and mutation guards, registration,
existing array ABI/driver and oracle/run/profile/report hooks. Five matched cases
pass untimed. Canonical verify record:
benchmarks/runtime/results/2026-10-06T22-22-20-862Z-verify.json. All 117 compiler
and 258 oracle input hashes were checked against final source; execution samples
are empty. implementationAccepted remains false.

All 40 original logs/evidence/verify records are imported and read back byte-for-byte
in benchmarks/records.sqlite. Loose copies were removed. Summary record:
target/editor/deferred-generic-value-evidence.json. Failed compiler/backend/test and
registration attempts remain preserved. All launched tool handles are terminal.
Five executable consumers are atomically refreshed and hash-verified. Obsolete
test executables, benchmark/native and Node reference scratch were removed; library
caches and five consumers remain.

Five original witnesses still fail on captured values: three generic_function_values
and two generic_value_signatures. The capture-free first part of the mixed witness
now succeeds before its captured declaration reaches the pending boundary.
Next implement explicit captured source contexts, one retained environment across
types, sound per-method read/mutate/consume access, independent inferred factory
environments, then result-only context propagation, polymorphic callback/storage
adaptation and full generic alias editor help. Retain all five red witnesses and
complete the wider gate173 audit and meaningful family regression/performance review.

Four owner fixtures remain untouched and uncommitted: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr, map-group-by/documented.tsr.
Use explicit Git paths, offline locked Cargo, the single warm target and
CARGO_INCREMENTAL=0. Hourly checks remain paused.
