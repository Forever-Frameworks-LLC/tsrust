# Gate 186: tagged Result and exhaustive statement matching

B10/S06 is accepted on 2026-10-10. Source closes at `8d431eed`; the publication
commit adds this record and updates the ledgers. The [acceptance map](b10-result-match-acceptance-map.md)
and [guide](../guides/language/result-match.md) define the supported scope.

## Behavior

`Result<T, E>` holds one success or error payload, constructed by contextual or
explicit `Ok`/`Err`. Exhaustive statement match evaluates its scrutinee once and
binds the selected payload inside its arm. Explicit `return Err(error)` propagates
a typed error without throwing. Owned values and captured reusable/Once callables
preserve their existing invocation and transfer rules. ReadOnly matching lends
payloads; Mutable matching lends exclusive access. No implicit clone is added.

Checked editor help covers constructors, public Result types, payload members,
arm-local definitions, imported aliases and nested matches. Incomplete headers
recover lexical assistance without guessed payload types. TextMate recognizes
Result, constructors and match. Formatting preserves tokens and stable arm layout.

Cleanup witnesses cover replacement, unused callbacks, early returns, throws,
finally, loop exits and recursive forwarding. A counting allocator restores the
exact live-byte baseline over 32 source-body invocations at optimization levels
zero and three. Process setup runs once, so its panic hook is not mistaken for a
payload leak. An independent TypeScript reference verifies behavior.

## Defects found during acceptance

The first complete compiler run found an existing optional-call witness failing:
a statically absent nongeneric callable accepted explicit type arguments after
Result constructor support enabled generic preparation in nongeneric programs.
Validation now precedes absence lowering. Six optional-call controls and a formatted
editor diagnostic witness pass. The first complete editor run found its exhaustive
catalog witness dispatcher missing Result/Ok/Err. It now checks their formatted
catalog examples; dedicated native example checks remain intact. Failed original
runs are preserved, and only affected or previously unrun suites were repeated.

## Correctness evidence

- Complete compiler coverage: 2,754 passed, zero failed, three intentional ignores,
  no aborted suite. The original 1,446 preceding passes combine with 1,308 resumed
  passes; the originally failed 88-test suite is counted only from its repaired run.
- Complete editor coverage: 472 passed, zero failed, one intentional ignore. The
  full affected intelligence suite passes 311; all other suites retain their full
  run evidence. Every started suite has a result summary.
- All ten depth canaries pass on normal worker stacks after the repair. Compiler
  doctests, extension compilation and TextMate grammar checks pass.
- All eighteen Result integration tests pass. Six Result catalog examples execute
  after idempotent formatting with exact output. Project assistance, module identity,
  formatter corpus and diagnostic controls retain their earlier focused records.
- The matched workload verifies five cases each on native and Node-after-tsc.

## Performance review

The final workload retains seven observations per 2,048-scenario batch: two readonly
inspections, owned transfer, two stateful callback calls, mutable record/array updates
and replaced fallible callback/catch/finally behavior. Its oracle uses independent
arithmetic; guarded sources, exact fingerprints, fallible native ABI, observers,
runner/profile hooks and registration rejection controls are complete.

Execution-only measurement preserves eight warmups, three processes and 31 samples
per process: 93 samples per engine. Pooled median batch duration is 0.111625 ms native
and 0.628750 ms Node-after-tsc, a 5.63 ratio of Node time to native time. Native process
medians are 0.112583, 0.111167 and 0.108083 ms; Node medians are 0.603958, 0.628750 and
0.638666 ms. This mixed workload establishes a baseline, not isolated operation
costs or a general runtime claim. There is no earlier uncontended measurement of
its final semantics to use for a regression comparison. No optimization was needed
for acceptance. An earlier run overlapped compiler regressions and is retained
separately, excluded from this review.

## Durable evidence and publication

Exact bytes are imported into `benchmarks/records.sqlite`, read back and removed
from loose storage:

- `runtime/results/result-match-cleanup-2026-10-10-checks.json`
- `runtime/results/result-match-harness-2026-10-10-checks.json`
- `runtime/results/result-match-family-2026-10-10-regressions.json` (original failure)
- `runtime/results/result-match-family-2026-10-10-resumed.json` (compiler pass and original editor failure)
- `runtime/results/result-match-family-2026-10-10-editor-repair.json`
- `runtime/results/result-match-family-2026-10-10-final-controls.json`
- `runtime/results/2026-10-10T12-41-00-897Z-verify.json`
- `runtime/results/2026-10-10T12-41-18-287Z.json` (contaminated; excluded)
- `runtime/results/2026-10-10T12-57-21-848Z.json` (acceptance measurement)
- `runtime/results/result-match-family-2026-10-10-consumers.json`
- `runtime/results/result-match-family-2026-10-10-publication.json`
- `runtime/results/result-match-family-2026-10-10-cleanup.json`

Five executable consumers were copied through temporary sibling files and atomically
renamed from source `8d431eed`; hashes match their verified build inputs. Three
installed compiler paths pass the cleanup fixture check. The installed extension
grammar was refreshed atomically. Reload the VS Code window to apply grammar changes
and restart the Tsrust server to use the refreshed executable.

## Boundaries and next work

Match expressions, guards, nested destructuring patterns and `?` remain separate.
General discriminated unions are D2. Borrowed callable payload storage is rejected;
context-free inline callbacks may need explicit generic arguments. These are explicit
limits, not implemented claims. The next dependency row is B11/S07: resources and
bytes, with a concrete ownership/drop/API plan before implementation. The larger
compiler completion goal remains open.

Cleanup removed 71 obsolete or temporary paths totaling 879,117,354 bytes,
while retaining the five current consumers and newest executable variants.
