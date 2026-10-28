# Milestone 100: dense homogeneous call spread

Status: focused implementation, documentation and source registration accepted; see [Focused acceptance](#focused-acceptance). Full regression and accumulated native/compiled-Node measurements remain queued for checkpoint 101 and are not measured.

## Observable behavior

Support array expansion in call argument lists, such as `sum(...values)` and `sum(1, ...values, 2)`. Cover empty, literal, dynamic and multiple spread sources with ordinary arguments before, between and after them. Each supported dense homogeneous array expands its elements in order. Expansion does not add array-literal spread, object spread, heterogeneous tuple types or arbitrary iterable protocols.

Evaluate an observable callee or method receiver once before argument expressions. Evaluate argument segments once, left to right, and retain their values before entering the target and running reached defaults. Preserve optional invocation: an absent callable skips argument and spread-source evaluation. Do not turn fixed-slot padding into supplied arguments or rest elements. Runtime supplied argument counts must survive nullable, abstract, returned, nested and finite-family callback dispatch.

Preserve current target argument bounds and type contracts. Statically known cardinality permits precise compile-time checks. Dynamic cardinality requires sound checks of every reachable slot and the actual minimum/maximum argument count. Where a native arity check is necessary, perform it after supplied expression evaluation and before callee defaults or body execution. Reject incompatible slot types or unsound access rather than silently converting, dropping extra arguments or choosing an unchecked overload.

## Ownership and native representation

Use the existing typed arrays and standard Rust slices, iterators and vectors. Copy scalar leaves where permitted; borrow read-only inputs without implicitly cloning their owners; use mutable iteration only for genuine mutation contracts. Transfer owning elements where the target consumes them or constructs an owning rest pack. A consumed source must become unavailable under the existing ownership rules. Check field sources, temporaries, source-root loans and later argument effects.

Mixed borrowed fixed prefixes and owning rest tails may use a staged local owner: separate the owning tail before borrowing the prefix. Do not reject an otherwise sound schedule merely to avoid implementing it. Preserve array-literal construction and capture effects before optimizing literal expansion; expanding raw element syntax must not change owning callable captures into borrowed direct arguments.

Keep exact nested and nominal identities, finite stored callable storage, drop behavior and unwind cleanup. Do not add generated unsafe code, trait-object boxing, hidden clones, new Copy permissions or raised recursion limits. Keep shared recursive compiler dispatch small.

## Complete current domain

Cover the currently supported int, number, boolean, null, undefined, string, record and owning stored-callable array element families, including supported nested ranks and generic identities. Check direct declarations, generic/imported calls, closures, callable parameters, record fields, conditional/nullable selections, optional invocation and returned environments.

Include supported built-in and native method call sites. Fixed, optional and variadic built-ins need their actual argument contracts; heterogeneous variadic console calls cannot be modeled as a fake homogeneous rest signature. Preserve current receiver access, callbacks, source-order effects, strict argument bounds and native behavior. Existing unsupported callable-array contexts, mixed/nullable element arrays and arbitrary iterables retain precise boundaries.

## Acceptance

- Focused parser, checker and native fixtures cover literal/dynamic cardinality, ordinary/multiple segments, current element families and dispatch contexts.
- Verify exact callee/argument/default ordering, skipped optional work, actual counts, source mutation and normal/panic cleanup at O0 and O3 where observable output or ownership needs native evidence.
- Preserve gate 99 rest/default behavior and precise ownership, type and arity diagnostics. Retain passing groups and original failures; replay only affected behavior.
- Verify completion, hover and parameter help for complete and incomplete spread calls, generic/imported targets and spread-source expression types. Any new built-in names require complete verified catalog documentation before acceptance.
- Register meaningful matched Node-after-tsc source scenarios and keep timings separately pending checkpoint 101. Store original records in canonical SQLite after exact verification.
- Publish executable language examples only after current compiler and native verification. Update coverage and developer evidence without rewriting historical measurements.

## Focused acceptance

The staged compiler, editor and registration work was integrated into the main tree, repaired and checked there. The user guide is [call-spread.md](https://forever-frameworks-llc.github.io/tsrust/language/call-spread/).

Implementation: source `...` arguments parse as call-only spread nodes. `spread_calls.rs` and `spread_builtins.rs` plan each call once into ordered segments, selected argument slots and an optional rest destination; `spread_emit.rs` and `spread_builtin_emit.rs` lower the plan to native slices, iterators and vectors with a runtime supplied count. Supplied counts use `CallArgumentCount`, so dynamic cardinality reaches concrete, abstract, materialized and finite-family dispatch.

Repairs made during integration:

- A nullable carrier slot holding an exclusive element loan is lent directly into a mutable nullable parameter. The previous lowering assumed an owned payload and reached an unreachable arm at run time.
- A statically empty source before a rest parameter, such as `total(...[])`, now emits an exhaustive count match.
- A spread call through a strictly narrowed nullable callable local checks the callee as its present payload.
- Spread argument parsing, the typed-arrow diagnostic scan, postfix operators and already planned spread expressions moved into dedicated helpers, restoring the normal-stack source-depth witnesses without a larger worker stack. `rest_normal_depth_and_over_limit_keep_the_original_worker_stack` failed before this integration and passes again.
- An annotated arrow whose result type ends in adjacent generic closers, such as `():Box<Box<T>>=>`, is recognized as an arrow header again.
- The staged `call_spread_mutate_callback` workload read a captured variable while its mutating closure was alive, and `call_spread_owned_once_optional` used `?.` on an `owned once` callable. Both sources, their TypeScript counterparts, oracle cells and manifest fingerprints were corrected to supported forms with the same observable work.

Evidence (logs under `target/editor`):

- `gate100-affected-compiler.log`: 33 compiler suites, 293 tests passed, none failed. This includes 10 `call_spread` tests, 5 `root_builtin_spread` built-in fixtures at O0 and O3, and the rest, optional/default, nullable, optional-chaining, arrow, array, math, string, formatting, storage and project suites.
- `gate100-editor-affected.log`: all 147 language-server tests passed, including spread hover, completion and conservative signature help, and the executable guide examples at both optimization levels.
- `gate100-guide-examples.log`: every example and boundary diagnostic quoted in the guide, with current compiler and native output.
- `gate100-workload-verify.log`: thirteen `call_spread_*` scenarios pass all correctness cases on native and Node after TypeScript compilation. These are untimed. `call-spread-registration.test.mjs` passes and preserves the 718 earlier manifest rows.
- `gate100-build.log`: compiler and language-server binaries rebuilt.
- `gate100-depth-sweep.log`: 36 depth-named tests across all 138 compiler suites; 34 passed. One failure is the specialization-limit test listed below. The other, `string_concat_integer_fusion::checked_target_types_loans_shadowing_and_normal_depth_stay_unchanged`, needs an artifact directory variable that this sweep did not set. The sweep ran after the frame reductions and before the three later lowering repairs; the affected-suite run above covers those.

No new built-in names were added, so the IntelliSense catalog has no new entries.

## Outstanding

- Timing, allocation profiles and the full regression are pending checkpoint 101. No performance claim is made.
- Raw evidence has not been imported into the SQLite analytics store; the staged work under `/tmp/tsrust-gate100-*` is retained for that.
- The callback specialization-limit failure and the conditional nullable callable initializer panic found during integration are fixed; see [callable and depth regression repairs](./callable-regression-repairs.md).
- An arrow function invoked directly where it is written cannot take spread arguments, and `?.` cannot invoke an `owned once` callable. Both are documented boundaries.
