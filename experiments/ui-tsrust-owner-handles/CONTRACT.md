# Experimental lifetime contract

The user dispatched the lifetime-contract assignment on 2026-10-15 using its recommended decisions. These decisions define the experimental target; they do not establish that every requirement is implemented. See `resource-matrix.json` for R01–R19 and its H01–H19 reconciliation.

## Identity decision and B1

A UI runtime should control one allocator. Owners allocated through that runtime receive independent monotonic identities; instances are monotonic within an owner, with zero reserved for its structural root. Handles contain identity only and must not be serialized, persisted or transported between runtimes. Checked exhaustion must reject before wrap or reuse.

Supported operations must prevent or reject cross-runtime misuse. Trusted callers are not assumed to forge internal fields, but ordinary construction of two runtimes must be safe. Documentation alone is insufficient.

The one boundary candidate in `runtime-boundary.tsr` hides a Domain in an owned retained closure. Callers cannot select its starting counter through `runtime()`. Repeated calls allocate owners 1 and 2, but a second runtime starts at 1. The native assertions reproduce wrong-runtime read, mutation, enqueue, removal and root-based reorder. They also demonstrate a handle escaping owner/allocator teardown and reading a fresh runtime's replacement state. The assertions deliberately confirm unsafe behavior; they are counterexamples, not acceptance tests for a safe API.

B1 remains blocked. Hiding the allocator solves caller-selected IDs within one scope but does not attach an enforceable runtime brand to the handle. No alternate representation was pursued: without an additional checked brand or stable identity primitive, rearranging these scalar fields cannot distinguish the counterexample. This is a bounded design finding, not a proof that ordinary Tsrust can never solve it.

A separate identity task should choose an enforceable runtime boundary: a statically scoped brand with no escaping mutable loan, or a verified runtime identity facility. It must reject every applicable cross-runtime access path, escaped handles after teardown/recreation, repeated initialization and exhaustion without mutation. Do not relax ownership rules, use user labels as identity, or adopt a global registry merely to pass this example.

## H10 decision: B3 resolved by adopted semantics

Self, ancestor and root removal immediately end live membership and handle-mediated access. `read` returns `undefined`; `add` and `enqueue` return `false`. Removing only a sibling leaves active state accessible. Callbacks continue normally, but have no separate surviving current-state loan. The existing `disposal.tsr` assertions cover these cases and stale queued deliveries.

For example, callback code can call `remove(owner, handle)`, check that `add(owner, handle, 1)` is false, then continue unrelated control flow. It must perform a desired final state mutation before removal. The experiment does not throw merely because the component was removed.

This intentionally replaces the Rust reference's surviving state loan. B3 is resolved by this explicit contract adoption and its native regressions, not by implementation of the old semantics. H10 remains blocked on B2. FIFO delivery, guarded recursive drain, fresh replacement identity, H11–H13 live-list behavior and ownership rejection rules are unchanged.

Logical invalidation does not permit early physical release. Retired state and captures must remain alive until the current callback returns normally, then release exactly once with retired descendants before ancestors. Outside delivery, explicit removal must finish cleanup before returning. Automatic owner destruction must release resources without an explicit remove. Outstanding handles must not retain them. Root zero carries no state resource.

Exception/panic recovery, unwinding, async cancellation, multithreading and serialization remain outside this experiment.

## Observation approaches and B2

Exactly two approaches were inspected:

1. **Existing testing hooks/assertions.** `crates/tsrust-compiler/src/stdlib/testing/api.rs` defines test/suite lifecycle hooks, including afterEach and afterAll. They run at test phases, not at destruction of arbitrary values. Hook counters would measure hook execution, not state/capture/payload release. No suitable source-level destruction observer was established from this surface. The existing `scope-exit.tsr` is the minimal observation-limit witness: an escaped integer handle says nothing about its owner's destruction.
2. **Existing owned File resource.** `resource-capture.tsr` moves an actual `std:fs.File` into a retained callback stored by the real owner implementation, delivers twice, then removes the registration. It opens the existing read-only owner fixture; no shared output or service is needed. This checks/emits and executes through direct rustc. It demonstrates usable native capture storage, not a destructor observation. The API exposes consuming close/read but no non-owning release audit or release sequence. A successful close, file existence, ability to reopen, or empty registry cannot establish exact-once release or child-before-parent ordering. The observer has therefore not passed R09 calibration; dependent resource branches stop here.

Read-only implementation evidence: `stdlib/fs/bindings.rs` declares consuming close and mutable read; `native/types.rs` declares File; `stdlib/fs/runtime/fs.rs` owns a native file through `Resource<FileBackend>`. In `stdlib/fs/runtime/resource.rs`, explicit close and Drop each take the optional backend, with Drop using the same close path. Its Rust-only tests have a custom Backend probe recording release IDs. That probe is not an existing source API, and importing it into these witnesses would require new support outside this assignment. These paths support an ownership explanation but do not measure when our stored closure drops. No generated Rust was edited.

State and event payloads in the current model remain integers. The real File capture result is not extrapolated to those paths. No queue-clear API exists; clear is inapplicable, while stale drain and owner drop remain required and unproven for resource payloads. Logical 1,000-cycle balance and last-valid-ID tests run, but cannot establish physical resource balance.

## Smallest proposed observation dependency

A separately assigned test-only native resource observer could expose an owned tracked resource plus an independent audit that does not retain it. The native resource's actual release path must record a unique resource ID and ordered release event; source code must not increment a cleanup counter itself. The audit must remain readable after resources drop. Specify whether it observes destruction of the wrapper, the underlying resource, or both.

Before using it for UI acceptance, calibrate no release while live, one event on explicit and automatic destruction, no second event on duplicate cleanup, and failure under deliberately wrong expected counts/order. Then place resources separately in actual state, captures and queued payloads; test callback-safe release, nested child-before-parent order, successful/stale delivery, owner scope exit and 1,000-cycle balance. Validate that audit handles do not extend resource lifetime. Keep those future native/compiler/editor changes separately owned and reviewed; this task implements none of them.

B2 is an observation gap, not a demonstrated leak or proof that a general destruction language feature is needed. Full feasibility remains blocked by B1, B2 and the compiler integration limitation B4.

## B4: named-handler self-replacement compiler limitation

`named-self-replacement.tsr` preserves the formatted review reproducer. On the pinned S15 compiler, both strict check and emit exit 101 with an analysis/mod.rs index-out-of-bounds panic (length 22, index 22). It replaces its active component using a named handler inside a retained callback. `factory-self-replacement.tsr` preserves the passing control using a closure factory; native assertions verify fresh identity, stale queued work, unaffected FIFO, recursive-drain rejection and final root removal.

This is a compiler defect observed at integration, not an accepted ownership rejection or a runtime failure. Simpler named-handler programs passed during review, so the limitation is tied to this reproducer rather than all named functions. No regression origin is established. H13/R08 retain their executed fixture results with B4 explicitly qualifying the supported forms; H19/R19 remain blocked reports. B3's adopted semantics are unchanged.

The harness verifies the exact panic for both commands and fails on success, a different diagnostic, timeout or signal. A future compiler fix requires deliberate provenance review and conversion of this reproducer to a successful native test. Do not normalize arbitrary panics as expected results. Compiler repair is separately scoped; no compiler files are modified here. The reproducer has no suppression directive: an internal compiler panic is not a suppressible source diagnostic.
