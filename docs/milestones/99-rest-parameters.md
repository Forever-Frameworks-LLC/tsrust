# Milestone 99: typed homogeneous rest parameters

Status: focused compiler, native, editor, documentation and benchmark-registration acceptance is complete. Exact SQLite artifact import, restoration and guarded duplicate cleanup are accepted; the gate is closed. Full regression and accumulated measurements remain queued for checkpoint 101.

## Observable behavior

Support one terminal rest parameter such as `function sum(first: int, ...values: int[]): int`. The rest parameter receives a fresh array containing the remaining supplied arguments in order. With no tail arguments it receives an empty array. Rest collection does not introduce optional element types or convert a supplied `undefined` into omission: each supplied tail value must satisfy the declared element type.

Required, optional and defaulted fixed parameters keep milestone 98 semantics. Required positions determine the minimum argument count; a rest suffix removes the fixed maximum. A caller must still supply an explicit `undefined` slot to reach a later fixed parameter when necessary. Evaluate the callee and all supplied arguments once, left to right, before entering the callee and evaluating reached fixed-parameter defaults. Pack each tail value once without duplicating side effects.

Permit rest syntax in declarations, arrows and callable annotations. A rest parameter must be last, must have a supported homogeneous array annotation, and cannot have an optional marker or initializer. Keep duplicate/reserved-name validation, normal source-depth limits and precise source diagnostics. Contextually typed rest arrows and generic `T[]` annotations must resolve through the same checked parameter contract as explicit annotations.

## Ownership and native representation

A rest array is an owning collection. Its construction follows the existing array-literal rules: primitive leaves copy, while owned strings, records, nested arrays and owning stored callable environments transfer into the fresh collection. A caller that needs an independent retained owner must explicitly clone it. Reading the rest parameter in the body does not silently clone tail arguments or introduce borrowed element storage. This differs from JavaScript's shared object references and must be stated in language documentation and editor help.

Use standard Rust collection storage and the existing typed array operations. Keep body access inference separate from the ownership needed to construct and pass the pack. Mutation of the rest array changes its own collection. Fixed-prefix read, mutate and consume contracts stay unchanged. Retain current move checks, capture loans, element access rules, finite callable family storage, drop behavior and unwind cleanup. Do not introduce production unsafe code, trait-object boxing, hidden clones, new Copy permissions or raised recursion limits.

Rest annotations need no additional data-access modifier to express ownership of the newly constructed array. If access modifiers are accepted, their meaning must be explicit and consistent across declarations and callable signatures; do not interpret a read modifier as permission to build an undocumented array of borrowed elements.

## Complete current domain

Cover every currently supported array element family: int, number, boolean, null, undefined, string, nominal records and owning finite stored callable families. Include supported nested ranks, generic record identities and generic element inference. Preserve existing precise rejection of mixed/nullable element arrays and borrowed callable array elements; adding rest syntax does not expand the array type system.

Carry rest identity, element types and accepted argument bounds through direct calls, generic specialization, imported signatures, concrete closures, read/mutate/once/owning callable parameters, callable record fields, conditional selections, optional invocation, returned environments and finite stored families. Validate callable compatibility against accepted source argument positions. Include valid rest-to-fixed callable adaptation where the accepted argument/result and ownership contracts match; reject incompatible tails or transfers without erasing the rest contract.

Existing array callbacks retain their documented supplied arguments. A callback with a rest suffix receives those remaining supplied arguments as a fresh pack. Check all current adapter paths rather than only direct declarations. Rest syntax must not turn optional/default parameter padding into actual supplied array elements.

## Acceptance

- Focused compiler fixtures must cover empty, one-element and multiple-element tails; all current element families; fixed-prefix omission/defaults; generic and imported signatures; callable adaptation and every supported dispatch context.
- Execute exact ordering, pack mutation, nested values, owning returns, stored environments and native examples at O0 and O3. Preserve source-bound expected outputs and original failures; rerun only affected checks after repairs.
- Check transferred-owner availability, unchanged fixed-prefix borrowing, capture lifetimes, normal drops, skipped work and panic cleanup. Ownership-only guarantees may be compile/resource evidence rather than a JavaScript throughput claim.
- Verify hover, completion, signature help and active-parameter selection for complete and incomplete calls. Help must distinguish fixed slots from a repeated rest element and explain the fresh owning pack. Check generic/imported forms and initializer scope with the integrated compiler.
- Register matched Node-after-tsc correctness workloads for each meaningful current runtime scenario. Keep measurement acceptance separate and pending checkpoint 101; SQLite remains canonical. No new timing run is required during this gate.
- Publish executable language examples only after current compiler and native verification. Every new built-in method or property still needs complete verified IntelliSense documentation in `crates/tsrust-lsp/src/builtins.json`; rest syntax itself introduces no built-in name.

## Reference and separate work

The [TypeScript handbook](https://www.typescriptlang.org/docs/handbook/2/functions.html#rest-parameters-and-arguments) describes terminal rest parameters and array-typed tails. [Untimed reference evidence](../../benchmarks/runtime/diagnostics/gate99-rest-reference/manifest.json) uses pinned TypeScript 5.9.3 and Node 24.11.1, with ten literal expected output lines and seven diagnostic cases. They establish reference behavior only; native focused implementation acceptance is recorded below.

Runtime call spread, array-literal spread, tuple/heterogeneous rest types and arbitrary iterables are separate gates. They do not excuse missing currently supported homogeneous rest types or callable contexts in this gate.

## Focused acceptance evidence

[Current acceptance](../../benchmarks/runtime/diagnostics/gate99-rest-parameter-implementation.json) binds 19 integrated compiler/editor paths, 34 accepted native cases, 17 unchanged benchmark/resource emissions and focused integrated editor checks. Thirteen comparisons are registered among 717 sources, with 575 complete benchmark cases per optimization and 24 resource scenarios. Original failures and repairs remain preserved. The [language guide](../../website/src/content/docs/language/rest-parameters.md) and 14 array callback catalog entries have verified examples; this gate adds no built-in names. Measurements and the full regression remain pending checkpoint 101.

[Gate closure](../../benchmarks/runtime/diagnostics/gate99-focused-closure.json) binds byte-exact SQLite restoration of 2,107 selected records and guarded duplicate cleanup. The 52 unsnapshotted historical intermediate source versions retain their original identities without a byte-restoration claim; the complete final3 source epoch and current native correspondence establish implementation acceptance.
