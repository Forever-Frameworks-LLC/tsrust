# Interoperability and secondary targets

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Restricted local Rust bridges are implemented; broader package, FFI and secondary-target designs remain proposals. See the current bridge contract below.

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

## Current local Rust bridge

Local `.rs` imports derive source signatures from safe public free functions.
The supported scalar, owned-buffer, borrowed-slice, Result and TaskContext boundary
is documented in [Rust/Rayon bridges](plans/rust-rayon-bridges.md). Direct calls run
on the caller; explicit `spawn` and owned `ThreadPool.task` retain their existing
ownership, cancellation and cleanup contracts. Nested Rayon work inside a pool
task uses the selected pool. The later removal of the global pool supersedes the
older automatic-default-pool proposal below.

Example: [owned Rust/Rayon work](../examples/rust-rayon/main.tsr).
Arbitrary Rust packages, record/trait/lifetime mapping and asynchronous exports
remain outside the restricted boundary.

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

### Declared crate bindings and owned data

Import a `*.bindings.json` file to select exact Cargo dependencies and a handwritten
Rust adapter module. The adapter's public free functions, named public structs and
enums form the explicit source API. Third-party types and functions are not imported
automatically. See the [regex example](../examples/rust-regex/main.tsr).

Run `tsrust bindings resolve path/to/api.bindings.json` after cloning or changing
versions/features. Commit the adjacent `api.bindings.lock`. Resolution may access
the registry and run Cargo's ordinary dependency machinery. Vendored inputs live
in the binding directory's ignored `target/tsrust-native/bindings` cache. Ordinary
check/emit operations only read project inputs; builds are offline and locked.
Exports carry the vendored dependency sources and licenses. Dependencies are trusted
build inputs, including their build scripts; local adapter code forbids unsafe code.

The manifest accepts schemaVersion 1, a relative source path, and dependencies with
exact version, optional features and defaultFeatures (true by default). Registry
crates are supported; git/path dependencies and custom registries are not. Binding
modules in one program must declare the same dependency configuration and lock.
Share the same manifest when several source modules use one binding API.

Public structs require concrete named public fields. Enums support unit, tuple and
named variants. `State::Ready(T)` maps to `StateReady { value: T }`; multiple tuple
payloads use item0, item1, etc. Named fields keep their names. The enum maps to a
union of those records, so ordinary match expressions and ownership checks apply.
String and primitive-buffer fields move without serialization or implicit cloning.
Nested acyclic owned data and Result outputs are supported. Returned references,
borrowed records, generic or recursive layouts, and vectors of records remain
unsupported. Adapter structs with custom Drop implementations cannot be projected
field by field. Keep resource-owning third-party objects private to the adapter.

Curated standard-library APIs and explicit crate bindings serve different needs.
A crate dependency does not create a std: module or guarantee compatibility with
all of its APIs. Networking, server and database libraries can be wrapped with
explicit lifecycle contracts; they are not all bundled into every compiler build.
