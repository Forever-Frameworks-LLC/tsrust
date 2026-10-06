# Milestone 84: canonical nullable type foundation

Status: complete. Focused implementation and documentation, the retained checkpoint 86 regression, and matched runtime/resource comparisons are accepted.

## Purpose

Array lookup and removal methods need a result that can contain either an element or undefined. Nullable records, parameters and returns need the same type machinery. This gate establishes that shared representation before exposing methods with optional results. The complete nullable-language stage also requires narrowing, optional fields and arguments, nullish coalescing and optional chaining; strict stable-place narrowing is accepted in Milestone 85; the other dependencies remain separate work.

## Initial nullable domain

The initial supported domain combines one existing payload type with null, undefined or both. Canonicalization removes duplicate members and collapses a single remaining member. Arbitrary unions of different payload types remain part of the broader value plan; this gate must not advertise them. Nullable callable annotations and inferred nullable closures remain unsupported in this gate: their origin substitution and borrowed-environment escape contracts need separate work. Existing nonnullable callables must keep their current behavior. Recursive-record-array payloads require the existing iterative cleanup model; direct unsized record cycles must retain precise diagnostics.

Named annotations use `T | null | undefined`, with parenthesized grouping. `int[] | null` describes an absent-or-whole-array value. `(int | null)[]` describes nullable elements and remains unsupported until the array element representation and all element operations support them. Optional fields and parameter defaults also remain separate work.

Absent-only `null | undefined` must preserve both alternatives. Contextual injection must not collapse their distinction, and singleton unions reduce to their existing type.

## Implementation requirements

- Canonicalize union members so member order and duplicate members do not create different checked types. Keep null and undefined distinct. Do not use void as a substitute for undefined.
- Support whole-union observation through typeof, strict singleton equality and explicit Boolean/String conversion without moving a borrowed payload. Existing Array.isArray must inspect an array alternative rather than classify every union as a non-array. Other unsupported union observers must reject precisely instead of producing static fallback results.
- Carry nullable scalar, string, array and record values through checked annotations, initialization, assignment, parameters, returns and conditional joins. Preserve existing callable ownership contracts wherever callable alternatives become supported; reject unsupported contracts precisely.
- Emit ordinary Rust tagged values without an interpreter, implicit owned clones or shared mutable ownership. Determine copying from the alternatives: an owned string, array or record must remain owned when wrapped.
- Inject and transfer an alternative exactly once. Retain source evaluation order, branch laziness, replacement disposal and existing capture/loan restrictions. Do not allow a union wrapper to conceal a moved value or bypass a borrowed value's lifetime.
- Require sound projection before accessing a potentially absent receiver. This foundation must not advertise unsafe member access, array indexing or narrowing that the compiler has not proved.
- Include union alternatives in record dependency analysis and storage accounting. Recursive layouts and drop behavior must remain valid; hidden recursive edges cannot become unsized native types or evade cleanup checks.
- Editor annotations must respect canonical union semantics regardless of member order. A nullable array must not receive unconditional array methods solely because its annotation text ends with `[]`.

## Accepted ownership contract

A tagged nullable value owns its present payload. Constructing a new tag from an owned string, array or record transfers that payload, including when the resulting temporary is passed to a read-only consumer. Borrowing an already constructed union of the same type must leave its source available. Widening an owned union to a different tag must transfer ownership without cloning; scalar-only unions remain copyable. Focused compiler diagnostics and generated native execution verify this contract.

Whole-union observers borrow the tag and inspect its current alternative. Borrowing must not create a hidden owned copy or allow a later argument to mutate or transfer the receiver while its loan remains active.

## Remaining observable limits

General unions of distinct payload types, nullable callable values, nullable array elements, optional fields and parameters, broader flow narrowing, nullish coalescing and optional chaining remain outside this foundation. General union coverage must remain in progress after this nullable gate is accepted.

The compiler rejects Number conversion of a nullable union, direct console inspection of a nullable union, and inspection of records or arrays whose field graph contains nullable values. Template interpolation of a nullable union also requires a separate supported conversion contract. Explicit String/Boolean conversion, typeof and strict comparison against null or undefined provide supported whole-value observations with focused native acceptance. Equality with a payload or another union remains unsupported until a sound comparison contract exists.

These limits must produce diagnostics. An unrelated nullable record definition must not break ordinary record inspection or trigger an unused inspector during code generation.

## Acceptance matrix

| Area | Required evidence |
|---|---|
| Type identity | Reordered, grouped and duplicate alternatives resolve consistently; absent-only unions preserve null and undefined |
| Scalar transport | Checked and native nullable int, number and boolean values, including both present and absent branches |
| Owned transport | Strings, arrays and records survive transfer and return; normal replacement and unwind release owned payloads |
| Evaluation | Conditional branches and injected expressions execute once, in source order; scalar/object observations use the actual alternative |
| Ownership | Borrowed captures, transfers, repeated moves and receiver loans retain precise diagnostics; unsupported nullable callables cannot conceal borrowed environments |
| Layout | Standard tagged layouts, recursive record checks, iterative disposal of existing recursive array-record payloads and no generated unsafe code |
| Editor | Nullable annotation hover, member/index guards and signature help agree with checked compiler behavior; Boolean/String documentation includes accepted union behavior |
| Examples | Language examples checked by the current compiler and executed natively where outputs matter |
| Comparisons | Equivalent source workloads registered separately from checkpoint measurements |

No new built-in method or property is implied by this foundation. Any built-in added during implementation must have complete catalog documentation and verified examples before acceptance. Final evidence must identify the actual supported syntax and remaining Stage 2 dependencies, rather than treating this milestone as completion of the whole nullable-language stage.

## Focused acceptance evidence

[Gate84 evidence](../../benchmarks/runtime/diagnostics/2026-10-03T10-48-34-000Z-nullable-type-foundation-implementation.json) retains actual command logs, failed attempts, narrow repairs, source identities and native documentation examples. Three arena groups, four compiler groups and one editor group pass. Generated programs execute at optimization levels 0 and 3; resource fixtures verify transfer pointers, allocation, normal/unwind disposal and recursive-array-record cleanup.

Six matched workloads are registered: 126 source cases, 22 literal hands, 119 negative guards and 12 rejected semantic mutants pass. Actual Tsrust workloads check and emit; timing is accepted at checkpoint 86. The previous 526 manifest objects and 514 measured report rows remain unchanged. Concurrent diagnostic quick fixes and unrelated editor changes are preserved.

## Checkpoint 86 measurements

[Completed checkpoint evidence](../../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) accepts 806 named checks on the tested revision, retaining successful targets and repairing failures without a full restart. [Runtime/resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 6 comparisons for this milestone across eight runtimes. Native leads compiled Node on 3/6 and the fastest scripting runtime on 2/6. Results measure complete setup, selection, operation and retained output; they do not establish isolated method latency or allocation parity.

Across gates 82–86, forty new rows and one existing-row refresh produce 30,504 timed batches and 123 native profiles, all with zero final live bytes. The report now contains 554 individually dated measured rows. The other 513 report sections, nine primitive memory tables and auxiliary baselines remain byte-exact. Concurrent project/editor revisions outside the tested checkpoint are explicitly excluded from its regression claim.
