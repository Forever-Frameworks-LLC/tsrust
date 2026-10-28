# Database integration

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Scope native drivers, typed queries, schema validation, and database ergonomics.

**Planning deliverable:** A first database integration proposal with ownership, nullability, and schema-validation rules.

**Dependencies to coordinate:** [async-and-runtime](async-and-runtime.md), [interop-and-targets](../tooling/interop-and-targets.md), [metadata-and-serialization](metadata-and-serialization.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-27"></a>

## 27. Database Layer

Backend adoption will require excellent database support.
Research native integrations for:
* PostgreSQL
* MySQL
* SQLite
* Redis
* MongoDB
Potential typed SQL:
const user = await db.query<User>`
    SELECT id, name, email
    FROM users
    WHERE id = ${id}
`;
Compiler could validate:
* parameter types
* returned columns
* nullable fields
Research offline schema snapshots versus live database introspection.
