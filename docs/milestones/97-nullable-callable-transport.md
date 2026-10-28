# Milestone 97: nullable callable transport

Status: focused implementation accepted. [Acceptance evidence](../../benchmarks/runtime/diagnostics/2026-10-04T04-22-29-000Z-nullable-callable-implementation.json) binds compiler, editor, ownership, executable examples, original failures and ten registered comparison pairs. Full regression and accumulated measurements remain scheduled for checkpoint 101.

## Why this comes first

An optional parameter can contain either its declared value or undefined. The compiler already transports nullable data, but explicitly rejects nullable callable payloads. Adding optional data parameters alone would leave supported callback ownership and invocation kinds behind. This gate extends the shared nullable representation before changing parameter arity or default evaluation.

## Required behavior

Support one callable payload with null, undefined, or both, using the existing canonical ordering and nominal/structural callable contracts. Cover declared functions, concrete closures, read/reusable/once/owned/owned-once callbacks and finite stored callable families. Transport these values through currently supported locals, parameters, returns, record fields, conditional selections and captures, including existing generic and recursive callable signatures.

A present payload retains its argument access modes, invocation mode, concrete origin and capture lifetime. Guarded invocation, optional invocation, nullish coalescing and nullish assignment must preserve those contracts. Absence must never evaluate call arguments or create a callable owner. Consuming a once payload must remain a checked move; wrapping a borrowed callback in a nullable tag must not permit escape. Nullable storage must not grant new Copy or clone permissions.

Use the current source ownership rules in every witness. Bare callable annotations own reusable environments; `once (...) => ...` owns an environment that can be invoked once, as does `owned once (...) => ...`. Explicit `read` and `mutate` callable annotations borrow their environments. Nullable factory results must preserve those rules: owning results may escape, while wrapping a borrowed result annotation in a nullable union must not bypass the existing escape restriction.

Nullish assignment retains the existing destination-alias contract. Its borrowed expression result is read-only: it may invoke a read callback, but cannot mutate or consume a callable environment. A reusable owning callback can instead be assigned with `action ??= fallback` and then invoked through the narrowed destination. Verify both that positive schedule and the rejected mutable/once alias calls; do not weaken the view contract to make a fluent call compile.

Use ordinary typed Rust tags, generic callback bounds and existing finite callable layouts. Preserve borrowed views and precise lifetimes. Do not add trait-object boxing, hidden owner clones, origin erasure, unsafe code, larger stacks or relaxed depth/layout limits.

Existing `typeof`, `Boolean` and `Array.isArray` observations must inspect the active tag without consuming reusable source storage. Function-source conversion through `String` remains unsupported, including callable-containing nullable types. Existing console and numeric conversion limitations must remain explicit rather than producing fabricated values.

## Acceptance

- Exercise every supported invocation/ownership kind through transport, guarded calls, optional calls, coalescing and assignment. Verify concrete and abstract callback layouts, generic forwarding, imported declarations and recursive signature/layout boundaries.
- Include nullable callbacks whose signatures contain ordinary callback parameters or results. Generated generic bounds must cover those nested contracts. Borrowed captures must retain the payload's generic type and all required lifetimes; an immutable nullable view must not permit mutable or once invocation.
- Verify callable record fields whose names overlap built-in methods, including `concat`. Dispatch must follow the checked receiver and field contract, including existing nonnullable fields, rather than the member spelling alone.
- Check strict absent-value diagnostics, preserved argument access modes, source-order effects, skipped arguments, once moves, borrowed capture escape and clone restrictions. Retain normal depth limits and meaningful deep-expression witnesses.
- Execute native fixtures at O0 and O3 for behavior that depends on output or disposal. Verify ordinary ownership transfer and balanced normal/unwind cleanup; include allocation checks only where they establish a specific representation guarantee.
- Keep hover, completion, signature help, parameter help and compiler facts consistent for nullable declarations and narrowed callable payloads. Preserve current-header, incomplete-body and project/import behavior.
- Verify language-guide examples against the compiler and native execution. Any new built-in method or property still requires complete catalog documentation before acceptance; this gate does not itself introduce a new built-in name.
- Register accurate TypeScript/Node counterparts for newly measurable behavior, or record a precise non-comparability reason. Queue measurements for checkpoint 101, preserving SQLite as the canonical analytics source.

## Following work

[Milestone 98](./98-optional-default-parameters.md) scopes optional/default parameters across currently supported data and callable parameter kinds together. Omission metadata belongs in callable contracts and every direct, generic, forwarded, conditional and stored call path. Expression defaults also require lazy evaluation in callee parameter order, earlier-parameter scope, correct required-position arity and retained ownership/loans. Rest parameters, destructuring, heterogeneous unions and nullable array elements remain separate language features; they do not justify excluding already supported parameter kinds.

## Accepted metadata prerequisite

The project metadata portion is integrated. `ProjectAnalysis::checked_type_facts()` retains original module coordinates and structured nominal types for nullable projections, successful optional receivers and checked call/nullish results. It collects these facts in the existing checked-body walk. Callable members in displayed nullable types are parenthesized, preserving the distinction between an absent callable and a callable returning an absent value.

Five focused checks pass against the integrated compiler: imported nullable views and optional/function results keep their declaration identity; direct callable results keep their expression coordinates; specialized arrow callback calls retain their checked result type; conflicting known types remain explicit; and an unresolved result specialization cannot expose another specialization's generic signature. Additional staged checks cover optional-call result spans, complete nested callback contracts and parenthesized callable array types. The guarded final compiler merge includes these repairs and preserves current peer changes. Metadata checks remain scoped prerequisites rather than a full regression claim.

## Accepted TypeScript counterpart prerequisite

Eight comparison drafts pass actual pinned TypeScript compilation and Node execution against independent complete-output expectations: 466 cases include 42 literal expectations, and a separate fixture verifies eight source-order lines. Each reported operation will count one complete scenario iteration; exact retained counters disclose skipped and reached callbacks separately. This was the initial counterpart prerequisite. The native acceptance below supersedes its pending source checks; checkpoint 101 measurements remain pending. Untimed counterpart checks alone do not establish native behavior or performance.

Two additional higher-order factory comparisons pass separate TypeScript/Node source acceptance across 120 complete cases, including fourteen literal expectations. They cover nullable owning factories with borrowed callback inputs and factories that return only absence. Six counters distinguish factory creation, argument evaluation, outer and child invocation, and fallback selection. The original eight comparisons were not rerun.

All ten staged comparison sources now pass native execution at O0 and O3 across 586 complete cases, including 56 literal expectations, plus eight source-order lines. A repaired conditional Read argument passes its affected 58 cases at both optimization levels; its emitted Rust matches the previously accepted source. Original failed attempts remain retained. These are untimed correctness checks. The final peer-preserving compiler merge and all ten comparison registrations are integrated. Final emission correspondence preserves the accepted native executions; performance and allocation measurements are queued for checkpoint 101.

## Accepted editor prerequisite

The editor changes are integrated after focused protocol acceptance on the staged compiler. Nullable declarations, narrowed payloads, imported and generic signatures, incomplete headers and callable-array results retain consistent hover and parameter help. Six executable language examples produce nine expected lines at O0 and O3. The merged editor regenerates identical Rust for those examples, so the native executions were not repeated. The final compiler merge and durable evidence are integrated. The [language guide](../guides/language/nullable-callables.md) publishes four of the verified examples. Pinned formatting is followed by a successful workspace library check; this does not replace checkpoint 101 regression testing.
