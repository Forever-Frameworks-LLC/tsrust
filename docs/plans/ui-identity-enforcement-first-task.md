# Parallel UI task: identity-enforcement feasibility

Status: proposed assignment, 2026-10-15; not dispatched. This plan does not authorize implementation, a new worktree, compiler changes, builds, consumer refresh or merge.

## Outcome

Determine whether existing supported Tsrust facilities can enforce the adopted runtime identity boundary without retaining UI state or mutable owner loans. Deliver either a checked, executable identity candidate integrated into the experimental owner API, or a reproducible blocker and the smallest proposed missing primitive. A completed investigation is not automatically an enforced identity contract.

Identity comes before resource observation: B1 already permits operations on the wrong component; B2 is an unproven resource-lifetime requirement. Do not reopen the adopted B3 immediate-retirement contract or implement a B2 observer here.

Read:

- [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), and [UI vision](../../UI_VISION.md).
- [Lifetime contract](../../experiments/ui-tsrust-owner-handles/CONTRACT.md), [H matrix](../../experiments/ui-tsrust-owner-handles/matrix.json), [R matrix](../../experiments/ui-tsrust-owner-handles/resource-matrix.json), and [B4 repair](../../experiments/ui-tsrust-owner-handles/B4-REPAIR.md).
- The existing `owner.tsr`, `domain-collision.tsr`, `runtime-boundary.tsr`, `cycles.tsr`, and named/factory self-replacement witnesses in that experiment.
- [SQLite acceptance](sqlite-acceptance.md) and the compiler worker's current handoff at dispatch.

B4 merged at `e33af7dba53a4b51cf47bd2b6a43ebbd2047ed39`, following SQLite source checkpoint `91e6b937`. At planning time SQLite still has uncommitted editor work and combined-build validation in progress. Neither its completion nor the availability of a repaired installed compiler may be assumed. The existing UI harness pins a historical B4 candidate; merging source does not update that executable.

## Contract to enforce

Use the existing runtime-local contract, with these explicit acceptance boundaries:

1. Ordinary supported construction can create two independent runtimes safely. Each owner belongs to exactly one runtime; each component instance belongs to exactly one owner. Equal local owner/instance counters in different runtimes must not alias.
2. A handle identifies runtime, owner and instance, or carries an equivalent checked distinction. Root handles need the same runtime/owner distinction; instance zero alone is not an identity.
3. Application keys are labels, not identity. Caller-provided names, timestamps, random collision probabilities, or a comment prohibiting cross-runtime calls are not enforcement.
4. Handles retain no component state, callback registration or mutable owner loan. An independent identity token is permissible only with a concrete ownership explanation showing it does not retain the runtime's owners/state. Do not claim physical destruction proof while B2 remains open.
5. Removing and reinserting a key creates a fresh instance. Escaped handles must not become valid after owner/runtime teardown and recreation. Define token allocation, retirement, comparison and exhaustion precisely; do not rely on reusable addresses or resettable counters.
6. Invalid operations must be rejected before accessing the wrong owner's state, invoking a callback or modifying either runtime. Preserve existing successful-operation behavior and same-owner idempotent removal.
7. Runtime-local handles are not serialized, persisted, sent between processes or transported across threads in this task. Deliberate tampering with internal structural fields is outside the threat model, but callers using documented constructors and operations must be safe. List those constructors and state which boundary enforces validity.

For a dynamically checked candidate, use the existing result conventions: foreign `valid` is false; foreign `read` is undefined; foreign `add`, `enqueue`, `remove` and `reorder` return false; `values` with a foreign parent returns `stale`; `mount` with a foreign parent throws before changing owner state. A stale same-owner removal may remain an idempotent success. Apply identity validation before the root special case and before stale/idempotent shortcuts. A statically enforced candidate may reject these programs instead, with stable source diagnostics and equivalent positive controls.

All entry points count: `valid`, `mount`, `read`, `add`, `enqueue`, `remove`, `values`, `reorder`, and the handle reconstruction used by `drain`. Do not fix only the shared node lookup while root paths, queued events or parent-based operations still bypass identity checks.

## Bounded investigation

Prefer a small checked runtime-identity token, if an existing supported facility supplies one. This is a preference to investigate, not permission to invent a native service. Do not introduce generative types, higher-rank lifetime machinery or a general global object registry solely for this experiment.

First inspect the facilities that could actually establish the distinction: construction, comparison, copying/ownership, escape restrictions and exhaustion. Cite existing source definitions and supported checked examples. If a static brand is proposed, prove it differs for independently constructed runtime values; a generic tag chosen by the caller or a structural alias is insufficient.

The closure-hidden allocator already failed in `runtime-boundary.tsr`. Retain it as historical evidence, not a new candidate. Adding another caller-selected integer or hiding the same resetting counter is not a distinct approach.

Budget: one primary candidate. One materially different alternative is allowed only after a concrete failure of the primary and a named existing facility that could repair that failure. Stop earlier if neither candidate has a credible enforcement mechanism. Do not launch an open-ended representation search.

If a viable source-level boundary exists, make the smallest changes to the existing experimental owner, handles and event representation. Identity must survive all internal handle reconstruction. Keep state, callbacks, retirement and delivery behavior otherwise intact. Do not fork a second owner implementation or replace callbacks with mock commands.

If a required facility is absent, stop dependent implementation and provide:

- The smallest counterexample or supported-syntax rejection, exact compiler identity and diagnostic, and a passing control.
- The missing operation or checked distinction and why existing facilities inspected do not provide it. Bound the conclusion to this investigation; do not claim that the language can never solve it.
- A minimal proposed primitive contract: construction authority, equality, copy/move behavior, non-retention, lifetime scope, reuse/exhaustion, failure behavior and required compiler/editor tests. Any process-wide allocator proposal must justify its necessity and scope; it is not approved or implemented here.
- Exact shared source surfaces a later implementation would touch, with a separately coordinated follow-up after SQLite.

No fake token generator, native shim, emitted-Rust patch, unsafe cast, suppression, hidden clone of owner state, SQL-backed ID allocator, or Rust-only owner implementation may stand in for checked source feasibility. SQLite is not a dependency of the identity model.

## Isolation alongside SQLite

- After explicit dispatch, use one worker in a separate managed Git worktree. Select a committed baseline containing B4 merge `e33af7db`, preferably a later verified SQLite/B4 checkpoint. Record its full hash. Never copy uncommitted main source. Supply this task brief explicitly if it is absent from the baseline.
- Own only `experiments/ui-tsrust-owner-handles/`: candidate source, witnesses, harness/tests, identity report/matrix and provenance. Existing compiler, native, LSP/editor, other experiments and project plans are read-only references during execution.
- No compiler repairs, native bindings, public built-ins, root manifests/locks, production CLI, benchmark registrations or shared interfaces. If the candidate requires one, report the precise dependency instead of broadening scope.
- Preserve original collision and B4 witnesses as historical evidence. If adapting the owner changes their API, retain original source attribution and separate old counterexample evidence from current enforcement assertions; do not rewrite an old failure into a claimed historical success.
- If an API change prevents an original B4 witness from using the candidate owner, execute the original committed witness/module pair in a disposable directory under the selected compiler. Retrieve it from Git, retain no source snapshot, and also test the equivalent behavior on the candidate API. Historical execution alone does not validate the candidate.
- No main-checkout writes, main analytics writes, merge, push, consumer refresh or replacement of installed/editor/release binaries. One writer per worktree; source isolation does not isolate build tools or timing.
- Use existing Node built-ins, `node:test`, supported Tsrust and installed rustc. No downloads, dependency installation, Cargo builds or new Cargo targets in this assignment. A missing suitable executable is a validation deferral, not proof of a language limitation.
- Coordinate directly with the compiler worker before compiler/rustc/native checks or whole-project formatting. Obtain an explicit quiet validation window with a stable selected executable; SQLite builds and measurements take priority. A queued message, unchanged hash or idle-looking process list is not permission. Never stop another worker's processes.
- Identify the candidate compiler by source commit/fingerprint, path, version and SHA-256. Require a verified build containing B4. Review any harness pin update explicitly, preserve prior provenance, and rerun the complete focused experiment before claiming continuity. Do not disable the pin or copy binaries as backups. Verify hashes after validation and preserve results if a tool changes.
- Use bounded subprocesses and disposable task-specific directories. Set `CARGO_INCREMENTAL=0` in inherited environments. If direct rustc execution needs shared Cargo outputs or unavailable dependencies, defer that branch; do not build implicitly.
- Before every source checkpoint, run whole-project `TSRUST=<verified-formatter> node tools/format-project.mjs` in the worktree. Verify token preservation, idempotence and formatted-source execution. Read-only reuse of an existing formatter dependency is allowed with provenance. Outside-ownership formatting changes are a checkpoint conflict: report them, do not commit them.
- Keep closed records in the experiment's ignored task-local SQLite through existing storage tooling. Verify restoration bytes, hashes and integrity before removing loose records. No archive directories or file snapshots. Main SQLite import belongs to separately authorized serial integration.

Documentation and read-only capability investigation can run alongside SQLite. Source-only implementation stays within the experiment. Native/compiler execution waits for the coordinated window. Any new identity primitive is a later compiler task, not parallel work silently added to this one.

## Acceptance matrix

Create `identity-matrix.json` with I01–I20 in order. Each row records **executed**, **analysis/emission only**, **expected rejection**, **blocked**, or **not applicable with reason**, plus its commands, witness and evidence. Record investigation completion and B1 enforcement as separate fields. A blocked dependent row names the missing capability and upstream row; no silent skips.

| ID | Case | Required result/evidence |
|---|---|---|
| I01 | Baseline and tools | Committed baseline includes B4; compiler source/hash verified; quiet window confirmed; compiler-worker files and consumers unchanged. |
| I02 | Existing failure | Retain/reproduce original independent-runtime and teardown/recreation aliasing under attributable historical source. Explain the failure without treating it as accepted behavior. |
| I03 | Candidate mechanism | Identify the actual checked distinction, all supported construction paths and ownership of its identity token. Document primary/alternate budget and eliminate already-failed counter-only approaches. |
| I04 | Positive runtime behavior | Two owners within one runtime and owners in two independent runtimes work correctly with their own handles, including identical application keys and colliding local counters. |
| I05 | Foreign reads and mutation | `valid`, `read`, `add`, and `remove` reject cross-runtime and wrong-owner handles in both directions; both owners retain their independently expected state. |
| I06 | Root and parent paths | Foreign roots/parents cannot mount children, read lists, reorder or remove roots. No root sentinel or idempotent-removal shortcut accepts them. Positive root/parent controls still work. |
| I07 | Queue and delivery | Foreign enqueue rejects without changing queue state or invoking callbacks. Identity is preserved in stored events and `drain` reconstruction; local FIFO and stale delivery remain correct. No fabricated queue-transfer API is required. |
| I08 | Retire and replace | Same-key replacement gets a fresh identity. Old handles/events stay stale through reorder and reinsertion; unaffected siblings still operate. Include named and factory handlers. |
| I09 | Teardown and recreation | Escaped handles cannot address fresh owners/runtimes, including repeated construction at the same call site. Prove runtime rejection or strict compile-time non-escape with a valid local-use control. Logical teardown is not destructor proof. |
| I10 | Identity exhaustion | Last valid runtime/owner/instance identities behave as specified; next allocation rejects before wrap or reuse without changing counters, membership, queues or existing state. Use an existing controlled seam or the same algorithm with a documented reduced limit; no enormous loop or fake identity generator. Unreachable limits without a valid seam remain explicitly unproved. |
| I11 | Handle ownership | Handles survive or reject escape according to the chosen contract without borrowing mutable owner state. Independent owner mutation remains possible. Inspect checked/emitted ownership to show tokens do not retain state; leave physical release claims under B2. |
| I12 | Boundary completeness | Audit every public constructor/operation and every internal handle/event construction. Unsupported internal-field tampering is documented; ordinary construction, copied handles where supported, and import/call-site variation cannot defeat enforcement. |
| I13 | Existing lifecycle behavior | Retained captures, extracted dispatch, active self/ancestor/root retirement, sibling-only access, list edits, duplicate keys, reentrant drain and 1,000 logical cycles retain native assertions. B3 semantics unchanged. |
| I14 | Ownership negatives and B4 | Strict check/emit still reject escaped loans, overlap, direct registry dispatch and use after transfer. Both original B4 named/factory integration cases still execute with the selected compiler; no panic is an accepted ownership diagnostic. |
| I15 | Assertion sensitivity | A deliberately incorrect successful-state or rejection expectation fails natively. For static rejection, a matching valid program passes and the exact invalid operation is diagnosed; arbitrary compiler failure does not count. |
| I16 | Formatting | Whole-project token preservation and repeat idempotence; successful formatted witnesses check, emit and execute with original assertions. Protected files are preserved. |
| I17 | Missing-capability branch | If blocked, retain a minimal failure plus passing control, inspected facilities, exact missing primitive contract and later ownership boundaries. Mark unexecuted dependent cases blocked. If fully enforced, mark this branch not applicable with evidence. |
| I18 | Historical reconciliation | Reconcile every H01–H19 and R01–R19 row with current results. B1 resolves only when all applicable enforcement rows pass; B2 stays open, B3 adopted and B4 resolved. Preserve historical counterexamples and compiler identities. |
| I19 | Focused suite and demo | Complete suite exits 0 with all rows dispositioned and no silent skips. Demo executes supported witnesses, reports identity separately, and still exits 2 because B2 remains open. Usage/unexpected tool or assertion failures exit 1. |
| I20 | Isolation and handoff | Only owned files committed; exact source/tool identities, commands, matrix, limitations and restorable local SQLite evidence supplied. Shared target/window released; no main import, consumer refresh or next milestone. |

I04–I12 are not waived by choosing a static brand: supply the corresponding strict rejection witnesses and positive controls. Native positive behavior remains required where applicable. Do not close B1 based solely on one passing access method, different owner counters, or a documented restriction callers can violate through the supported API.

A capability blocker is a valid completed investigation only when reproducible under a verified tool and bounded by I17. An active build window, stale executable, timeout of unexplained cause or missing local tool is an execution problem, not a capability result. Preserve unfinished acceptance explicitly.

## Execution sequence and deliverables

1. Resolve baseline, SQLite ownership, compiler provenance and build-window constraints. Record the exact assignment and concrete owned-file list before editing.
2. Inspect the enforcement mechanism and perform only the small probes needed to choose a viable candidate or establish I17. Do not implement identity operations around an assumed primitive.
3. Complete the coherent candidate and focused regressions before batch validation. Reuse the current harness and existing owner implementation; do not introduce another test framework.
4. Add `IDENTITY.md` and `identity-matrix.json` inside the experiment. Reconcile the H/R matrices and provenance. Historical source results remain historical; new candidate results have their own source identity.
5. During the reserved validation window, run whole-project formatting and the complete focused entry points:

   ```sh
   TSRUST=<verified-B4-capable-compiler> node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
   TSRUST=<verified-B4-capable-compiler> node experiments/ui-tsrust-owner-handles/cli.mjs demo
   ```

   Expected results: tests exit 0; demo exits 2 for B2, also listing B1 if identity remains blocked. A passing investigation suite does not imply full UI acceptance. Unexpected panic/rejection/signal/timeout fails validation. Repair and rerun affected checks only; no full compiler/LSP milestone or performance measurements.
6. Commit the owned implementation and acceptance evidence, or the reproducible bounded blocker report. Tool/window deferrals remain incomplete acceptance. Hand off worktree path, commit, exact checks, compiler identity, matrix dispositions, remaining dependencies and isolated SQLite location.

Stop after this task. Do not merge, refresh consumers, implement the proposed native primitive, add resource observation, integrate SQLite into the UI, or begin rendering, JSX, routing execution or another UI/compiler milestone.
