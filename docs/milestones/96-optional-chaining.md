# Milestone 96: optional chaining

Status: focused implementation accepted. [Implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) binds retained compiler checks, scoped boundary/resource supplements, one editor group plus a scoped protocol supplement and six native language examples at O0/O3. Combined regression and native/compiled-Node measurements are accepted at checkpoint 96.

## Scope

Implement contiguous typed optional chains containing field access, indexed access and current callable or built-in method calls. Evaluate each base once. A reached `?.` boundary returns undefined for null or undefined and skips the remaining contiguous suffix, including indices and arguments. Parentheses end that short-circuit region: `value?.field.method()` and `(value?.field).method()` have different nullable-access requirements. Follow the [ECMAScript optional-chain evaluation rules](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-optional-chains) within the supported typed domain.

Reuse current nullable data payloads, array ranks, generic record fields, namespace calls and built-in method contracts. Support reached mutation, callbacks and nullable method results with their existing restrictions. Nullable callable storage, arbitrary payload unions and class features remain separately scoped prerequisites. Optional chains cannot be assignment or update destinations.

Copy selections are snapshots. Owned field or indexed selections from retained named storage are scoped readonly views; explicit clone creates independent ownership. Fresh receivers may transfer selected payloads, and fresh method results remain owned. Preserve source loans, partial transfers, branch joins, capture and escape checks, normal disposal and unwind behavior. Do not add implicit clones or a new runtime container.

Use standard Rust branches and matches with checked selected-value provenance. Keep native indexed bounds, UTF-8 behavior and method-specific limits. Add undefined to results when a nullish boundary is reachable, while preserving existing nullable members. Successful-branch facts remain local; optional evaluation does not unconditionally narrow the original receiver afterward.

## Acceptance

Four unique focused compiler groups and scoped boundary/resource/mutator supplements are accepted. Editor help uses cached compiler-checked whole-chain results and successful-hop receiver facts; a scoped protocol supplement covers statically present receivers and mutators. Six language examples passed native execution at optimization levels 0 and 3; the final merged compiler emits identical Rust. This syntax adds no new built-in method or property.

Eight accepted matched comparison pairs cover scalar record fields, String length, array indexing and joining, owned record fields and repeated nullable boundaries. Retain natural values, tags and effect counts; disclose setup, cloning and serialization costs. They are registered across 674 source rows and are measured at checkpoint 96. Store accepted timing and allocation records in SQLite; Markdown reports remain optional exports.

[Optional chaining guide](../guides/language/optional-chaining.md) explains lazy contiguous evaluation, grouping, native bounds and scoped readonly ownership. The original borrowed-view example rejection is preserved; its explicit-clone repair produces the same six documented output lines. No builtin catalog entry changes.

[Checkpoint96 completed evidence](../../benchmarks/runtime/diagnostics/2026-10-04T01-06-08-000Z-checkpoint96-completed-proof.json) binds 1,033 retained baseline checks plus six focused concat-refinement groups, with targeted repairs and no full restart. The 48 new comparisons are measured; 18 older affected workloads have fresh measurements. Whole-workload native/compiled-Node results and normalized resources remain in SQLite, with original records preserved. Native owned copying and retained serialization costs differ from JavaScript sharing; measurements do not imply allocation parity or universal wins.
