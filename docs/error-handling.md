# Error handling

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof.

**Scope:** Choose recoverable-error propagation and fatal-failure behavior.

**Planning deliverable:** An error-model decision with synchronous, async, and Rust interoperability examples.

**Dependencies to coordinate:** [type-system](./type-system.md)

## Owner direction (2026-10-05)

The owner clarified that Tsrust should automate safe code generation and keep native failures within Tsrust's error model. The implementation target is therefore to prevent ordinary supported operations from leaking Rust panics, and to represent runtime failures such as checked bounds and integer arithmetic errors as recoverable Tsrust errors. The earlier recommendation to leave these failures permanently fatal does not meet that target.

This is a design direction, not an implemented guarantee. Error types, propagation, catch behavior, ownership during failure, cleanup and library boundaries still require implementation and verification. Unhandled errors need controlled Tsrust reporting at the entry boundary. Failures outside language control, such as process termination or an operating-system resource failure that aborts execution, cannot be promised recoverable. Compiler defects must remain identifiable rather than masquerade as expected application errors.

## Current experiment

[Milestone 12](./milestones/12-scalar-operators.md) provisionally makes integer overflow and division/remainder errors fatal runtime failures. Generated checked operations report the operator’s source byte range, independent of backend optimization settings. This does not implement recoverable errors, exceptions, or a general panic/cleanup contract.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-10"></a>

## 10. Error Handling

This needs substantial research.
Possible mechanisms:
Result types
Result<T, E>
Throw/catch syntax backed by Result
Allow:
try {
    const user = await getUser();
} catch (error) {
    ...
}
while compiling ordinary recoverable errors into explicit result types.
Potential ergonomic propagation operator:
const user = try getUser();
or:
const user = getUser()?;
But ? is Rust-like and may violate the design goal of hiding Rust vocabulary.
Research:
* checked versus unchecked errors
* fatal/panic conditions
* stack traces
* error causes
* error wrapping
* async errors
* automatic propagation
* interoperability with Rust libraries
Goal: retain Node-like ease without making error behavior implicit or unpredictable.

## Releasing callback captures during cleanup

A Map containing borrowing callbacks keeps their captures borrowed until the
callbacks are removed. Clearing that Map in `finally` releases those captures;
subsequent statements in the same cleanup block may mutate them. Cleanup runs
before a pending return, error, break or continue resumes. A cleanup error or
control exit replaces the pending outcome. Clearing does not copy captures or
invoke callbacks, and removed callback environments are disposed once.

The executable regression `catch_cleanup_clear_releases_captures_before_cleanup_mutation`
covers normal, caught, continue and break paths for both a local Map and a Map
stored in a record field.

Cleanup may clear several independent Maps, including different fields of the
same record. Each clear releases that Map's callbacks in statement order. Empty
Maps outside a loop remain empty when the loop executes zero times. Renewing an
empty native storage epoch does not discard callbacks retained by another Map.

Nested handlers preserve cleanup order: an inner cleanup runs before an outer
catch observes a rethrown error. An outer cleanup then runs before loop control
resumes. A break in that cleanup can replace a pending continue, after preceding
clear and capture-mutation statements have executed.

A callback inserted after cleanup clears a Map remains stored and keeps its
captures borrowed. A later explicit clear releases it. A view returned by `get`
must leave its scope before the Map can be cleared or otherwise mutated.

An empty local Map of records containing borrowing callbacks supports size,
lookup, membership, deletion, iteration, movement and clearing even when no
record is inserted. Lookup returns undefined and iteration executes no body.
Its absent callback environment cannot execute or throw; it does not introduce
an unknown failure into a surrounding catch. Later insertions retain their
concrete callback environments, capture loans and failure contracts.

The same release boundary applies to counted for loops and do-while loops whose
finally body clears each scoped Map before changing its captured values.
Continue runs finally before the for update or do-while condition. Break runs
finally and skips that tail. A counted loop that executes zero iterations leaves
the original empty Map available. Loop conditions and updates must not retain
the callback payload across the release boundary.

For-of loops use the same cleanup boundary for array, Set, Map-entry and string
iteration. The iterable is evaluated once with its usual borrowing rules.
Finally runs before continue or break leaves an iteration, and clearing the
scoped Map releases its callbacks before captured values change. Empty
iteration leaves the original empty Map available. Iterating the Map being
cleared remains subject to the ordinary iterator borrow restrictions.

### Releasing individual Map callback captures

In straight-line local storage, deleting a known scalar-literal key releases that
entry's callback captures. Other entries keep their captures borrowed. The same
rule applies to a Map stored in a local record field. Replacing a known entry
evaluates the replacement before releasing the previous payload; a replacement
that throws leaves the previous entry intact. A later insertion can borrow a
released value again. Clear releases all remaining entry captures.

The following pattern is covered by native disposal tests:

```ts
type Holder = { run: ReadOnly<Function<int>>; };

function main(): void {
    const first = [3];
    const second = [5];
    const handlers = new Map<string, Holder>();
    handlers.set("first", { run: (): int => first[0] });
    handlers.set("second", { run: (): int => second[0] });
    handlers.delete("first");
    first[0] = 9;
    {
        const saved = handlers.get("second");
        if (saved !== undefined) console.log(saved.run()); // 5
    }
    handlers.clear();
    second[0] = 11;
}
```

A lookup view must leave scope before mutation of its Map. If another entry
captures `first`, deleting only one such entry does not permit changing `first`.
Dynamic keys and uncertain branch outcomes conservatively retain potentially live
captures. At an if statement's normal completion join, entries deleted or
replaced on every reaching path release their old captures. An early return does
not retain its entries on the remaining path. Reinsertion retains the newly stored
capture. Loop completion uses the same rule: all reaching exit states must release
the entry. Zero-iteration paths and early break or continue paths retain any
potentially live captures. This covers while, counted for, do-while and for-of
completion. A later loop may refill a narrowed local Map with new callback
environments while keeping surviving entries. Its storage representation widens
before entry, including the zero-iteration path. Multiple incoming origins and
counted-loop block transport are supported. A branch may delete a known entry
and change its released capture before the branch ends. Surviving storage returns
through the branch join; returning arms dispose it at their exit. Refilling a
previously narrowed Map inside a branch widens its storage before entry, so new
callback origins survive the join even when the other arm inserts nothing. Other retained
entries continue to protect their captures. Normally completing while, counted
for, do-while and array/string for-of bodies may delete a known entry and mutate
its released capture before the iteration ends. Surviving entries retain their
key order and captures, including Maps stored in local record fields. Do-while
preserves its initial entries for the first body and tests its condition after
each body. Empty carried storage is supported. Abrupt controls, nested loops,
cleanup transport and release within the same expression remain under development.
Changing a callback environment representation for later capture access can scan
surviving entries. Known entry deletion avoids that conversion when the released
capture has no later source use or opaque named call. The release feature does
not yet promise constant-time updates in every supported program.
