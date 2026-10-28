# Parallel UI task: B2 resource-release observer and lifetime validation

Status: proposed assignment, 2026-10-15; not dispatched. This document authorizes no implementation, worktree creation, compiler repair, build, merge or consumer refresh.

## Outcome and architectural direction

Build the smallest trustworthy test-only resource observer, then measure the actual experimental Tsrust owner's release behavior. Close B2 only for requirements demonstrated by checked, formatted, emitted native execution. Preserve unresolved cases as precise blockers.

The intended UI must support async operations and background workers, potentially with painting confined to one thread. Synchronous execution in this assignment is a validation stage, not a permanent runtime restriction. Do not adopt an ownership representation that requires all application work to finish synchronously or all work to execute on one thread. Document the extension boundaries before implementation; do not silently convert prototype limits into UI requirements.

Identity enforcement is merged. B1 is resolved within the documented source API; B3 adopts immediate logical retirement; B4 and the captured-factory dependency are repaired. B2 remains unproven resource release, not a demonstrated leak. This task must distinguish observer correctness, synchronous lifetime acceptance, and future async/thread acceptance.

Read:

- [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), [UI vision](../../UI_VISION.md).
- [Lifetime contract](../../experiments/ui-tsrust-owner-handles/CONTRACT.md), [identity contract](../../experiments/ui-tsrust-owner-handles/IDENTITY.md), [review](../../experiments/ui-tsrust-owner-handles/REVIEW.md), and current provenance.
- H01–H19 in `matrix.json`, R01–R19 in `resource-matrix.json`, I01–I20 in `identity-matrix.json`, existing harness, demo, owner and witnesses.
- Current native adapter implementations and tests, including supported custom resource/drop boundaries. Older adapter plans are historical, not evidence of current support.
- [Standard-library roadmap](standard-library-roadmap.md), current compiler handoff, and S18 worker's current ownership/build window.

Planning baseline: main `49704b4f46050e420238148fe9804d9f9483e151`, containing UI merge `bea68611514760a9d3f6a13d1d73f289f3023447` and dependency `66dd80d87c0af9a009981c3ed72a6d310a2463f0`. Installed compiler at planning time has SHA256 `12f069627c4e253b6656028edf384d99c9fcba8eb956c45364e401318fd7cc89`. Reverify at dispatch; S18 is active and may change tools. Never copy uncommitted S18 source into the UI worktree.

## Async and worker compatibility requirements

Record these boundaries in the design before writing the observer or adapting the owner:

1. **Owner execution authority.** Prefer one serialized mutation context per owner, compatible with a UI/paint executor and background workers. This does not require one thread for the entire application. Do not introduce locks around every owner operation or a scheduler in this task.
2. **Identity across deferred work.** Runtime/owner/instance identity must support rejecting a completion after removal, same-key replacement, or owner teardown. A future typed completion envelope must return through the owner's validation boundary. Do not pass mutable owner loans across worker boundaries. Current handles remain non-transportable until separately checked transport support exists.
3. **Logical retirement versus physical release.** Removal immediately prevents handle-mediated access. Storage remains alive while an authorized active use exists. The current callback-return safe point is one synchronous case, not the universal definition of cleanup completion. Async suspension must not hold an unbounded exclusive owner loan; explain which ownership transfer or completion boundary will be needed later.
4. **Cancellation and shutdown.** Cancellation request, task termination, and completion of resource cleanup are distinct events. A dropped future does not prove backend cleanup completed. Future shutdown needs an explicit completion boundary for outstanding work; this assignment adds no cancellation or shutdown API.
5. **Thread affinity.** Some resources may require release on the UI or owning thread. The synchronous observer must report what it measures without promising that arbitrary destructors can run on any worker. A future executor-dispatched cleanup path must be able to retain the same identity and ordering assertions.
6. **Identity allocation audit.** Inspect the current private module scalar and closure-owned allocator. Do not assume they are safe for concurrent runtime construction. Document the exact race/authority boundary and alternatives such as serialized construction or a checked synchronized allocator. Do not introduce per-thread counters that can alias. Any transport/allocation limitation remains an explicit architectural dependency, not a production guarantee.
7. **Observable events.** Identify allocation, logical retirement, active-use completion and physical resource release separately in evidence. Only the native release path may produce release events. Source markers may establish execution phases but must not impersonate release.

Deliver a compatibility table: current mechanism, future async/worker requirement, reusable part, unresolved dependency, and whether a representation change is required. A known fundamental incompatibility must be reported before presenting the prototype as a suitable UI foundation. This task does not prove Send-like safety, scheduling fairness, cancellation, thread-affine cleanup or distributed identity.

## Observer contract and bounded implementation

Prefer an experiment-local Rust adapter through the existing checked adapter mechanism. No downloaded dependencies. Reuse existing native test helpers when they genuinely observe this resource's lifecycle.

The observer owns a unique tracked resource with a stable ID and an independent audit. The resource's actual release path appends an ordered release event. Audit reads must remain possible after the resource is gone and must not keep the resource alive. The resource may retain audit storage; the audit must not retain the resource or UI owner. Bound and reset audit storage between cases only after all resources have released. Never erase evidence of live resources.

Specify whether events represent wrapper destruction or underlying resource release. Prefer one underlying release operation shared by explicit close and automatic destruction, with a guard against duplicate release. If wrapper events are also needed, label them separately. Do not equate a synthetic resource event with completion of File, SQL connection or asynchronous backend cleanup.

Calibrate the observer independently before using it to judge the owner:

- No release while a resource is live; exactly one release after explicit consumption and after ordinary scope exit.
- Moving through a function, record, array and retained closure does not release prematurely. Include replacement and array removal.
- Keeping an audit alive does not keep a resource alive. Keeping a live resource after an audit handle leaves scope remains valid.
- Duplicate logical cleanup cannot cause a second release. A consumed resource's illegal reuse must reject; do not invent an API that can safely close the same owned value twice.
- Wrong count, wrong ID, missing release, premature release and reversed dependency-order expectations are detected. Use independent expected values, not expected results derived from the observed log.
- Record release order in one audit sequence when comparing parent and child; separate unrelated logs cannot establish a total order.

Budget: one primary existing-adapter route. One alternative is allowed only after a concrete failure and a named supported facility that addresses it. No open-ended observer search. The old testing-hook and File-reopen approaches do not establish the required observations and are not new candidates.

If adapters cannot express the needed resource, produce a minimal rejection and passing control, then coordinate the precise missing support with the compiler worker. Experiment implementation may pause that branch while independent documentation/tests continue. A separate compiler dependency is allowed only after explicit scope/ownership confirmation under the later implementation authorization; this plan alone grants none.

Do not patch emitted Rust, inject cleanup counters into generated code, substitute a Rust-only owner, weaken ownership checks, add unsafe casts, or use a mock registry. Native observer implementation is legitimate instrumentation; bypassing checked source ownership is not.

## Integration into the existing owner

Use `experiments/ui-tsrust-owner-handles/` and its existing owner, harness and demo. Add observer support, `release-matrix.json`, `RESOURCE-RELEASE.md`, and focused tests/witnesses there. Keep one real owner implementation.

Use the smallest concrete resource-bearing representation supported by existing syntax. Extend actual node state, retained captures and queued payloads separately. A narrowly scoped concrete fixture representation is acceptable; a generic UI framework solely for this test is not. Preserve integer-value behavior, identity checks, real callbacks and existing public experimental operations. Update all affected callers, including named/factory B4 witnesses. Keep original historical witness/module pairs attributable to their commits.

For each new resource-bearing operation, document transfer on success and rejection. A by-value rejected enqueue may consume and release its payload; it must neither leak nor mutate the wrong owner. Do not assume rejected ownership transfer gives the value back. Match the chosen checked signature with a native positive control and an illegal reuse rejection.

Required behavior:

- Outside delivery, explicit removal finishes applicable state/capture cleanup before returning.
- During a callback, self/ancestor/root retirement denies access immediately but does not physically release retired state/captures until callback return. Sibling-only removal must not invalidate active state. Cover multiple retirements in one callback.
- Retired descendants release before ancestors. Compare every relevant descendant resource against its ancestor resources. Do not impose sibling order or state-versus-capture order within one node unless the existing contract requires it.
- Same-key replacement creates fresh resources and identity; releasing old resources cannot release the replacement.
- Accepted queued payloads remain owned until successful delivery, stale discard, or owner destruction. Observe their release separately from node/capture cleanup. No queue-clear operation currently exists; do not add one just to satisfy a hypothetical row.
- Automatic owner destruction without explicit remove releases all resources exactly once. Measure ordering independently from explicit removal; array field drop order is not proof of tree order. If the existing contract cannot be met automatically, report a concrete lifecycle blocker rather than requiring callers to remember cleanup.
- Outstanding scalar handles and allocator values must not retain a destroyed owner's resources. Other live owners remain unaffected.

Fix a demonstrated experiment-local lifecycle bug within the bounded assignment, with a failing regression and unchanged contract. Do not silently weaken descendant ordering or callback safety to obtain a pass. A compiler/runtime dependency or architectural redesign requires the coordinated dependency branch.

## Isolation alongside S18

At planning time, the compiler worker confirms S18 targets SQLx SQLite, Postgres and MySQL with opt-in database checks. Transactions must rollback on cancellation/drop, pool/transaction handles remain owned, and async receiver loans must prevent overlapping exclusive operations. UI observation must not infer backend cleanup completion from dropping a future.

- At dispatch, create a separate managed worktree from a committed, verified baseline containing the identity repair. One agent writes that tree. Record the full baseline and exact owned paths. Supply this plan explicitly if not committed in the baseline.
- Primary ownership is only `experiments/ui-tsrust-owner-handles/`. Main checkout, S18 SQL modules/tests, compiler/LSP/editor internals, root manifests/locks, production CLI, benchmark registrations and other experiments remain read-only unless the dependency protocol grants specific ownership.
- Do not use SQL, live databases, network services, or S18's opt-in integration databases as the observer. No connection-pool or transaction implementation changes. No new package downloads or dependency installation.
- Source-only UI work and read-only investigation may overlap S18. Builds, native tests and formatting require an explicitly acknowledged quiet window from the compiler worker. An apparently idle process or a queued message is not permission. S18 timing and database validation retain priority.
- Reuse `/private/tmp/tsrust-gate100-editor-target` when Cargo is required, with `CARGO_INCREMENTAL=0` and `CARGO_BUILD_JOBS=1`. Never create another Cargo target or implicitly rebuild shared consumers. Any local adapter manifest must stay inside the experiment and direct builds into the reserved target. Use existing offline dependencies only.
- Pin compiler path, source commit/fingerprint, version and SHA256. Preserve prior provenance. Reverify before and after checks. A changed pin requires review and the complete focused suite; a stale tool or unavailable window is validation deferral, not a language blocker.
- Direct rustc/native probes also require the quiet window because they can disturb S18 measurements. Use bounded subprocesses, disposable native output and no retained source snapshots.
- Before source checkpoints, run whole-project `TSRUST=<verified-compiler> node tools/format-project.mjs` in the worktree. Verify token preservation, idempotence and formatted native behavior. Outside-ownership changes are a checkpoint conflict; preserve and report them rather than committing unrelated files.
- Keep closed evidence in ignored task-local SQLite with relative record paths. Verify exact restoration, blob hashes and integrity before removing loose copies. Do not write main analytics during execution. The known main-store historical absolute-path issue is outside this task; do not replicate it or repair it incidentally.
- No automatic main merge, consumer refresh or evidence integration. Reserve serial integration explicitly if later authorized. Prune only this task's obsolete native/test outputs; preserve current consumers and S18 outputs.

### Compiler dependency protocol

Before editing shared code, send the compiler worker the minimal reproducer, passing control, exact tool identity, suspected surface, proposed files, ownership/runtime contract, tests and expected build needs. Obtain explicit acknowledgment of file ownership and a validation window. If a shared file is actively owned by S18, let that worker implement the dependency or wait for a checkpoint; never race edits.

For an approved dependency, include compiler and native regressions, editor diagnostics/completion/hover where the checked surface changes, and complete IntelliSense documentation if introducing a built-in. Prefer test-local adapter support over a new public destruction feature. Keep dependency and UI checkpoints separately reviewable. Do not alter SQL cancellation/transaction semantics to accommodate the UI observer.

Coordination confirms technical ownership; it does not independently authorize a merge or consumer refresh. Follow the eventual user's dispatch permissions. If dependency integration is needed by both agents, agree its serial handoff explicitly before either changes main or shared executables.

## Acceptance matrix

Create `release-matrix.json` with L01–L26 in order. Each row includes witness, exact commands, expected and observed results, compiler identity, evidence references and one disposition: **executed**, **expected rejection**, **analysis/emission only**, **blocked**, or **not applicable with reason**. Split mixed rows into subcases; do not promote an entire row from one passing case. Link blocked subcases to their minimal reproducer and upstream dependency.

Record investigation completion, observer calibration, synchronous B2 closure, and async/thread readiness separately. Async/thread readiness remains unproven in this task.

| ID | Case | Required evidence |
|---|---|---|
| L01 | Baseline, ownership and tools | Identity repairs present; exact pin; acknowledged S18 window; owned files and unchanged shared consumers recorded. |
| L02 | Async/worker compatibility | All seven architectural boundaries assessed; current scalar identity concurrency limitation explicit; no synchronous-only production requirement adopted. |
| L03 | Observer boundary | Actual native release path, audit ownership graph, event meanings and checked adapter boundary identified; source markers cannot synthesize release. |
| L04 | Live/move calibration | Zero premature releases through function, record, array and retained-closure moves; replacement/removal release only the displaced resource. |
| L05 | Exact-once calibration | Explicit close and automatic scope exit each release once; duplicate logical cleanup adds no event; consumed-resource reuse rejects with a passing control. |
| L06 | Non-retaining audit | Audit survives resource release without retaining resource/owner; resource can outlive an audit handle; audit storage teardown is safe. |
| L07 | Assertion sensitivity | Wrong count, ID, missing/premature event and reversed dependency order fail; corresponding correct expectations pass. |
| L08 | Resource-bearing state | Real node state owns tracked resources; source storage/access/moves work; no scalar-only proxy claim. |
| L09 | Retained captures | Actual stored named/factory callbacks retain captures through repeated delivery; capture releases exactly once when registration is destroyed. |
| L10 | Removal outside delivery | Leaf/subtree/root removal completes applicable cleanup before return; stale same-owner repeated removal adds no release. |
| L11 | Active self retirement | Access ends immediately; active state/capture remains unreleased inside callback; releases after return, exactly once. |
| L12 | Active ancestor/root retirement | Active child removes ancestor/root; all retired resources remain safe through callback; descendants precede ancestors at cleanup. |
| L13 | Sibling and repeated retirement | Sibling removal preserves active access; repeated/overlapping retirements cannot double-release; multiple retirement requests preserve dependency order. |
| L14 | Nested order | Depth-three tree with siblings; independently expected descendant-before-ancestor constraints for state and captures; incidental sibling order not asserted. |
| L15 | Same-key replacement | Old resources release once; replacement remains live and fresh; stale queued work cannot invoke or release replacement. |
| L16 | Successful queued payload | Real resource payload follows checked ownership into FIFO delivery; no premature release; release after its documented last use. |
| L17 | Rejected/stale payload | Foreign/rejected enqueue respects documented transfer behavior and unchanged owner state; stale drain releases payload once without callback. Queue-clear subcase explicitly not applicable. |
| L18 | Automatic owner destruction | No explicit cleanup call; live state/captures/queued payloads release once; required descendant ordering measured independently, including a nontrivial tree. |
| L19 | Non-retaining identities | Escaped handles and surviving allocator do not retain dead-owner resources; independent live owners remain intact. |
| L20 | Repeated balance | At least1,000 native create/deliver/remove/drop cycles; all allocated IDs release exactly once and return to resource baseline; audit growth managed without hiding live resources. |
| L21 | Existing behavior and negatives | Complete I/H/R logical regressions including original/current B4, reentrant drain, live-list edits, fresh identity and strict ownership rejections remain valid. |
| L22 | Formatting and source fidelity | Whole-project checkpoint formatting; token preservation/idempotence; checked/emitted execution of formatted witnesses; emitted Rust unmodified. |
| L23 | Dependency or defect | Minimal failing case plus passing control; precise compiler/lifecycle/observer limit; approved fixes include regression evidence; no unexplained panic/timeout counted as expected blocker. |
| L24 | Matrix reconciliation | Every H01–H19, R01–R19 and I01–I20 row mapped to current or explicitly historical evidence; B1/B3/B4 preserved; B2 only closed for proved synchronous contract. |
| L25 | Suite and demo | All applicable rows dispositioned without silent skips; actual native demo; exit0 only for adopted synchronous contract fully proved, exit2 for evidenced remaining blocker, exit1 for unexpected error or invalid invocation. |
| L26 | Handoff and isolation | Commit(s), exact commands/results, adapter contract, compatibility table, remaining dependencies and isolated SQLite supplied; shared window released; no unauthorized merge/refresh/new milestone. |

Automatic owner destruction and callback-safe release cannot be waived as not applicable. Resource-bearing state and payload failures remain blockers rather than extrapolations from capture success. A calibrated synthetic observer establishes the tested ownership paths; it does not certify every native backend or asynchronous cleanup protocol.

A real observer may expose a cleanup defect. Preserve that failing observation, fix only within approved ownership, then rerun affected cases. Unsupported required behavior is a bounded blocker; an unexpected failure must first be diagnosed. Tool/window unavailability leaves acceptance incomplete.

## Execution sequence and deliverables

1. Resolve baseline, worktree ownership, S18 coordination and the async/worker compatibility table. Read all affected callers before changing the owner representation.
2. Establish the checked adapter route using the smallest native calibration program. If blocked, follow the compiler dependency protocol before dependent owner work.
3. Complete observer calibration, concrete resource-bearing owner integration and focused witnesses as one coherent section. Reuse the existing harness and test framework. Avoid running broad regressions after each edit.
4. During the reserved window, run the complete focused commands with the verified compiler:

   ```sh
   TSRUST=<verified-compiler> node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
   TSRUST=<verified-compiler> node experiments/ui-tsrust-owner-handles/cli.mjs demo
   ```

   The harness must include observer calibration and adapter/native build steps with explicit offline configuration and bounded subprocesses. Record any separate required native unit command exactly. Never silently build outside the agreed window. Tests exit0 only when assertions and matrix accounting pass; a blocker report passing its tests is not full contract acceptance.
5. Update `RESOURCE-RELEASE.md`, `release-matrix.json`, H/R/I reconciliation, README/demo and provenance. Preserve prior evidence. Clearly distinguish synchronous acceptance from future async/cancellation/thread work.
6. Format before source checkpoints, commit accepted owned work or a reproducible bounded blocker report, verify local evidence restoration, prune obsolete task outputs and release the shared window.

Hand off worktree path, commit(s), full matrix dispositions, exact checks, compiler/native provenance, observer semantics, actual lifetime findings, compatibility table and precise remaining compiler dependencies. No full unrelated compiler/LSP regression or performance milestone is required without a separately justified dependency change.

Stop after this bounded task. Do not begin rendering, JSX, route execution, a scheduler, worker transport, async UI cancellation, SQL integration or another UI milestone. Async and multithreading remain intended product capabilities with explicit unproved contracts, not rejected features.
