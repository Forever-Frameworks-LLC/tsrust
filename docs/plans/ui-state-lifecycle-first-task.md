# Parallel UI task: component state and lifecycle feasibility

Status: proposed assignment, 2026-10-15; not dispatched. Planning only. The route-manifest prototype and regression fix are merged through `01f3a1d45aa6e6f81422d059f84a341f13581df2`. S13/S14 are accepted; S15 HTTP remains pending. Older status paragraphs in the UI plans predate those acceptances.

Read [AGENTS.md](../../AGENTS.md), [development workflow](../development-workflow.md), [UI vision](../../UI_VISION.md), [feasibility decision register](ui-feasibility-review.md#remaining-decisions-and-when-they-block), [parallel boundaries](parallel-ui-foundations-plan.md), and [S13](async-tasks-acceptance.md)/[S14](tokio-tasks-acceptance.md) acceptance. This assignment narrows the lifecycle research described in those documents; it does not change the product vision.

## Outcome and decision

Prove or disprove a small synchronous component-owner model with retained events, independent state, stable list identity and deterministic disposal. Deliver executable evidence and a precise inventory of what the accepted Tsrust compiler can express. The task can succeed as a feasibility investigation while concluding that generated interactive UI remains blocked.

This addresses a runtime risk that async/HTTP acceptance does not resolve. It selects neither browser execution technology nor native renderer, and does not freeze a public signal/component API. Keep shared web/native behavior as the eventual goal; a headless model is not paired-target acceptance.

## Isolation and prerequisites

- One worker in a separate managed Git worktree. Suggested committed baseline: `01f3a1d45aa6e6f81422d059f84a341f13581df2`, which includes S13/S14 and the route prototype. The integration owner must provide this brief if it is not in that baseline. Record the resolved full commit. Do not copy main's uncommitted S15 source.
- Own only `experiments/ui-state-lifecycle/`. Read other source as needed. Do not change the route prototype, compiler, LSP, editor, root manifests/locks, production CLI, benchmark registrations, installed consumers, shared native/build caches or main analytics database.
- Use installed Node built-ins and Rust standard library only. No downloads, dependencies, Cargo workspace members, second Cargo target or new HTTP runtime. Prefer `rustc --test` for the isolated Rust harness, with disposable outputs under the experiment or a task-specific temporary directory.
- Use `/Users/seancotter/.cargo/bin/tsrust` only after identifying its verified source provenance and recording its hash/version. Treat installed tools as read-only. Check their identities again after checks; if S15 replaces a tool, retain completed evidence and rerun affected checks against a stable verified tool. Do not rebuild, replace or archive consumers.
- Inspect actual CLI help before constructing compiler commands. Use check/emit and a direct native compilation only if they can avoid shared output/cache writes. If generated native execution requires shared Cargo outputs or unavailable dependencies, record that blocker; do not bypass isolation with an implicit build. A check-only witness must not be called an execution proof.
- Follow whole-project formatting before every source checkpoint in the isolated tree: `TSRUST=/Users/seancotter/.cargo/bin/tsrust node tools/format-project.mjs`. Verify token preservation, idempotence and formatted-source behavior. If the formatter needs unavailable tooling or changes files outside ownership, report the checkpoint conflict; do not commit those changes, weaken the formatter or install dependencies. Preserve unrelated edits.
- If capacity or shared tool contention prevents isolation, pause this experiment behind S15. S15 has priority. This document authorizes no dispatch, worktree creation, implementation or merge by itself.

## Experimental behavioral contract

Use one UI thread and an explicit owner for a component subtree. The owner owns mutable state and registered handlers; externally retained handles must not keep removed state alive. Start with the smallest concrete counter/list implementation that can test these rules. No generic component engine is required.

1. **Identity:** each mounted instance has an identity distinct from its application item key. Handles validate owner and instance identity. A stale handle must never address a replacement, even when the same item key or storage slot is reused. If using generation counters, check exhaustion rather than wrapping into an old identity.
2. **State access:** event handlers read current state at delivery time, then mutate it synchronously. No mutable borrow escapes a delivery. Reads produce ordinary values for assertions; no reactive subscriptions, dependency graph or automatic rendering is implied.
3. **Retained events:** registration outlives the construction call and supports repeated delivery. A retained callback must exercise actual access to owned state; a list of canned counter outputs or a command-only simulation is insufficient evidence for callback feasibility.
4. **Ordering:** a small explicit FIFO holds pending deliveries. An event queued during delivery runs after the current callback returns. Recursive draining is rejected with a defined result; there is no recursive mutable access or hidden task scheduler. Tests drive draining explicitly, without timers or sleeping.
5. **Removal:** removal immediately invalidates access and prevents new events for the removed instance. An already running callback may finish its current access, but pending deliveries must not run afterward. Resources held by that callback are released on its return. Disposing an owner removes its descendants and their registrations. Repeated disposal is a defined no-op.
6. **Stale delivery:** return an explicit stale/disposed outcome without mutation or panic. Queue entries must not retain component state or handler captures after disposal; draining stale entries releases their payloads. Clearing/dropping the queue also releases payloads.
7. **List identity:** use opaque application keys `1`, `2`, `4`. Reordering preserves each item's state; removal disposes only that item; reinsertion creates a fresh instance. Reject duplicate live keys before changing the list. Empty keys may be rejected explicitly; no numeric coercion or route/database inference.
8. **Scope:** no implicit global/shared component state. Parent state survives child removal. Two owners may use identical item keys without interference. Define child-before-parent cleanup, with stable sibling insertion order for the test trace; this is an experimental policy, not a public guarantee.

An explicit safe Rust owner/registry with non-owning validated handles is a candidate, not mandated structure. Standard-library borrowing/interior mutability may be used where justified, but invalid event sequences must yield defined results rather than `RefCell` borrow panics. No `unsafe`, leaked allocations, process-global state, arbitrary deep clones or strong reference cycles to hide an ownership problem.

## Acceptance matrix

Every row needs a named witness and independent expected values, delivery trace or drop counts. Record whether it is proven by the Rust model, Tsrust analysis, or generated native execution. Do not collapse those columns into one pass mark.

| ID | Witness | Required observable result |
|---|---|---|
| L01 | Two counters mounted with equal initial values | Updating A twice and B once gives A=2, B=1; no shared state |
| L02 | Handler retained after construction returns | Repeated delivery reads the latest value; real captured/handled state remains valid until removal |
| L03 | Two owners with the same item keys | A handle from one owner cannot read or mutate the other's state |
| L04 | Edit then reorder items `1`, `2`, `4` | Edited values remain attached to keys, independent of position; no remount/drop on reorder |
| L05 | Duplicate live key; invalid key under chosen policy | Explicit rejection before mutation; values, ordering and live-resource counts unchanged |
| L06 | Remove item `2` with a queued event and retained handle | Remaining items unchanged; new access/delivery is stale; queued event never invokes removed handler |
| L07 | Reinsert key `2`; reuse internal storage if applicable | Fresh initial state; old handle/event cannot address replacement; fresh handle works |
| L08 | Drop child then parent; dispose twice | Child cleanup precedes parent; each resource drops once; parent remains usable until its disposal |
| L09 | Callback queues another callback | Exact FIFO trace; second callback observes completed first mutation; no overlapping mutable access |
| L10 | Callback tries recursive drain | Defined rejection, no panic; pending queue stays valid and can later drain |
| L11 | Callback removes itself or a sibling with pending work | Current delivery completes; removed instance receives no later delivery; cleanup occurs at the defined safe point |
| L12 | Dispose root with retained handles and pending deliveries | State and handler captures release without waiting for external handles; queued payloads release on drain/clear/drop |
| L13 | Owner dropped without explicit dispose | Same exact-once resource release; surviving handles return stale rather than retaining state |
| L14 | 1,000 fixed mount/edit/reorder/remove cycles | Tracked state, registrations and payload counts return to baseline each cycle; no sleeps or timing assertions |
| L15 | Invalid ownership source witnesses | Strict compiler analysis/emission rejects escaped local loans, overlapping mutable access and use after consuming owner, where expressible; record actual diagnostics |
| L16 | Successful Tsrust source witnesses | Format preserves tokens; second pass stable; checked/emitted source retains assertions; execute natively where isolation permits |
| L17 | Identity exhaustion if finite reusable generations are used | Exhaustion rejects allocation/reuse without wrapping, corrupting existing state or reviving stale handles; otherwise document why mechanism is inapplicable |
| L18 | Deterministic demonstration | Fixed event script shows independent counters, preserved reordered edits, stale delivery after removal, fresh reinsertion and zero final tracked live resources |

Tracked drop counts are scoped evidence about owned test resources, not an allocator-wide leak proof or performance benchmark. Panic recovery, allocation failure recovery, multithreading and async cancellation are outside this contract; do not claim they passed.

## Tsrust capability inventory and stop rules

Use minimal ordinary `.tsr` programs and existing `std:testing` assertions where available. Probe the underlying ownership operations without inventing `.tsrx`, UI built-ins, decorators or compiler annotations. Reuse accepted callable/ownership behavior; do not assume async tasks provide persistent UI state handles.

For each required operation—retaining an invocable handler, accessing mutable owned state across deliveries, non-owning invalidation, instance identity, subtree cleanup—record:

- Source witness and expected observable behavior or rejection.
- Exact compiler command, exit status, diagnostics and executable result if obtained.
- Status: supported and executed; supported by analysis/emission only; expected rejection; or missing language/runtime capability.
- Smallest missing contract, candidate integration location identified from source, and whether it requires compiler, library, editor or eventual renderer work.

Do not call unsupported syntax a positive rejection proof of a lifetime rule. Keep expected ownership errors separate from missing language features. Intentionally malformed lexical sources belong in inline harness strings; checked-in rejection fixtures follow the workflow's editor suppression rules and still undergo strict rejection checks.

A missing capability is a result, not permission to implement it. Continue the independent Rust witnesses, then report the gap. If the Rust model fails an acceptance row, repair within this scope or report a no-go with its smallest reproducer. Never report an incomplete or failing matrix as accepted. No automatic follow-on compiler task.

## Deliverables and validation

```text
experiments/ui-state-lifecycle/
  README.md             provisional contract, commands, matrix results, limitations
  lifecycle.rs          minimal model, Rust tests and deterministic demo
  witnesses/            minimal Tsrust positives and reasoned rejections
  tests/                Node built-in harness for Rust and compiler witnesses
  cli.mjs               demo runner only
  .gitignore            task-local disposable output and isolated evidence
```

These are target deliverables, not files to create during planning. Avoid helper layers unless shared by the tests and demo. The Node harness orchestrates installed tools; it is not a proposed application runtime. It must propagate subprocess failures and missing-tool/blocker status, bound process duration, and clean its own temporary outputs. A capability blocker must appear explicitly in the report, never as a silent skipped test or successful execution.

Implement the coherent model and regression witnesses first, then batch its focused checks. An earlier compiler capability probe is appropriate only where the result determines implementation feasibility. Repair failures and rerun affected checks; no compiler/LSP full suite or timing run for this experiment.

Required entry points from the isolated worktree:

```sh
node --test experiments/ui-state-lifecycle/tests/*.test.mjs
node experiments/ui-state-lifecycle/cli.mjs demo
```

The tests compile/run the Rust assertions using installed `rustc` and exercise the Tsrust inventory. The demo uses the same model, not a second simulation. Record expanded subprocess commands, versions/hashes and actual results. Before committing, complete the required whole-project formatting and verify the formatted owned fixtures. If outside-ownership formatting prevents a checkpoint, retain the implementation and evidence and report the conflict instead of committing unrelated changes.

## Evidence, handoff and stop

Store closed command output, source/tool identities, expected versus observed results and failures in a task-local ignored SQLite database. Use existing storage tooling where practical; do not build another analytics system. Verify byte-exact restoration and database integrity before deleting closed loose logs. Keep no archives or source snapshots. Prune only this task's obsolete generated outputs.

Commit only the completed owned experiment after focused acceptance and formatting, or clearly report any checkpoint blocker. Hand off worktree path, full commit, exact checks/results, per-row proof level, capability gaps and a recommendation to proceed, narrow the design or wait. Keep the closed database isolated for owner-authorized serial integration into canonical SQLite; do not merge or import into the main store from this worker.

Passing this prototype permits a later proposal for real paired browser/native input; it does not authorize it. S15 acceptance still gates HTTP/UI middleware, request providers, loaders and full/fragment transport. Browser execution, JSX lowering, reactive subscription tracking, styling, text/IME/focus/accessibility, routing lifecycle integration and public API/editor support remain unproven.

Stop after this task. No rendering, server, scaffolding, component catalog, general scheduler, signal graph, DI, production library or next UI milestone.
