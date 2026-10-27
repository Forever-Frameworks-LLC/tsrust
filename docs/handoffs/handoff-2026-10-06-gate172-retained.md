Superseded by [the gate172 accepted handoff](handoff-2026-10-06-gate172-accepted.md).

# Handoff: retained owned nested callbacks

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md,
docs/plans/nested-function-declarations-plan.md and
docs/milestones/172-nested-function-owned-escape.md first. Gate171 is accepted at
1c6dd58. Gate172 remains incomplete; do not mark the full plan complete. Inspect Git
for the actual checkpoint hash. Preserve four uncommitted owner fixtures under
crates/tsrust-compiler/tests/fixtures: comma-contexts/guards.tsr, interfaces/shapes.tsr,
map-for-each/values.tsr and map-group-by/documented.tsr. Explicit Git paths only.

## Implemented and verified

The prior fca1825 checkpoint enables ownership-taking captures through existing
transfer/argument-loan checks and hidden native owned parameters. This checkpoint
adds retained record/array storage, callable-array inference, independent factory
instances, once callbacks removed from inferred arrays, recursive retained array
owners, callback failure and instrumented native disposal. Five captured record
owners drop exactly once in both debug and optimized builds, including recursive
retention, failure and consuming owner return. ReadOnly data cannot escape through
an owned named factory; the compiler reports parameter-access with ownership advice.

inferred_callable_arrays.rs derives owned/owned-once storage from a checked first
closure element, preserving single evaluation and rejecting incompatible signatures.
It marks direct first-element arrows as escaping before checking, uses existing
origin validation and subsequent stored_callable_value conversion. Shared callable
shape/arity logic is exposed by callable_resolution::intern_callable_shape. Duplicate
family inference does not add unused callback declarations. Source signatures and
parameter help remain consistent for indexed inferred arrays.

Untimed verification exposed an emitter panic for an unused throwing factory.
needs_exception_transport now examines all emitted non-template definitions, not
only main-reachable functions. Saved Result contracts therefore cover unused
throwing callbacks too. The dedicated native red witness now passes, along with
exception library and handling suites.

The new nested_function_owned_escape workload is appended at the END of manifest.
Matched TypeScript sources, independent eight-observation oracle, mutation guards,
registration test, result-aware ABI, run/profile/report hooks and coverage exports
are present. It covers stateful retained factories, inferred arrays, both recursive
parity paths with owned arrays, consuming record return, failure and input length.
implementationAccepted remains false until gate172 acceptance. No timing is run.

## Next required work

Fix dead inner-function capture propagation. local_functions.rs currently adds a
dependency from an enclosing local function to every nested declaration, called or
not. This supplies typed contexts for uncalled-body checking but also imposes
unnecessary runtime capture arguments and borrows on the outer call. Separate
typechecking requirements from live call/value capture requirements. Keep invalid
uncalled bodies diagnosed; do not bypass their checks or eager-borrow uninitialized
data. Review named_capture_context, prepare_uncalled_capturing_functions, readiness,
access summaries, capture-slot ordering and lowering together. Add a genuine failing
native/diagnostic witness before changes. Preserve exact hidden-slot mappings and
recursive/callback/generic source identity.

Then audit every gate172 requirement against direct evidence, finish editor/guide
acceptance and set benchmark implementation acceptance only when warranted. Gate173
omitted-result inference and generic/default/optional capturing function values remain
open. Preserve its original red witness. The full nested family milestone follows
completed scope; do not trigger heavy runs by gate count.

Use one agent and the single target /private/tmp/tsrust-gate100-editor-target with
CARGO_INCREMENTAL=0, offline locked Cargo and Node v24.11.1. Source borrowing syntax
uses ReadOnly<T>, not the retired read keyword. TDD, std:testing, Node reference,
all eight canaries after dispatch changes and whole editor before acceptance. No
routine full compiler matrix, timing, archives or binary backups. SQLite is canonical;
import exact original logs/results, read back, then remove loose copies. Five
consumers are refreshed atomically. Library caches remain; pruned tests will relink.
Hourly checks stay paused.

## Checkpoint evidence

The final focused suite passes nine tests, including exact callback failure payloads
and debug/optimized disposal assertions. All eight depth canaries pass (63 tests).
The complete editor crate passes 277 tests, including 146 intelligence tests.
Exception handling passes 20 tests and the bounded exception library selection
passes 140 tests. The registration test passes. The guide example executes natively
and prints `11 21 12`. Native and Node-after-tsc pass all five untimed workload cases
with eight observations each. No performance measurement is claimed.

The verification record is stored byte-for-byte in benchmarks/records.sqlite at
benchmarks/runtime/results/2026-10-06T20-08-05-035Z-verify.json. Its 112 compiler
fingerprints, oracle inputs and matched source hashes match this checkpoint.

All 35 original checkpoint logs/results, including failed witnesses and
repairs, were imported, read back byte-for-byte and removed from loose storage.

Obsolete executable tests, examples and native comparison outputs were pruned.
Library caches and five current consumers remain; tests will relink when needed.

## Dead-inner capture red witness

The next source checkpoint retains a genuine failing native test named
`dead_inner_declarations_do_not_require_outer_capture_initialization` in
nested_function_owned_escape.rs. Its dead-inner.tsr fixture calls outer before
initializing an array used only by an unused nested declaration. The call must
return 7. Current checking instead reports unassigned at outer(), demanding later
initialization. The independent Node reference passes. Exact original red and Node
logs were imported and byte-verified in SQLite under target/editor/dead-inner-red.log
and target/editor/dead-inner-node.log. No compiler implementation changed.

The first rejection occurs in local_readiness::check, which constructs aliases
from Plan::group_requirements. local_functions::body adds an enclosing-to-child
dependency for every declaration. That same graph supplies Contexts::prepare,
checked readiness, recursive layouts and native hidden argument requirements.
Removing its declaration edge alone would lose the typed defining slots needed
by prepare_uncalled_capturing_functions. Filtering only access summaries would
also violate the contiguous parameter prefix assumed by emission and lowering.

Introduce distinct live call/value dependencies and declaration-body typing
requirements. Preserve checking of invalid unused bodies, including existing
nested_function_captures::uncalled_capturing_functions_are_still_checked. Deferred
checking must obtain types from the completed lexical owner, without making an
uncalled declaration a runtime borrow or forcing an uninitialized native argument.
Keep recursive groups and hidden slot mappings consistent across both phases.
