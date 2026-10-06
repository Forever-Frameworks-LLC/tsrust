# Validation

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Compare schema APIs and type-derived validation, including HTTP integration.

**Planning deliverable:** A checked input-validation design with typed success values and useful failure reports.

**Dependencies to coordinate:** [type-system](./type-system.md), [metadata-and-serialization](./metadata-and-serialization.md), [error-handling](./error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-29"></a>

## 29. Validation Framework

Build a Zod-like API, but investigate compile-time integration.
Potential syntax:
const CreateUser = schema({
    email: string().email(),
    age: int().min(18),
});
Potential stronger language-native approach:
type CreateUser = {
    email: Email;
    age: int where age >= 18;
};
Research:
* refinement types
* branded/opaque types
* runtime validation generation
* JSON parsing
* error reporting
* schema export
* OpenAPI generation

<a id="web-25"></a>

## 25. Zod as the Second Compiler Stress Test

After Hono, use Zod’s developer experience as another compatibility target.
Do not necessarily reproduce its runtime implementation.
Investigate:
const User = z.object({
    email: z.string().email(),
    age: z.number().int().min(18),
});
versus deeper compiler integration.
The important benchmark is:
Can the language provide at least the same validation ergonomics while exploiting static type information that JavaScript cannot?

<a id="web-26"></a>

## 26. Hono + Validation Integration

Eventually target something like:
app.post(
    "/users",
    validate.json(CreateUser),
    async ctx => {
        const input = ctx.valid("json");

        const user = await db.users.insert(input);

        return ctx.json(user, 201);
    }
);
The compiler should know the type of input.
No runtime casts.
No separate hand-written interface if the schema already determines the type.
