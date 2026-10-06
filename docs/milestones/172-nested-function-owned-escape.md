# Gate 172: owned escape of nested functions

Status: in progress, not accepted. Gate 171 is accepted at 1c6dd58. Preserve the
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
