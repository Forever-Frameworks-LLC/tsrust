# Security and supply chain

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork; ongoing.

**Scope:** Define unsafe boundaries, build/install permissions, and dependency visibility.

**Planning deliverable:** A security policy covering source code, native wrappers, packages, and build scripts.

**Dependencies to coordinate:** [principles-and-boundaries](../../plans/principles-and-boundaries.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-23"></a>

## 23. Unsafe Code

Decide whether the source language permits unsafe code at all.
Possible model:
* ordinary language has no unsafe memory operations
* unsafe/native extensions require Rust
* unsafe packages must declare that fact
* package tooling surfaces unsafe dependencies
Example:
Dependency safety report:

32 packages
29 safe-language only
2 Rust-backed safe wrappers
1 contains unsafe Rust
This could provide a significant security advantage.

<a id="core-36"></a>

## 36. Security Model

Research security as a first-class design dimension.
Potential advantages:
* no arbitrary eval
* no dynamic prototype mutation
* immutable package versions
* deterministic builds
* limited install scripts
* safe native APIs
* auditable unsafe dependencies
Investigate sandboxed build scripts if build scripts exist at all.

<a id="core-37"></a>

## 37. NPM Supply-Chain Improvements

Do not copy npm’s postinstall model without evaluating its security implications.
Potential rules:
* packages cannot execute arbitrary code during installation by default
* build scripts require explicit permission
* native extensions are declared
* network access during builds can be disabled
* dependency capabilities are visible
Potential manifest:
{
  "permissions": {
    "build": false,
    "network": false,
    "filesystem": ["./generated"]
  }
}
