# Async and runtime

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Backend proof.

**Scope:** Define execution, task lifetimes, cancellation, concurrency, and runtime initialization.

**Planning deliverable:** An async/runtime design tested against captures, borrowing across await, and cancellation.

**Dependencies to coordinate:** [ownership-and-memory](./ownership-and-memory.md), [error-handling](./error-handling.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-09"></a>

## 9. Async and Concurrency

Preserve:
async
await
Promise<T>
at the language surface if practical.
Example:
async function getUser(id: string): Promise<User> {
    return await db.users.find(id);
}
Research:
* whether Promise<T> should really be named Promise<T> or something more runtime-neutral
* structured concurrency
* task cancellation
* async drop/destructors
* executor/runtime selection
* work stealing
* blocking vs nonblocking APIs
* async streams
* backpressure
* timers
* channels
* task-local storage
Do not blindly reproduce JavaScript’s event-loop semantics.
Determine which Node behaviors developers actually rely on versus which should be intentionally replaced.

<a id="core-32"></a>

## 32. Global Runtime Model

Explicitly research what runtime the language actually needs.
Questions:
* Is there a global async executor?
* Who initializes it?
* Is there thread-local state?
* Is there a runtime allocator?
* How large is a minimal binary?
* Can synchronous programs avoid linking async components?
* Can applications opt into different executors?
* How much runtime machinery is acceptable?
The ideal is a very small runtime rather than “Node rewritten in Rust.”
