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
