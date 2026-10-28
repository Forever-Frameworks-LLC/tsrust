# Gate 172: owned escape of nested functions

Status: accepted. Gate 171 is accepted at 1c6dd58. Preserve the
four owner fixture edits and use the existing development workflow and full nested
function plan. The settled model transfers owned escaping captures, preserves scalar
copy semantics and uses existing loans for borrowed captures. No implicit clones or
shared JavaScript capture cells are introduced.

## Implementation chunks and gates

1. Ownership-taking captures. A named function can return or otherwise transfer a
   captured owner. Replace blanket rejection with existing availability, argument
   loan and transfer checks. Verify independent factory instances, direct owner reuse,
   once-callable reuse and overlapping argument/capture access. Use the arrow wrapper
   and existing owned callable transport, with source capture identities preserved.
2. Retained values and aggregate storage. Verify multiple factory instances, record
   and array storage, explicit moves, borrowed-data escape rejection and disposal.
   Add missing callable transport support if actual witnesses expose gaps.
3. Recursive environments and callback failure. Prove recursive captured groups have
   finite retained storage, no duplicated ownership and correct activation identity.
   Verify exception propagation and disposal on ordinary and failure paths.
4. Correctness and usability. Investigate the known dead-inner-declaration capture
   propagation: checking unused bodies must not create unnecessary runtime loans or
   transfers. Keep invalid unused bodies diagnosed. Complete editor diagnostics,
   hover/signature examples and guide coverage without exposing hidden arguments.
5. Acceptance support. Append a matched owned-escape workload at the END of the
   manifest; add the TypeScript counterpart, independent oracle, source contracts,
   registrations, observer/ABI and run/profile/report hooks. Verify untimed, run all
   eight canaries and whole editor, import/read back evidence into SQLite, refresh
   consumers atomically and commit. No timing or full compiler matrix for this gate.

## First ownership-taking chunk

The native take-owner fixture failed at the ownership rejection. It now returns two
independent captured User owners through the named-function arrow wrapper. The direct
call checks existing capture access against active argument loans and marks consumed
owners transferred. Native emission already supports consuming hidden parameters.
The final lowering validator no longer rejects every consuming capture. An unused
ownership-taking declaration is now typechecked without demanding execution.

Gate acceptance remains owed for aggregate storage, disposal, callback failures,
recursive retained lifetime, dead declaration propagation, editor examples and the
matched workload. Default/optional/generic function values and omitted-result
inference remain gate 173; preserve its genuine red witness.

Checkpoint evidence: capture suite17 and owned-escape tests2 pass, including native
independent factories and precise transfer/once/conflict diagnostics. Node reference
passes the factory fixture. Canaries63 and editor276 pass. Existing matched capture
cases5 pass untimed; new owned-escape workload remains owed before gate acceptance.
Canonical result 2026-10-06T19-52-48-838Z-verify.json has verified source fingerprints
and no timing samples. 11 exact original records are imported/read back in SQLite;
failed assertion and retired-keyword test drafts are retained. Five consumers are
refreshed atomically. 59 scratch executables and runtime output are pruned.


## Retained storage and failure checkpoint

Native witnesses now cover record storage, explicit and inferred callable arrays,
independent factories, recursive retained array owners, consuming callbacks removed
from inferred arrays and callback failure propagation. Unannotated callable arrays
were a genuine array-element error. inferred_callable_arrays.rs now derives owned
or owned-once storage from the checked first element, registers its origin and checks
subsequent values against the same signature. It reuses callable_resolution's shared
shape/arity logic and avoids adding unused duplicate callback declarations. The first
element is evaluated once; subsequent construction follows existing ownership checks.
An editor witness verifies indexed signature help for the inferred family.

An instrumented native witness checks five captured record owners drop exactly once
across independent success cases, failure, consuming return and recursive retained
groups. Both debug and optimized native execution verify the exact count and owner
identity mask. No runtime disposal feature or implicit clone was added.

The new matched workload is appended at the manifest end, with eight independent
observations covering retained state, inferred arrays, both recursive parity paths,
consuming owner return, callback failure and input length. TypeScript counterpart,
independent oracle, mutation guards, ABI registration, run/profile/report hooks and
coverage exports are supplied. implementationAccepted remains false pending full
gate acceptance; no timing is performed.

Untimed verification exposed an unused throwing factory emitter panic. A native
regression preserves it. Exception transport preparation now covers every emitted
non-template definition, rather than only definitions reachable from main. Saved
error contracts therefore exist even for unused throwing callbacks. Existing
exception-library and exception-handling checks verify this correction.

Borrowed-data escape is rejected by the existing ReadOnly parameter ownership
contract, with a focused diagnostic witness. Still pending: dead-inner-declaration
capture propagation, final editor/guide acceptance and a full gate audit. Gate173 limitations
and its omitted-result red witness remain unchanged. Gate172 is not accepted here.

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

## Final acceptance audit

Gate172 is accepted for owned nested callable escape. Independent factories,
record and array storage, inferred and explicitly typed callable arrays, consuming
capture return, once invocation, borrowed-escape rejection, transfer conflicts,
recursive retained owners, callback error payloads and exact debug/optimized
disposal are covered by the eleven owned-escape tests. The seventeen capture tests
preserve existing borrowed behavior and invalid unused-body diagnostics.

The dead-inner witness now passes. Runtime capture dependencies include only
actual calls and value uses. Completed lexical contexts supply types separately
for unused body checking; deferred placeholders are replaced before real checking.
Lexical contexts are retained only for ancestors that supply captured declarations.
No hidden argument slot filtering or ABI remapping is required. Invalid unused
bodies still report type errors; live calls and returned callable values still
require initialized captures. The native fixture verifies that an unused inner
declaration also cannot conflict with consuming an outer-call argument.

All eight depth canaries pass (63 tests), the local compiler library selection
passes 19 tests, and the complete editor crate passes 278 tests. The editor witness
retains hover and navigation for unused declarations. Its initial spacing assertion
was corrected to match the actual source signature; no hover defect was found.
The expanded Node reference passes. Matched native/Node-after-tsc workload cases
pass five untimed inputs with eight observations each and an additional dead-inner
assertion. The independent oracle, source mutation guards, ABI, registrations and
run/profile/report hooks are complete. Measurement remains deferred to the full
named-declaration family milestone. Gate173 and the full plan remain incomplete.

Canonical untimed record: benchmarks/runtime/results/2026-10-06T20-29-06-606Z-verify.json
in benchmarks/records.sqlite. All 112 compiler inputs, all oracle inputs and matched
source hashes match the accepted source. The manifest acceptance flag was promoted
after this verification; the final registration test verifies that metadata.
All 20 original logs/results from the final repair were imported, read back
byte-for-byte and removed from loose storage. Earlier red evidence remains intact.

## Reconciliation against the plan (2026-10-07)

Each requirement of gate 172 in
[the nested function plan](../plans/nested-function-declarations-plan.md) was checked
against a current witness. All pass on the sources accepted with the scoped callable
record gate (commit 5c5cc68).

| Plan requirement | Witness |
| --- | --- |
| Return, pass and store nested functions through owned callable transport | `escaping_named_functions_transfer_independent_captured_owners`, `retained_named_callbacks_preserve_storage_recursion_and_failure` |
| Transfer captured owners once; scalar captures copy | `take-owner.tsr`, `consuming_named_captures_reject_reuse_and_conflicting_argument_borrows` |
| Once versus reusable invocation | `inferred_once_callable_arrays_move_each_retained_owner`, `inferred_callable_arrays_check_contracts_and_invocation` |
| Reject escaping borrowed data | `borrowed_data_cannot_be_retained_by_an_owned_named_factory` |
| Multiple factory instances | `take-owner.tsr`, `retained.tsr` |
| Aggregate storage | `retained.tsr` (record), `retained-inferred.tsr` (arrays) |
| Disposal | `retained_named_environments_drop_owners_once_on_success_and_failure` at optimization levels 0 and 3 |
| Callback failures | `retained.tsr`, `unused_throwing_named_factory_emits_its_checked_error_contract` |
| Recursive group lifetime | `retained.tsr` |
| No shared capture cells, no implicit clone | generated code of the fixtures; `dead_inner_declarations_*` for unused bodies |
| Editor and guide | hover and navigation witnesses in the editor crate (339 pass); `docs/guides/language/nested-functions.md` |
| Matched workload | `nested_function_owned_escape`, native and Node-after-tsc verified untimed again in `2026-10-07T23-32-57-916Z-verify.json` (SQLite) |

One correction belongs to gate 171 rather than this gate: the project test
`retained_capture_plans_map_diagnostics_after_prior_module_declarations` still
expected the pending capture diagnostic. It now asserts the `unassigned` diagnostic
for a capture read before initialization in a later module, with the same location
mapping. No requirement of gate 172 is open.
