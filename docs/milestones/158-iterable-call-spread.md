# Gate 158: iterable call spread

Ordinary calls now expand dense arrays, strings, Sets and Maps in iteration order.
Each source evaluates once and stages its elements before the next argument.
Strings construct owning Unicode scalar character values without consuming their
source. Sets supply keys; Maps supply stored entry tuples. Arbitrary iterator
objects remain unsupported.

## Contracts and runtime behavior

Expanded elements satisfy the existing fixed, optional, default and rest parameter
contracts. Scalars copy; readonly owning arguments borrow; consuming owning
arguments transfer. Stored Set keys and Map entry tuples cannot be lent mutably,
because changing keys would invalidate lookup. Borrowed Map tuples can contain
noncloneable values. Empty constructors receive contextual types. Generic inference
uses expanded element types rather than treating a collection as one argument.

Dynamic argument-count mismatches now throw a structured TypeError with code
`call-arity` after source evaluation and before defaults or the target body. Catch
can recover and finally runs normally. This repairs the previous native assertion
panic and ensures error-effect checking sees the expanded call's failure. Constant
absent optional-call targets skip source effects; this uses a local immutable
absence proof rather than changing general lexical borrowing or flow narrowing.
String character values captured by a returned callable do not retain an alias to
the original source. String.raw retains its explicit output-copy behavior for
string and Set<string> substitutions. String code constructors and variadic Math
operations accept compatible iterable elements.

## Defects and focused evidence

- Red compiler/native witnesses preceded changes. The std:testing value fixture
  passes pinned Node unchanged and native execution. It covers Unicode, empty
  sources, readonly and consuming contracts, noncloneable entries, defaults,
  generics, source order, skipped optional sources and catch/finally propagation.
  Static checks reject collection-key mutation, readonly owner transfer, moved
  reuse and record sources. Native arity-recovery assertions pass. Its independent
  Node counterpart adds explicit argument-count guards because JavaScript itself
  does not enforce native fixed-parameter counts.
- Existing optional_chaining tests uncovered a pre-existing type-hint failure for
  chained selections followed by `??`; accepted gate157 consumers reproduce it.
  Nested optional field/index hints now retain the boundary's undefined alternative
  while resolving the underlying selection. Grouped-chain and normal depth checks
  remain intact. Existing native pointer, allocation and unwind checks pass without
  weakening their assertions.
- Compiler: 73 distinct focused checks. Retained coherent batch successes cover
  call_spread 10, exception_handling 20, higher_order_function_values 6,
  iterable_array_spread 2, native_completion 1, native_exception_region 3,
  string_from_codes 3, string_raw 5 and type_syntax 8. Final repair batch passes
  iterable_call_spread 3, optional_chaining 8 and nullish_coalescing 4. Unaffected
  passing suites were retained rather than replayed after the hint repair.
- Editor: iterable_call_spread 2 and iterable_array_spread 2 pass. Checked RuntimeError
  field facts now supply catch error-code hover types. Signatures, generic result
  types and seven documented examples are checked. Only string.concat,
  StringConstructor.fromCodePoint/fromCharCode/raw and Math.min/max/hypot catalog
  entries changed; other catalog entries remain identical.
- Seven catalog examples produce the expected output in native and pinned Node.
  Fixtures and expected output live under tests/fixtures/iterable-call-spread.
- Harness: 385 checks pass after refresh; final accepted-manifest repeat is retained
  with the acceptance logs. No full regression or timing.
- Untimed native/node-tsc verification passes the five cases recorded in
  runtime/results/2026-10-06T10-39-29-284Z-verify.json. The result is verifiedOnly and
  contains no timed execution rows. TypeScript requires rest signatures for dynamic
  iterable spread; its three helper signatures use rest arrays while scenarios
  supply the exact native counts and identical arguments. This difference is guarded
  by the source contract. Matched sources, independent closed-form result oracle,
  hashes, registration and runtime/profile/report support are complete.
- Workload checked before atomic refresh of all five consumers. Exact previous
  bytes remain in archive/iterable-call-spread-consumers-start; final hashes are in
  target/editor/iterable-call-spread-consumers.log.

Source, support, counterpart and record snapshots retain exact relative paths.
Logs are retained under target/editor/iterable-call-spread-*; initial failures are
preserved. New workload remains unmeasured pending the collection milestone.
