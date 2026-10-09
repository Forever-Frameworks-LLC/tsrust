# B7 mutable captured installers checkpoint

B7/gate 183 remains open. D2 discriminated unions are a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md first.

## Accepted behavior

Reusable installers can capture a mutable array, install a callback borrowing it, invoke that callback, clear it and repeat. The retained exclusive capture borrows the installer receiver. Clear the callback before another mutable installer call or before disposing of the installer. After clearing and leaving the installer scope, the original array becomes available. Readonly captures still retain the original caller-owned source independently of installer disposal.

`captured_record_borrows.rs` attaches mutable captured retention to the receiver. `closure_record_signature_emit.rs` gives the native method an explicit receiver lifetime and uses it in retained callback conversion bounds. Generic trait adapters retain higher-ranked receiver obligations. `emit.rs` uses that receiver only when the planned signature requires it. No source invocation contract, capture ownership or clone rule changes.

Explicit owning transfer already works with existing contracts: an installer value `Once<Function<void, [target: Mutable<Holder>]>>`, an `Owned<int[]>` input, and an owning `Function<int> | undefined` field. The stored callback owns the array and can mutate it repeatedly after consuming the installer. The original array binding remains transferred.

A first probe incorrectly combined a borrowed `Mutable<Function<int>>` field with a callee-owned array. This exposed a real source diagnostic gap rather than a valid ownership transfer. Nullable injection hid the callback environment from the existing escape eligibility check. `record_parameter_carriers.rs` now unwraps the injection and rejects the borrow with `callback-escape`. Keep this distinction: owning transfer works; borrowed retained sources through arbitrary consuming wrappers are not established.

## Verification and evidence

The eight affected integration suites total 107 passes. Eight owner-provenance units, 27 capture-loan units, one strengthened editor witness and eight depth canaries pass. Editor controls cover safe repeated borrowing, conflicting installer reuse, retained callee-owned escape and use of the original transferred array. Compiler scope control checks the exact escape diagnostic. Native assertions exercise repeated scoped borrowing and owning once transfer. No complete compiler/LSP regression or timing run occurs.

The matched workload observes 64 values per iteration. Six new observations cover scoped callback results, released source state, exact installer effects and owned callback results. Five native and Node-after-tsc cases pass; registration and mutation controls pass. A stale clear/effect count and a too-weak duplicate-call guard were repaired, with failed attempts retained. Whole-project formatting changes one workload/fingerprint, then zero files.

SQLite preserves `runtime/results/2026-10-09T22-33-57-820Z-verify.json` and `runtime/results/2026-10-09T22-34-26-050Z-verify.json`, plus content-addressed source/text/log evidence under `runtime/source-evidence/mutable-captured-installers/`. Both records' compiler and harness inputs match their hashes, including the first guard version. CLI emission, trimmed emission and generated instrumented prefix match. Exact readback succeeds and SQLite quick_check is ok. Five disposable outputs totaling 1,092,118 bytes are removed. Closed loose evidence is removed only after readback.

## Continue

Next implement hidden retained sources through additional closure-call edges. Preserve source-slot versus binding-ID distinctions and source invocation contracts. Captured record targets, callback capture vectors, abstract calls, aliases, consuming wrappers that retain borrowed sources, methods/defaults, field/index sources, computed owners, retained control flow and recursive stores remain in the B7 plan. Current conservative guards stay until their full call and lifetime interfaces exist. Complete family regressions, accumulated measurement and consumer refresh only at B7 family acceptance.

Five installed consumers remain B5. Fifteen unrelated fixture edits remain untouched and uncommitted. Use explicit Git paths, the shared Cargo target, incremental off, Node 24.11.1 and Homebrew Python. Keep D2 separate and the full compiler goal active.

## Continuation prompt

Continue Tsrust from this checkpoint. Reusable mutable captured installers now lend through receiver lifetimes and require clearing before reuse/disposal. Owning Once/Owned transfer executes with owning callback fields. Nullable borrowed fields now diagnose retention of callee-owned storage. Implement the next B7 interface, starting with transitive hidden-source composition, using focused TDD, matched workloads, editor parity and exact SQLite evidence. Preserve unrelated fixtures, conservative unsupported-use guards and B5 consumers. Do not treat this checkpoint as B7 family acceptance.
