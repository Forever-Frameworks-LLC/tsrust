# Parallel UI task: Tsrust owner/handle feasibility

Status: proposed assignment, 2026-10-15; not dispatched. This document authorizes no implementation, worktree creation or merge.

Read [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), [UI vision](../../UI_VISION.md), [lifecycle assignment](ui-state-lifecycle-first-task.md), [merged lifecycle results](../../experiments/ui-state-lifecycle/README.md), and [S15 status](http-acceptance.md).

## One outcome

Determine whether ordinary Tsrust library code can express the synchronous owner/handle contract already demonstrated in Rust. Deliver executable source witnesses for supported operations and minimal, reproducible blockers for the rest. Do not implement new compiler features or publish a UI API.

The Rust prototype and callback-removal fix are merged at `5ff76855b9f2445a6c6b9bd26a369686b8dd16ec`. Its 20 Rust cases establish a reference model, not Tsrust support. Existing Tsrust witnesses already prove retained mutable counter closures and reject unsafe loan/transfer patterns. Reuse that evidence; focus on independent invalidation, registration ownership and cleanup.

S15 remains in acceptance and has changed task ownership and native emission. This task can run alongside it because it is confined to source experiments against a verified compiler. Any compiler/native-binding/editor implementation discovered here waits for separately assigned work after S15 acceptance.

## Isolation and baseline

- One worker in a separate managed Git worktree. Suggested baseline: `5ff76855b9f2445a6c6b9bd26a369686b8dd16ec`. Record the full resolved commit; do not import uncommitted main files. Supply this brief explicitly if it is absent from the selected commit.
- Own only `experiments/ui-tsrust-owner-handles/`. Read the existing lifecycle experiment and compiler source as references; do not modify them. No compiler, native-binding declarations, LSP, editor, root manifests/locks, production CLI, installed consumers, benchmark registrations, shared outputs or main SQLite writes.
- Use installed Node built-ins, existing Tsrust libraries and installed `rustc`. No downloads, Cargo builds, new Cargo targets, new Rust dependencies or hand-written native owner/handle adapter. A Rust implementation behind a new binding would bypass the question this task must answer.
- Identify the verified compiler's source commit, version and binary hash before using it. The prior experiment used the accepted S14 compiler from `6fce2b90`; do not assume the installed binary still matches that identity. An installed compiler and the checkout may describe different source revisions; record both. Do not depend on unaccepted S15 APIs.
- Keep tools read-only. Inspect actual CLI help. Prefer check/emit followed by direct `rustc` compilation in a task-specific temporary directory. If execution needs shared native caches, Cargo or unavailable dependencies, record an execution blocker rather than building implicitly. Recheck tool hashes after validation; preserve completed evidence and rerun only affected checks if a tool changed.
- Follow whole-project formatting before source checkpoints with `TSRUST=<verified-compiler> node tools/format-project.mjs`. Prove token preservation, repeat idempotence and formatted-source behavior. Existing formatter dependencies may be reused read-only, as in the previous experiment, with their identities and any resolver hook recorded. Do not install dependencies or bypass checks. Report unavailable tooling or outside-ownership formatting changes as checkpoint conflicts; never commit unrelated changes.
- Give S15 priority if shared tool replacement or resource contention prevents isolation. Do not rebuild or replace consumers to unblock this experiment. Keep `CARGO_INCREMENTAL=0` for any inherited tool environment; this task should not invoke Cargo.

## Bounded investigation

Start with an explicit owner passed to operations and an opaque handle value, rather than requiring a handle to mutate state by itself. Handle access must validate both owner and instance identity. This does not select public syntax or require a general-purpose `Weak` facility.

Investigate one small concrete representation using existing records, collections and callable contracts. A single alternate representation is permitted if a demonstrated blocker rules out the first. Avoid open-ended representation searches. Do not add a generic signal system, allocator, component engine or framework abstraction to make the example work.

The handle must not own the component state or a retained mutable loan into it. An application-supplied label is not sufficient owner identity unless the library enforces uniqueness within an explicitly documented scope. Repeated caller-selected IDs must not silently alias owners. Distinguish library-managed uniqueness from unforgeable language capabilities; this is trusted in-process code, not a wire security token.

Use real retained callbacks, constructed before delivery and invoked repeatedly after construction returns. At least two different captured environments must coexist in the registry. Do not replace callbacks with event-kind enums, canned outputs, copied state snapshots, global variables or a switch that merely simulates registration.

Keep integer counter/list state, one thread and a manually drained FIFO. Preserve the reference model's immediate handle invalidation and live-list removal, deferred cleanup while a callback is active, stale-event outcomes, checked identity exhaustion and child-before-parent disposal. If existing Tsrust cannot express a required operation, stop that dependent branch and record the gap; continue independent witnesses.

Do not edit emitted Rust to repair type errors, add missing ownership machinery or manufacture destructor observations. Generated source may be inspected. If exact resource release cannot be observed using existing supported facilities, report it as unproven. An empty registry or a manually incremented cleanup counter does not prove captured resources were dropped.

## Acceptance matrix

Each row must have an explicit disposition: **executed**, **analysis/emission only**, **expected rejection**, **blocked**, or **not applicable with reason**. A blocked row references a minimal witness, the actual failure or observation limit, and the dependent rows it prevents. Missing features must never become silent skipped tests or successful UI acceptance.

| ID | Witness | Required result or evidence |
|---|---|---|
| H01 | Tool and source provenance | Verified executable identity, checkout commit and isolated commands recorded; no shared output writes |
| H02 | Two owners with identical application keys | Independent state; wrong-owner read, mutation and event submission fail without affecting either owner; explain how identity uniqueness is enforced |
| H03 | Two retained callback environments | Construction returns; both registrations remain callable and read current state on repeated delivery; distinct captures do not alias accidentally |
| H04 | Non-owning handle access | Owner remains independently operable between deliveries; a retained handle does not impose a long-lived mutable loan or transfer component ownership |
| H05 | Remove with retained handle and queued event | New access/submission is stale; old queued delivery invokes no removed callback; unaffected items retain values |
| H06 | Reinsert the same key | Fresh initial state and identity; old handle/event cannot target the replacement, including after any storage reuse |
| H07 | Keyed list edit and reorder | Keys `1`, `2`, `4` retain edited values independently of position; duplicate live keys and invalid orders reject before mutation |
| H08 | Callback queues more work | Exact FIFO trace; later delivery sees the completed prior mutation; actual callback storage remains valid |
| H09 | Recursive drain and overlapping access | Explicit runtime rejection or documented compile-time rejection; no overlapping mutation, lost pending events or borrow panic |
| H10 | Callback removes itself/sibling/ancestor | Current permitted access finishes; subsequent delivery is stale; ancestor disposal preserves child-before-parent cleanup |
| H11 | Callback removes `2`, then reads list | Immediate result contains only surviving `1`, `4`; no stale entry contaminates list reads |
| H12 | Callback removes `2`, then reorders survivors | Reordering to `4`, `1` succeeds immediately and persists after callback return |
| H13 | Callback removes and reinserts `2` | Reads/reorders expose only the replacement; duplicate replacement rejects; old queued event remains stale and new delivery works |
| H14 | Explicit disposal with outstanding handles/events | State and captured resources release exactly once at the defined safe point; handles do not retain them; payloads release on drain/clear/drop |
| H15 | Owner leaves scope without explicit disposal | Automatic resource release is observed where supported; distinguish rejection of later owner access from a runtime stale result requiring an owner; never access freed memory to test it |
| H16 | Repeated cycles and finite identity space | 1,000 bounded cycles return observable resources to baseline; exhaustion rejects before reuse/wrap; evidence limits stated if actual drops cannot be observed |
| H17 | Ownership rejection controls | Strict analysis/emission rejects escaped loans, overlapping mutable access and use after transfer using valid supported syntax; preserve existing safety rules |
| H18 | Formatting and independent assertions | Successful source preserves tokens, formats idempotently, checks/emits and runs with `std:testing`; deliberately wrong expected value must fail native execution |
| H19 | Deterministic demo or blocked report | Demonstrate supported counter/list/handle behavior from the actual Tsrust implementation; report blocked operations explicitly without substituting the Rust model |

H02–H16 refine the existing L01–L14/L17 lifetime cases. H11–H13 are mandatory regressions from the merged bug fix. H17–H19 preserve L15/L16/L18 evidence requirements. The earlier Rust suite remains reference evidence; rerunning it is not a substitute for these source-level results.

Compile-time rejection can be a useful alternative for unsafe reentrancy, but it does not automatically satisfy an API promised to return a runtime error. Document any semantic narrowing and its consequences for shared UI callbacks. Full feasibility requires either the original semantics or an explicitly reviewed narrower contract.

## Sequence and blocker handling

1. Inspect current callable/collection/resource support and the existing witnesses. Document the candidate ownership relationships and who owns each registration, event payload and state value. No speculative public API design is required.
2. Probe only the capabilities that determine feasibility: owner identity, non-owning access, retained callback storage and scoped mutation. Early checks are justified here because their results determine whether the implementation branch can proceed.
3. Complete the coherent supported slice and its regression witnesses before batch validation. If callback registration is blocked, do not build a command-only substitute or duplicate the Rust prototype. Independent identity/list witnesses may still proceed, clearly labeled as partial.
4. Validate formatted Tsrust through the project/module path for imports and direct native execution where isolated. Use `std:testing` assertions and expected traces independent of the implementation. Keep expected ownership errors separate from missing syntax/type support. Editor suppression comments must not replace strict rejection checks.
5. Finish the matrix, demo or blocked report, and the smallest follow-up recommendation. Do not broaden into compiler fixes. No full compiler/LSP regression or benchmark timing is required for this experiment.

For each blocker, retain the smallest supported-syntax source that demonstrates it, compiler command/output, expected behavior, actual diagnostic or native failure, and inspected compiler locations. State whether it appears to need an ordinary-library design change, compiler callable/ownership support, native resource binding, editor support, or merely a better observation method. Do not infer a new language requirement from one failed representation.

Expected inspection starting points include `crates/tsrust-compiler/src/analysis/checked_callable_storage.rs`, `analysis/capture_loan_binding.rs`, `callables/borrowed_callable_contracts.rs`, `emit/callback_capture_emit.rs`, and `native/{types,bindings,signatures}.rs`. These are read-only leads, not approved edit targets.

## Deliverables and checks

```text
experiments/ui-tsrust-owner-handles/
  README.md       representation, proof levels, matrix, blockers and handoff
  witnesses/      supported Tsrust implementation/examples and minimal rejections
  tests/          Node built-in orchestration and independent assertions
  cli.mjs         demo / explicit blocked report
  .gitignore      isolated evidence and disposable task outputs
```

Do not create this directory during planning. During implementation, keep helpers minimal and reuse existing reference material read-only. Do not refactor the prior experiment to share a harness.

Required entry points:

```sh
node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
node experiments/ui-tsrust-owner-handles/cli.mjs demo
```

Tests may pass when they correctly establish an expected compiler rejection or a documented capability blocker; the printed matrix must still say **blocked**, not supported. A missing tool, unexpected rejection, incorrect output or failed assertion fails validation. The demo exits zero only for a completed Tsrust demonstration of the required contract; otherwise it emits the explicit blocker report and exits with a documented nonzero status. Do not hardcode a successful transcript.

Every subprocess needs bounded duration, captured exit/output and cleanup of task-created outputs. Record the exact expanded compiler/native commands. Successful analysis/emission alone is not native execution. Pure algorithms may have an independent Node oracle, but Node must not execute the supposed Tsrust owner implementation.

## Completion, evidence and stop

Completion means every row is dispositioned with evidence, not necessarily that every capability exists. Distinguish **investigation complete** from **owner/handle contract feasible**. A blocked investigation can be committed as useful evidence, provided the handoff names the blockers and never claims full UI acceptance.

Before the source checkpoint, run required whole-project formatting in the isolated worktree and affected checks. Commit only the owned experiment. If formatting requires changes outside ownership, report that conflict rather than committing unrelated changes.

Use an ignored task-local SQLite database with the existing storage tooling for closed evidence. Record source/tool hashes, commands, original failures, successful repairs and matrix results. Verify byte-exact restoration and database integrity before removing loose copies. No archives or source snapshots. Delete obsolete task-created native executables; never prune shared S15 outputs.

Hand off the worktree path, commit, exact check/demo results, evidence location and recommendation: proceed with an ordinary library, review a narrower contract, or schedule a specific compiler dependency after S15 acceptance. Keep evidence isolated for owner-authorized serial integration. Do not merge, modify main analytics or refresh consumers.

No rendering, browser execution choice, JSX lowering, HTTP/loader integration, reactive graph, general shared-state API, DI, component catalog, public built-in or next UI milestone. Stop after this feasibility task.
