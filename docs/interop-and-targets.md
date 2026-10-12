# Interoperability and secondary targets

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof; later targets.

**Scope:** Scope safe Rust wrappers, foreign interfaces, and eventual WebAssembly support.

**Planning deliverable:** A restricted Rust wrapper contract; separate feasibility notes for C/C++ and WebAssembly.

**Dependencies to coordinate:** [ownership-and-memory](./ownership-and-memory.md), [type-system](./type-system.md), [error-handling](./error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-13"></a>

## 13. Rust Interoperability

This may become critical to ecosystem development.
Investigate a controlled FFI system that allows Rust libraries to be wrapped.
Potential syntax:
extern rust package "serde";
or build-time bindings.
Research:
* automatic trait mapping
* ownership mapping
* async interoperability
* generic interoperability
* macros
* unsafe Rust boundaries
* lifetime-heavy APIs
* procedural macros
* C ABI fallback
* build scripts
A major design goal should be:
Rust can act as the native ecosystem beneath the language without forcing ordinary users to write Rust.

<a id="core-39"></a>

## 39. Foreign Function Interface

Eventually support:
* Rust
* C
* possibly C++
Potential:
extern "C" function sqlite3_open(...);
But ordinary users should consume safe wrapper packages instead.
Research automatic header/binding generation.

<a id="core-40"></a>

## 40. WebAssembly

Investigate WASM as a secondary target.
Potential uses:
* edge runtimes
* serverless
* plugins
* browser compute modules
* sandboxed extensions
Determine whether language semantics translate cleanly to WASM without creating a second runtime model.

## Local Rust modules and native workers: owner discussion

The owner approved designing local Rust source integration alongside S12, before
its public worker interface is finalized. Compile explicit Rust exports with the
generated program, allowing direct calls and execution through the native thread
API. The owner additionally requested an automatically managed default pool for
threaded interop and supported package wrappers. Exact import/configuration syntax
and implementation remain pending. This does not block S11 JSON/schema work or
imply arbitrary Rust package compatibility.

Recommended initial boundary: local modules with explicitly exposed functions,
checked scalar/owned-buffer/record/Result mappings, generated adapters and automatic
build/dev dependency tracking. Ordinary users should not supply --extern on each
build. Thread execution remains separate from Rust import: a synchronous native
call should not require a worker. Owned buffers should transfer without payload
serialization or hidden cloning where the representations support it. Rust
compilation alone is not proof of compatible layouts, zero-copy conversion or
cross-module inlining; validate those properties with allocation and native tests.

Preserve the approved explicit join/detach obligation. Reject borrowed captures
that could outlive owners. Define panic conversion, supported lifetime signatures,
thread-transfer eligibility, unsafe-code trust and build-script policy before
exposing the boundary. Native threads cannot forcibly cancel arbitrary Rust code.
A bounded worker pool belongs to repeated-job execution, not the import mechanism.
Do not accept an API solely from handwritten Rust controls: require source/editor
contracts, normal build/dev operation, ownership rejection cases and measured
transfer/setup costs. Node comparisons must account for transferable buffers and
shared memory rather than assume every worker message serializes its payload.

### Approved pool direction and implementation recommendations

Provide automatic pool setup for threaded Rust interop; ordinary callers should
not need to construct a pool. Rayon is the preferred CPU-work backend to evaluate,
not a promise that every Rust crate shares its scheduler. Generate pool integration
only when used and initialize workers lazily. Reuse one toolchain-managed CPU pool
across compatible wrappers, with configurable worker and outstanding-job limits.
Default worker sizing should follow available parallelism; exact configuration
keys, admission/backpressure behavior and shutdown policy require the S12 contract.
Rayon worker count alone does not bound queued jobs or retained input memory.

Keep direct synchronous imports direct. Importing a Rust module must not silently
change a function into an asynchronous call or schedule every call on a worker.
The pool automatically supports the explicit worker-execution path. Cooperating
Rayon wrappers should execute within the selected pool so nested Rayon work can
reuse it. Wrappers with private pools or other runtimes must declare that behavior;
do not promise to merge arbitrary package pools or add another layer blindly.
Separate CPU work from blocking I/O and the later Tokio async runtime. Preserve
owned transfer, explicit completion obligations and cooperative cancellation.

Acceptance must cover lazy/no-use startup, concurrent wrappers, nested parallel
work, bounded submissions, saturation, panic propagation, shutdown and build/dev
restarts. Measure cold setup, repeated-job throughput, thread counts and allocation
transfer; verify no hidden per-call pools or payload serialization. Pin Rayon when
implementing the backend. Reference: Rayon [ThreadPoolBuilder](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html)
and [ThreadPool](https://docs.rs/rayon/latest/rayon/struct.ThreadPool.html).
