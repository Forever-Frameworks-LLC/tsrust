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
