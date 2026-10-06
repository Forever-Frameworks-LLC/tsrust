# Gate 120: general data unions

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior and scope

Supported data alternatives can now share a canonical tagged union: int, number, boolean, string, records, arrays, positional tuples, null and undefined. Alternative order and duplicates do not change identity. Transport, explicit cloning, readonly observations, non-null projections and lazy coalescing preserve the active alternative. A non-null assertion removes only null and undefined. Coalescing retains every present alternative and may introduce a different data fallback, including after an early-exit typeof guard. Mutable parameters borrow supplied owners; omitted/defaulted parameters retain fresh local default owners. Spread staging retains each possible alternative and its owner or loan instead of assuming a single present payload.

Strict typeof comparisons narrow stable bindings and record fields in either operand order. The number category includes int and number; object includes null, records, arrays and tuples. Replacement and overlapping mutation invalidate proofs. Ambiguous optional member receivers require explicit narrowing. Generic union patterns can infer one unresolved alternative after subtracting concrete alternatives; multiple unresolved patterns require explicit type arguments.

Callable unions retain the existing one-callable-payload contract. Void alternatives, union array elements, general union arithmetic/equality and implicit common-member access remain unsupported. General data unions provide the representation foundation for heterogeneous array leaves and Object.values/entries; they do not themselves complete record value enumeration.

## Implementation and repairs

Union canonicalization and residual projections retain all alternatives. Record layout and recursive graph discovery follow every union alternative, rejecting hidden direct cycles and preserving iterative clone/drop for recursive array edges. Native owner enums use separate storage parameters for owning alternatives; normalized views borrow canonical data types. Mutable data carriers reborrow each owning alternative without detaching source mutations. Tagged spread carriers preserve distinct alternatives at dynamically selected positions.

Acceptance repaired borrowed generic view normalization, temporary copy projections, required mutable carriers, mixed spread staging and narrowed fallback inference. The callable fallback path retains contextual callable conversion rather than creating an incompatible union of callable representations. The prepared rank-driver harness caught that regression; its original failures and accepted supplement are retained.

The normal-stack 100-call witness initially aborted during type checking. Phase instrumentation identified analysis before emission, and actual compiled dispatch frames were inspected. Exact nongeneric unshadowed union-returning calls now avoid redundant injection recursion. The unchanged normal worker stack passes; diagnostic instrumentation was removed. A separate 12,000-node recursive union graph clones and drops successfully.

## Focused evidence

- `target/editor/data-unions-callable-fallback-repair.log`: all 16 data-union tests and 24 nullable-callable tests pass.
- `target/editor/data-unions-final-compiler.log`: 10 call-spread and 21 optional-parameter tests pass, alongside the retained 15-test data-union pass before the final fallback supplement. Earlier accepted checks retain nullable narrowing (4), nullable unions (4) and Object.keys (6).
- `target/editor/data-unions-node-accepted.log`: 14 source fixtures match Node v24.11.1. Only explicit native clone() is adapted with independent iterative data copying; tags, typeof, guards, fallback selection and source mutations execute directly in Node. The generated depth witness is tested natively separately.
- `target/editor/data-unions-editor-current.log`: two focused editor tests pass, covering guarded hover/completion and all 15 native catalog examples. The changed catalog entries are null, undefined, void, typeof, String, Boolean and Array.isArray.
- `target/editor/data-unions-stack-phase.log`, `data-unions-frames.log`, `data-unions-frames-repaired3.log`, and `data-unions-stack-repair.log` preserve the failing phase, actual frames and normal-stack repair.
- `target/editor/data-unions-registration.log`: 18 report and registration checks pass, including matched sources, fingerprints, independent oracle routing and mutation guards.
- `target/editor/data-unions-benchmark-current.log` and `benchmarks/runtime/results/2026-10-04T22-05-09-391Z-verify.json`: data_union_values and the affected nullable_callable_generic_coalesce pair each pass all five untimed native/Node-after-tsc cases. The new pair retains all six result cells for each complete scenario.
- `target/editor/data-unions-harness-final.log`: 321 of 322 checks pass initially. The affected prepared rank-driver check passes with its three neighboring checks in `data-unions-harness-accepted-supplement.log`; all 322 checks have passing evidence across retained runs.
- `target/editor/data-unions-build-debug-current.log` and `data-unions-build-release-current.log`: compiler/editor consumers rebuilt. The catalog-only final refresh is recorded separately.

The byte-exact baseline is `archive/data-unions-start`. Original failed checks remain alongside repairs. No full compiler/LSP regression, timing or allocation profiling was run. SQLite measurement records are unchanged. The new comparison is ready and unmeasured; earlier checkpoint measurements remain historical evidence.

Next gate: union array leaves, including ownership, cloning, array methods and editor representation, before heterogeneous Object.values/entries.
