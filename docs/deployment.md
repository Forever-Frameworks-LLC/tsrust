# Native deployment

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Define executable deployment and runtime resource requirements.

**Planning deliverable:** A deployment checklist validated against a produced native server binary.

**Dependencies to coordinate:** [toolchain-and-development](./toolchain-and-development.md), [interop-and-targets](./interop-and-targets.md), [async-and-runtime](./async-and-runtime.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-33"></a>

## 33. Startup and Deployment

Target:
lang build --release
producing one deployable native executable where practical.
Investigate:
* static binaries
* dynamic linking
* CA certificates
* timezone database
* native dependencies
* database drivers
* TLS implementations
Goal:
COPY app /app
ENTRYPOINT ["/app"]
with no language runtime installation required.
