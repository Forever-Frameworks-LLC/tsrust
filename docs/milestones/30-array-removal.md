# Milestone 30: owned array removal

[Roadmap](../roadmap.md) · [Owned array elements](./22-owned-array-elements.md) · [Nested record fields](./25-nested-record-fields.md)

**Status:** Complete. All 386 regression groups pass, including 16 array-removal groups. Formatting and Clippy pass.

## Scope and contract

`array.remove(index)` accepts one `int` index and returns the exact element type. It removes the selected element, shifts later elements left, and preserves their order. Scalar values copy into the result; strings, records, and nested arrays transfer ownership without cloning. The array remains owned and usable after removal.

Receivers include named arrays and mixed named field/index storage paths. Receiver indices evaluate once before the removal index. Selected parent storage stays stable through index evaluation under the existing root-owner loans. A named array may be changed by index evaluation before final removal. The removal index converts and checks against the final receiver length after index evaluation. Negative and out-of-range indices fail with the full removal call's source byte range in every optimization profile. Earlier parent selection failures retain their own source ranges and occur before removal-index evaluation.

Removal infers exclusive mutation on the root owner. Argument, capture, iteration, callback, recursive access, and transfer rules remain enforced. Returned owned elements may move independently, be returned, enter another aggregate, or be retained by an escaping closure.

## Acceptance evidence

The [fixture](../../crates/tsrust-compiler/tests/fixtures/array-removal/queue.tsr) removes a record, independently mutates its owned tags, reads the remaining records in order, and moves the removed value back into the array. It prints `Sam`, `saved`, `2`, `Pat`, `Alex`, and `Sam`. Generated Rust uses native `Vec::remove` and contains no source-value clone, boxing, or unsafe code. Native removal shifts later elements, so time is linear in the number shifted; this gate does not add a runtime allocation benchmark.

The [16-group suite](../../crates/tsrust-compiler/tests/array_removal.rs) verifies scalar/string/record/nested results, nominal and rank matching, moves into calls and aggregates, independent returned ownership, named and mixed receiver paths, root access summaries, named index-side mutation, final-length checking, once-only parent evaluation, read-safe operand access, selection/argument/capture/iteration conflicts, all four callback contracts, recursive data effects, retained mutable factories, consuming closures, branch/loop ownership, arity/type/transfer diagnostics, deterministic metadata, and backend-independent CLI operation.

Eight runtime failure cases run with optimization levels 0 and 3 and overflow checks disabled. They cover negative, empty, out-of-range, maximum integer, side-effecting index, early parent selection, later removal, and index-driven shrinking. Each verifies stdout before failure and the precise original source range. Deep evidence includes a 50-level mixed record/index path, a 100-dimensional native nested-array removal, accepted frontend rank 128, and rejected rank 129/excessive mixed nesting.

## Boundaries

This is an indexed removal operation rather than JavaScript `splice` compatibility. Empty arrays fail at any removal index. `pop`, `shift`, optional results, arbitrary range removal, partial record field moves, consuming/mutable iteration, and iterator pipelines remain separately scoped.

Milestone 31 subsequently adds [direct named record field moves and restoration](./31-record-field-moves.md), tracking partial availability through joins and loops. Indexed and iteration element fields remain borrowed; conservative root loans and whole environment capture storage remain in force.
