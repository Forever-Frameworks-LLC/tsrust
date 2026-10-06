# Source migration and compatibility tests

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Measure application migration and independently test documented public HTTP behavior.

**Planning deliverable:** An original compatibility corpus and a migration report with explicit semantic changes.

**Dependencies to coordinate:** [http-and-hono](./http-and-hono.md), [validation](./validation.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="web-29"></a>

## 29. Migration Experience

An ideal experiment:
Take a small real Hono TypeScript application.
Do not directly compile its dependencies.
Port only its application source.
Measure:
Lines unchanged
Lines changed
Imports changed
Unsupported constructs
Semantic changes required
Performance
Memory usage
Binary size
A useful project KPI might eventually be:
Percentage of ordinary Hono application code that transfers unchanged.
Do not establish a target until experimentation.

<a id="web-30"></a>

## 30. Compatibility Test Corpus

Build original compatibility tests around documented Hono behavior.
Categories:
* routing
* route precedence
* path parameters
* query parameters
* headers
* cookies
* middleware ordering
* context variables
* error handling
* JSON
* text
* HTML
* redirects
* streaming
* JSX
* nested routers
Do not blindly copy Hono’s internal source tests.
Write independent tests describing observable public behavior.
