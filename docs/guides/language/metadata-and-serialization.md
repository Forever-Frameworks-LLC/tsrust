# Metadata and serialization

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Scope reflection, annotations, code generation, and checked serialization.

**Planning deliverable:** A checked JSON design and a decision on the minimum metadata/code-generation mechanism.

**Dependencies to coordinate:** [type-system](type-system.md), [error-handling](error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-24"></a>

## 24. Reflection and Runtime Type Information

Research carefully.
Questions:
* Are types available at runtime?
* Can developers inspect field names?
* Can frameworks automatically serialize objects?
* Can decorators/annotations attach metadata?
* Is reflection compile-time generated?
* Should runtime reflection be opt-in?
Avoid carrying heavyweight type metadata into every binary unless needed.
Potential syntax:
@reflect
type User = {
    id: int;
    email: string;
};

<a id="core-25"></a>

## 25. Decorators / Attributes / Metadata

TypeScript developers increasingly use decorators.
Research a safer compile-time equivalent.
Example:
@route("/users/:id")
async function getUser(id: string): Promise<User> {}
This could generate routing tables at compile time.
Potential uses:
* serialization
* validation
* database mappings
* dependency injection
* HTTP routes
* CLI parsing
* documentation generation
Avoid runtime decorator machinery where compile-time expansion works.

<a id="core-26"></a>

## 26. Macros and Code Generation

Research whether macros are necessary.
Potential goals:
* derive implementations
* schema generation
* SQL compile-time checking
* route generation
* serialization
* FFI bindings
Avoid exposing Rust-style procedural macro complexity unless absolutely necessary.
Potential simpler model:
derive(Serialize, Clone)
type User = {
    ...
};
or annotations.

<a id="core-30"></a>

## 30. Serialization

Investigate whether types can automatically generate serializers.
Example:
const json = JSON.stringify(user);
could compile to specialized serialization code without reflection.
Likewise:
const user = JSON.parse<User>(body);
should validate the incoming data.
Never make unchecked JSON parsing silently produce typed values.
