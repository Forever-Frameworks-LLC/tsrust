# Gate 141: callable expression invocation

Status: focused compiler/native, editor, guide, refreshed-consumer and matched benchmark acceptance complete. Full regressions and measurements remain deferred.

## Behavior

A call may invoke a factory result or an immediately invoked arrow/function expression, including chained factory results. Record fields and indexed callable arrays retain their existing call support. The callee is evaluated once before arguments, and argument evaluation remains left to right. Defaults, rest arguments, spread counts, generic factories and once/reusable ownership contracts remain checked.

Required IIFE parameters may infer a type from an argument with a known type. A spread position may infer a type when every possible contributing source agrees; `undefined` is a valid inferred type. Arguments without an available type hint need an explicit parameter annotation. Conditional alternatives retain the existing same-origin requirement. No implicit clone was added.

See [the syntax guide](https://forever-frameworks-llc.github.io/tsrust/language/callable-expressions/). Its assertion example passes the current native compiler and independent Node reference.

## Implementation and repairs

Fresh Call/Closure callees enter the existing conditional-call machinery, checked once to obtain their concrete origin. The checked invocation helper reuses that value rather than creating or consuming it twice. Abstract result signatures resolve through their actual result shape. Fresh spread callees use the same staging and contextual parameter inference.

Inline-never helpers keep contextual inference and capture-result traversal outside shared recursive frames. Temporary fresh-callee capture loans end when the result cannot retain a callable. Returned callables and aggregates containing them retain their capture restrictions; owners already transferred remain transferred. A red mutable IIFE witness exposed an obsolete temporary loan that incorrectly blocked the subsequent array read.

Additional red witnesses exposed missing untyped spread inference, rejection of an actual undefined argument as unknown, and a misleading void-argument diagnostic when no hint exists. Those cases now infer valid types or report parameter-inference directly. Non-callable computed expressions report a type error about the callable contract.

Editor result inference recognizes checked invocation spans through leading grouping parentheses. Checked closure facts supply IIFE signature help; a factory result supplies its returned callable signature rather than the factory's own parameters. Hover, parameter definitions and active parameter selection have focused protocol coverage. Computed signature help never searches inside a factory's arguments to invent a callable type.

## Fixtures and counterpart corrections

The initial conditional fixture selected different arrow origins, which the existing contract rejects. It now selects two results of one factory. An unsupported Function annotation in a tuple fixture was replaced with a supported record field and indexed arrow-typed callable array; this does not claim callable arrays are unsupported.

The first TypeScript verification found an unconverted nested Function return annotation and an ordinary array used as a fixed-arity spread. The counterpart now uses a TypeScript callable return and a tuple annotation. The native source retains its dense array and inferred IIFE parameters. The failed verification and corrected untimed records are retained.

## Evidence

- Exact original snapshots: archive/callable-expression-start, archive/callable-expression-benchmark-start, archive/callable-expression-consumers-start and archive/callable-expression-records-start.
- Initial and focused red logs remain under target/editor/callable-expression-*. Compiler module-path and command working-directory mistakes were corrected; their failures do not represent accepted feature evidence.
- Eight focused compiler suites account for 104 distinct passing tests in callable-expression-final-compiler.log: expression calls3, spread10, callable fields35, function expressions2, higher-order6, optional parameters21, rest parameters19 and type syntax8. Later inference changes pass expression3, higher-order6 and type syntax8 in callable-expression-inference-final.log. The final expanded expression fixture passes all3 tests in callable-expression-contracts-final.log, with seven static rejections.
- Two gate-specific editor checks cover factory/IIFE result and inferred-parameter hover, definition, computed signature contracts and active parameter selection. Two adjacent existing signature-help checks also pass; exact logs remain under target/editor.
- Current executable assertion fixtures pass the independent Node reference in callable-expression-accepted-node.json. The guide has native and Node evidence in callable-expression-guide.json.
- Five consumers are atomically refreshed and source/destination SHA256 matches recorded in callable-expression-final-consumers.json. Both debug and release builds pass.
- Runtime harness365/365 passes in callable-expression-final-harness.log. Corrected source guards and registrations pass in callable-expression-registration-final.log. Final native/node-tsc untimed verification of callable_expressions and function_expressions passes in benchmarks/runtime/results/2026-10-05T14-26-52-167Z-verify.json (callable-expression-accepted-verify.log).

## Benchmark support

callable_expressions retains nine cells for direct factory invocation, nested factory results, arrow and function IIFEs, inferred spread, a default parameter, ordered factory/argument calls, their trace and dynamic input text. One complete scenario is the denominator. The oracle uses direct arithmetic and a fixed trace rather than the callable implementation; registration tests cross-check full observed values and reject altered source/hashes/denominators. Native and TypeScript sources, current hashes, observer/ABI support and run/profile/report hooks are ready. Timing remains pending the callable feature-family milestone.
