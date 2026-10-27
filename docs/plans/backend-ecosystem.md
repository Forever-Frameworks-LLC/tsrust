# Backend ecosystem

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof; later packages.

**Scope:** Define native framework ergonomics and the first ecosystem packages.

**Planning deliverable:** A staged package backlog tied to actual backend prototype needs.

**Dependencies to coordinate:** [standard-library](../guides/language/standard-library.md), [http-and-hono](../guides/language/http-and-hono.md), [validation](../guides/language/validation.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-15"></a>

## 15. Backend-First Framework Strategy

A later project stage should port or recreate popular JavaScript/TypeScript libraries around language-native primitives.
Priority candidates:
* Hono
* Zod
* Express-style routing concepts
* Fastify concepts
* tRPC concepts
* Prisma-like database ergonomics
* Drizzle-style typed SQL
* date utilities
* JWT libraries
* validation libraries
* logging libraries
* testing frameworks
Do not necessarily line-by-line port these libraries.
Instead:
1. Identify why developers like the library.
2. Identify its public API and ergonomic conventions.
3. Reimplement those concepts natively.
4. Remove runtime behaviors that are unnecessary in a statically compiled language.
5. Take advantage of compile-time generation where possible.
Example:
Zod exists partly because TypeScript’s compile-time types disappear at runtime.
A native version could potentially derive validation metadata directly from types:
const UserSchema = schema<User>();
or:
@validate
type User = {
    email: Email;
    age: int >= 18;
};
Research whether the language can eliminate entire classes of runtime JavaScript tooling through compiler support.

<a id="core-57"></a>

## 57. First Native Ecosystem Packages

After the core language stabilizes, build first-party/native equivalents inspired by:
1. Hono
2. Zod
3. dotenv/config
4. logging
5. PostgreSQL client
6. Redis client
7. JWT
8. test runner
9. mocking
10. date/time utilities
Focus on API ergonomics rather than literal source compatibility.
