# Parallel UI task: lifetime contracts and resource-observation feasibility

Status: proposed assignment, 2026-10-15; not dispatched. Planning only. This document does not authorize implementation, worktree creation, compiler changes or merge.

Read [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), [UI vision](../../UI_VISION.md), [owner/handle assignment](ui-tsrust-owner-handles-first-task.md), [merged feasibility results](../../experiments/ui-tsrust-owner-handles/README.md), [H01–H19 matrix](../../experiments/ui-tsrust-owner-handles/matrix.json), [Rust lifecycle reference](../../experiments/ui-state-lifecycle/README.md), and [current S15 status](http-acceptance.md).

## One outcome

Specify the runtime identity boundary and the narrower active-callback contract, then determine whether existing supported Tsrust facilities can provide trustworthy observations of resource release. Deliver executable evidence where possible and precise blockers elsewhere. This is a bounded follow-up to the merged experiment, not a public UI runtime or a compiler milestone.

The feasibility work and H10 regressions merged at `169f34bbba97aa43d41c99d26ea9314ddf8a52b3`; experimental source came from `8ec68bf097e7a3f2da207b59394d259da7caee70`. It proves retained callbacks and logical invalidation, but leaves B1 (identity scope), B2 (resource observation) and B3 (active callback state access) explicit. Original source, matrix dispositions and evidence must remain attributable to those commits.

Recommended order: settle the two contracts, establish whether an observer is credible, then execute only the lifetime cases that observer can actually distinguish. Do not add a large runtime to manufacture an observable result.

## Proposed contract decisions

Dispatching this assignment using these recommendations adopts the following experimental contract. It does not claim that its implementation or full resource behavior has already been proven.

### Identity: one allocator per UI runtime

- One library-controlled allocator owns the owner-ID sequence for a runtime. Instance IDs remain monotonic within their owner; zero remains the structural root sentinel. Same application keys in different owners are independent.
- Handles contain identity only; they do not own state or retain mutable loans. Handles are runtime-local, not serialized, persisted, sent between processes, or accepted from external input. No promise of stable IDs across runs.
- The supported API must prevent cross-runtime handle use through checked construction/access boundaries or reject it before reading, mutating, removing, reordering or queueing work. A comment saying “do not mix runtimes,” caller-selected labels, or resetting a counter is not enforcement. Two independently created runtimes are a mandatory counterexample.
- Distinguish trusted in-process misuse from deliberate mutation of internal structural fields. Unforgeable security capabilities are not required, but normal supported operations must not silently alias owners across runtimes. Explain what callers can construct and which boundary enforces validity.
- Check owner and instance exhaustion before wrap/reuse. A retired handle must never address a replacement, including after runtime teardown and recreation if such a handle remains representable.

Investigate one minimal ordinary-library boundary using existing language facilities. One alternate is permitted only after a concrete failure of the first. If scope cannot be enforced under this budget, retain B1 with the smallest counterexample and a specific follow-up proposal. Do not introduce global registries, random identity infrastructure, generic shared ownership, or a new native identity service in this task.

### H10: retirement ends state access immediately

- Self, ancestor or root removal immediately invalidates affected handles and removes affected items from live list membership.
- The current callback may finish executing, but access to retired state through its handle ends immediately: `read` returns `undefined`, `add` returns `false`, and new event submission fails. There is no separate surviving mutable state loan.
- Removing only a sibling leaves the active callback's state accessible. Operations on unaffected live nodes remain permitted. Recursive draining still rejects; existing FIFO and stale-event behavior remain unchanged.
- Removal is not a callback exception and does not force an early return. User code can finish unrelated work, but cannot rely on a final mutation of retired state. Record this consequence in an ordinary callback example.
- Resource release remains deferred until the active callback finishes normally; invalidation and physical release are separate events. Retired descendants must release before their retired ancestors. Outside a callback, explicit removal completes cleanup before returning.
- Same-key reinsertion creates a fresh identity. It must not revive the old callback, old handle or queued event. Preserve H11–H13 in full.

This explicitly supersedes only the Rust reference's surviving current-state loan. It does not weaken child-before-parent disposal, exact-once release, automatic owner cleanup or non-owning handles. Record B3 as resolved by an explicit experimental contract decision only when the decision and regressions are present. H10 stays blocked while B2 prevents its resource assertions. Do not rewrite historical evidence as if the original contract had passed.

Exception/panic recovery, async cancellation, multithreading and behavior during callback unwinding remain excluded. Do not infer those guarantees from normal-return witnesses or from S15 HTTP resource tests.

## Resource-observation investigation

First identify the observable resource, its actual owner, the event indicating release, and how the observer survives without retaining that resource. Inspect existing testing/resource facilities and their implementation read-only. S15 disposal evidence is a useful lead, not proof that a source-level UI observer is available.

Evaluate at most two existing observation approaches. A useful observer must distinguish retained from released, release during a callback from release after callback return, one release from two, and child release from parent release when those claims are made. State any weaker proof explicitly. A released descriptor becoming unusable, for example, does not by itself prove exact-once destruction or total ordering.

Allowed:

- Ordinary Tsrust witnesses using existing supported libraries and owned captured resources.
- Read-only inspection of emitted Rust to explain ownership and drop paths, labeled analysis/emission evidence.
- An external test observer for real supported resource effects, provided it does not implement the owner/handle behavior or patch emitted code. Use only disposable local resources; no network/service dependency.
- The smallest experiment-only changes needed to carry an already supported observable resource through the actual state, registration or event path. Prove those paths separately; an unrelated resource sample does not establish UI cleanup.

Not allowed:

- New compiler/native bindings, custom native owner adapters, custom Rust destructors added behind the source witness, or instrumentation inserted into emitted Rust.
- Manually incremented cleanup counters, empty arrays, successful reads, process exit, RSS measurements or integer payload disposal presented as destructor proof.
- Refactoring the owner into a generic framework solely to accommodate observation. If resource-bearing state or event payloads cannot fit this bounded experiment, mark those cases blocked rather than extrapolating from callback captures.

Define release requirements separately for state, captured resources and queued payloads. Explicit removal, normal callback completion, stale-event drain and automatic owner scope exit are different paths. Test each supported path. Queue clearing is required only if an explicit clear API is introduced; otherwise mark it not applicable and still test stale drain and owner drop. Root instance zero has no state resource in the existing representation; do not fabricate a root destructor assertion.

If no credible observer exists, stop that dependent branch. Deliver the smallest supported-syntax witness, exact diagnostic or observation limit, inspected source locations, and a narrowly scoped observation-support proposal with its required tests. This is a successful investigation outcome, not full lifetime acceptance. Do not implement the proposed compiler/native dependency.

## Isolation alongside S15

At planning time S15 reports correctness accepted, with final consumer refresh and matched timings in progress. Re-read its status at dispatch; do not assume the pending work has finished.

- One worker in a separate managed Git worktree. Select a committed baseline containing merge `169f34bb`; prefer the latest relevant verified S15 checkpoint available at dispatch. Record the resolved full commit. Do not copy uncommitted main files. Supply this brief explicitly if it is absent from that baseline.
- Own only `experiments/ui-tsrust-owner-handles/`. Keep contract decisions and the follow-up matrix there. Read earlier experiments, compiler source and plans without modifying them. No main-checkout edits during execution.
- No compiler, native-binding declarations, LSP/editor, root manifests/locks, production CLI, benchmark registrations, installed consumers, shared build outputs or main SQLite writes. No merge or worktree cleanup that discards evidence.
- Use installed Node built-ins, existing supported Tsrust facilities and installed rustc. No downloads, dependency installs, Cargo builds, new Cargo targets or consumer replacement. Set `CARGO_INCREMENTAL=0` in inherited tool environments.
- Establish the compiler source commit, version and binary hash from verified provenance. The old experiment pins S14 and will reject the replaced executable. Update that pin only as an explicit experiment change after provenance review; preserve the old compiler identity and results in evidence. Revalidate the whole focused experiment under the selected compiler before claiming continuity. A new pin alone is not validation.
- Recheck tool hashes after validation. If S15 replaces a tool, preserve completed results under the old identity and defer or rerun affected checks after the new executable is verified. Do not copy old binaries as backups, rebuild them or silently disable the pin.
- Give S15 timing priority. Documentation and read-only inspection can proceed during measurements. Schedule compiler/rustc/native test execution and whole-project formatting outside active S15 timing windows. Do not start those loads without evidence the timing window is clear; if that cannot be established, finish independent work and report validation pending. Do not stop S15 processes.
- Direct check/emit/rustc operations use disposable task-specific directories, bounded subprocess durations, captured status/stdout/stderr and cleanup. If a candidate needs shared native caches, Cargo or unavailable dependencies, report that execution constraint instead of building implicitly.
- Before source checkpoints, run whole-project `TSRUST=<verified compiler> node tools/format-project.mjs` in the worktree. Prove token preservation, repeat idempotence and formatted-source behavior. Existing formatter dependencies may be reused read-only with recorded identities. Any required formatting outside ownership is a checkpoint conflict: report it; never commit unrelated changes.
- Preserve unrelated owner changes. Keep closed evidence in an ignored task-local SQLite using existing storage tooling. Verify byte-exact restoration and integrity before removing loose records; no archive directories or source snapshots. Keep main analytics untouched until separately authorized serial integration.

## Acceptance matrix

Give every row a proof disposition: **executed**, **analysis/emission only**, **expected rejection**, **blocked**, or **not applicable with reason**. Record contract decisions separately from proof status. A blocked row names its minimal witness, exact failure/observation limit and dependent rows. No silent skips or full-acceptance claim from a passing harness.

| ID | Case | Required result/evidence |
|---|---|---|
| R01 | Baseline, tools and timing isolation | Full source/tool identities, S15 status and clear validation window recorded; no shared builds or consumer changes |
| R02 | Explicit contract decisions | Runtime-local identity and immediate loss of retired state access documented, with changed Rust semantics and unaffected guarantees named |
| R03 | Independent owners in one runtime | Same keys remain independent; wrong-owner reads/mutations/submissions/removal reject without side effects; allocator is library-controlled at the supported boundary |
| R04 | Independent runtimes | Reproduce the old collision; proposed boundary rejects or statically prevents cross-runtime use through all applicable supported access paths, or retain B1 |
| R05 | Stale identity and exhaustion | Remove/reinsert, surviving handles after teardown, and runtime recreation cannot alias through supported operations; last valid identity succeeds, next rejects before wrap without damaging live state |
| R06 | Self, ancestor and root removal | Callback continues normally; affected reads/mutations/submissions reject after removal; repeated queued events are stale; no surviving state loan claimed |
| R07 | Sibling-only removal | Active callback state remains readable/mutable; removed sibling access is stale; unaffected queued work retains FIFO behavior |
| R08 | Existing logical regressions | H03–H09 and H11–H13 retain native assertions, including two real retained captures, immediate list reads/reorder/reinsertion and guarded recursive drain |
| R09 | Observer calibration | Positive live-resource control, release observation and deliberate wrong expectation fail/pass as appropriate; exact-once and ordering claims have independent support, or are explicitly blocked |
| R10 | Explicit removal outside delivery | Actual supported state and registration capture resources release once before removal returns; retained handles do not keep them alive; duplicate removal releases nothing again |
| R11 | Removal during delivery | No retired resource release before normal callback return; subsequent safe-point cleanup releases once, with immediate logical invalidation independently asserted |
| R12 | Child-before-parent release | A nested tree with siblings produces an independently expected descendant-before-ancestor release order; cover active-child ancestor/root removal; distinguish capture and state observations |
| R13 | Event payload paths | Real resource-bearing payloads release on successful delivery, stale delivery and owner drop; explicit clear if supported; integer deltas alone do not satisfy this row |
| R14 | Automatic scope exit | Supported state/captures/payloads release once without explicit remove; no resource retained by an escaped identity-only handle; no access to freed memory used as a test |
| R15 | Repeated lifetime cycles | 1,000 bounded cycles restore actual observable resources to baseline; include retained stale handles/events; logical collection lengths are supplementary only |
| R16 | Ownership controls | Strict analysis/emission preserves escaped-loan, overlapping-access and use-after-transfer rejection; no suppression, implicit cloning or ownership relaxation used to pass |
| R17 | Formatting and native assertions | Whole-project tokens preserved, second pass stable, formatted source checks/emits/runs; a deliberately false resource expectation fails when an observer is available |
| R18 | Old/new matrix reconciliation | Every H01–H19 row maps to current evidence, contract change or retained blocker; B1/B2/B3 statuses and dependencies are explicit; original evidence remains historical |
| R19 | Demo/report and handoff | Actual supported native witnesses run; exit 0 only when the adopted contract is fully demonstrated, exit 2 for a reproduced capability/observation blocker, exit 1 for unexpected validation failure; commands and limits recorded |

An unavailable/replaced tool or an active timing window is a validation deferral, not proof of a language blocker. It must not turn into a passing investigation report. Exact-once release, ordering and balance may have different proof levels; never promote the whole row based on one successful observation. Full H10 acceptance requires R06/R07 and the relevant resource rows, even after the B3 contract decision.

## Execution sequence and deliverables

1. Establish baseline, verified compiler and S15 isolation. Record proposed decisions in `CONTRACT.md` within the existing experiment; do not expand public API design.
2. Bound the identity investigation and evaluate observer candidates. Early minimal probes are justified only to decide whether dependent work is possible. Retain failed probes and avoid an open-ended representation search.
3. Finish the coherent supported witness slice and regression assertions before batch validation. Keep the H10 narrowed semantics and existing callback-list regressions explicit.
4. Reconcile H01–H19 in `matrix.json`; add `resource-matrix.json` for R01–R19, decision status, proof levels and blocker dependencies. Update README and the existing test/demo entry points. Reuse the current harness; no separate runtime or duplicated framework.
5. Run whole-project formatting and focused checks outside S15 timing windows. Repair and rerun affected checks only. No full compiler/LSP suite, benchmarks or performance claims.
6. Commit only the owned experiment after validated checks, or a reproducible capability-blocker report. If external validation is still pending, clearly label the handoff incomplete rather than claiming acceptance.

Required entry points remain:

```sh
node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
node experiments/ui-tsrust-owner-handles/cli.mjs demo
```

A passing suite may establish a real blocker. A successful demo must not substitute the Rust reference, print a canned success trace, or count contract decisions as resource evidence. A blocked report should run the supported independent witnesses and identify exactly what was not demonstrated.

Hand off the worktree path, source commit, exact commands/results, compiler provenance, adopted contract differences, both matrices, blockers and isolated SQLite location. Recommend either proceeding with the ordinary library, accepting a documented remaining restriction, or a specific separately scoped compiler/native observation task after S15 completion.

Stop after this assignment. Do not merge, refresh consumers, integrate main SQLite, start rendering/JSX/HTTP integration, add a reactive graph, implement DI or begin another UI milestone.
