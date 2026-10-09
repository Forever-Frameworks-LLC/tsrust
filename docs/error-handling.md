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
each body. Empty carried storage is supported. Break and continue after deletion return
surviving entries before leaving the body. Counted continue still executes its
update; break skips the update and the next condition. A break before deletion
keeps the original capture live. Counted updates may inspect and delete entries created by the body. A temporary
entry survives continue until the update deletes it; break skips the update.
A capture remains protected until that deletion. Nested loops, cleanup transport
and release within the same expression remain under development.
Changing a callback environment representation for later capture access can scan
surviving entries. Known entry deletion avoids that conversion when the released
capture has no later source use or opaque named call. The release feature does
not yet promise constant-time updates in every supported program.


Known-key deletion inside a try body can release that entry's callback capture
before later mutation in the body. Remaining entries stay available in catch
and finally. Borrowed iteration in cleanup finishes its iterator borrow before
an error transfers completion storage. Deletion does not release captures held
by other entries. These guarantees also apply to straight-line handler and
cleanup entry conversions. In while, counted, post-test and for-of loops,
finally runs before a pending break or continue resumes. Cleanup can replace
that control with its own break or continue. Surviving entries remain available
in cleanup; deleted captures are released before later mutation in the body.
Nested finally blocks run from the inner block outward before the loop resumes.
A populated first entry remains visible even when cleanup empties later
iterations. Pending returns and errors also pass through these cleanup loops.
A return expression is evaluated before cleanup. A return or throw in cleanup
replaces the pending result; a continue in cleanup can cancel it and resume the
loop. This includes a loop whose body exits without any backedge. If a failing
operation before deletion can retain the entry, cleanup must delete that entry
(or clear the Map) before changing its capture. Labelled while, counted, post-test and for-of loops preserve the same storage
rules. In nested cleanup loops, an inner break keeps its surviving entries and
stays inside the enclosing phase. A break or continue targeting an outer loop
runs each exited finally from the inside outward. An inner cleanup continue can
cancel an outer exit and remain in the inner loop; outer cleanup runs when that
phase subsequently exits. Catching an inner error also preserves survivor
storage before outer cleanup. Arbitrary labelled blocks, nonlocal direct
controls outside completion regions and independently specialized phase storage
still require acceptance.


Nested loop controls also preserve local Map owners without an exception boundary.
A break or continue to an enclosing loop transfers the current live storage to
that loop's carried body before leaving the inner scopes. The enclosing loop
then selects its break or backedge conversion. Deleted callback captures are
released; surviving entries remain callable and keep their captures borrowed.
Controls crossing try/finally still run each required cleanup before dispatch.
Labelled block breaks return their planned Map storage after required cleanup.


Labelled blocks preserve local Map callback storage across normal fallthrough and
breaks to the block. The compiler joins entries retained by each reaching exit.
An early break before deletion retains that entry; a refill with a different
callback remains callable after the block. Deleted captures can be mutated once
all reaching owners have released them. Remaining entries keep their captures
borrowed until deletion, clear or disposal.

A break from an inner loop to a labelled block carries the live owner to the block
result. Required finally bodies run before that result is returned. Cleanup can
replace a pending completion with its own block break. This storage transport
moves entries and does not clone captured values. Source ownership checks still
reject mutation of surviving captures.


Ordinary loops can carry different callback origins at first entry, on a backedge
and on break. A zero-iteration path retains its original entries. Deletion can
release an old capture before mutation, and a replacement callback remains
callable on later iterations. An empty break remains empty instead of inheriting
backedge entries. If a break happens before deletion, that entry remains live.
The compiler keeps the first owner's representation separate from the working
family required by later iterations and by cleanup observation.

A loop does not need a reachable next iteration to release a deleted entry.
Direct break, return and throw paths release the deleted callback before a later
mutation of its capture. Remaining entries retain their callbacks and borrowing
restrictions. This applies to while, counted, post-test and for-of loops. A
post-test body ending in break still reaches the code after the loop; its surviving
Map is converted to the checked exit layout before that code executes.

When a try body and its catch handler both delete the same known Map entry, the
normal result releases that entry's captures before following statements. The
handler can mutate the deleted capture after its own deletion. Entries retained
on either normal path remain borrowed. A handler that keeps the entry cannot
make a later mutation safe merely because the try body's successful path deletes
it. Normal result storage is independent of the storage carried by an error or a
pending return/control completion.

After finally completes normally, the Map result uses the cleaned normal payload
layout. A callback deleted by cleanup does not become part of the following
normal storage merely because it existed before cleanup. Surviving entries retain
their borrowing restrictions. Cleanup still runs before a pending error or return
resumes, and a return from cleanup overrides the pending completion with the usual
callback disposal order.

For a try/catch, a thrown error carries the Map entries still
live at that error point into the handler. Deleting a known entry before a
fallible call releases its capture in the handler; an error before deletion
retains that capture. Other entries remain borrowed. This also applies when the
Map has been moved into a record field. The handler's normal result and the
body's error payload have independent storage layouts.

A finally clause does not restore a capture already released before handler
entry. The handler observes the surviving Map entries, then cleanup runs before
its completion resumes. Errors before deletion still retain the original capture.

For finally without a catch, cleanup starts with the Map entries surviving the
pending body error. It observes those entries and their callbacks before the
error resumes. A capture deleted before every body error can be mutated during
cleanup; an error before deletion keeps that capture borrowed. Cleanup still
runs on normal completion, and its own abrupt completion overrides the pending
one with the normal disposal rules.


When a catch removes a known Map entry and rethrows, an enclosing catch receives
the remaining entries. A capture used only by the removed entry can be changed
in that enclosing catch. Captures still used by surviving entries remain
borrowed. Native cleanup disposes removed and surviving entries exactly once.


A finally block can remove a known Map entry before propagating an earlier error
or throwing a replacement error. The enclosing catch receives the survivors in
either case. A replacement error takes precedence over the pending error; removal
does not release captures belonging to surviving entries.


A catch body may remove a known Map entry and then rethrow. Its finally block
receives the survivors, so it can change a capture used only by the removed
entry. This also applies when finally throws a replacement error. Captures used
by surviving entries remain borrowed until those entries are removed or dropped.
