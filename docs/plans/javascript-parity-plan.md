# JavaScript values and namespace implementation plan

Requested scope: progressively implement missing JavaScript value and namespace behavior in Tsrust. This plan is the shared compiler/editor dependency contract. It does not declare the entire language equivalent to JavaScript. Reference behavior comes from ECMAScript and the repository's pinned Node.js v24.11.1; emitted executables remain native and dependency-free. Preserve ownership checks and the existing numbered compiler work. These compatibility stages do not reserve or renumber another compiler agent's gates.

## Progress

Stage 1 and Array.isArray, Boolean(...) and String(...) are implemented with focused acceptance; see [evidence and remaining limits](../milestones/value-types-and-namespaces.md). Stage 2 is in progress: [canonical nullable types](../milestones/84-nullable-type-foundation.md) have focused compiler, editor and native acceptance; [strict nullable narrowing and payload projection](../milestones/85-nullable-flow-narrowing.md) now have focused acceptance; [nullish coalescing](../milestones/94-nullish-coalescing.md) and [nullish assignment](../milestones/95-nullish-assignment.md) and [optional chaining](../milestones/96-optional-chaining.md) now have focused acceptance. Broader guards and optional fields/arguments remain pending. Stages 4 and 5 and the remaining Stage 3 APIs are planned. Checkpoint 81 accepts the existing conversion comparisons. Gates 82–85 add Array.join, Array.toString, canonical nullable transport and strict nullable narrowing with focused implementation and documentation acceptance; their measurements and the affected String(array) refresh are accepted at checkpoint 86. Nullable storage and strict guarded payload access now support implementing optional array results.

## Audit baseline (before this implementation)

The compiler currently represents int, number, boolean, string, homogeneous arrays, nominal records and typed callable values. It rejects null/undefined. Its existing global namespaces are Math, Number and console. The editor catalog covers their supported methods plus primitive/array methods; namespace availability must follow checked compiler behavior rather than editor-only suggestions.

Missing foundations include null/undefined, nullable/optional unions and narrowing, dynamic heterogeneous values, arbitrary object keys/property metadata, BigInt, Symbol and host/error/async objects. Missing standard surfaces include Object, JSON, Array static methods, String/Boolean conversion and constructors, Number.parseInt and number.toString, Math.random, Map/Set, Date/RegExp/Error/Promise and binary data APIs. Console still has documented long-inspection/Unicode/host differences. The implementation order below follows dependency cost and utility, not an unbounded list of method names.

## Stage 1 — null and undefined (implemented)

Observable acceptance:

- Parse null and undefined literals and type annotations. Keep them distinct from one another and from void, which is still the unusable result of a procedure.
- Infer either singleton type for local variables; support parameters, returns, record fields, homogeneous arrays, nested arrays, callbacks/captures and assignment without allocating singleton payloads.
- Support strict equality/inequality with either singleton, including comparisons against other already-supported types. Evaluate operands once, preserve their order and retain loans for borrowed values. Different types compare unequal; null === undefined is false. Do not enable coercive ==/!=.
- typeof returns the JavaScript spelling for supported values, including typeof null == "object", undefined == "undefined", native int/number == "number" and callable == "function". Evaluate resolved operands once without consuming owned values. Handle an unresolved bare identifier deliberately; never erase errors in a member/call expression.
- void expression evaluates its operand once and returns undefined. !null and !undefined return true. Existing native arithmetic and boolean conditions do not silently become general dynamic coercion.
- Number(null) is zero; Number(undefined) is NaN. Template interpolation prints their names.
- Every console method accepts them. Log/dir/table rendering, placeholders, assert truthiness, labels/defaults, JSON placeholder omissions/null array slots and dir options must match the pinned Node reference where supported. Optional undefined values trigger console defaults; an explicit null retains its distinct behavior.
- Extend hover/completion/type inference and update affected builtin descriptions with examples. Add precise diagnostics for illegal type assignment, arithmetic, dereference and mixed arrays.

Implementation seams: syntax.rs owns literals/type/unary representation; analysis.rs owns type resolution, copy/loan rules, equality and conversions; emit.rs owns distinct zero-sized native singleton types and evaluation order; storage/fixed_storage/record_runtime retain valid layouts and cleanup; runtime_console.rs owns inspected and JSON-compatible representations; intelligence.rs/builtins.json own editor discovery and documented examples.

Proof: Node stdout/stderr comparison at O0/O3, focused source diagnostics and access contracts, nested storage/callback fixtures, no generated unsafe code, and compiler/editor documentation checks. Register equivalent value workloads for the next scheduled checkpoint; allocation/throughput evidence must remain pending until actually measured.

Stage 1 deliberately does not claim string | null, optional fields, heterogeneous arrays or optional chaining. These require Stage 2's type representation and control-flow proof rather than a console formatting shortcut.

## Stage 2 — nullable types and optional results

Compiler prerequisite: canonical union type representation (rather than more one-off enum cases), tagged native layouts, joins/injection/projection, and loan/move tracking through alternatives. Start with T | null and T | undefined, including owned string/array/record alternatives. Parse optional record fields/parameters with undefined defaults. Require explicit narrowing before unsafe member access; handle == null only if coercive equality is separately approved as language behavior.

Strict singleton ===/!== narrowing, conditional joins, nullish coalescing, nullish assignment and typed optional chaining are accepted in the current canonical domain. Optional fields/arguments and broader guards remain pending. Prove short-circuit order, optional argument evaluation, branch ownership, replacement/drop, callback returns and no implicit owned clones. Expose narrowed editor types and diagnostics only after the compiler supplies them. This stage unblocks Array.find/findLast/pop/shift/at and JSON.stringify's potentially undefined result. All method overloads/defaults and examples must match the actual supported signatures.

## Stage 3 — immediately useful static APIs

| Surface | Compiler work | Must wait for | Acceptance |
|---|---|---|---|
| Array.isArray | Borrow/evaluate one supported value; distinguish array types | Stage 1 value coverage | Scalars/records/callables false, arrays true; optional/undefined input; editor namespace/signature |
| Object.is | SameValue for numbers (NaN and signed zero), primitives and native object identity | Stage 1; identity lifetime proof | NaN true; -0 versus +0 false; aliases versus distinct allocations; cross-type false; no structural-equality shortcut |
| String and Boolean | Explicit conversions, truthiness and array/object conversion rules | Stage 1; full coercion contract | Defaults/null/undefined, NaN/-0, text/empty collections; source order and ownership |
| Number.parseInt / number.toString | Prefix/radix parsing and floating-point radix formatting | Standalone numeric algorithms; Stage 1 defaults | Radix/default/coercion/error boundaries, negative zero/infinities and rounding; Node differential cases |
| JSON.stringify | JSON value conversion, undefined omission, replacer/space behavior and result type | Stage 2 optional result; property model for complete replacers | Root undefined; object omission versus array null; escaping; ordering; indentation; nonfinite values; cycles/errors |
| Object.keys/values/entries/hasOwn | Observable own-property enumeration and key ordering | Keyed object model; heterogeneous results for values/entries | Numeric-key ordering, empty/missing properties, safe borrowing, typed result arrays |
| Array.from/of | Iterable conversion and typed result inference | Stage 2/4 heterogeneous/iterable contracts | Element order, optional mapping/index args, iteration cleanup and independent output |

Implement a coherent supported overload set with explicit remaining differences; never advertise absent overloads as working. Each namespace gets root hover, completion, member hover and parameter help before acceptance. Reuse compiler checking for catalog witnesses.

## Stage 4 — broader values and collections

Design a tagged dynamic value domain for explicitly heterogeneous containers without weakening existing statically specialized arrays/records. Define ownership/identity first: wrapping an owned value, borrowing it, moving it out, mutation, drop and cycle handling. Avoid a blanket any escape hatch. Add string-keyed property storage/enumeration and a clear distinction between absent keys and present undefined. Only then implement full Object/JSON overloads, Map/Set, sparse array/property behavior and callable inspection. BigInt requires exact arbitrary-width arithmetic, division/rounding/radix and mixed-number diagnostics; Symbol requires unique identity/property keys and registry semantics. Both need an explicit runtime/dependency/performance decision before implementation.

## Stage 5 — native host and stateful standard objects

Date requires wall-clock/calendar/time-zone parsing/formatting contracts; RegExp needs a standards-compatible engine and UTF-16 indexing policy; Error needs throw/catch/unwind and source-stack metadata; Promise requires async scheduling/lifetime/cancellation contracts. TypedArray/ArrayBuffer need binary storage, sharing/endian/bounds and identity contracts. Math.random needs a documented native entropy/PRNG contract and distribution tests, not falsely identical random sequences. Node modules are a separate host boundary; do not imply all Node APIs from global console compatibility.

## Console completion track

Proceed independently where foundations are available: long collection wrapping/column grouping, complete Unicode display widths, inspect options, terminal color behavior, warning scheduling and timer unit formatting. Stage 1 removes null/undefined gaps; Stage 2/4 unlock heterogeneous/cyclic/property inspection. Native traces cannot print the same frames as a separate Node executable; source maps/debug metadata can improve Tsrust source traces. Inspector integration remains host work. Maintain the precise difference ledger in ../milestones/console-parity.md.

## Shared progression and release checks

For each stage, record planned/implemented/verified/checkpoint-pending separately in a milestone. Compiler progression must provide checked types, argument/default rules, result types, side effects, ownership and errors before the editor publishes a capability. Editor work can prepare examples/test fixtures in parallel, but must gate completion and documentation on compiler witnesses. Runtime values must pass generated native execution, not just parsing.

Run focused affected checks during implementation. Preserve running work and passed checkpoint evidence. The full regression/performance checkpoint remains the numbered compiler workflow (next checkpoint gate 96), not a full-suite restart for each namespace. Add equivalent benchmark sources and pending records; do not mark an unmeasured method as measured. Node is a reference in tests, never an executable dependency of generated programs.

Sources: [ECMAScript value types](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-ecmascript-language-types), [Node.js 24.11.1 console](https://nodejs.org/download/release/v24.11.1/docs/api/console.html). Per-stage behavior must also be checked against the relevant primary specification sections and pinned executable.
