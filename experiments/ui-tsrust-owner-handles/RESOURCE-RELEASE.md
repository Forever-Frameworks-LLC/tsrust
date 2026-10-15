# B2 resource-release investigation

## Result and boundary

The native observer candidate can be calibrated independently, but the current checked external Rust bridge deliberately rejects its resource type. B2 remains open. No resource-bearing UI state, capture or event claim is inferred from native-only execution. The current owner is unchanged.

The minimal reproducer is `release/blocked.ts`, copied into a disposable `.tsr` file with `release/tracked.rs`. Both strict check and emit must reject with:

> Boundary data cannot implement Drop; keep resource owners private to the Rust adapter.

The diagnostic is `rust-bridge`. The paired `release/control.tsr` and `plain.rs` carry owned plain data across the same bridge, format, check, emit and execute assertions. The rejection is not a panic or a missing package. The template uses `.ts` so ordinary recursive workspace checking does not mistake an intentional rejection for application source.

## Observer contract

`tracked.rs` owns a synthetic resource with a monotonic native ID. Its actual Rust `Drop` implementation appends that ID to a mutex-protected ordered audit. Explicit consuming `close` delegates to Drop. The event means destruction of this synthetic resource wrapper; it does not measure SQL rollback, file close completion or async backend cleanup.

The audit owns only integer events, never resources or owners. Snapshots copy integer events. Resources do not borrow the audit; process-lifetime audit storage remains available after destruction. Each calibration process starts fresh. No reset can hide a live resource. The audit is deliberately process-scoped test instrumentation, not a proposed UI global registry or production API. IDs use checked atomic increment; exhaustion fails before reuse. Public Rust fields exist only to make the bridge rejection minimal, not to enforce an opaque production resource boundary.

Native calibration covers moves through functions/records/arrays/closures, repeated closure calls, replacement, explicit close, scope exit, duplicate logical cleanup, snapshots surviving release, one ordered child/parent sequence, and1,000 release cycles with unique IDs. Wrong count, wrong ID, missing release, premature release and reversed order expectations must fail natively. A separate native illegal-use-after-close control must reject with E0382.

These controls calibrate the measurement mechanism only. Explicitly dropping a native child then parent does not establish automatic UI tree ordering. Native1,000-cycle balance does not establish UI balance. Audit snapshots are values, not owned audit handles; audit-handle drop semantics are not claimed. Tsrust resource moves and calibration remain blocked at the bridge.

## Why the compiler guard is justified

At baseline49704b4f, `crates/tsrust-compiler/src/project/rust_data.rs` rejects exported data implementing Drop. Its fields must be public concrete data. `project/rust.rs::shape` restricts borrowed inputs and rejects returned traits/borrows; a private opaque owner or returned Rust closure is not an existing alternative resource bridge. `emit/rust.rs` reconstructs imported records field by field through `rust_data_output` and `rust_input`; it does not preserve an opaque native owner and its destructor. The existing `tests/rust_bridge.rs::data_boundaries_reject_private_recursive_and_borrowed_layouts` requires this rejection.

Removing the guard is not a repair: field projection would change destruction timing or fail native moves. Keeping a resource private inside a Rust function can observe that function's local cleanup but cannot observe the Tsrust owner's retained resources. Returning an integer and retaining resources in a Rust map would require manual source cleanup and would hide the exact automatic-destruction property under test. Neither is an accepted alternative.

Only one primary adapter route was attempted. No second credible existing route was established. Prior File/testing-hook observations remain valid but insufficient. The conclusion is bounded to the inspected checked bridge; it does not claim the language can never support resource observation.

## Compiler coordination and precise dependency

The S18 worker confirmed that its compiler-owned SQL resource types and returned-borrow contracts do not safely generalize external projected structs with Drop. It recommended evidence-backed B2 blocker handoff and approved no shared opaque-resource changes during S18. No compiler, LSP, SQL, root manifest or lock changes are included.

A separately assigned dependency needs an owned opaque resource boundary that preserves native storage and destructor identity through construction, moves, arrays/records, closure retention, removal and automatic scope exit. It needs explicit borrowed inspection, consuming close, non-clonability unless genuinely supported, use-after-transfer diagnostics, and editor agreement. An independent audit must not retain the resource. The dependency must define thread affinity and transfer eligibility rather than infer Send-like safety from a structural record. It must distinguish destruction from asynchronous cleanup completion.

Expected review surfaces: project native type discovery/declarations, checked ownership/type representation, native input/output emission, resource storage and callable capture paths, source diagnostics, and LSP checked facts. Exact files must be agreed with the compiler worker before edits; a new builtin would additionally require complete catalog documentation. Do not begin by deleting the Drop rejection. The scope may be a narrow checked test-resource facility instead of a public generalized bridge, but either needs independent native/source/editor acceptance.

A second possible dependency is automatic owner cleanup ordering. Current structural Owner stores nodes and handlers in separate arrays. Its explicit retire/cleanup traversal does not prove descendant-first destruction on ordinary owner scope exit. Do not assert an observed bug without the calibrated checked resource path. Once that path exists, L18 must test this independently; any required owner finalization or representation change must preserve async/worker compatibility.

## Async and worker compatibility

Async and background workers remain required UI capabilities. The following are design obligations, not proofs or new production restrictions.

| Boundary | Current mechanism | Reusable part | Unresolved dependency / potential change |
|---|---|---|---|
| Owner mutation | Synchronous exclusive `Mutable<Owner>` calls and extracted dispatch | Serialized owner mutation and entry validation | Define owner executor authority; painting may be thread-affine. Worker computation should return through that authority, not share mutable owner loans. No scheduler exists here. |
| Deferred completion | Scalar runtime/owner/instance identity; FIFO synchronous events | Reject stale identity after retirement/replacement | Typed completion envelope and checked transport remain absent. Current handles are not certified cross-thread. |
| Lifetime safe point | Logical retirement immediately; physical explicit cleanup after callback return | Separate retirement from resource release | Async work needs owned active-use storage/leases or another explicit completion boundary. A callback-return-only cleanup rule cannot be generalized across await. No mutable owner loan may remain indefinitely suspended. |
| Cancellation/shutdown | Outside current experiment | Separate logical cancellation from access eligibility | Request, task termination and backend cleanup completion need distinct evidence. Dropping a future is insufficient; draining/joining cleanup needs its own contract. |
| Release affinity | Native calibration runs on one thread | Ordered release IDs and independent audit | UI-affine resources may need cleanup dispatched to their executor. The synthetic observer's mutex is not proof arbitrary resources are movable or releasable on any thread. |
| Identity allocation | Private module scalar; allocator owns mutable Domain | Nonreuse and checked exhaustion within current sequential API | Concurrent runtime construction is unproved. Serialized creation or a checked synchronized allocator is required before concurrency; per-thread reset counters would alias. This is a known future representation/authority decision. |
| Events and ordering | Release audit plus source/native phase assertions | Independent allocation/retirement/use-completion/release distinctions | Cross-thread happens-before and async completion need explicit synchronization; a global event sequence alone does not prove causality. |

The candidate does not introduce a global UI lock, thread-local identity or synchronous-only cleanup API. Automatic finalization and async active-use ownership remain architectural dependencies, not promises that all future work is a local extension. B2 closure for synchronous paths would still leave worker transfer, cancellation and thread-affine cleanup unproved.

## Acceptance, reproducibility and handoff

`release-matrix.json` records L01–L26 and reconciles all H/R/I rows. Use the existing entry points with the pinned compiler:

```sh
TSRUST=/Users/seancotter/.cargo/bin/tsrust node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
TSRUST=/Users/seancotter/.cargo/bin/tsrust node experiments/ui-tsrust-owner-handles/cli.mjs demo
```

The demo executes native calibration, the expected checked rejection, the passing control and existing UI behavior, then exits2 for B2. Unexpected tool errors, panics in checked compilation, assertion failures or invalid invocation fail; no silent skip promotes blocked rows. Calibration failures deliberately panic only in separate native negative-control processes.

No Cargo build, dependency installation, emitted-Rust patch, native owner substitute, async UI implementation, merge or consumer refresh is part of this result. Shared tool execution requires the S18 worker's explicit quiet window. Closed logs, coordination, assignment hash and exact restoration evidence remain in ignored task-local `evidence.sqlite` until separately authorized integration. Main analytics is untouched.

Native linkage: the plain-data emitted program imports Rayon. Bare rustc initially failed with E0432; this was a harness linkage failure, not the B2 blocker. The repaired harness links existing pinned release libraries read-only, verifies their hashes before/after execution, and does not edit emitted Rust or run Cargo. See `release-native-provenance.json`.

## Completed validation

- Full focused command:16 passed,0 failed,0 skipped; final exit0.
- Demo:exit2 for the reproduced B2 bridge blocker; native calibration, positive checked control and existing UI witnesses execute.
- Native controls: five deliberately wrong expectations fail with native assertion diagnostics; illegal consumed-value reuse rejects with E0382.
- Both strict check and emit reject the Drop boundary with the expected rust-bridge message, without panic.
- Whole-project formatting: one new control formatted; second and checkpoint passes change zero files/fingerprints. All tracked Tsrust source tokens match the committed baseline. New source tokens/idempotence and native behavior pass in the release tests.
- Initial full run:15 passed,1 harness linkage failure; preserved. Affected release recheck:3 passed. Final full run:16 passed.
- First formatter invocation lacked the existing TypeScript dependency; reused main's installed TypeScript through an ignored worktree-local symlink, read-only, without downloads. Formatting then passed.
- CLI and both installed/configured LSP hashes are unchanged. No Cargo builds or shared-target writes. Native binaries and disposable source directories are removed by the harness.

Closed evidence paths use `resource-release/` in the worktree's ignored SQLite store. Evidence includes initial failures, affected/final checks, demo, token/format proofs, compiler/library/source hashes and S18 coordination. This is a completed bounded blocker investigation, not synchronous B2 acceptance or an approved opaque-resource implementation.
