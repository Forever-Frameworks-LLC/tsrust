# Feature and comparison inventory

This inventory makes implementation gaps and comparison gaps visible independently. It covers primitive types, their built-in methods and properties, collections, binary data, control flow, iterators, generators, decorators, TypeScript types, and other core language constructs. It is a work queue rather than a conformance claim.

The API catalog is extracted from the project's pinned TypeScript 5.9.3 standard-library declarations, including their `esnext` declarations. Every used declaration file has a recorded SHA-256 identity. Interface overloads share one row; inherited operations remain under their declaring interface. Namespace functions in Reflect and Intl are included. Declaration availability alone does not establish that an API is standardized or present in each pinned runtime.

Sources: [TypeScript library declarations](https://github.com/microsoft/TypeScript/tree/v5.9.3/src/lib), [ECMAScript specification](https://tc39.es/ecma262/), [TypeScript iteration reference](https://www.typescriptlang.org/docs/handbook/iterators-and-generators.html), and [modern decorator reference](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-0.html#decorators). Legacy decorators are tracked separately. Browser and Node host APIs require their own inventories as the backend and desktop workstreams expand.

## Updating

1. Add language constructs to `language.json`; standard-library members are extracted automatically.
2. Update `status.json` with implementation state, actual evidence, and genuine workload IDs. An implemented feature can still have a missing comparison. A paired group identifies registered matched scenarios; checkpoint evidence separately establishes whether those sources have measured execution results.
3. Run the pinned Node executable with `benchmarks/coverage/update.mjs` from the repository root. It regenerates `inventory.json` and `docs/guides/language/feature-coverage.md`, checks unique IDs, rejects unknown workload or allocation-probe references, and verifies local evidence paths.
4. Review the generated diff. Do not mark an isolated comparison complete because a compound workload contains a similar operation.

## Acceptance for each operation

Define the intended native Rust semantics and deliberate JavaScript differences first. Cover ordinary values, boundaries, empty inputs, invalid inputs, ownership and aliasing, evaluation order, and disposal where applicable. Numeric coverage includes overflow, division rules, signed zero, NaN, infinities, precision, and conversions for the relevant type. String coverage includes ASCII, multiple UTF-8 widths, combining sequences, NUL, case expansion, and boundary behavior. Collection coverage includes empty/singleton inputs, storage transitions, growth, mutation, independent clones, callback effects, and nested elements.

Implement an independent expected-result oracle and matched TypeScript counterpart. Keep exactly the same observable work in both implementations and disclose representation costs. Use ops/s for execution and normalized numbers for memory; retain raw records, versions, source hashes, warmups, and disposal checks. Pure type-system constructs require diagnostic and compile-time evidence rather than fabricated runtime throughput.

Full compiler regression and accumulated performance comparisons run every five gates after implementation settles. Use focused checks while developing. After a failed full run, preserve passed suites, rerun affected cases after repairs, and resume unrun suites. Keep gates coherent enough to cover related methods together, rather than requiring a full suite for each trivial method addition. Previously measured operations do not need retiming for an unrelated new API; refresh only affected comparisons.

## Implementation order

Homogeneous array concatenation follows accepted callback traversal, native strings, Number/Math operations, classic loops, slicing, map/filter, fixed scalar storage, callback predicates and both reduction directions. Declared-function values follow; complete statically resolved higher-order flows require recursive callable annotations and native specialization foundations. Close each gate's focused compiler, editor, executable documentation and relevant ownership/disposal evidence before implementing the next. Register matched runtime sources during the gate; collect accumulated performance measurements at the five-gate checkpoint. Continue filling missing primitive comparisons and coherent families of standard Rust methods, then expand switch and optional/result types needed by collection searches and removals. General iterators, lazy pipelines, generators, async iteration, classes and decorators remain explicit dependency-backed work. TypeScript type features, modules and backend/desktop APIs retain their documented workstreams.

Original vision exclusions such as unrestricted `any`, arbitrary prototype mutation, and `eval` are recorded as policy rows. The user accepts native Rust quirks; this inventory does not revive the retired UTF-16 compatibility requirement.
