# Gate 142: checked exception handling

Status: accepted. Focused compiler/native, editor, grammar, guide, current consumer, runtime harness and matched untimed benchmark checks pass. One bounded compiler and editor test pass was run for this gate because its changes reach every function; its findings are repaired and listed below. No performance measurement was performed.

## Behavior

Try/catch/finally and explicit throw are enabled through checked source execution. Completion transport distinguishes normal execution, return, throw, break and continue. Finally executes before a pending exit, and an exit produced by finally replaces it. Errors from called functions propagate into handlers. Catch values use inferred alternatives and require narrowing before type-specific operations.

Supported arithmetic and array-bound failures use structured runtime errors. Project emission resolves their source filename and byte range to the original module. Uncaught checked errors execute applicable cleanup, report a diagnostic and return unsuccessful process status without a native panic. External native bindings require audited contracts; allocation failure, external termination and native defects are not claimed recoverable.

See [the error-handling guide](../exception-handling.md). Its example produces identical native and independent Node output: failed, cleanup, 4.

## Ownership and callable integration

Checked Result transport carries owned values without implicit cloning. Named, concrete, abstract and specialized callback paths use saved error layouts. Stored reusable callback environments can transfer into owned test registration. Their errors are reported and later tests execute; source use after transfer rejects. Single-use stored callbacks cannot satisfy reusable test contracts.

The synchronous runner handles typed test/hook failures, runs teardown and preserves later tests. Registration and nested execution guards produce checked testing errors. Prepared suite registration restores its entry scope on callable exit. Prepared test-command finish returns a checked failure instead of process::exit. Legacy unprepared paths retain their existing behavior.

## Supported contracts and limits

- A caught value needs a resolved error contract. Functions, function values, stored callables and the audited `std:testing` bindings have one. A call whose contract cannot be resolved inside a `try` with a named catch is an `error-effect` error; nothing is treated as non-failing by default.
- `expect(...)` matchers and the `std:testing` assertions fail with a catchable assertion error. `expect(1).toBe(2)` inside `try` is caught and `error.name` is `"AssertionError"`.
- Audited native bindings: `std:path` `basename` (cannot fail); testing-runtime `shareCallback`, `numericValue`, `lengthValue`, `hasLength` (construction and reads, cannot fail); `checkAssertion`, `runTests`, `finishTests`, `registerTest`, `registerHook`, `beginSuite`, `beginFile`, `endSuite` (runtime error). Every other native binding is unresolved.
- Reusable stored callbacks transfer into test registration. Single-use stored callbacks are rejected there. Shared registration beyond the existing `shareCallback` path is not added by this gate.
- Prepared suite and file entries restore the registration scope when registration throws.
- Allocation failure, external termination and native defects are not recoverable.

## Repairs made during acceptance

- Catching a matcher failure was rejected as an unresolved contract. The first analysis of such a program ends with `recursion-inference`, and the speculative access passes that follow did not record stored-callable origins, so every stored call looked unresolved. Speculative passes now note origins for effect lookup (`speculative_access_run`, `Registry::note_speculative_origin`); definitive passes still validate and register them.
- After recursive inference the analyzer renumbers closures. Saved call origins (`Expr::callable_origin`) were not renumbered, which caused out-of-bounds panics in thirteen tests of `recursive_callable_results` and `nullable_callables`. They are renumbered now.
- Three depth-limit tests overflowed the 2 MB test stack because recursive frames had grown: the statement and arrow-function parser (`closures`), `Analyzer::coalesce` (`nullish_coalescing`) and `Analyzer::block_with_scope` (`for_of`). Each was split into smaller frames. No limit changed.
- Tests that still expected older behavior were updated, each after confirming the current behavior: `formatting` (try/catch now checks), `array_pop_shift` and `value_types` (mixed literals infer a union element type), `array_removal` (an indexed element binding is a borrowed view), `for_of` (`for...in` is supported), `owned_arrays` (emitted loops carry a label), and the unit test that lists audited native bindings. Previous files are in `archive/exception-acceptance-stale-tests-start` and `archive/exception-opaque-source-diagnosis-start`.

## Editor

- The language server declares the catch name in the handler body: hover shows `(catch) error` with the checked type when a use carries one, definition goes to the catch clause, and the name is not visible after the handler.
- The grammar scopes `try`, `catch`, `finally` and `throw` as control keywords, together with `switch`, `case`, `default` and `in`, which were missing.
- Formatting keeps `} catch (error) {` and `} finally {` on the closing line.

## Evidence

- Red witness: `/private/tmp/exception-matcher-boundary-red.log`. Repair: `/private/tmp/exception-speculative-origin-repair.log` (exception_handling 20/20).
- Bounded complete pass. Part one ran 138 suites on the sources before the later repairs (`/private/tmp/exception-acceptance-complete-compiler.log`, 5 failing suites) and was then stopped. Part two ran the other 48 suites and the repaired suites on the final sources (`/private/tmp/exception-acceptance-remaining-suites.log`: 55 suites, 390 pass, 1 stale row in `value_types`, repaired in `/private/tmp/exception-acceptance-value-types.log`). The 44 callable, exception and parser suites were also rerun after the parser and renumbering repairs (`/private/tmp/exception-repair-final-batch.log`, `/private/tmp/exception-origin-remap-repair.log`). Suites that passed in part one and are outside those groups were not rerun after the frame splits, which move code without changing it.
- Compiler unit tests 289/289 (`/private/tmp/exception-acceptance-lib.log`) and editor tests 225/225 (`/private/tmp/exception-acceptance-complete-editor.log`) on the final sources. The catch-binding editor test passed on its first run; there is no red log for it.
- Grammar: `/private/tmp/exception-grammar-red.log` fails, `/private/tmp/exception-grammar.log` passes.
- Five consumers replaced by atomic rename from the final builds; hashes in `target/editor/exception-acceptance-consumers.log`; previous consumers in `archive/exception-acceptance-consumers-start`.
- Runtime harness on the refreshed consumers: `/private/tmp/exception-acceptance-harness.log`, 368/368.
- Untimed native/node-tsc verification on the refreshed consumers: `benchmarks/runtime/results/2026-10-06T04-04-38-673Z-verify.json`.
- The guide example prints `failed`, `cleanup`, `4` natively and in Node (recorded earlier in this gate).
- Earlier evidence of this gate is indexed by the handoff sections. No Git operations occurred.

## Benchmark readiness

Exception_completion observes a caught integer throw or normal return, finally cleanup trace, finally replacing a return, and dynamic input text. Its matched TypeScript source uses the same scenario. Independent oracle uses closed-form arithmetic; registration tests guard source hashes, scenario operations, denominator and prior manifest history, and compare platform execution across sizes and seeds.

Result-aware native ABI adaptation extracts successful workload results and fails verification explicitly on unexpected checked errors. Existing result observers, argument preparation and measurement loops are retained. Runtime and allocation profiling hooks include source guards and oracle fingerprints. The workload is unmeasured. Its source and implementation acceptance flags are set.
