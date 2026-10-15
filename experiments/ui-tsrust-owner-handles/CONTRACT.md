# Experimental lifetime contract

The user dispatched the lifetime-contract assignment on 2026-10-15 using its recommended decisions. These decisions define the experimental target; they do not establish that every requirement is implemented. See `resource-matrix.json` for R01–R19 and its H01–H19 reconciliation.

## Identity decision and B1

A UI runtime should control one allocator. Owners allocated through that runtime receive independent monotonic identities; instances are monotonic within an owner, with zero reserved for its structural root. Handles contain identity only; foreign-runtime operations reject. Handles must not be serialized or persisted. Checked exhaustion must reject before wrap or reuse.

Supported operations must prevent or reject cross-runtime misuse. Trusted callers are not assumed to forge internal fields, but ordinary construction of two runtimes must be safe. Documentation alone is insufficient.

The historical closure-hidden allocator reset local counters and permitted cross-runtime
aliasing. The identity follow-up now enforces runtime/owner/instance identity through
a private monotonic module scalar and a closure-owned private Domain. See
[identity contract and evidence](IDENTITY.md). Original counterexamples remain in Git
and execute against their original module in disposable directories.

B1 is resolved within the documented source API. Deliberate fabrication or mutation
of internal structural fields is outside this contract; handles are not security
capabilities. Exact resource destruction remains unobserved under B2.

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

B2 is an observation gap, not a demonstrated leak or proof that a general destruction language feature is needed. Full feasibility remains blocked by B2. B4 is resolved by the bounded compiler repair.

## B4: named-handler compiler defect repaired

The historical S15 and pre-repair TLS compilers panic during strict check and emit of `named-self-replacement.tsr`. Original diagnostics remain in SQLite. The defect was missing access-summary coordinates for a callable synthesized during record discovery, not an ownership rejection or a runtime failure.

The repair admits those coordinates when the specialization is created, using the existing recursive inference lattice. Both original named and factory witnesses now check, emit and execute their unchanged assertions. H13/R08 no longer carry the B4 limitation. The harness requires successful native execution and fails on any panic, rejection, timeout or signal. See [repair acceptance and provenance](B4-REPAIR.md).

B4 repair alone established no identity or resource destruction proof. B1 is now resolved by the separate identity candidate; B2 remains open and B3's adopted semantics are unchanged.
