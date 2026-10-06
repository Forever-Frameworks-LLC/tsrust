# Packages and registry

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Ecosystem proof.

**Scope:** Scope content-addressed storage, version identity, registry requirements, and the package-manager prototype.

**Planning deliverable:** Two projects sharing stored package files, plus a version/lockfile design and separate registry backlog.

**Dependencies to coordinate:** [compiler-and-modules](./compiler-and-modules.md), [security-and-supply-chain](./security-and-supply-chain.md), [toolchain-and-development](./toolchain-and-development.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-17"></a>

## 17. Package Management

Do not reproduce node_modules.
The package manager should use a global or content-addressed package store.
Potential structure:
~/.language/packages/
    sha256/...
Projects contain only:
project.lang.json
project.lock
src/
Packages should be globally cached and referenced by version/hash.
Example manifest:
{
  "name": "my-api",
  "version": "1.0.0",
  "dependencies": {
    "hono": "4.2.1",
    "zod": "3.24.0"
  }
}
No package code should need to be copied into every project.
Research models from:
* pnpm
* Cargo
* Go modules
* Nix
* Bun
* Deno
* Zig
* modern package content-addressable stores
Goals:
* reproducible installs
* deterministic builds
* minimal disk duplication
* lockfiles
* dependency graph inspection
* straightforward version pinning
* offline builds
* package integrity verification
* package signing/provenance where possible

<a id="core-18"></a>

## 18. Dependency Versioning

Avoid npm’s dependency-chaos failure modes.
Research whether the package system should allow multiple versions of the same dependency in one binary.
Potential behavior:
App
├── framework A -> parser 2.0
└── library B   -> parser 1.7
Possible approaches:
* allow both versions
* deduplicate compatible versions
* force single version
* namespaced dependency identities
Investigate consequences for:
* binary size
* shared types
* trait identity
* dynamic loading
* security patching

<a id="core-19"></a>

## 19. Package Registry

Eventually define a first-party registry.
Potential requirements:
* immutable published versions
* signed packages
* checksums
* dependency metadata
* source availability
* license declaration
* native-code declaration
* unsafe-code declaration
* package health indicators
* compiler-version compatibility
* security advisories
* yank/deprecate functionality
Research whether packages should publish source, intermediate representation, or compiled artifacts.

<a id="core-55"></a>

## 55. Package Manager Prototype

Implement:
lang add package
lang remove package
lang install
lang update
Use a global content-addressed store.
Demonstrate two projects sharing the same physical package files.
