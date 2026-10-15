# Tsrust owner/handle feasibility

Investigation complete; the full owner/handle contract remains **blocked**. Ordinary Tsrust can express considerably more of the lifecycle model than the earlier counter witness established. This directory is experimental source and evidence, not a production runtime or public API.

Baseline: `5ff76855b9f2445a6c6b9bd26a369686b8dd16ec`. The task brief was supplied from `docs/plans/ui-tsrust-owner-handles-first-task.md` in main; it is not part of that baseline. S15 code in the checkout is not treated as accepted or used by these programs.

## Commands and outcomes

```sh
node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
node experiments/ui-tsrust-owner-handles/cli.mjs demo
```

The focused suite validates native source witnesses, strict rejection controls, formatting and the 19-row matrix. A passing suite means the investigation is reproducible; it does not make blocked rows supported. The demo executes the real Tsrust programs, then prints the blocker matrix and exits **2** deliberately. Exit 1 indicates usage, tool or unexpected validation failure. There is no Rust owner implementation or canned successful demo transcript behind the CLI.

Tools: installed Node, rustc and the verified S14 Tsrust compiler from `6fce2b90`, SHA-256 `cae21ac7d026d98f879656cc0776cb5080a46a681251ec7e3e9160bbc0b20c6d`. The hash is checked before and after native execution. A changed compiler requires provenance review, not a silent pin update. Check/emit use the module path; native code is compiled directly with rustc in disposable directories. No Cargo, downloads, package installation, compiler build, consumer replacement or shared build output is involved. `UI_EVIDENCE_LOG` optionally retains exact subprocess records for SQLite import.

## Tested representation

Candidate one invoked a callback while it remained inside the owner registry. `direct-dispatch.tsr` is rejected with `conflicting-access`: invocation borrows the registry while the callback requires mutable access to its containing owner. This is a correct safety diagnostic, not a request to relax ownership.

The one alternate representation removes the owned registration before invocation, passes the owner explicitly, restores the registration, then removes retired resources. `extracted-dispatch.tsr` proves this works without a native adapter. `retained-registry.tsr` independently proves that two mutable captured environments can coexist and retain their own current state. The owner implementation uses real stored callable values, not command enums or state snapshots.

`owner.tsr` owns flat node, display-order, registration, retirement and event arrays. Nodes hold integer state. Handles contain only owner/instance integers and retain neither state nor a loan into the owner. Root instance zero is a structural sentinel, not a stateful node. IDs are monotonic within their allocator and fail before the explicit ceiling of 2147483647. Application keys are opaque strings.

Retirement removes live display membership immediately. Callback-time reads, reorders and same-key reinsertion see the current list. Physical removals are queued in child-before-parent traversal order and performed after the active registration returns to the registry. This ordering is visible in source, but actual destructor order is not measured. Recursive drain returns `reentrant`; pending events preserve FIFO order. Stale events are discarded without invoking a registration.

Copying the two integers in a handle is permitted. The implementation does not clone owner state or callback environments. Source-level records are not unforgeable capabilities: the experiment assumes trusted callers use its operations instead of editing internal fields. This restriction and the allocator scope below must be reviewed before a public API.

## Matrix and witnesses

The machine-readable [matrix](matrix.json) is the authoritative disposition list. Every H01–H19 row is explicit; there are no skipped tests masquerading as support.

| Rows | Disposition | Evidence |
|---|---|---|
| H01 | Executed | Tool pins, isolated commands and source identities |
| H02 | Blocked for general owner identity; scoped behavior executes | `owners-lists.tsr` rejects wrong-owner reads/mutations/submission within one allocator; `domain-collision.tsr` demonstrates cross-allocator collision |
| H03–H04 | Executed | Retained registry, extracted dispatch, independent owner mutation and integer-only handles |
| H05–H07 | Executed | Queued stale delivery, replacement identity, preserved keyed edits, duplicate/invalid-order rejection |
| H08–H09 | Executed | Callback FIFO observations 1,3,6; reentrant drain rejection; strict direct-dispatch rejection |
| H10 | Blocked on actual resource destruction | `disposal.tsr` executes self/sibling/ancestor invalidation and stale deliveries; registry emptiness is not destructor evidence |
| H11–H13 | Executed | `callback-lists.tsr` runs all three callback-removal regressions, including fresh delivery and duplicate replacement rejection |
| H14–H15 | Blocked on resource observation | Explicit logical cleanup and `scope-exit.tsr` run; exact release and automatic destructor observations are absent |
| H16 | Blocked on resource baseline; logical cycles execute | 1,000 mount/deliver/remove cycles leave no registry entries; checked owner/instance exhaustion rejects; no allocator/drop claim |
| H17 | Expected rejection | Existing escaped-loan, overlapping-access and consumed-owner fixtures reused read-only with strict check/emit |
| H18 | Executed | Token preservation, repeat formatting, native assertions and a deliberately incorrect expected value causing native failure |
| H19 | Blocked report | Actual partial native demo followed by explicit blocker matrix; exit 2 |

`owners-lists.tsr`, `disposal.tsr`, `callback-lists.tsr` and `cycles.tsr` are separate programs to keep compiler work bounded. The combined program exceeded the 30-second subprocess limit; the original timeout is retained in evidence. Splitting preserved the assertions. A typed catch binding inside a stored callback initially produced `error-effect`; the exception value was unused, so the supported catch-without-binding form suffices here. No required behavior is replaced by a mock. The assertion mutation exits 1 with `Uncaught AssertionError`, rather than the earlier simple counter's panic exit 101; the harness checks the actual diagnostic.

## Blockers and next decision

**B1: identity allocator scope.** `Domain.next` allocates unique owners only within that Domain. Two independently constructed Domains can each issue owner 1/instance 1. The collision witness asserts that a handle then reads the wrong owner's value. This is a deliberate counterexample, not an accepted implementation behavior. The scoped tests do not justify a general cross-runtime handle API.

One possible narrower contract is a single library-controlled allocator per UI runtime, with handles forbidden across runtime boundaries. That restriction is not enforced by these structural records and needs review. Alternatively, a future library/native contract could supply stable owner identity. This investigation does not establish that a general `Weak` type or compiler change is necessary. No additional representation search was attempted.

**B2: destruction observability.** Existing source assertions establish state values, callback invocation, stale results and logical registry removal. They do not establish when captured resources or event payloads actually drop, or prove exact-once child-before-parent destruction. A surviving integer handle after `scope-exit.tsr` returns is not evidence that an owning resource was released. No generated Rust was patched or instrumented to invent this proof.

The standard testing surface and native type declarations inspected here did not establish a source-level resource-drop observer suitable for these witnesses. The next dependency should be an agreed observation method or a narrowly scoped resource contract, rather than claiming a leak or adding a generic destruction API without evidence. Scope changes, observation mechanisms and any compiler work require a separate task.

Read-only compiler inspection leads:

- `crates/tsrust-compiler/src/analysis/checked_callable_storage.rs` and `callables/storage/mod.rs`: existing concrete owned environment admission; no changes needed for the successful registry.
- `analysis/capture_loan_binding.rs` and `callables/borrowed_callable_contracts.rs`: preserve loan and invocation safety when considering any new resource contract.
- `emit/callback_capture_emit.rs` and `emit/expression_work.rs`: generated callback storage/invocation integration if later required.
- `native/types.rs`, `native/bindings.rs`, `native/signatures.rs`: possible explicit identity/resource declarations; these are not approved edits.
- `error_effects/mod.rs`: the unresolved typed catch observation is separate from the successful catch-without-binding witness. No error payload inspection is required by this task.

Recommendation: review allocator scope and select a resource-lifetime observation strategy before promoting this library model. Existing owned callable storage already supports the tested dispatch mechanism. Do not start a broad compiler owner/handle feature merely because the Rust prototype used `Weak`. Actual shared compiler work still waits for S15 acceptance and a separate assignment.

## Isolation, evidence and limits

Only `experiments/ui-tsrust-owner-handles/` is owned. The prior prototype and its rejection fixtures are read-only references. No main analytics writes, root manifest/lock changes, compiler/LSP/editor modifications, renderer, JSX, HTTP, loader, signal graph or DI work occurred.

Whole-project formatting is required before the source checkpoint. The existing formatter's TypeScript dependency may be resolved read-only from main, with the exact helper and dependency identities recorded. This does not add a dependency to the prototype. The small Node token checker intentionally supports only the lexical subset used by these witnesses and rejects unsupported input; it is not a new application parser.

Closed records live in ignored `evidence.sqlite`, using `benchmarks/storage/store.py` and verified byte-exact restoration. Original timeouts, failed probes, corrected harness expectations, final checks and demo output are retained. Loose verified records and task-created executables are removed. Evidence remains isolated for owner-authorized serial integration.

Callbacks in the executed slice complete normally. Exception/panic recovery, cancellation, multithreading, untrusted internal-field mutation, cross-Domain transport, rendering and performance are not accepted. Physical allocation counts and drop order remain unmeasured. Do not call the blocked contract production-ready or merge without review. Stop after this task.
