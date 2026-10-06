# Milestone 01 amendment: integer values

[Ownership proof](./01-ownership-proof.md) · [Primitive workstream](../primitives-and-collections.md) · [Original prototype](../core-and-backend-prototypes.md#core-52)

**Status:** Implemented and tested as a provisional experiment on September 30, 2026. These rules extend the ownership proof enough to run the original numeric `User`/`normalize` example. They do not settle the full numeric type system.

**Subsequent extension:** [Milestone 12](./12-scalar-operators.md) adds arithmetic, unary negation, comparisons, and boolean operators. It separates minus tokens and permits whitespace before digits, while preserving literal bounds. The original rules below record this amendment's earlier scope.

## Representation and syntax

`int` is a signed 64-bit value, lowered to Rust `i64`, with the inclusive range `-9223372036854775808` through `9223372036854775807`. Its width is independent of the target's pointer width. There is no implicit conversion between `int`, `string`, and records.

Literals contain ASCII decimal digits with an optional adjacent leading minus sign. Leading zeros are decimal: `00042` is `42`, and `-0` is `0`. A negative literal is a single token; whitespace after the minus sign and general unary negation are unsupported. Fractions, exponents, radix prefixes, separators, and suffixes are unsupported. Arithmetic operators are also unsupported, so runtime overflow behavior remains undecided.

The frontend validates literal bounds, including the negative minimum, before Rust generation or backend lookup. Out-of-range literals produce an `integer-range` diagnostic covering the complete literal. Emission uses the validated value with an explicit `i64` suffix.

## Copy and ownership behavior

- Integer literals, locals, parameters, return values, and record fields are supported. Local types may be inferred.
- An integer assignment or argument copies the value. The original binding remains available. An integer expression statement also leaves its binding available.
- Every integer parameter has the inspectable contract `copy`, including an unused parameter. It lowers to an `i64` value parameter. This contract is separate from the `read`/`mutate`/`consume` ordering for owned values.
- Reading or returning an integer field copies it and retains the record. A function that only returns `user.id` therefore has a `read` contract for `user`.
- Replacing an integer field requires mutable access to its record. `const` still prevents rebinding a local; it permits replacing record fields.
- Records remain owned values even if all their fields are integers. Assigning or consuming such a record still transfers it. There is no inferred record copying.
- `clone(record)` explicitly creates an independent record with copied integers and independent owned strings. Direct `clone(int)` remains unsupported because copying integers needs no clone operation.
- `console.log(int)` prints the signed decimal value followed by a newline. String methods still require strings.

## Argument evaluation and access

Arguments are evaluated from left to right. Once an integer argument has been copied, it retains no loan on its original binding or record. Existing loans from earlier owned arguments still constrain later argument evaluation.

For a function that takes a copied integer followed by a mutable record, `update(user.id, user)` is valid: the first argument captures the old integer value before the record is borrowed. In the reverse order, `update(user, user.id)` is rejected if the first parameter requires mutable access: the earlier exclusive loan is already active when the field is read. Earlier consumption likewise makes later field reads invalid. This deliberately retains the current conservative whole-record access rules.

## Acceptance evidence

The [original example fixture](../../crates/tsrust-compiler/tests/fixtures/integers/original_user.tsr) contains `id: int`, initializes it with `1`, and prints `pat` through `normalize(user)`. The inferred contract is `normalize.user: read`; the generated program requires no implicit clone.

The [integer acceptance suite](../../crates/tsrust-compiler/tests/integers.rs) has eight test groups covering the original example, boundary values, repeated scalar use, transitive copy contracts, typed fields, mutation, cloning, record transfers, argument evaluation order, frontend rejection, and diagnostic spans. Accepted inputs are checked through both the library and CLI, compiled with the pinned Rust backend, and compared against exact native output. A rejected native build is checked with an empty `PATH` to establish that range validation precedes backend lookup.

Run from the repository root:

```sh
cargo test --offline --test integers
cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/integers/original_user.tsr
```

At completion of this amendment, all 37 Cargo test groups passed, including the unchanged 11-case ownership corpus and handwritten Rust witnesses. Formatting and Clippy are checked separately. This is bounded implementation evidence, not a general soundness proof.

## Deferred decisions

[Milestone 02](./02-control-flow.md) adds boolean copying and branch ownership joins, bringing the suite to 48 test groups. Arithmetic and its overflow policy, comparisons, floats, `number`, explicit numeric widths, casts, serialization, FFI layout, and numeric collection behavior still need separate scope. Closure captures and escaping lifetimes are next; async follows separately.
