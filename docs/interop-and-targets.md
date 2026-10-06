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
