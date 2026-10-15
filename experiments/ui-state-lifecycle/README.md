# Component state and lifecycle experiment

This is a synchronous, single-threaded feasibility prototype. It does not render UI or provide a public component API. Baseline: `01f3a1d45aa6e6f81422d059f84a341f13581df2`, including accepted S13/S14 and the merged route-manifest experiment. S15 is independent and not a dependency of these tests.

## Run

From the worktree root:

```sh
node --test experiments/ui-state-lifecycle/tests/*.test.mjs
node experiments/ui-state-lifecycle/cli.mjs demo
```

Requires existing Node, `rustc` and the verified installed Tsrust compiler. No packages, Cargo builds or network access. `TSRUST` may select the same verified compiler at another path; the suite rejects a different hash until its provenance is reviewed. Node orchestrates the experiment; it is not a proposed application runtime.

The accepted S14 compiler comes from `6fce2b90`, SHA-256 `cae21ac7d026d98f879656cc0776cb5080a46a681251ec7e3e9160bbc0b20c6d`, documented in `docs/plans/tokio-tasks-acceptance.md`. Validation uses Node v26.3.0 and rustc 1.96.1. Tests compile the Rust model and emitted Tsrust to temporary executables with direct `rustc`, then delete them. Each subprocess has a timeout and bounded output; missing tools and failed commands fail the suite. Optional `UI_EVIDENCE_LOG` records exact subprocess arguments, results and output for SQLite import.

## Model and provisional policy

`Owner` owns a tree of state and actual boxed `FnMut` handlers. State is an integer for this bounded counter/list slice. Handlers read and mutate current state, including after their construction function returns. This is not a list of canned commands or generated outputs.

A private handle contains a weak owner identity and an instance ID. Handles and queued events do not retain state or handler captures. Owner identity is checked on every access. IDs increase monotonically, never reuse storage identities, and reject allocation before overflow. An application key may be reused after removal; its replacement always has a different instance ID. No global identity allocator, shared application state, unsafe code or state clones are used.

Each node retains insertion order for disposal and independent display order for list reordering. Empty or duplicate sibling keys are rejected before mutation. Reordering requires exactly the current keys, with no omissions, duplicates or unknowns. Keys are opaque strings. Removing a child preserves its parent and siblings.

Delivery uses a manually drained FIFO. The callback receives exclusive state access plus the owner for queuing or removal. Nested drain and independent read of the currently borrowed state return `Reentrant`. Newly queued events run after already pending events. Removal invalidates handles immediately. During delivery, retired resources remain until that callback returns; cleanup then releases children before parents. Sibling insertion order is retained for one subtree disposal, regardless of display reorder. Separate explicit removals follow their invocation order. Repeated removal is a no-op; foreign-owner operations are rejected.

Stale events report `Stale` without invoking the old handler. Their payloads release when drained, cleared or when the owner drops. Disposed state and handler captures do not wait for queued payloads or retained handles. Weak identity metadata can outlive the owner, but component state cannot. Dropping the owner performs subtree and queue cleanup without requiring explicit disposal.

## Acceptance matrix and proof levels

All 18 plan rows are accounted for. The Rust harness has 17 tests: L11 has two; L15/L16 are compiler tests in Node. The Node suite has three tests, including harness failure checks. No tests are silently skipped. The model rows pass in Rust; they are not all implemented in Tsrust.

| Row | Named witness | Rust model | Tsrust analysis / native execution |
|---|---|---|---|
| L01 | `l01_independent_counters` | Pass: A=2, B=1 | `retained-counter.tsr` passes both |
| L02 | `l02_retained_capture_current_state` | Pass: retained captured multiplier, values 7 then 23 | Returned mutable counter closure passes both; not the Rust registry |
| L03 | `l03_cross_owner_handles` | Pass: same keys/IDs cannot cross owners | Not implemented; non-owning handle contract pending |
| L04 | `l04_reorder_preserves_state_and_mounts` | Pass: edits follow keys; no drops/remounts | Not implemented |
| L05 | `l05_invalid_keys_and_order_are_atomic` | Pass: keys/order/resources unchanged on rejection | Not implemented |
| L06 | `l06_removed_pending_event_is_stale` | Pass: no removed callback invocation | Not implemented |
| L07 | `l07_reinsert_does_not_revive_handle` | Pass: fresh instance, old event stale | Not implemented |
| L08 | `l08_child_first_idempotent_disposal` | Pass: child-first, stable sibling order, exact drop trace | Not implemented |
| L09 | `l09_fifo_and_completed_mutations` | Pass: exact trace 1,3,6 | Not implemented |
| L10 | `l10_recursive_drain_rejected` | Pass: rejection preserves later deliveries | Not implemented |
| L11 | `l11_self_and_sibling_removal`, `l11_active_child_disposes_root_child_first` | Pass: active access finishes, later work stale, safe cleanup | Not implemented |
| L12 | `l12_disposed_resources_do_not_wait_for_handles_or_payloads` | Pass: only queued payload remains until drain/clear | Not implemented |
| L13 | `l13_owner_drop_releases_queue_and_captures` | Pass: drop releases resources; weak identity expires | Not implemented |
| L14 | `l14_thousand_cycles_return_to_baseline` | Pass: 1,000 cycles; one root remains per cycle, final zero | Not implemented |
| L15 | `escaped-loan`, `overlapping-access`, `consumed-owner` | Not applicable | Strict check/emit reject with `closure-escape`, `conflicting-access`, `transferred` respectively |
| L16 | Positive formatting/native harness | Not applicable | Tokens preserved, second format stable, check/emit/native assertions pass; corrupted expectation fails natively |
| L17 | `l17_checked_identity_exhaustion` | Pass: allocation rejects at exhausted ID, no revival or mutation | Not implemented |
| L18 | `l18_demo_exact_output` and CLI demo | Pass: same model and exact output | No Tsrust renderer/demo implied |

`retained-owner-conflict.tsr` additionally proves that ordinary mutable capture prevents independent owner access while the handler remains live (`capture-conflict`). Rejection tests check emission with editor suppression still present, then remove only the suppression comment in temporary source and require strict check and emit failures. These are ownership diagnostics from valid syntax, not unsupported-token errors.

The fixture token scanner deliberately supports only the simple lexical subset used here. It preserves comments and literal spelling, rejects unsupported input, and has mutation controls for changed strings and operators. It is not an application-source parser or production formatter. The compiler performs formatting; the scanner checks the before/after witnesses.

## Capability inventory and precise dependencies

1. **Retained invocation and mutable owned state: supported and executed.** `retained-counter.tsr` returns closures from a factory, invokes them repeatedly, and asserts independent state with `std:testing`. Generated Rust builds using only `rustc`. No S15/async dependency.
2. **Borrow safety: supported rejection evidence.** The three L15 witnesses establish borrowed callback escape rejection, incompatible simultaneous access and transfer enforcement. Relevant existing implementation: `src/callables/borrowed_callable_contracts.rs`, `src/analysis/capture_loan_binding.rs`, under `crates/tsrust-compiler/`. Do not weaken these rules to make a UI model fit.
3. **Externally invalidated non-owning handles: missing integration contract.** The Rust model uses `Weak<()>` plus checked IDs. No equivalent UI owner/handle API is published by this experiment. `retained-owner-conflict.tsr` shows why directly retaining a mutable closure loan is not independent owner access. It does not prove every alternative Tsrust implementation impossible. A later task must choose a library-owned registry or checked native resource API and prove non-owning access, stale results and cross-owner rejection at the Tsrust boundary. Inspect `src/native/types.rs`, `src/native/bindings.rs` and `src/native/signatures.rs` for an explicit native contract, rather than introducing assumed public `Weak` syntax.
4. **Instance identity and list operations: Rust-only here.** They may be ordinary library algorithms; no new language feature is established as necessary. Translate and test only after choosing the owner representation. Preserve checked overflow and opaque keys.
5. **Subtree disposal, callback access during delivery and registration storage: Rust-only here.** A future runtime/library needs deterministic ownership and cleanup. Compiler integration must verify callback environment storage and invocation modes, especially moving state out temporarily for a callback. Relevant starting points: `src/analysis/checked_callable_storage.rs`, `src/emit/callback_capture_emit.rs`, and existing native resource bindings. This prototype does not claim compiler support for its `Box<dyn FnMut>` representation or arbitrary heterogeneous registrations.
6. **Public API/editor support: pending.** If a built-in/native API is chosen, its checked declarations, emitter/native behavior, `crates/tsrust-lsp/src/builtins.json`, completion/hover and ownership diagnostics need their own acceptance. Ordinary library algorithms do not automatically require compiler edits. No files in those areas changed here.

Recommendation: the isolated Rust model is feasible. Existing Tsrust retained counters are feasible. Generated UI remains blocked on the owner/handle integration contract and browser execution choice. Review this evidence before assigning a translation or paired real-input task. S15 acceptance still gates HTTP middleware, loaders and full/fragment transport; this experiment neither implements nor validates them.

## Demo

```text
counters: a=2, b=1
reordered: [("4", 40), ("2", 23), ("1", 10)]
removed event: [Err(Stale)]; old handle: Err(Stale)
reinserted: 1
tracked live resources: 0
```

## Isolation, evidence and limitations

Only this directory is owned. No root manifests/locks, compiler/LSP/editor source, installed consumers, route prototype, shared build outputs or main SQLite writes. Whole-project formatting passed across 1,317 Tsrust sources with token preservation; it changed only the five owned fixtures, then changed zero files/fingerprints on repetition. All tracked files outside ownership remained byte-identical. The worktree lacks a local TypeScript module required by the existing formatter. A temporary Node resolver hook redirected only that import to the already installed main-checkout module, read-only; no dependency was downloaded or installed. Hook bytes and the module hash are retained in isolated evidence. The prototype itself uses no TypeScript dependency. The assignment is `docs/plans/ui-state-lifecycle-first-task.md` in the main checkout; it was supplied separately from this baseline.

`evidence.sqlite` is ignored and retained locally for serial integration. It uses the existing `benchmarks/storage/store.py` immutable records format. Exact command logs, failed/expected rejection output, source/tool identities and validation results are imported and read back byte-for-byte before loose closed logs are removed. Never merge or import this database into main from the experimental worker.

Tracked drops cover state, handler captures and event payloads, not all allocator activity. The audit trace intentionally retains test history. No memory-size, speed or broad leak claim follows. Callbacks are trusted, finite synchronous functions; no guarantee against infinite enqueue loops, callback panic recovery, integer overflow inside application callbacks or allocation failure. No async cancellation, threads, reactive subscriptions, JSX, browser execution, native rendering, text/IME/focus/accessibility, production routing lifecycle or security-policy execution is tested. No new milestone is authorized.
